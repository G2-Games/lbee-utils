use std::{
    io::{self, Read, Seek, SeekFrom},
    path::PathBuf
};

use byteorder::{LittleEndian, ReadBytesExt};
use image::DynamicImage;

use crate::compression::{decompress, line_diff, line_diff_cz4, parse_chunk_info};
use crate::common::{CommonHeader, CzError, CzHeader, CzImage};

#[derive(Debug, Clone, Copy)]
pub struct Cz4Header {
    /// Common CZ# header
    common: CommonHeader,
}

impl CzHeader for Cz4Header {
    fn new<T: Seek + ReadBytesExt + Read>(bytes: &mut T) -> Result<Self, CzError>
    where
        Self: Sized,
    {
        let common = CommonHeader::new(bytes)?;

        if common.version() != 4 {
            return Err(CzError::VersionMismatch(common.version(), 3));
        }

        Ok(Self {
            common,
        })
    }

    fn version(&self) -> u8 {
        self.common.version()
    }

    fn length(&self) -> usize {
        self.common.length()
    }

    fn width(&self) -> u16 {
        self.common.width()
    }

    fn height(&self) -> u16 {
        self.common.height()
    }

    fn depth(&self) -> u16 {
        self.common.depth()
    }

    fn color_block(&self) -> u8 {
        self.common.color_block()
    }

    fn to_bytes(&self) -> Result<Vec<u8>, io::Error> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct Cz4Image {
    header: Cz4Header,
    bitmap: Vec<u8>,
}

impl CzImage for Cz4Image {
    type Header = Cz4Header;

    fn decode<T: Seek + ReadBytesExt + Read>(bytes: &mut T) -> Result<Self, CzError> {
        let header = Cz4Header::new(bytes)?;
        bytes.seek(SeekFrom::Start(header.length() as u64))?;

        let block_info = parse_chunk_info(bytes)?;
        bytes.seek(SeekFrom::Start(block_info.length as u64))?;

        let bitmap = decompress(bytes, &block_info)?;

        let mut picture = image::RgbaImage::new(header.width() as u32, header.height() as u32);

        let pixel_byte_count = 3;
        line_diff_cz4(&mut picture, 3, pixel_byte_count, &bitmap);

        Ok(Self {
            header,
            bitmap: picture.into_vec()
        })
    }

    fn save_as_png(&self, name: &str) -> Result<(), image::error::ImageError> {
        let img = image::RgbaImage::from_raw(
            self.header.width() as u32,
            self.header.height() as u32,
            self.bitmap.clone(),
        ).unwrap();

        img.save(name)?;

        Ok(())
    }

    fn header(&self) -> &Self::Header {
        &self.header
    }

    fn set_header(&mut self, header: Self::Header) {
        self.header = header
    }

    fn into_bitmap(self) -> Vec<u8> {
        self.bitmap
    }

    fn save_as_cz<T: Into<PathBuf>>(&self, path: T) -> Result<(), CzError> {
        todo!()
    }

    fn set_bitmap(&mut self, bitmap: &[u8], header: &Self::Header) {
        todo!()
    }
}
