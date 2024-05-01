//! Shared types and traits between CZ# files

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CzError {
    #[error("Version in header does not match expected version")]
    VersionMismatch
}

pub trait CzHeader {
    fn new(bytes: &[u8]) -> Result<Self, CzError> where Self: Sized;

    fn version(&self) -> u8;

    fn header_length(&self) -> usize;

    fn width(&self) -> u16;

    fn height(&self) -> u16;

    fn depth(&self) -> u8;
}

/// The common first part of a header of a CZ# file
#[derive(Debug)]
pub(crate) struct CommonHeader {
    /// Format version from the magic bytes, (eg. CZ3, CZ4)
    pub version: u8,

    /// Length of the header in bytes
    pub length: u8,

    /// Width of the image in pixels
    pub width: u16,

    /// Height of the image in pixels
    pub height: u16,

    /// Bit depth in Bits Per Pixel (BPP)
    pub depth: u8,
}

impl CommonHeader {
    pub fn new(bytes: &[u8]) -> Self {
        Self {
            version: bytes[2] - b'0',
            length: bytes[4],
            width: u16::from_le_bytes(bytes[8..10].try_into().unwrap()),
            height: u16::from_le_bytes(bytes[10..12].try_into().unwrap()),
            depth: bytes[12],
        }
    }
}

pub trait CzImage {
    type Header;

    /// Create a [CZImage] from bytes
    fn decode(bytes: &[u8]) -> Result<Self, CzError> where Self: Sized;

    /// Save the image as a PNG
    fn save_as_png(&self, name: &str);

    /// Get the header for metadata
    fn header(&self) -> &Self::Header;

    /// Get the raw underlying bitmap for an image
    fn raw_bitmap(&self) -> &Vec<u8>;
}
