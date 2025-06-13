use std::collections::HashMap;
use bevy::prelude::*;

/// Bit-mask of opaque (alpha > threshold) pixels for an image.
/// A simple row-major flat vector where `x + y * width` indexes the pixel.
#[derive(Clone)]
pub struct AlphaMask {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<bool>, // true = opaque
}

impl AlphaMask {
    /// Test whether the pixel at `(x, y)` is opaque.
    #[inline]
    pub fn is_opaque(&self, x: u32, y: u32) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        self.pixels[(x + y * self.width) as usize]
    }

    /// Build an alpha mask from a loaded Bevy `Image`.
    pub fn from_image(img: &Image) -> Self {
        let width = img.texture_descriptor.size.width;
        let height = img.texture_descriptor.size.height;
        let mut pixels = Vec::with_capacity((width * height) as usize);

        let Some(bytes) = img.data.as_ref() else {
            // Image not yet loaded – treat as fully opaque placeholder.
            pixels.resize((width * height) as usize, true);
            return Self { width, height, pixels };
        };

        // Image data is stored in RGBA8 sRGB format by default.
        // Every 4 bytes correspond to RGBA channels.
        for chunk in bytes.chunks_exact(4) {
            // Alpha channel is at index 3
            let alpha = chunk[3];
            pixels.push(alpha > 127); // treat >50% opacity as solid
        }

        Self {
            width,
            height,
            pixels,
        }
    }
}

/// Global cache of alpha masks for all loaded textures we care about.
#[derive(Resource, Default)]
pub struct AlphaMasks(pub HashMap<Handle<Image>, AlphaMask>); 