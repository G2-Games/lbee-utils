use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Seek, SeekFrom, Write};

use crate::common::{CzError, CommonHeader};
use crate::compression::{compress, decompress, get_chunk_info};

pub fn decode<T: Seek + ReadBytesExt + Read>(
    bytes: &mut T,
    header: &CommonHeader,
) -> Result<Vec<u8>, CzError> {
    let block_info = get_chunk_info(bytes)?;
    bytes.seek(SeekFrom::Start(block_info.length as u64))?;

    let data = decompress(bytes, &block_info)?;

    let bitmap = line_diff(header, &data);

    Ok(bitmap)
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

fn line_diff(header: &CommonHeader, data: &[u8]) -> Vec<u8> {
    let width = header.width() as u32;
    let height = header.height() as u32;

    let mut output_buf = Vec::with_capacity((width * height * 4) as usize);

    let block_height = (f32::ceil(height as f32 / 3.0) as u16) as u32;

    let mut curr_line;
    let mut prev_line = Vec::with_capacity(width as usize * 3);

    let mut curr_alpha;
    let mut prev_alpha = Vec::with_capacity(width as usize);

    let mut rgb_index = 0;
    let mut alpha_index = (width * height * 3) as usize;
    for y in 0..height {
        curr_line = data[rgb_index..rgb_index + width as usize * 3].to_vec();
        curr_alpha = data[alpha_index..alpha_index + width as usize].to_vec();

        if y % block_height != 0 {
            curr_line.iter_mut().zip(&prev_line).for_each(|(curr_p, prev_p)| {
                *curr_p = curr_p.wrapping_add(*prev_p);
            });
            curr_alpha.iter_mut().zip(&prev_alpha).for_each(|(curr_a, prev_a)| {
                *curr_a = curr_a.wrapping_add(*prev_a);
            });
        }

        // Write the decoded RGBA data to the final buffer
        curr_line
            .windows(3)
            .step_by(3)
            .zip(&curr_alpha)
            .for_each(|(curr_p, alpha_p)| {
                output_buf.extend_from_slice(&[
                    curr_p[0],
                    curr_p[1],
                    curr_p[2],
                    *alpha_p,
                ]);
            });

        prev_line.clone_from(&curr_line);
        prev_alpha.clone_from(&curr_alpha);

        rgb_index += width as usize * 3;
        alpha_index += width as usize;
    }

    output_buf
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
