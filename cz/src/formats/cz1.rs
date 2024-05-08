use byteorder::ReadBytesExt;
use std::io::{Read, Seek, SeekFrom};

use crate::common::CzError;
use crate::compression::{decompress, get_chunk_info};

pub fn decode<T: Seek + ReadBytesExt + Read>(bytes: &mut T) -> Result<Vec<u8>, CzError> {
    // Get the information about the compressed chunks
    let block_info = get_chunk_info(bytes)?;
    bytes.seek(SeekFrom::Start(block_info.length as u64))?;

    // Get the bitmap
    let bitmap = decompress(bytes, &block_info).unwrap();

    Ok(bitmap)
}
