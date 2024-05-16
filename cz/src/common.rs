//! Shared types and traits between CZ# files

use std::{
    collections::HashMap,
    io::{self, Read, Seek, Write},
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use image::Rgba;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CzError {
    #[error("Expected CZ{}, got CZ{}", 0, 1)]
    VersionMismatch(u8, u8),

    #[error("Could not parse color index palette")]
    PaletteError,

    #[error("Bitmap size does not match image size")]
    BitmapFormat,

    #[error("File data is incorrect, it might be corrupt")]
    Corrupt,

    #[error("File is not a CZ image")]
    NotCzFile,

    #[error("Failed to read/write input/output")]
    IoError(#[from] io::Error),

    #[error("Problem while decoding file")]
    DecodeError,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CzVersion {
    CZ0,
    CZ1,
    CZ2,
    CZ3,
    CZ4,
    CZ5,
}

impl TryFrom<u8> for CzVersion {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let value = match value {
            0 => Self::CZ0,
            1 => Self::CZ1,
            2 => Self::CZ2,
            3 => Self::CZ3,
            4 => Self::CZ4,
            5 => Self::CZ5,
            _ => return Err("Value is not a valid CZ version"),
        };

        Ok(value)
    }
}

pub trait CzHeader {
    fn from_bytes<T: Seek + ReadBytesExt + Read>(bytes: &mut T) -> Result<Self, CzError>
    where
        Self: Sized;

    /// The [CommonHeader] header from the image
    fn common(&self) -> &CommonHeader;

    /// Turn the header into bytes equivalent to the original header from the file
    fn write_into<T: Seek + WriteBytesExt + Write>(
        &self,
        output: &mut T,
    ) -> Result<usize, io::Error>;

    /// The version of the image
    fn version(&self) -> CzVersion;

    /// Set the version of the image
    fn set_version(&mut self, version: CzVersion);

    /// The length of the header in bytes
    fn length(&self) -> usize;

    /// The width of the image
    fn width(&self) -> u16;

    /// Set the width of the image
    fn set_width(&mut self, width: u16);

    /// The height of the image
    fn height(&self) -> u16;

    /// Set the height of the image
    fn set_height(&mut self, height: u16);

    /// The bit depth of the image (BPP)
    fn depth(&self) -> u16;

    fn set_depth(&mut self) {
        unimplemented!()
    }

    /// An unknown value?
    fn color_block(&self) -> u8;
}

/// The common first part of a header of a CZ# file
#[derive(Debug, Clone, Copy)]
pub struct CommonHeader {
    /// Format version from the magic bytes, (eg. CZ3, CZ4)
    version: CzVersion,

    /// Length of the header in bytes
    length: u32,

    /// Width of the image in pixels
    width: u16,

    /// Height of the image in pixels
    height: u16,

    /// Bit depth in Bits Per Pixel (BPP)
    depth: u16,

    /// Color block? This byte's purpose is unclear
    unknown: u8,
}

impl CommonHeader {
    pub fn new(version: CzVersion, width: u16, height: u16) -> Self {
        Self {
            version,
            length: 15,
            width,
            height,
            depth: 32,
            unknown: 0,
        }
    }

    pub fn set_length(&mut self, length: u32) {
        self.length = length
    }
}

impl CzHeader for CommonHeader {
    fn from_bytes<T: Seek + ReadBytesExt + Read>(bytes: &mut T) -> Result<Self, CzError>
    where
        Self: Sized,
    {
        let mut magic = [0u8; 4];
        bytes.read_exact(&mut magic)?;

        if magic[0..2] != [b'C', b'Z'] {
            return Err(CzError::NotCzFile);
        }

        // Ensure the version matches a CZ file type
        let version = match CzVersion::try_from(magic[2] - b'0') {
            Ok(ver) => ver,
            Err(_) => return Err(CzError::NotCzFile),
        };

        let mut header = Self {
            version,
            length: bytes.read_u32::<LittleEndian>()?,
            width: bytes.read_u16::<LittleEndian>()?,
            height: bytes.read_u16::<LittleEndian>()?,
            depth: bytes.read_u16::<LittleEndian>()?,
            unknown: bytes.read_u8()?,
        };

        // Lock the color depth to 8 if it's over 32
        // This is obviously wrong, but why is it wrong?
        if header.depth() > 32 {
            header.depth = 8
        }

        Ok(header)
    }

    fn common(&self) -> &CommonHeader {
        self
    }

    fn version(&self) -> CzVersion {
        self.version
    }

    fn set_version(&mut self, version: CzVersion) {
        self.version = version
    }

    fn length(&self) -> usize {
        self.length as usize
    }

    fn width(&self) -> u16 {
        self.width
    }

    fn set_width(&mut self, width: u16) {
        self.width = width
    }

    fn height(&self) -> u16 {
        self.height
    }

    fn set_height(&mut self, height: u16) {
        self.height = height
    }

    fn depth(&self) -> u16 {
        self.depth
    }

    fn color_block(&self) -> u8 {
        self.unknown
    }

    fn write_into<T: Seek + WriteBytesExt + Write>(
        &self,
        output: &mut T,
    ) -> Result<usize, io::Error> {
        let pos = output.stream_position()?;
        let magic_bytes = [b'C', b'Z', b'0' + self.version as u8, b'\0'];

        output.write_all(&magic_bytes)?;
        output.write_u32::<LittleEndian>(self.length() as u32)?;
        output.write_u16::<LittleEndian>(self.width())?;
        output.write_u16::<LittleEndian>(self.height())?;
        output.write_u16::<LittleEndian>(self.depth())?;
        output.write_u8(self.color_block())?;

        Ok((output.stream_position()? - pos) as usize)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ExtendedHeader {
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

    unknown_2: Option<u32>,
}

impl ExtendedHeader {
    pub fn new() -> Self {
        ExtendedHeader {
            unknown_1: [0u8; 5],
            crop_width: 0,
            crop_height: 0,
            bounds_width: 0,
            bounds_height: 0,
            offset_width: None,
            offset_height: None,
            unknown_2: None,
        }
    }

    pub fn with_crop(mut self, crop_width: u16, crop_height: u16) -> Self {
        self.crop_width = crop_width;
        self.crop_height = crop_height;

        self
    }

    pub fn with_bounds(mut self, bounds_height: u16, bounds_width: u16) -> Self {
        self.bounds_width = bounds_width;
        self.bounds_height = bounds_height;

        self
    }

    pub fn with_offset(mut self, offset_width: u16, offset_height: u16) -> Self {
        self.offset_width = Some(offset_width);
        self.offset_height = Some(offset_height);
        self.unknown_2 = Some(0);

        self
    }

    pub fn from_bytes<T: Seek + ReadBytesExt + Read>(
        input: &mut T,
        common_header: &CommonHeader,
    ) -> Result<Self, CzError> {
        let mut unknown_1 = [0u8; 5];
        input.read_exact(&mut unknown_1)?;

        let crop_width = input.read_u16::<LittleEndian>()?;
        let crop_height = input.read_u16::<LittleEndian>()?;

        let bounds_width = input.read_u16::<LittleEndian>()?;
        let bounds_height = input.read_u16::<LittleEndian>()?;

        let mut offset_width = None;
        let mut offset_height = None;
        let mut unknown_2 = None;
        if common_header.length() > 28 {
            offset_width = Some(input.read_u16::<LittleEndian>()?);
            offset_height = Some(input.read_u16::<LittleEndian>()?);

            unknown_2 = Some(input.read_u32::<LittleEndian>()?);
        }

        Ok(Self {
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

    pub fn write_into<T: Seek + WriteBytesExt + Write>(
        &self,
        output: &mut T,
    ) -> Result<usize, io::Error> {
        let pos = output.stream_position()?;

        output.write_all(&self.unknown_1)?;
        output.write_u16::<LittleEndian>(self.crop_width)?;
        output.write_u16::<LittleEndian>(self.crop_height)?;
        output.write_u16::<LittleEndian>(self.bounds_width)?;
        output.write_u16::<LittleEndian>(self.bounds_height)?;

        if self.offset_width.is_some() {
            output.write_u16::<LittleEndian>(self.offset_width.unwrap())?;
            output.write_u16::<LittleEndian>(self.offset_height.unwrap())?;
            output.write_u32::<LittleEndian>(self.unknown_2.unwrap())?;
        }

        Ok((output.stream_position()? - pos) as usize)
    }
}

impl Default for ExtendedHeader {
    fn default() -> Self {
        Self::new()
    }
}

pub fn get_palette<T: Seek + ReadBytesExt + Read>(
    input: &mut T,
    num_colors: usize,
) -> Result<Vec<Rgba<u8>>, CzError> {
    let mut colormap = Vec::with_capacity(num_colors);
    let mut rgba_buf = [0u8; 4];

    for _ in 0..num_colors {
        input.read_exact(&mut rgba_buf)?;
        colormap.push(rgba_buf.into());
    }

    Ok(colormap)
}

pub fn apply_palette(
    input: &[u8],
    palette: &[Rgba<u8>]
) -> Result<Vec<u8>, CzError> {
    let mut output_map = Vec::new();

    for byte in input.iter() {
        let color = palette.get(*byte as usize);
        if let Some(color) = color {
            output_map.extend_from_slice(&color.0);
        } else {
            dbg!(byte);
            return Err(CzError::PaletteError);
        }
    }

    Ok(output_map)
}

pub fn rgba_to_indexed(
    input: &[u8],
    palette: &[Rgba<u8>]
) -> Result<Vec<u8>, CzError> {
    let mut output_map = Vec::new();
    let mut cache = HashMap::new();

    for rgba in input.windows(4).step_by(4) {
        let value = match cache.get(rgba) {
            Some(val) => *val,
            None => {
                let value = palette.iter().position(|e| e.0 == rgba).unwrap_or_default() as u8;
                cache.insert(rgba, value);
                value
            }
        };

        output_map.push(value)
    }

    Ok(output_map)
}

pub fn default_palette() -> Vec<Rgba<u8>> {
    let mut colormap = Vec::new();

    for i in 0..=0xFF {
        colormap.push(Rgba([0xFF, 0xFF, 0xFF, i]))
    }

    colormap
}
