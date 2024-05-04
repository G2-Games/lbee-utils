use std::io::{Read, Seek};
use byteorder::{LittleEndian, ReadBytesExt};
use crate::common::CzError;

#[derive(Debug, Clone, Copy)]
pub struct ChunkInfo {
    pub size_compressed: usize,
    pub size_raw: usize,
}

#[derive(Debug, Clone)]
pub struct CompressionInfo {
    pub chunk_count: usize,
    pub total_size_compressed: usize,
    pub total_size_raw: usize,
    pub chunks: Vec<ChunkInfo>,

    /// Length of the compression chunk info
    pub length: usize,
}

/// Get info about the compression chunks
pub fn parse_chunk_info<T: Seek + ReadBytesExt + Read>(bytes: &mut T) -> Result<CompressionInfo, CzError> {
    let parts_count = bytes.read_u32::<LittleEndian>()?;

    let mut part_sizes = vec![];
    let mut total_size = 0;
    let mut total_size_raw = 0;

    // Loop over the compressed bytes
    for _ in 0..parts_count {
        let compressed_size = bytes.read_u32::<LittleEndian>()? * 2;
        total_size += compressed_size;

        let raw_size = bytes.read_u32::<LittleEndian>()?;
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
        length: bytes.stream_position()? as usize,
    })
}

/// Decompress an LZW compressed stream, like CZ1
pub fn decompress<T: Seek + ReadBytesExt + Read>(
    input: &mut T,
    chunk_info: &CompressionInfo,
) -> Result<Vec<u8>, CzError> {
    let mut m_dst = 0;
    let mut bitmap = vec![0; chunk_info.total_size_raw];
    for chunk in &chunk_info.chunks {
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
