use std::io::{Read, Write, Seek, SeekFrom};
use byteorder::{ReadBytesExt, WriteBytesExt};

use crate::common::{CommonHeader, CzError, CzHeader};
use crate::compression::{compress, decompress, diff_line, get_chunk_info, line_diff};

pub fn decode<T: Seek + ReadBytesExt + Read>(
    bytes: &mut T,
    header: &CommonHeader
) -> Result<Vec<u8>, CzError> {
    let block_info = get_chunk_info(bytes)?;
    bytes.seek(SeekFrom::Start(block_info.length as u64))?;

    let bitmap = decompress(bytes, &block_info)?;
    let bitmap = line_diff(header, &bitmap);

    Ok(bitmap)
}

pub fn encode<T: WriteBytesExt + Write, H: CzHeader>(
    output: &mut T,
    bitmap: &[u8],
    header: &H,
) -> Result<(), CzError> {
    let bitmap = diff_line(header, bitmap);

    let (compressed_data, compressed_info) = compress(&bitmap, 65277);

    compressed_info.write_into(output)?;

    output.write_all(&compressed_data)?;

    Ok(())
}
