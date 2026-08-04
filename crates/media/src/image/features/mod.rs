#![allow(clippy::needless_range_loop)]

use image::{GenericImageView, Pixel, imageops::FilterType};

use crate::image::im::Image;

pub mod dct;
pub mod hash;

/// Image feature extractor
pub struct ImageFeatures<'a> {
    pub(crate) img: &'a Image,
}

impl<'a> ImageFeatures<'a> {
    pub fn p_hash(&self) -> i64 {
        let resized = self.img.inner.resize_exact(32, 32, FilterType::Triangle);
        let luma = resized.to_luma8();
        drop(resized);
        let matrix: [[u8; 32]; 32] = luma
            .rows()
            .map(|i| {
                i.map(|p| p.0[0])
                    .collect::<Vec<_>>()
                    .try_into()
                    .expect("Prepared image must be 32 pixels heigh")
            })
            .collect::<Vec<_>>()
            .try_into()
            .expect("Prepared image must be 32 pixels width");
        drop(luma);
        hash::p_hash(matrix)
    }

    pub fn a_hash(&self) -> i64 {
        let resized = self.img.inner.resize_exact(8, 8, FilterType::Triangle);
        let luma = resized.to_luma8();
        drop(resized);
        let pixels = luma
            .pixels()
            .map(|p| p.0[0])
            .collect::<Vec<_>>()
            .try_into()
            .expect("After preparation there should be 64 pixels");
        drop(luma);
        hash::a_hash(pixels)
    }

    pub fn avg_color(&self) -> (u8, u8, u8) {
        let resized = self.img.inner.resize_exact(1, 1, FilterType::Nearest);
        let [r, g, b] = resized.get_pixel(0, 0).to_rgb().0;
        (r, g, b)
    }
}
