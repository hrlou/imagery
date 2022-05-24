pub mod prelude {
    pub use image::{
        ImageBuffer, Rgb, Luma,
        io::Reader as ImageReader,
    };
    // pub extern crate imageproc as proc;
    // pub use proc::{haar};
    pub extern crate dssim_core as dssim;
    pub use crate::driver::{self, *};
}
use std::{fmt::Debug, usize};

use image::DynamicImage;
use prelude::*;

pub type RgbBuffer = ImageBuffer<Rgb<u8>, Vec<u8>>;
pub type Process = dyn FnMut(&RgbBuffer, usize) -> bool;

pub trait Driver {
    type Error: Debug + std::error::Error;

    fn img(&self) -> Option<RgbBuffer> {
        None
    }

    fn frames<P: FnMut(&RgbBuffer, usize) -> bool>(&mut self, process: P) -> Result<bool, Self::Error> {
        Ok(false)
    }

    fn frame(&mut self, mut idx: usize) -> Option<DynamicImage> {
        let mut dynamic: RgbBuffer = RgbBuffer::default();
        // frames are indexed from one
        if idx == 0 {
            idx = 1;
        }
        match self.img() {
            Some(img) => {
                dynamic = img;
            },
            None => {
                match self.frames(|frame, frame_idx| {
                    if idx == frame_idx {
                        dynamic = frame.clone();
                        return false
                    }
                    // if the frame is out of bound then it should select the last one
                    dynamic = frame.clone();
                    true
                }) {
                    Err(_) => { return None; },
                    _ => {},
                }
            }
        };
        let dynamic = DynamicImage::from(dynamic);
        Some(dynamic)
    }
}

// pub trait AnimatedDriver: Driver {
    // fn frames<P: FnMut(&RgbBuffer, usize) -> bool>(&mut self, process: P) -> Result<(), Self::Error>;
// }

/// Video driver using ffmpeg
pub mod video;