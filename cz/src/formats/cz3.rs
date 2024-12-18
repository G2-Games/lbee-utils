use std::io::{Read, Seek, SeekFrom, Write};

use crate::common::{CommonHeader, CzError};
use crate::compression::{compress, decompress, get_chunk_info};

pub fn decode<T: Seek + Read>(bytes: &mut T, header: &CommonHeader) -> Result<Vec<u8>, CzError> {
    let block_info = get_chunk_info(bytes)?;
    bytes.seek(SeekFrom::Start(block_info.length as u64))?;

    let data = decompress(bytes, &block_info)?;

    let bitmap = line_diff(header, &data);

    Ok(bitmap)
}

pub fn encode<T: Write>(
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

/// Function to extract the data from a CZ3 file after compression
///
/// Uses the previous line to determine the characterisitcs of the
/// following lines
fn line_diff(header: &CommonHeader, data: &[u8]) -> Vec<u8> {
    let width = header.width() as u32;
    let height = header.height() as u32;
    let mut output_buf = data.to_vec();

    let block_height = (f32::ceil(height as f32 / 3.0) as u16) as usize;
    let pixel_byte_count = header.depth() >> 3;
    let line_byte_count = (width * pixel_byte_count as u32) as usize;

    let mut curr_line;
    let mut prev_line = Vec::with_capacity(line_byte_count);

    let mut index = 0;
    for y in 0..height {
        curr_line = data[index..index + line_byte_count].to_vec();

        if y % block_height as u32 != 0 {
            curr_line
                .iter_mut()
                .zip(&prev_line)
                .for_each(|(curr_p, prev_p)| *curr_p = curr_p.wrapping_add(*prev_p));
        }

        prev_line.clone_from(&curr_line);
        if pixel_byte_count == 4 {
            output_buf[index..index + line_byte_count].copy_from_slice(&curr_line);
        } else if pixel_byte_count == 3 {
            for x in (0..line_byte_count).step_by(3) {
                let loc = (y * 3 * width) as usize + x;

                output_buf[loc..loc + 3].copy_from_slice(&[
                    curr_line[x],
                    curr_line[x + 1],
                    curr_line[x + 2],
                ])
            }
        } else if pixel_byte_count == 1 {
            for (x, rgba) in curr_line.iter().enumerate().take(line_byte_count) {
                let loc = (y * width) as usize + x;

                output_buf[loc] = *rgba;
            }
        }

        index += line_byte_count;
    }

    output_buf
}

/// Function to encode data into the CZ3 format before compression
///
/// Read more in [`line_diff`]
fn diff_line(header: &CommonHeader, input: &[u8]) -> Vec<u8> {
    let width = header.width() as u32;
    let height = header.height() as u32;

    let mut data = Vec::with_capacity(input.len());

    let block_height = (f32::ceil(height as f32 / 3.0) as u16) as usize;
    let pixel_byte_count = header.depth() >> 3;
    let line_byte_count = (width * pixel_byte_count as u32) as usize;

    let mut curr_line;
    let mut prev_line: Vec<u8> = Vec::with_capacity(line_byte_count);

    let mut i = 0;
    for y in 0..height {
        curr_line = input[i..i + line_byte_count].to_vec();
        if y % block_height as u32 != 0 {
            for x in 0..line_byte_count {
                curr_line[x] = curr_line[x].wrapping_sub(prev_line[x]);
                prev_line[x] = prev_line[x].wrapping_add(curr_line[x]);
            }
        } else {
            prev_line.clone_from(&curr_line);
        }

        data.extend_from_slice(&curr_line);
        i += line_byte_count;
    }

    data
}
