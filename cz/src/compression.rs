use std::{collections::BTreeMap, io::{Read, Seek, Write}};
use byteorder::{LittleEndian, ReadBytesExt};
use image::{buffer, ColorType, DynamicImage, GenericImage, GenericImageView, RgbImage, Rgba, RgbaImage};

use crate::common::{CzError, CzHeader};
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

    bitmap.truncate(chunk_info.total_size_raw);

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

pub fn line_diff<T: CzHeader>(header_info: &T, data: &[u8]) -> Vec<u8> {
    let width = header_info.width() as u32;
    let height = header_info.height() as u32;
    let mut pic = image::RgbaImage::new(width, height);

    let block_height = (f32::ceil(height as f32 / 4 as f32) as u16) as usize;
    let pixel_byte_count = header_info.depth() >> 3;
    let line_byte_count = (width * pixel_byte_count as u32) as usize;

    let mut curr_line: Vec<u8>;
    let mut prev_line: Vec<u8> = Vec::with_capacity(line_byte_count);

    let mut i = 0;
    for y in 0..height {
        curr_line = data[i..i+line_byte_count].to_vec();
        dbg!(curr_line.len());

        if y % block_height as u32 != 0 {
            for x in 0..line_byte_count {
                curr_line[x] += prev_line[x]
            }
        }

        prev_line = curr_line.clone();
        if header_info.version() == 4 {
            for x in 0..width {
                pic.get_pixel_mut(x as u32, y).0[3] = curr_line[x as usize];
            }
        } else if pixel_byte_count == 4 {
            let mut raw = pic.into_raw();
            raw[i..i+line_byte_count].copy_from_slice(&curr_line);

            pic = RgbaImage::from_raw(width, height, raw).unwrap();
        } else if pixel_byte_count == 3 {
            for x in 0..line_byte_count {
                pic.get_pixel_mut((x/3) as u32, y).0 = [curr_line[x], curr_line[x + 1], curr_line[x + 2], 0xFF];
            }
        }

        i += line_byte_count;
    }

    pic.into_vec()
}

pub fn line_diff_cz4(picture: &mut RgbaImage, color_block: u8, pixel_byte_count: usize, data: &[u8]) {
    let width = picture.width() as u32;
    let height = picture.height() as u32;
    let block_height = (f32::ceil(height as f32 / color_block as f32) as u16) as u32;


    let mut curr_line = vec![0u8; width as usize * pixel_byte_count];
    let mut prev_line = vec![0u8; width as usize * pixel_byte_count];

    let mut i = 0;
    for y in 0..height {
        curr_line = data[i..i + width as usize * pixel_byte_count].to_vec();

        if y % block_height != 0 {
            for x in 0..(width as usize * pixel_byte_count) {
                curr_line[x] += prev_line[x]
            }
        }


        for x in 0..width as usize {
            if pixel_byte_count == 1 {
                picture.get_pixel_mut(x as u32, y).0[3] = curr_line[x];
            } else if pixel_byte_count == 4 {
                *picture.get_pixel_mut(x as u32, y) = Rgba([
                    curr_line[x * pixel_byte_count + 0],
                    curr_line[x * pixel_byte_count + 1],
                    curr_line[x * pixel_byte_count + 2],
                    curr_line[x * pixel_byte_count + 3]
                ]);
            } else if pixel_byte_count == 3 {
                *picture.get_pixel_mut(x as u32, y) = Rgba([
                    curr_line[x * pixel_byte_count + 0],
                    curr_line[x * pixel_byte_count + 1],
                    curr_line[x * pixel_byte_count + 2],
                    0xFF
                ]);
            }
        }

        prev_line = curr_line.clone();
        i += width as usize * pixel_byte_count;
    }
}
