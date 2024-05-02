use std::io::Read;

use crate::cz_common::{parse_colormap, CommonHeader, CzError, CzHeader, CzImage};

#[derive(Debug)]
pub struct Cz1Header {
    /// Common CZ# header
    common: CommonHeader,
}

#[derive(Debug)]
pub struct Cz1Image {
    header: Cz1Header,
    bitmap: Vec<u8>,
    palette: Vec<[u8; 4]>,
}

impl CzHeader for Cz1Header {
    fn new(bytes: &[u8]) -> Result<Self, CzError> {
        let common = CommonHeader::new(bytes);

        if common.version != 1 {
            return Err(CzError::VersionMismatch)
        }

        Ok(Self {
            common,
        })
    }

    fn version(&self) -> u8 {
        self.common.version
    }

    fn header_length(&self) -> usize {
        self.common.length as usize
    }

    fn width(&self) -> u16 {
        self.common.width
    }

    fn height(&self) -> u16 {
        self.common.height
    }

    fn depth(&self) -> u16 {
        self.common.depth
    }
}

impl CzImage for Cz1Image {
    type Header = Cz1Header;

    fn decode(bytes: &[u8]) -> Result<Self, CzError> {
        let mut position = 0;

        // Get the header from the input
        let header = Cz1Header::new(bytes)?;
        position += header.header_length();

        // The color palette
        let (palette, palette_length) = parse_colormap(&bytes[position..], 0x100);
        position += palette_length;

        dbg!(&bytes[position..position + 4]);

        let parts_count = u32::from_le_bytes(bytes[position..position + 4].try_into().unwrap());
        position += 4;
        dbg!(parts_count);
        let mut part_sizes = vec![0; parts_count as usize];
        let mut total_size = 0;

        for size in &mut part_sizes {
            let part_size = u32::from_le_bytes(bytes[position..position + 4].try_into().unwrap()) * 2;
            *size = part_size;
            total_size += part_size;

            dbg!(part_size);

            position += 8;
        }

        if position + total_size as usize > bytes.len() {
            return Err(CzError::InvalidFormat)
        }

        let mut m_dst = 0;
        let bitmap = vec![0; 4882176];

        let mut image = Self {
            header,
            bitmap,
            palette
        };

        for size in part_sizes {
            let part = &bytes[position..position + size as usize];
            position += size as usize;

            for j in (0..part.len()).step_by(2) {
                let ctl = part[j + 1];

                if ctl == 0 {
                    image.bitmap[m_dst] = part[j];
                    m_dst += 1;
                } else {
                    m_dst += image.copy_range(part, get_offset(part, j), m_dst);
                }
            }
        }

        Ok(image)
    }

    fn save_as_png(&self, name: &str) {
        image::save_buffer(
            name,
            &self.bitmap,
            self.header.common.width as u32,
            self.header.common.height as u32,
            image::ExtendedColorType::Rgba8
        ).unwrap()
    }

    fn header(&self) -> &Self::Header {
        &self.header
    }

    fn raw_bitmap(&self) -> &Vec<u8> {
        &self.bitmap
    }
}

fn get_offset(input: &[u8], src: usize) -> usize {
    (((input[src] as usize) | (input[src+1] as usize) << 8) - 0x101) * 2
}

impl Cz1Image {
    fn copy_range(&mut self, input: &[u8], src: usize, dst: usize) -> usize {
        let mut dst = dst;
        let start_pos = dst;

        if input[src + 1] == 0 {
            self.bitmap[dst] = input[src];
            dst += 1;
        } else if get_offset(input, src) == src {
            self.bitmap[dst] = 0;
            dst += 1;
        } else {
            dst += self.copy_range(input, get_offset(input, src), dst);
        }

        if input[src + 3] == 0 {
            self.bitmap[dst] = input[src + 2];
            dst += 1;
        } else if get_offset(input, src + 2) == src {
            self.bitmap[dst] = self.bitmap[start_pos];
            dst += 1;
        } else {
            self.bitmap[dst] = copy_one(input, get_offset(input, src + 2));
            dst += 1;
        }

        dst - start_pos
    }
}

fn copy_one(input: &[u8], src: usize) -> u8 {
    if input[src + 1] == 0 {
        input[src]
    } else if get_offset(input, src) == src {
        0
    } else {
        copy_one(input, get_offset(input, src))
    }
}
