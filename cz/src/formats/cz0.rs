use std::{
    fs::File,
    io::{self, BufWriter, Cursor, Read, Seek, Write},
    path::PathBuf,
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::common::{CommonHeader, CzError, CzHeader, CzImage};

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

#[derive(Debug)]
pub struct Cz0Image {
    header: Cz0Header,
    bitmap: Vec<u8>,
}

impl CzHeader for Cz0Header {
    fn new<T: Seek + ReadBytesExt + Read>(bytes: &mut T) -> Result<Self, CzError>
    where
        Self: Sized,
    {
        let common = CommonHeader::new(bytes)?;

        if common.version() != 0 {
            return Err(CzError::VersionMismatch(common.version(), 0));
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

    fn common(&self) -> &CommonHeader {
        &self.common
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
        let mut buf = vec![];

        buf.write_all(&self.common.to_bytes()?)?;
        buf.write_all(&self.unknown_1)?;
        buf.write_u16::<LittleEndian>(self.crop_width)?;
        buf.write_u16::<LittleEndian>(self.crop_height)?;
        buf.write_u16::<LittleEndian>(self.bounds_width)?;
        buf.write_u16::<LittleEndian>(self.bounds_height)?;

        if self.length() > 28 {
            buf.write_u16::<LittleEndian>(self.offset_width.unwrap())?;
            buf.write_u16::<LittleEndian>(self.offset_height.unwrap())?;
            buf.write_all(&self.unknown_2.unwrap())?;
        }

        Ok(buf)
    }
}

impl CzImage for Cz0Image {
    type Header = Cz0Header;

    fn decode<T: Seek + ReadBytesExt + Read>(bytes: &mut T) -> Result<Self, CzError> {
        // Get the header from the input
        let header = Cz0Header::new(bytes)?;
        bytes.seek(io::SeekFrom::Start(header.length() as u64))?;

        // Get the rest of the file, which is the bitmap
        let mut bitmap = vec![];
        bytes.read_to_end(&mut bitmap)?;

        let bpp = (header.depth() >> 3) as usize;

        if bitmap.len() != (header.width() as usize * header.height() as usize) * bpp {
            return Err(CzError::Corrupt)
        }

        Ok(Self { header, bitmap })
    }

    fn save_as_cz<T: Into<PathBuf>>(&self, path: T) -> Result<(), CzError> {
        let mut output_file = BufWriter::new(File::create(path.into())?);

        output_file.write_all(&self.header().to_bytes()?)?;
        output_file.write_all(&self.bitmap)?;
        output_file.flush()?;

        Ok(())
    }

    fn header(&self) -> &Self::Header {
        &self.header
    }

    fn set_header(&mut self, header: &Self::Header) {
        self.header = *header
    }

    fn bitmap(&self) -> &Vec<u8> {
        &self.bitmap
    }

    fn into_bitmap(self) -> Vec<u8> {
        self.bitmap
    }

    fn set_bitmap(&mut self, bitmap: &[u8], width: u16, height: u16) {
        self.bitmap = bitmap.to_vec();

        self.header.common.width = width;
        self.header.common.height = height;
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
