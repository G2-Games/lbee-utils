//! Shared types and traits between CZ# files

use std::io::{self, Cursor, Read};

use byteorder::{LittleEndian, ReadBytesExt};
use image::Rgba;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CzError {
    #[error("Version in header does not match expected version")]
    VersionMismatch,

    #[error("Format of supplied file is incorrect; expected {} bytes, got {}", expected, got)]
    InvalidFormat{expected: usize, got: usize},

    #[error("Failed to read input")]
    ReadError(#[from] io::Error),
}

pub trait CzHeader {
    fn new(bytes: &[u8]) -> Result<Self, CzError> where Self: Sized;

    fn version(&self) -> u8;

    fn header_length(&self) -> usize;

    fn width(&self) -> u16;

    fn height(&self) -> u16;

    fn depth(&self) -> u16;
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
}

impl CommonHeader {
    pub fn new(bytes: &mut Cursor<&[u8]>) -> Result<Self, io::Error> {
        let mut magic = [0u8; 4];
        bytes.read_exact(&mut magic)?;

        Ok(Self {
            version: magic[2] - b'0',
            length: bytes.read_u32::<LittleEndian>()?,
            width: bytes.read_u16::<LittleEndian>()?,
            height: bytes.read_u16::<LittleEndian>()?,
            depth: bytes.read_u16::<LittleEndian>()?,
        })
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
    fn into_bitmap(self) -> Vec<u8>;
}

pub fn parse_colormap(input: &mut Cursor<&[u8]>, num_colors: usize) -> Result<Vec<Rgba<u8>>, CzError> {
    let mut colormap = Vec::with_capacity(num_colors);
    let mut rgba_buf = [0u8; 4];

    for _ in 0..num_colors {
        input.read_exact(&mut rgba_buf)?;
        colormap.push(Rgba(rgba_buf));
    }

    Ok(colormap)
}

pub struct ChunkInfo {
    pub size_compressed: usize,
    pub size_raw: usize,
}

pub struct CompressionInfo {
    pub chunk_count: usize,
    pub total_size_compressed: usize,
    pub total_size_raw: usize,
    pub chunks: Vec<ChunkInfo>,

    /// Length of the compression chunk info
    pub length: usize,
}

/// Get info about the compression chunks
pub fn parse_chunk_info(bytes: &mut Cursor<&[u8]>) -> Result<CompressionInfo, CzError> {
    let parts_count = bytes.read_u32::<LittleEndian>()?;

    dbg!(parts_count);
    let mut part_sizes = vec![];
    let mut total_size = 0;
    let mut total_size_raw = 0;

    for _ in 0..parts_count {
        let compressed_size = bytes.read_u32::<LittleEndian>()? * 2;
        total_size += compressed_size;

        let raw_size = bytes.read_u32::<LittleEndian>()? * 4;
        total_size_raw += raw_size;

        part_sizes.push(ChunkInfo {
            size_compressed: compressed_size as usize,
            size_raw: raw_size as usize,
        });
    }

    Ok(CompressionInfo {
        chunk_count: parts_count as usize,
        total_size_compressed: total_size as usize,
        total_size_raw: total_size_raw as usize,
        chunks: part_sizes,
        length: bytes.position() as usize,
    })
}


pub fn decompress(input: &mut Cursor<&[u8]>, chunk_info: CompressionInfo) -> Result<Vec<u8>, CzError> {
    let mut m_dst = 0;
    let mut bitmap = vec![0; chunk_info.total_size_raw];
    for chunk in chunk_info.chunks {
        let mut part = vec![0u8; chunk.size_compressed];
        input.read_exact(&mut part)?;

        for j in (0..part.len()).step_by(2) {
            let ctl = part[j + 1];

            if ctl == 0 {
                bitmap[m_dst] = part[j];
                m_dst += 1;
            } else {
                m_dst += copy_range(&mut bitmap, &part, get_offset(&part, j), m_dst);
            }
        }
    }

    Ok(bitmap)
}

fn get_offset(input: &[u8], src: usize) -> usize {
    (((input[src] as usize) | (input[src + 1] as usize) << 8) - 0x101) * 2
}

fn copy_range(bitmap: &mut Vec<u8>, input: &[u8], src: usize, dst: usize) -> usize {
    let mut dst = dst;
    let start_pos = dst;

    if input[src + 1] == 0 {
        bitmap[dst] = input[src];
        dst += 1;
    } else if get_offset(input, src) == src {
        bitmap[dst] = 0;
        dst += 1;
    } else {
        dst += copy_range(bitmap, input, get_offset(input, src), dst);
    }

    if input[src + 3] == 0 {
        bitmap[dst] = input[src + 2];
        dst += 1;
    } else if get_offset(input, src + 2) == src {
        bitmap[dst] = bitmap[start_pos];
        dst += 1;
    } else {
        bitmap[dst] = copy_one(input, get_offset(input, src + 2));
        dst += 1;
    }

    dst - start_pos
}

fn copy_one(input: &[u8], src: usize) -> u8 {
    if input[src + 1] == 0 {
        input[src]
    } else if get_offset(input, src) == src {
        0
    } else {
        copy_one(input, get_offset(input, src))
    }
}
