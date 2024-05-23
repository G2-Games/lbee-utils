use byteorder::{ReadBytesExt, WriteBytesExt};
use image::RgbaImage;
use std::io::{Read, Seek, SeekFrom, Write};

use crate::common::{CommonHeader, CzError};
use crate::compression::{compress, decompress, get_chunk_info};

pub fn decode<T: Seek + ReadBytesExt + Read>(
    bytes: &mut T,
    header: &CommonHeader,
) -> Result<Vec<u8>, CzError> {
    let block_info = get_chunk_info(bytes)?;
    bytes.seek(SeekFrom::Start(block_info.length as u64))?;

    let data = decompress(bytes, &block_info)?;


    let mut picture = image::RgbaImage::new(header.width() as u32, header.height() as u32);

    line_diff(&mut picture, &data);

    Ok(picture.into_raw())
}

pub fn encode<T: WriteBytesExt + Write>(
    output: &mut T,
    bitmap: &[u8],
    header: &CommonHeader,
) -> Result<(), CzError> {
    let bitmap = diff_line(header, bitmap);

    let (compressed_data, compressed_info) = compress(&bitmap, 0xFEFD);

    compressed_info.write_into(output)?;

    output.write_all(&compressed_data)?;

    Ok(())
}

fn line_diff(picture: &mut RgbaImage, data: &[u8]) {
    let width = picture.width();
    let height = picture.height();
    let block_height = (f32::ceil(height as f32 / 3.0) as u16) as u32;

    let mut curr_line;
    let mut prev_line = Vec::with_capacity(width as usize * 3);

    let mut curr_alpha;
    let mut prev_alpha = Vec::with_capacity(width as usize);

    let pcount = (width * height * 3) as usize;

    let mut i = 0;
    let mut z = 0;
    for y in 0..height {
        curr_line = data[i..i + width as usize * 3].to_vec();
        curr_alpha = data[pcount + z..pcount + z + width as usize].to_vec();

        if y % block_height != 0 {
            for x in 0..(width as usize * 3) {
                curr_line[x] = curr_line[x].wrapping_add(prev_line[x])
            }
            for x in 0..width as usize {
                curr_alpha[x] = curr_alpha[x].wrapping_add(prev_alpha[x])
            }
        }

        for x in 0..width as usize {
            picture.get_pixel_mut(x as u32, y).0 = [
                curr_line[x * 3],
                curr_line[x * 3 + 1],
                curr_line[x * 3 + 2],
                curr_alpha[x],
            ];
        }

        prev_line.clone_from(&curr_line);
        prev_alpha.clone_from(&curr_alpha);
        i += width as usize * 3;
        z += width as usize;
    }
}

fn diff_line(header: &CommonHeader, input: &[u8]) -> Vec<u8> {
    let width = header.width() as u32;
    let height = header.height() as u32;

    let mut data = Vec::with_capacity(width as usize * 3);
    let mut alpha_data = Vec::with_capacity(width as usize);

    let block_height = (f32::ceil(height as f32 / 3.0) as u16) as usize;
    let pixel_byte_count = header.depth() >> 3;
    let line_byte_count = (width * pixel_byte_count as u32) as usize;

    let mut curr_line: Vec<u8>;
    let mut prev_line: Vec<u8> = Vec::with_capacity(width as usize * 3);

    let mut curr_alpha: Vec<u8>;
    let mut prev_alpha: Vec<u8> = Vec::with_capacity(width as usize);

    let mut i = 0;
    for y in 0..height {
        curr_line = input[i..i + line_byte_count].windows(4).step_by(4).flat_map(|r| &r[0..3]).copied().collect();
        curr_alpha = input[i..i + line_byte_count].iter().skip(3).step_by(4).copied().collect();

        if y % block_height as u32 != 0 {
            for x in 0..width as usize * 3 {
                curr_line[x] = curr_line[x].wrapping_sub(prev_line[x]);
                prev_line[x] = prev_line[x].wrapping_add(curr_line[x]);
            }
            for x in 0..width as usize {
                curr_alpha[x] = curr_alpha[x].wrapping_sub(prev_alpha[x]);
                prev_alpha[x] = prev_alpha[x].wrapping_add(curr_alpha[x]);
            }
        } else {
            prev_line.clone_from(&curr_line);
            prev_alpha.clone_from(&curr_alpha);
        }

        data.extend_from_slice(&curr_line);
        alpha_data.extend_from_slice(&curr_alpha);
        i += line_byte_count;
    }

    data.extend_from_slice(&alpha_data);

    data
}
