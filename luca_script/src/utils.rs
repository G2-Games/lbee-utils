use std::error::Error;

use encoding_rs::*;

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

pub fn get_u16(bytes: &[u8], offset: usize) -> Result<(usize, u16), Box<dyn Error>> {
    Ok((
        offset + 2,
        u16::from_le_bytes(bytes[offset..offset + 2].try_into()?)
    ))
}

pub fn get_u32(bytes: &[u8], offset: usize) -> Result<(usize, u32), Box<dyn Error>> {
    Ok((
        offset + 4,
        u32::from_le_bytes(bytes[offset..offset + 4].try_into()?)
    ))
}

pub fn get_string(
    bytes: &[u8],
    offset: usize,
    format: Encoding,
    len: Option<usize>
) -> Result<(usize, String), Box<dyn Error>> {
    let slice = &bytes[offset..];

    // Find the end of the string
    let mut end = 0;
    if let Some(l) = len {
        end = l;
    } else {
        match format {
            Encoding::UTF8 | Encoding::ShiftJIS => {
                while (end < slice.len()) && (slice[end] != 0) {
                    end += 1
                }
            },
            Encoding::UTF16 => {
                while (end + 1 < slice.len()) && !((slice[end] == 0) && (slice[end + 1] == 0)) {
                    end += 2
                }
            },
        }
    };

    // Get the actual string data using the proper decoder
    let string = match format {
        Encoding::UTF8 => String::from_utf8(slice[..end].to_vec())?,
        Encoding::UTF16 => {
            String::from_utf16(
                &slice[..end].chunks_exact(2)
                    .map(|e| u16::from_le_bytes(e.try_into().unwrap()))
                    .collect::<Vec<u16>>()
            )?
        }
        Encoding::ShiftJIS => SHIFT_JIS.decode(&slice[..end]).0.to_string(),
    };

    Ok((offset + end + format.width(), string))
}
