use std::io::{Read, Seek, SeekFrom};
use byteorder::ReadBytesExt;

use crate::common::{CommonHeader, CzError};
use crate::compression::{decompress, line_diff, get_chunk_info};

pub fn decode<T: Seek + ReadBytesExt + Read>(bytes: &mut T, header: &CommonHeader)
    -> Result<Vec<u8>, CzError>
{
    let block_info = get_chunk_info(bytes)?;
    bytes.seek(SeekFrom::Start(block_info.length as u64))?;

    let bitmap = decompress(bytes, &block_info)?;
    let bitmap = line_diff(header, &bitmap);

    Ok(bitmap)
}
