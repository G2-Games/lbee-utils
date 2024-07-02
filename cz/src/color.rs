use std::{
    collections::HashMap,
    io::{Read, Seek},
};

use byteorder::ReadBytesExt;
use imagequant::Attributes;

use crate::common::{CommonHeader, CzError};

#[derive(Debug, Clone, Copy)]
pub struct Rgba(pub [u8; 4]);

impl From<[u8; 4]> for Rgba {
    fn from(value: [u8; 4]) -> Self {
        Self([value[0], value[1], value[2], value[3]])
    }
}

#[derive(Debug)]
pub struct Palette {
    pub colors: Vec<Rgba>
}

pub fn get_palette<T: Seek + ReadBytesExt + Read>(
    input: &mut T,
    num_colors: usize,
) -> Result<Palette, CzError> {
    let mut colormap = Vec::with_capacity(num_colors);
    let mut rgba_buf = [0u8; 4];

    for _ in 0..num_colors {
        input.read_exact(&mut rgba_buf)?;
        colormap.push(rgba_buf.into());
    }

    Ok(Palette { colors: colormap })
}

/// Take a bitmap of indicies, and map a given palette to it, returning a new
/// RGBA bitmap
pub fn apply_palette(input: &[u8], palette: &Palette) -> Result<Vec<u8>, CzError> {
    let mut output_map = Vec::new();

    for byte in input.iter() {
        let color = palette.colors.get(*byte as usize);
        if let Some(color) = color {
            output_map.extend_from_slice(&color.0);
        } else {
            return Err(CzError::PaletteError);
        }
    }

    Ok(output_map)
}

pub fn rgba_to_indexed(input: &[u8], palette: &Palette) -> Result<Vec<u8>, CzError> {
    let mut output_map = Vec::new();
    let mut cache = HashMap::new();

    for rgba in input.windows(4).step_by(4) {
        let value = match cache.get(rgba) {
            Some(val) => *val,
            None => {
                let value = palette.colors.iter().position(|e| e.0 == rgba).unwrap_or_default() as u8;
                cache.insert(rgba, value);
                value
            }
        };

        output_map.push(value)
    }

    Ok(output_map)
}

pub fn indexed_gen_palette(
    input: &[u8],
    header: &CommonHeader,
) -> Result<(Vec<u8>, Vec<Rgba>), CzError> {
    let size = (header.width() as u32 * header.height() as u32) * 4;

    let mut buf: Vec<u8> = vec![0; size as usize];
    buf[..input.len()].copy_from_slice(input);
    let buf: Vec<imagequant::RGBA> = buf
        .windows(4)
        .step_by(4)
        .map(|c| imagequant::RGBA::new(c[0], c[1], c[2], c[3]))
        .collect();

    let mut quant = Attributes::new();
    quant.set_speed(1).unwrap();

    let mut image = quant.new_image(
        buf,
        header.width() as usize,
        header.height() as usize,
        0.0
    ).unwrap();

    let mut quant_result = quant.quantize(&mut image).unwrap();

    let (palette, indicies) = quant_result.remapped(&mut image).unwrap();

    let gen_palette: Vec<Rgba> = palette
        .iter()
        .map(|c| Rgba([c.r, c.g, c.b, c.a]))
        .collect();

    let mut output_palette = vec![Rgba([0, 0, 0, 0]); 256];
    output_palette[0..gen_palette.len()].copy_from_slice(&gen_palette);

    Ok((indicies, output_palette))
}

pub fn _default_palette() -> Vec<Rgba> {
    let mut colormap = Vec::new();

    for i in 0..=0xFF {
        colormap.push(Rgba([0xFF, 0xFF, 0xFF, i]))
    }

    colormap
}
