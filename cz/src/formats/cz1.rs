use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Seek, SeekFrom, Write};

use crate::common::CzError;
use crate::compression::{compress, decompress, get_chunk_info};

pub fn decode<T: Seek + ReadBytesExt + Read>(bytes: &mut T) -> Result<Vec<u8>, CzError> {
    // Get the information about the compressed chunks
    let block_info = get_chunk_info(bytes)?;
    bytes.seek(SeekFrom::Start(block_info.length as u64))?;

    dbg!(&block_info);

    // Get the bitmap
    let bitmap = decompress(bytes, &block_info).unwrap();

    Ok(bitmap)
}

pub fn encode<T: WriteBytesExt + Write>(output: &mut T, bitmap: &[u8]) -> Result<(), CzError> {
    let (compressed_data, compressed_info) = compress(bitmap, 0xFEFD);

    dbg!(&compressed_info);

    compressed_info.write_into(output)?;

    output.write_all(&compressed_data)?;

    Ok(())
}
