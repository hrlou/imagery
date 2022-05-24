use image::DynamicImage;

pub use crate::driver::prelude::*;

// type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn thumbnail<I: Driver>(media: &mut I) -> Option<DynamicImage> {
    // thumb.as_dy
    let thumb = media.frame(1).unwrap();
    let thumb = thumb.resize(200, 200, image::imageops::FilterType::Lanczos3);
    Some(thumb)
}