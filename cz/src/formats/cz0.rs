use std::io::{self, Read, Seek, Write};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::common::{CommonHeader, CzError, CzHeader, CzVersion};

#[derive(Debug, Clone, Copy)]
pub struct Cz0Header {
    /// Common CZ# header
    pub common: CommonHeader,

    /// Unknown bytes
    unknown_1: [u8; 5],

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

    unknown_2: Option<[u8; 4]>,
}

impl Cz0Header {
    fn new<T: Seek + ReadBytesExt + Read>(bytes: &mut T) -> Result<Self, CzError>
    where
        Self: Sized,
    {
        let common = CommonHeader::new(bytes)?;

        if common.version() != CzVersion::CZ0 {
            return Err(CzError::VersionMismatch(common.version() as u8, 0));
        }

        let mut unknown_1 = [0u8; 5];
        bytes.read_exact(&mut unknown_1)?;

        let crop_width = bytes.read_u16::<LittleEndian>()?;
        let crop_height = bytes.read_u16::<LittleEndian>()?;

        let bounds_width = bytes.read_u16::<LittleEndian>()?;
        let bounds_height = bytes.read_u16::<LittleEndian>()?;

        let mut offset_width = None;
        let mut offset_height = None;
        let mut unknown_2 = None;
        if common.length() > 28 {
            offset_width = Some(bytes.read_u16::<LittleEndian>()?);
            offset_height = Some(bytes.read_u16::<LittleEndian>()?);

            let mut un_2 = [0u8; 4];
            bytes.read_exact(&mut un_2)?;

            unknown_2 = Some(un_2);
        }

        Ok(Self {
            common,

            unknown_1,

            crop_width,
            crop_height,

            bounds_width,
            bounds_height,

            offset_width,
            offset_height,

            unknown_2,
        })
    }

    fn to_bytes(&self) -> Result<Vec<u8>, io::Error> {
        let mut buf = vec![];

        buf.write_all(&self.common.to_bytes()?)?;
        buf.write_all(&self.unknown_1)?;
        buf.write_u16::<LittleEndian>(self.crop_width)?;
        buf.write_u16::<LittleEndian>(self.crop_height)?;
        buf.write_u16::<LittleEndian>(self.bounds_width)?;
        buf.write_u16::<LittleEndian>(self.bounds_height)?;

        if self.common.length() > 28 {
            buf.write_u16::<LittleEndian>(self.offset_width.unwrap())?;
            buf.write_u16::<LittleEndian>(self.offset_height.unwrap())?;
            buf.write_all(&self.unknown_2.unwrap())?;
        }

        Ok(buf)
    }
}

#[derive(Debug)]
pub struct Cz0Image {
    header: Cz0Header,
    bitmap: Vec<u8>,
}

pub fn decode<T: Seek + ReadBytesExt + Read>(bytes: &mut T) -> Result<Vec<u8>, CzError> {
    // Get the rest of the file, which is the bitmap
    let mut bitmap = vec![];
    bytes.read_to_end(&mut bitmap)?;

    Ok(bitmap)
}
