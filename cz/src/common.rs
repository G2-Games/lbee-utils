//! Shared types and traits between CZ# files

use std::{
    io::{self, Read, Seek, Write}, path::PathBuf
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use image::Rgba;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CzError {
    #[error("Version in header does not match expected version")]
    VersionMismatch(u8, u8),

    #[error("Format of supplied file is not a CZ#")]
    InvalidFormat,

    #[error("Failed to read/write input/output")]
    IoError(#[from] io::Error),
}

pub trait CzHeader {
    fn new<T: Seek + ReadBytesExt + Read>(bytes: &mut T) -> Result<Self, CzError>
    where
        Self: Sized;

    /// Turn the header into bytes equivalent to the original header from the file
    fn to_bytes(&self) -> Result<Vec<u8>, io::Error>;

    /// The version of the [CzImage] file
    fn version(&self) -> u8;

    /// The length of the header in bytes
    fn length(&self) -> usize;

    /// The width of the image
    fn width(&self) -> u16;

    /// The height of the image
    fn height(&self) -> u16;

    /// The bit depth of the image (BPP)
    fn depth(&self) -> u16;

    /// An unknown value?
    fn color_block(&self) -> u8;
}

/// The common first part of a header of a CZ# file
#[derive(Debug, Clone, Copy)]
pub struct CommonHeader {
    /// Format version from the magic bytes, (eg. CZ3, CZ4)
    pub version: u8,

    /// Length of the header in bytes
    pub length: u32,

    /// Width of the image in pixels
    pub width: u16,

    /// Height of the image in pixels
    pub height: u16,

    /// Bit depth in Bits Per Pixel (BPP)
    pub depth: u16,

    /// Color block
    pub color_block: u8,
}

impl CzHeader for CommonHeader {
    fn new<T: Seek + ReadBytesExt + Read>(bytes: &mut T) -> Result<Self, CzError>
    where
        Self: Sized,
    {
        let mut magic = [0u8; 4];
        bytes.read_exact(&mut magic)?;

        if magic[0..2] != [b'C', b'Z'] {
            return Err(CzError::InvalidFormat)
        }

        Ok(Self {
            version: magic[2] - b'0',
            length: bytes.read_u32::<LittleEndian>()?,
            width: bytes.read_u16::<LittleEndian>()?,
            height: bytes.read_u16::<LittleEndian>()?,
            depth: bytes.read_u16::<LittleEndian>()?,
            color_block: bytes.read_u8()?,
        })
    }

    fn version(&self) -> u8 {
        self.version
    }

    fn length(&self) -> usize {
        self.length as usize
    }

    fn width(&self) -> u16 {
        self.width
    }

    fn height(&self) -> u16 {
        self.height
    }

    fn depth(&self) -> u16 {
        self.depth
    }

    fn color_block(&self) -> u8 {
        self.color_block
    }

    fn to_bytes(&self) -> Result<Vec<u8>, io::Error> {
        let mut buf = vec![];

        let magic_bytes = [b'C', b'Z', b'0' + self.version, 0];
        buf.write_all(&magic_bytes)?;
        buf.write_u32::<LittleEndian>(self.length() as u32)?;
        buf.write_u16::<LittleEndian>(self.width())?;
        buf.write_u16::<LittleEndian>(self.height())?;
        buf.write_u16::<LittleEndian>(self.depth())?;
        buf.write_u8(self.color_block())?;

        Ok(buf)
    }
}

pub trait CzImage {
    type Header;

    /// Create a [crate::CzImage] from bytes
    fn decode<T: Seek + ReadBytesExt + Read>(bytes: &mut T) -> Result<Self, CzError>
    where
        Self: Sized;

    /// Save the image as a PNG
    fn save_as_png(&self, name: &str) -> Result<(), image::error::ImageError>;

    /// Save the image as its corresponding CZ# type
    fn save_as_cz<T: Into<PathBuf>>(&self, path: T) -> Result<(), CzError>;

    /// Get the header for metadata
    fn header(&self) -> &Self::Header;

    /// Set the header with its metadata
    fn set_header(&mut self, header: Self::Header);

    /// Get the raw underlying bitmap for an image
    fn into_bitmap(self) -> Vec<u8>;

    /// Set the bitmap the image contains
    fn set_bitmap(&mut self, bitmap: &[u8], header: &Self::Header);
}

pub fn parse_colormap<T: Seek + ReadBytesExt + Read>(
    input: &mut T,
    num_colors: usize,
) -> Result<Vec<Rgba<u8>>, CzError> {
    let mut colormap = Vec::with_capacity(num_colors);
    let mut rgba_buf = [0u8; 4];

    for _ in 0..num_colors {
        input.read_exact(&mut rgba_buf)?;
        colormap.push(Rgba(rgba_buf));
    }

    Ok(colormap)
}

pub fn apply_palette(input: &mut Vec<u8>, palette: &[Rgba<u8>]) -> Vec<u8> {
    let mut output_map = Vec::new();

    for byte in input.iter() {
        let color = palette[*byte as usize].0;
        output_map.extend_from_slice(&color);
    }

    output_map
}
