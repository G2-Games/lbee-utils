use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Seek, Write};

use crate::common::CzError;

pub fn decode<T: Seek + ReadBytesExt + Read>(
    input: &mut T
) -> Result<Vec<u8>, CzError> {
    // Get the rest of the file, which is the bitmap
    let mut bitmap = vec![];
    input.read_to_end(&mut bitmap)?;

    Ok(bitmap)
}

pub fn encode<T: WriteBytesExt + Write>(
    output: &mut T,
    bitmap: &[u8]
) -> Result<(), CzError> {
    output.write_all(bitmap)?;

    Ok(())
}
