use std::io::{Read, Seek, SeekFrom, Write};

use crate::common::CzError;
use crate::compression::{compress2, decompress2, get_chunk_info};

pub fn decode<T: Seek + Read>(bytes: &mut T) -> Result<Vec<u8>, CzError> {
    // Get information about the compressed chunks
    let block_info = get_chunk_info(bytes)?;
    bytes.seek(SeekFrom::Start(block_info.length as u64))?;

    // Get the bitmap
    let bitmap = decompress2(bytes, &block_info).unwrap();

    Ok(bitmap)
}

pub fn encode<T: Write>(output: &mut T, bitmap: &[u8]) -> Result<(), CzError> {
    let (compressed_data, compressed_info) = compress2(bitmap);

    compressed_info.write_into(output)?;

    output.write_all(&compressed_data)?;

    Ok(())
}
