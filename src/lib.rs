//! Simplify the creation of image variations such as thumbnails from varying media sources
//!
//! Provides abstractions over images and videos

#[cfg(test)]
mod tests;

pub mod convert;

/// Traits for different media
pub mod driver;

/// Metadata types for media
pub mod meta;


/*pub mod prelude {
    pub use crate::{
        imager::{self, prelude::*}, 
        video::{self, prelude::*}
    };
}*/
