use std::{io::{self, Cursor, Read, Seek, SeekFrom}, path::PathBuf};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::compression::{decompress, parse_chunk_info};
use crate::common::{CommonHeader, CzError, CzHeader, CzImage};

#[derive(Debug, Clone, Copy)]
pub struct Cz3Header {
    /// Common CZ# header
    common: CommonHeader,

    /// Width of cropped image area
    pub crop_width: u16,

    /// Height of cropped image area
    pub crop_height: u16,

    /// Bounding box width
    pub bounds_width: u16,

    /// Bounding box height
    pub bounds_height: u16,

    /// Offset width
    pub offset_width: Option<u16>,

    /// Offset height
    pub offset_height: Option<u16>,
}

impl CzHeader for Cz3Header {
    fn new<T: Seek + ReadBytesExt + Read>(bytes: &mut T) -> Result<Self, CzError>
    where
        Self: Sized,
    {
        let common = CommonHeader::new(bytes)?;

        if common.version() != 3 {
            return Err(CzError::VersionMismatch);
        }

        let mut unknown_1 = [0u8; 5];
        bytes.read_exact(&mut unknown_1)?;

        let crop_width = bytes.read_u16::<LittleEndian>()?;
        let crop_height = bytes.read_u16::<LittleEndian>()?;

        let bounds_width = bytes.read_u16::<LittleEndian>()?;
        let bounds_height = bytes.read_u16::<LittleEndian>()?;

        let mut offset_width = None;
        let mut offset_height = None;
        if common.length() > 28 {
            offset_width = Some(bytes.read_u16::<LittleEndian>()?);
            offset_height = Some(bytes.read_u16::<LittleEndian>()?);
        }

        Ok(Self {
            common,

            crop_width,
            crop_height,

            bounds_width,
            bounds_height,

            offset_width,
            offset_height,
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
pub struct Cz3Image {
    header: Cz3Header,
    bitmap: Vec<u8>,
}

impl CzImage for Cz3Image {
    type Header = Cz3Header;

    fn decode<T: Seek + ReadBytesExt + Read>(bytes: &mut T) -> Result<Self, CzError> {
        let header = Cz3Header::new(bytes)?;
        bytes.seek(SeekFrom::Start(header.length() as u64))?;

        let block_info = parse_chunk_info(bytes)?;

        let mut bitmap = decompress(bytes, &block_info)?;

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

        Ok(Self { header, bitmap })
    }

    fn save_as_png(&self, name: &str) -> Result<(), image::error::ImageError> {
        image::save_buffer(
            name,
            &self.bitmap,
            self.header.width() as u32,
            self.header.height() as u32,
            image::ExtendedColorType::Rgba8,
        )
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
