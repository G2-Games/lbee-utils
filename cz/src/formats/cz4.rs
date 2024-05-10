use byteorder::ReadBytesExt;
use image::RgbaImage;
use std::io::{Read, Seek, SeekFrom};

use crate::common::{CommonHeader, CzError, CzHeader};
use crate::compression::{decompress, get_chunk_info};

pub fn decode<T: Seek + ReadBytesExt + Read>(
    bytes: &mut T,
    header: &CommonHeader,
) -> Result<Vec<u8>, CzError> {
    let block_info = get_chunk_info(bytes)?;
    bytes.seek(SeekFrom::Start(block_info.length as u64))?;

    let pcount = (header.width() as usize * header.height() as usize) * 3;
    let data = decompress(bytes, &block_info)?;
    let data2 = data[pcount..].to_vec();

    let mut picture = image::RgbaImage::new(header.width() as u32, header.height() as u32);

    let pixel_byte_count = 3;
    line_diff_cz4(&mut picture, pixel_byte_count, &data);

    let pixel_byte_count = 1;
    line_diff_cz4(&mut picture, pixel_byte_count, &data2);

    Ok(picture.into_raw())
}

pub fn line_diff_cz4(picture: &mut RgbaImage, pixel_byte_count: usize, data: &[u8]) {
    let width = picture.width();
    let height = picture.height();
    let block_height = (f32::ceil(height as f32 / 3.0) as u16) as u32;

    let mut curr_line;
    let mut prev_line = vec![0u8; width as usize * pixel_byte_count];

    let mut i = 0;
    for y in 0..height {
        curr_line = data[i..i + width as usize * pixel_byte_count].to_vec();

        if y % block_height != 0 {
            for x in 0..(width as usize * pixel_byte_count) {
                curr_line[x] = u8::wrapping_add(curr_line[x], prev_line[x])
            }
        }

        for x in 0..width as usize {
            if pixel_byte_count == 1 {
                picture.get_pixel_mut(x as u32, y).0[3] = curr_line[x];
            } else if pixel_byte_count == 4 {
                picture.get_pixel_mut(x as u32, y).0 = [
                    curr_line[x * pixel_byte_count],
                    curr_line[x * pixel_byte_count + 1],
                    curr_line[x * pixel_byte_count + 2],
                    curr_line[x * pixel_byte_count + 3],
                ];
            } else if pixel_byte_count == 3 {
                picture.get_pixel_mut(x as u32, y).0 = [
                    curr_line[x * pixel_byte_count],
                    curr_line[x * pixel_byte_count + 1],
                    curr_line[x * pixel_byte_count + 2],
                    0xFF,
                ];
            }
        }

        prev_line.clone_from(&curr_line);
        i += width as usize * pixel_byte_count;
    }
}
