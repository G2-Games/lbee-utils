use byteorder::ReadBytesExt;
use std::io::{Read, Seek, SeekFrom};

use crate::common::CzError;
use crate::compression::{decompress_2, get_chunk_info};

pub fn decode<T: Seek + ReadBytesExt + Read>(bytes: &mut T) -> Result<Vec<u8>, CzError> {
    let block_info = get_chunk_info(bytes)?;
    bytes.seek(SeekFrom::Start(block_info.length as u64))?;

    let bitmap = decompress_2(bytes, &block_info).unwrap();

    Ok(bitmap)
}
