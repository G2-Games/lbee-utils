use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::{
    collections::HashMap,
    io::{Read, Seek, Write},
};

use crate::binio::BitIo;
use crate::common::CzError;

/// The size of compressed data in each chunk
#[derive(Debug, Clone, Copy)]
pub struct ChunkInfo {
    /// The size of the data when compressed
    pub size_compressed: usize,

    /// The size of the original uncompressed data
    pub size_raw: usize,
}

/// A CZ# file's information about compression chunks
#[derive(Default, Debug, Clone)]
pub struct CompressionInfo {
    /// Number of compression chunks
    pub chunk_count: usize,

    /// Total size of the data when compressed
    pub total_size_compressed: usize,

    /// Total size of the original uncompressed data
    pub _total_size_raw: usize,

    /// The compression chunk information
    pub chunks: Vec<ChunkInfo>,

    /// Length of the compression chunk info
    pub length: usize,
}

impl CompressionInfo {
    pub fn write_into<T: WriteBytesExt + Write>(
        &self,
        output: &mut T,
    ) -> Result<(), std::io::Error> {
        output.write_u32::<LittleEndian>(self.chunk_count as u32)?;

        for chunk in &self.chunks {
            output.write_u32::<LittleEndian>(chunk.size_compressed as u32)?;
            output.write_u32::<LittleEndian>(chunk.size_raw as u32)?;
        }

        Ok(())
    }
}

/// Get info about the compression chunks
///
/// These are defined by a length value, followed by the number of data chunks
/// that length value says split into compressed and original size u32 values
pub fn get_chunk_info<T: Seek + ReadBytesExt + Read>(
    bytes: &mut T,
) -> Result<CompressionInfo, CzError> {
    let parts_count = bytes.read_u32::<LittleEndian>()?;

    let mut part_sizes = vec![];
    let mut total_size = 0;
    let mut total_size_raw = 0;

    // Loop over the compressed bytes
    for _ in 0..parts_count {
        let compressed_size = bytes.read_u32::<LittleEndian>()?;
        total_size = i32::wrapping_add(total_size, compressed_size as i32);

        let raw_size = bytes.read_u32::<LittleEndian>()?;
        total_size_raw = u32::wrapping_add(total_size_raw, raw_size);

        part_sizes.push(ChunkInfo {
            size_compressed: compressed_size as usize,
            size_raw: raw_size as usize,
        });
    }

    Ok(CompressionInfo {
        chunk_count: parts_count as usize,
        total_size_compressed: total_size as usize,
        _total_size_raw: total_size_raw as usize,
        chunks: part_sizes,
        length: bytes.stream_position()? as usize,
    })
}

/// Decompress an LZW compressed stream like CZ1
pub fn decompress<T: Seek + ReadBytesExt + Read>(
    input: &mut T,
    chunk_info: &CompressionInfo,
) -> Result<Vec<u8>, CzError> {
    let mut output_buf: Vec<u8> = vec![];

    for block in &chunk_info.chunks {
        let mut buffer = vec![0u16; block.size_compressed];

        for word in buffer.iter_mut() {
            *word = input.read_u16::<LittleEndian>().unwrap();
        }

        let raw_buf = decompress_lzw(&buffer, block.size_raw);

        output_buf.write_all(&raw_buf)?;
    }

    Ok(output_buf)
}

fn decompress_lzw(input_data: &[u16], size: usize) -> Vec<u8> {
    let mut dictionary: HashMap<u16, Vec<u8>> = HashMap::new();
    for i in 0..256 {
        dictionary.insert(i as u16, vec![i as u8]);
    }
    let mut dictionary_count = dictionary.len() as u16;

    let mut w = vec![0];
    let mut result = Vec::with_capacity(size);

    input_data.iter().for_each(|element| {
        let mut entry;
        if let Some(x) = dictionary.get(element) {
            entry = x.clone();
        } else if *element == dictionary_count {
            entry = w.clone();
            entry.push(w[0]);
        } else {
            panic!("Bad compressed element: {}", element)
        }

        result.write_all(&entry).unwrap();
        w.push(entry[0]);

        dictionary.insert(dictionary_count, w.clone());
        dictionary_count += 1;

        w = entry;
    });

    result
}

/// Decompress an LZW compressed stream like CZ2
pub fn decompress2<T: Seek + ReadBytesExt + Read>(
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

fn decompress_lzw2(input_data: &[u8], size: usize) -> Vec<u8> {
    let mut data = input_data.to_vec();
    data[0] = 0;
    let mut dictionary = HashMap::new();
    for i in 0..256 {
        dictionary.insert(i as u64, vec![i as u8]);
    }
    let mut dictionary_count = dictionary.len() as u64;
    let mut result = Vec::with_capacity(size);

    let data_size = input_data.len();
    data.extend_from_slice(&[0, 0]);
    let mut bit_io = BitIo::new(data);
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
            break;
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

        result.write_all(&entry).unwrap();
        w.push(entry[0]);
        dictionary.insert(dictionary_count, w.clone());
        dictionary_count += 1;
        w.clone_from(&entry);
    }
    result
}

