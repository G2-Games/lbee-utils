use std::{error::Error, io::{Read, Write}};

use encoding_rs::*;
use byteorder_lite::{LE, ReadBytesExt};

pub enum Encoding {
    UTF8,
    UTF16,
    ShiftJIS,
}

impl Encoding {
    pub fn width(&self) -> usize {
        match self {
            Self::UTF8 | Self::ShiftJIS => 1,
            Self::UTF16 => 2,
        }
    }
}

pub fn decode_string_v1<R: Read>(
    input: &mut R,
    format: Encoding,
) -> Result<String, Box<dyn Error>> {

    // Find the end of the string
    let mut string_buf = Vec::new();
    match format {
        Encoding::UTF8 | Encoding::ShiftJIS => {
            let mut string_byte = input.read_u8()?;
            while string_byte != 0 {
                string_buf.push(string_byte);
                string_byte = input.read_u8()?;
            }
        },
        Encoding::UTF16 => {
            let mut string_u16 = input.read_u16::<LE>()?;
            while string_u16 != 0 {
                string_buf.write_all(&string_u16.to_le_bytes()).unwrap();
                string_u16 = input.read_u16::<LE>()?;
            }
        },
    }

    // Get the actual string data using the proper decoder
    let string = match format {
        Encoding::UTF8 => String::from_utf8(string_buf)?,
        Encoding::UTF16 => {
            String::from_utf16(
                &string_buf.chunks_exact(2)
                    .map(|e| u16::from_le_bytes(e.try_into().unwrap()))
                    .collect::<Vec<u16>>()
            )?
        }
        Encoding::ShiftJIS => SHIFT_JIS.decode(&string_buf).0.to_string(),
    };

    Ok(string)
}

pub fn encode_string_v1(string: String, format: Encoding) -> Vec<u8> {
    match format {
        Encoding::UTF8 => string.as_bytes().to_vec(),
        Encoding::UTF16 => string.encode_utf16().flat_map(|b| b.to_le_bytes()).collect(),
        Encoding::ShiftJIS => SHIFT_JIS.encode(&string).0.to_vec(),
    }
}
