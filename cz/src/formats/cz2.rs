use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Seek, SeekFrom, Write};

use crate::common::CzError;
use crate::compression::{compress2, decompress_2, get_chunk_info};

pub fn decode<T: Seek + ReadBytesExt + Read>(bytes: &mut T) -> Result<Vec<u8>, CzError> {
    let block_info = get_chunk_info(bytes)?;
    bytes.seek(SeekFrom::Start(block_info.length as u64))?;

    let bitmap = decompress_2(bytes, &block_info).unwrap();

    Ok(bitmap)
}

pub fn encode<T: WriteBytesExt + Write>(output: &mut T, bitmap: &[u8]) -> Result<(), CzError> {
    let (compressed_data, compressed_info) = compress2(bitmap, 0x87BDF);

    dbg!(&compressed_info);

    compressed_info.write_into(output)?;

    output.write_all(&compressed_data)?;

    Ok(())
}