pub fn compress(data: &[u8], size: usize) -> (Vec<u8>, CompressionInfo) {
    let mut size = size;
    if size == 0 {
        size = 0xFEFD
    }

    let mut part_data;

    let mut offset = 0;
    let mut count;
    let mut last = Vec::new();

    let mut output_buf: Vec<u8> = vec![];
    let mut output_info = CompressionInfo {
        _total_size_raw: data.len(),
        ..Default::default()
    };

    loop {
        (count, part_data, last) = compress_lzw(&data[offset..], size, last);
        if count == 0 {
            break;
        }
        offset += count;

        for d in &part_data {
            output_buf.write_all(&d.to_le_bytes()).unwrap();
        }

        output_info.chunks.push(ChunkInfo {
            size_compressed: part_data.len(),
            size_raw: count,
        });

        output_info.chunk_count += 1;
    }

    if output_info.chunk_count == 0 {
        panic!("No chunks compressed!")
    } else if output_info.chunk_count != 1 {
        output_info.chunks[0].size_raw -= 1;
        output_info.chunks[output_info.chunk_count - 1].size_raw += 1;
    }

    output_info.total_size_compressed = output_buf.len() / 2;

    (output_buf, output_info)
}

fn compress_lzw(data: &[u8], size: usize, last: Vec<u8>) -> (usize, Vec<u16>, Vec<u8>) {
    let mut count = 0;
    let mut dictionary = HashMap::new();
    for i in 0..=255 {
        dictionary.insert(vec![i], i as u16);
    }
    let mut dictionary_count = (dictionary.len() + 1) as u16;

    let mut element = Vec::new();
    if !last.is_empty() {
        element = last
    }

    let mut compressed = Vec::with_capacity(size);
    for c in data {
        let mut entry = element.clone();
        entry.push(*c);

        if dictionary.contains_key(&entry) {
            element = entry
        } else {
            compressed.push(*dictionary.get(&element).unwrap());
            dictionary.insert(entry, dictionary_count);
            element = vec![*c];
            dictionary_count += 1;
        }

        count += 1;

        if size > 0 && compressed.len() == size {
            break;
        }
    }

    let last_element = element;
    if compressed.is_empty() {
        if !last_element.is_empty() {
            for c in last_element {
                compressed.push(*dictionary.get(&vec![c]).unwrap());
            }
        }
        return (count, compressed, Vec::new());
    } else if compressed.len() < size {
        if !last_element.is_empty() {
            compressed.push(*dictionary.get(&last_element).unwrap());
        }
        return (count, compressed, Vec::new());
    }

    (count, compressed, last_element)
}

pub fn compress2(data: &[u8], size: usize) -> (Vec<u8>, CompressionInfo) {
    let size = if size == 0 { 0x87BDF } else { size };

    let mut part_data;

    let mut offset = 0;
    let mut count;
    let mut last: Vec<u8> = Vec::new();

    let mut output_buf: Vec<u8> = Vec::new();
    let mut output_info = CompressionInfo {
        _total_size_raw: data.len(),
        ..Default::default()
    };

    loop {
        (count, part_data, last) = compress_lzw2(&data[offset..], size, last);
        if count == 0 {
            break;
        }
        offset += count;

        output_buf.write_all(&part_data).unwrap();

        output_info.chunks.push(ChunkInfo {
            size_compressed: part_data.len(),
            size_raw: count,
        });

        output_info.chunk_count += 1;
    }

    if output_info.chunk_count == 0 {
        panic!("No chunks compressed!")
    } else if output_info.chunk_count != 1 {
        output_info.chunks[0].size_raw -= 1;
        output_info.chunks[output_info.chunk_count - 1].size_raw += 1;
    }

    output_info.total_size_compressed = output_buf.len();
    (output_buf, output_info)
}

fn compress_lzw2(data: &[u8], size: usize, last: Vec<u8>) -> (usize, Vec<u8>, Vec<u8>) {
    let mut data = data.to_vec();
    if !data.is_empty() {
        data[0] = 0;
    }
    let mut count = 0;
    let mut dictionary = HashMap::new();
    for i in 0..=255 {
        dictionary.insert(vec![i], i as u64);
    }
    let mut dictionary_count = (dictionary.len() + 1) as u64;

    let mut element = Vec::new();
    if last.is_empty() {
        element = last
    }

    let mut bit_io = BitIo::new(vec![0u8; size + 2]);
    let write_bit = |bit_io: &mut BitIo, code: u64| {
        if code > 0x7FFF {
            bit_io.write_bit(1, 1);
            bit_io.write_bit(code, 18);
        } else {
            bit_io.write_bit(0, 1);
            bit_io.write_bit(code, 15);
        }
    };

    for c in data.iter() {
        let mut entry = element.clone();
        entry.push(*c);

        if dictionary.contains_key(&entry) {
            element = entry
        } else {
            write_bit(&mut bit_io, *dictionary.get(&element).unwrap());
            dictionary.insert(entry, dictionary_count);
            element = vec![*c];
            dictionary_count += 1;
        }

        count += 1;

        if size > 0 && bit_io.byte_size() >= size {
            count -= 1;
            break;
        }
    }

    let last_element = element;
    if bit_io.byte_size() == 0 {
        if !last_element.is_empty() {
            for c in last_element {
                write_bit(&mut bit_io, *dictionary.get(&vec![c]).unwrap());
            }
        }
        return (count, bit_io.bytes(), Vec::new());
    } else if bit_io.byte_size() < size {
        if !last_element.is_empty() {
            write_bit(&mut bit_io, *dictionary.get(&last_element).unwrap());
        }
        return (count, bit_io.bytes(), Vec::new());
    }

    (count, bit_io.bytes(), last_element)
}
