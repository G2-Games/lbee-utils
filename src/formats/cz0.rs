use std::io::{self, Cursor, Read};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::cz_common::{CommonHeader, CzError, CzHeader, CzImage};

#[derive(Debug)]
pub struct Cz0Header {
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

#[derive(Debug)]
pub struct Cz0Image {
    header: Cz0Header,
    bitmap: Vec<u8>,
}

impl CzHeader for Cz0Header {
    fn new(bytes: &mut Cursor<&[u8]>) -> Result<Self, CzError>
    where
        Self: Sized,
    {
        let common = CommonHeader::new(bytes)?;

        if common.version() != 0 {
            return Err(CzError::VersionMismatch);
        }

        let _unknown = bytes.read_u32::<LittleEndian>()?;
        let _unknown = bytes.read_u8()?;

        let crop_width = bytes.read_u16::<LittleEndian>()?;
        let crop_height = bytes.read_u16::<LittleEndian>()?;

        let bounds_width = bytes.read_u16::<LittleEndian>()?;
        let bounds_height = bytes.read_u16::<LittleEndian>()?;

        let mut offset_width = None;
        let mut offset_height = None;
        if common.header_length() > 28 {
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

    fn header_length(&self) -> usize {
        self.common.header_length()
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
}

impl CzImage for Cz0Image {
    type Header = Cz0Header;

    fn decode(bytes: &[u8]) -> Result<Self, CzError> {
        let mut input = Cursor::new(bytes);

        // Get the header from the input
        let header = Cz0Header::new(&mut input)?;

        // Get the rest of the file, which is the bitmap
        let mut bitmap = vec![0u8; bytes.len() - header.header_length()];
        input.read_exact(&mut bitmap)?;

        Ok(Self { header, bitmap })
    }

    fn save_as_png(&self, name: &str) -> Result<(), image::error::ImageError> {
        Ok(image::save_buffer(
            name,
            &self.bitmap,
            self.header.width() as u32,
            self.header.height() as u32,
            image::ExtendedColorType::Rgba8,
        )?)
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

    fn save_as_cz(&self) -> Result<(), CzError> {
        todo!()
    }

    fn set_bitmap(&mut self, bitmap: Vec<u8>, header: Self::Header) {
        self.bitmap = bitmap;

        self.header = header;
    }
}

impl TryFrom<&[u8]> for Cz0Image {
    type Error = CzError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut input = Cursor::new(value);

        // Get the header from the input
        let header = Cz0Header::new(&mut input)?;

        // Get the rest of the file, which is the bitmap
        let mut bitmap = vec![];
        input.read_to_end(&mut bitmap)?;

        Ok(Self { header, bitmap })
    }
}
