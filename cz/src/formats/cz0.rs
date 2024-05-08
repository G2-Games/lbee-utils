use std::io::{Read, Seek};
use byteorder::ReadBytesExt;

use crate::common::CzError;

pub fn decode<T: Seek + ReadBytesExt + Read>(bytes: &mut T) -> Result<Vec<u8>, CzError> {
    // Get the rest of the file, which is the bitmap
    let mut bitmap = vec![];
    bytes.read_to_end(&mut bitmap)?;

    Ok(bitmap)
}
