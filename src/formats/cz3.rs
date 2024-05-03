use image::ImageFormat;

use crate::cz_common::{CzError, CzHeader, CzImage};

use super::cz1::Cz1Header;

#[derive(Debug)]
pub struct Cz3Image {
    header: Cz1Header,
    bitmap: Vec<u8>,
}

impl CzImage for Cz3Image {
    type Header = Cz1Header;

    fn decode(bytes: &[u8]) -> Result<Self, CzError> {
        let cz1_image = crate::formats::cz1::Cz1Image::decode(bytes)?;

        let header = cz1_image.header().clone();
        let mut bitmap = cz1_image.into_bitmap();

        dbg!(bitmap.len());

        let stride = (header.width() * (header.depth() / 8)) as usize;
        let third = ((header.height() + 2) / 3) as usize;
        for y in 0..header.height() as usize {
            let dst = y * stride;
            if y % third != 0 {
                for x in 0..stride {
                    bitmap[dst + x] += bitmap[dst + x - stride];
                }
            }
        }

        dbg!(bitmap.len());

        Ok(Self {
            header,
            bitmap
        })
    }

    fn save_as_png(&self, name: &str) {
        let img = image::RgbaImage::from_raw(
            self.header.width() as u32,
            self.header.height() as u32,
            self.bitmap.clone()
        ).unwrap();

        img.save_with_format(name, ImageFormat::Png).unwrap();
    }

    fn header(&self) -> &Self::Header {
        &self.header
    }

    fn into_bitmap(self) -> Vec<u8> {
        self.bitmap
    }
}
