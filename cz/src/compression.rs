use std::{collections::BTreeMap, io::{Cursor, Read, Seek, Write}};
use byteorder::{LittleEndian, ReadBytesExt};
use bitstream_io::{read::BitReader, BitRead};

use crate::common::CzError;
use crate::binio::BitIO;

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
        let compressed_size = bytes.read_u32::<LittleEndian>()?;
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

/// Decompress an LZW compressed stream like CZ1
pub fn decompress<T: Seek + ReadBytesExt + Read>(
    input: &mut T,
    chunk_info: &CompressionInfo,
) -> Result<Vec<u8>, CzError> {
    let mut m_dst = 0;
    let mut bitmap = vec![0; chunk_info.total_size_raw];
    for chunk in &chunk_info.chunks {
        let mut part = vec![0u8; chunk.size_compressed * 2];
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

/// Decompress an LZW compressed stream like CZ2
pub fn decompress_2<T: Seek + ReadBytesExt + Read>(
    input: &mut T,
    chunk_info: &CompressionInfo,
) -> Result<Vec<u8>, CzError> {
    let mut output_buf: Vec<u8> = vec![];

    for block in &chunk_info.chunks {
        let mut buffer = vec![0u8; block.size_compressed];
        input.read_exact(&mut buffer).unwrap();

        let raw_buf = decompress_lzw2(&buffer, block.size_raw);

        output_buf.write_all(&raw_buf).unwrap();
    }

    Ok(output_buf)
}

pub fn decompress_lzw2(input_data: &[u8], size: usize) -> Vec<u8> {
    let mut data = input_data.to_vec();
    data[0..2].copy_from_slice(&[0, 0]);
    let mut dictionary = BTreeMap::new();
    for i in 0..256 {
        dictionary.insert(i as u64, vec![i as u8]);
    }
    let mut dictionary_count = dictionary.len() as u64;
    let mut result = Vec::with_capacity(size);

    let data_size = input_data.len();
    data.extend_from_slice(&[0, 0]);
    let mut bit_io = BitIO::new(data);
    let mut w = dictionary.get(&0).unwrap().clone();

    let mut element;
    loop {
        let flag = bit_io.read_bit(1);
        if flag == 0 {
            element = bit_io.read_bit(15);
        } else {
            element = bit_io.read_bit(18);
        }

        if bit_io.byte_offset() > data_size {
            break
        }

        let mut entry;
        if let Some(x) = dictionary.get(&element) {
            // If the element was already in the dict, get it
            entry = x.clone()
        } else if element == dictionary_count {
            entry = w.clone();
            entry.push(w[0])
        } else {
            panic!("Bad compressed element: {}", element)
        }

        //println!("{}", element);

        result.write(&entry).unwrap();
        w.push(entry[0]);
        dictionary.insert(dictionary_count, w.clone());
        dictionary_count += 1;
        w = entry.clone();
    }
    result
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
