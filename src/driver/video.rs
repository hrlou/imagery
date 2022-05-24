pub mod prelude {
    pub use crate::driver::{self, prelude::*};
    pub use crate::driver::video::*;

    pub extern crate ffmpeg_next as ffmpeg;
    pub use ffmpeg::{
        decoder,
        format::{input, Pixel, context::Input, stream::Stream},
        frame::Video as Frame,
        media::Type as MediaType,
        software::scaling::{
            flag::Flags as ContextFlags,
            context::Context,
        },
        codec::{
            decoder::Video as VideoDecoder,
            context::Context as CodecContext,
        },
    };
}

use std::{path::Path};
// public exposure
pub use prelude::*;
pub use ffmpeg::init;
use std::cell::RefCell;

pub mod frame {
    use image::DynamicImage;

    use crate::driver::video::prelude::*;

    pub fn parse(frame: &Frame) -> Option<Vec<u8>> {
        use std::io::Write;

        let data = frame.data(0);
        let stride = frame.stride(0);
        let chunk_size: usize = 3;
        let byte_width: usize = chunk_size * frame.width() as usize;
        let height: usize = frame.height() as usize;
    
        let mut buf: Vec<u8> = vec![];
        buf.reserve((frame.width() * frame.height() * chunk_size as u32) as usize);
        for line in 0..height {
            let begin = line * stride;
            let end = begin + byte_width;
            match buf.write_all(&data[begin..end]) {
                Ok(_) => {},
                Err(_) => return None,
            }
        }
        Some(buf)
    }
    
    pub fn image(frame: &Frame) -> Option<RgbBuffer> {    
        match parse(frame) {
            // DynamicImage::
            // image::DynamicImage::ImageRgb8(())
            Some(buf) => {
                RgbBuffer::from_raw(frame.width(), frame.height(), buf)
            },
            None => None,
        }   
    }
}

pub struct Video {
    pub input: RefCell<Input>,
    pub stream_idx: usize,
}

impl Video {
    pub fn new<T>(path: T) -> Option<Video>
    where
        T: AsRef<Path>,
    {
        let i = input(&path);
        match i {
            Ok(i) => {
                let stream = Video {
                    input: RefCell::new(i),
                    stream_idx: 0,
                };
                Some(stream)
            },
            Err(_) => None,
        }
    }

    pub fn setup_stream<'a>(&mut self, kind: Option<MediaType>) -> Result<(), ffmpeg::Error> {
        let kind = kind.unwrap_or(MediaType::Video);
        let input = self.input.borrow();
        let stream = input.streams().find(|stream| {
            let codec = CodecContext::from_parameters(stream.parameters()).unwrap();
            codec.medium() == kind
        }).ok_or(ffmpeg::Error::StreamNotFound)?;
        self.stream_idx = stream.index();
        Ok(())
    }

    pub fn video(&mut self) -> Result<(VideoDecoder, Context), ffmpeg::Error> {
        let input = self.input.borrow();
        let stream = match input.stream(self.stream_idx) {
            Some(stream) => {
                stream
            },
            None => return Err(ffmpeg::Error::StreamNotFound),
        };
        let context = CodecContext::from_parameters(stream.parameters())?;
        let video = context.decoder().video()?;
        let context = Context::get(
            video.format(),
            video.width(),
            video.height(),
            Pixel::RGB24,
            video.width(),
            video.height(),
            ContextFlags::BILINEAR,
        )?;
        Ok((video, context))
    }
}

impl Driver for Video {
    type Error = ffmpeg::Error;

    fn frames<P: FnMut(&RgbBuffer, usize) -> bool>(&mut self, mut process: P) -> Result<bool, Self::Error> {
        let (mut video, mut context) = self.video()?;
        let mut frame_idx: usize = 0;

        let mut receive =
            |decoder: &mut VideoDecoder, idx: usize| -> Result<bool, ffmpeg::Error> {
                let mut decoded = Frame::empty();
                while decoder.receive_frame(&mut decoded).is_ok() {
                    let mut frame = Frame::empty();
                    // resize the frame
                    context.run(&decoded, &mut frame)?;
                    let frame = frame::image(&frame).unwrap();
                    if !process(&frame, idx) {
                        return Ok(false);
                    }
                }
                Ok(true)
            };

        let mut input = self.input.borrow_mut();
        for (s, packet) in input.packets() {
            if s.index() == self.stream_idx {
                video.send_packet(&packet)?;
                if receive(&mut video, frame_idx)? == false {
                    break;
                }
                frame_idx += 1;
            }
        }
        video.send_eof()?;
        Ok(true)
    }
}