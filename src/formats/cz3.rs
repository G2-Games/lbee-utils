use std::io::Cursor;

use image::ImageFormat;

use crate::cz_common::{decompress, parse_chunk_info, CommonHeader, CzError, CzHeader, CzImage};

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
    fn new(bytes: &[u8]) -> Result<Self, CzError> where Self: Sized {
        let mut input = Cursor::new(bytes);
        let common = CommonHeader::new(&mut input)?;

        if common.version != 3 {
            return Err(CzError::VersionMismatch)
        }

        let mut offset_width = None;
        let mut offset_height = None;
        if common.length > 28 {
            offset_width = Some(u16::from_le_bytes(bytes[28..30].try_into().unwrap()));
            offset_height = Some(u16::from_le_bytes(bytes[30..32].try_into().unwrap()));
        }

        Ok(Self {
            common,

            crop_width: u16::from_le_bytes(bytes[20..22].try_into().unwrap()),
            crop_height: u16::from_le_bytes(bytes[22..24].try_into().unwrap()),

            bounds_width: u16::from_le_bytes(bytes[24..26].try_into().unwrap()),
            bounds_height: u16::from_le_bytes(bytes[26..28].try_into().unwrap()),

            offset_width,
            offset_height,
        })
    }

    fn version(&self) -> u8 {
        self.common.version
    }

    fn header_length(&self) -> usize {
        self.common.length as usize
    }

    fn width(&self) -> u16 {
        self.common.width
    }

    fn height(&self) -> u16 {
        self.common.height
    }

    fn depth(&self) -> u16 {
        self.common.depth
    }
}

#[derive(Debug, Clone)]
pub struct Cz3Image {
    header: Cz3Header,
    bitmap: Vec<u8>,
}

impl CzImage for Cz3Image {
    type Header = Cz3Header;

    fn decode(bytes: &[u8]) -> Result<Self, CzError> {
        let mut input = Cursor::new(bytes);
        let header = Cz3Header::new(bytes)?;
        input.set_position(header.header_length() as u64);

        let block_info = parse_chunk_info(&mut input)?;

        let mut bitmap = decompress(&mut input, block_info)?;

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
