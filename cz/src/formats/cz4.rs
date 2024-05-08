use std::io::{Read, Seek, SeekFrom};
use byteorder::ReadBytesExt;

use crate::common::{CommonHeader, CzError, CzHeader};
use crate::compression::{decompress, line_diff_cz4, get_chunk_info};

pub fn decode<T: Seek + ReadBytesExt + Read>(bytes: &mut T, header: &CommonHeader) -> Result<Vec<u8>, CzError> {
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
