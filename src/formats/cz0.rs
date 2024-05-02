use crate::cz_common::{CommonHeader, CzError, CzHeader, CzImage};

#[derive(Debug)]
pub struct Cz0Header {
    /// Common CZ# header
    common: CommonHeader,

    /// Width of cropped image area
    crop_width: u16,

    /// Height of cropped image area
    crop_height: u16,

    /// Bounding box width
    bounds_width: u16,

    /// Bounding box height
    bounds_height: u16,

    /// Offset width
    offset_width: Option<u16>,

    /// Offset height
    offset_height: Option<u16>,
}

#[derive(Debug)]
pub struct Cz0Image {
    header: Cz0Header,
    bitmap: Vec<u8>,
}

impl CzHeader for Cz0Header {
    fn new(bytes: &[u8]) -> Result<Self, CzError> {
        let common = CommonHeader::new(bytes);

        if common.version != 0 {
            return Err(CzError::VersionMismatch)
        }

        let mut offset_width = None;
        let mut offset_height = None;
        if common.length < 28 {
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

impl CzImage for Cz0Image {
    type Header = Cz0Header;

    fn decode(bytes: &[u8]) -> Result<Self, CzError> {
        // Get the header from the input
        let header = Cz0Header::new(bytes)?;

        // Get the rest of the file, which is the bitmap
        let bitmap = bytes[header.header_length()..].to_vec();

        Ok(Self {
            header,
            bitmap
        })
    }

    fn save_as_png(&self, name: &str) {
        image::save_buffer(
            name,
            &self.bitmap,
            self.header.common.width as u32,
            self.header.common.height as u32,
            image::ExtendedColorType::Rgba8
        ).unwrap()
    }

    fn header(&self) -> &Self::Header {
        &self.header
    }

    fn raw_bitmap(&self) -> &Vec<u8> {
        &self.bitmap
    }
}
