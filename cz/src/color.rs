use imagequant::Attributes;
use rgb::{ComponentSlice, RGBA8};
use std::{
    collections::HashMap,
    io::{Read, Seek},
};

use crate::common::{CommonHeader, CzError};

/// A palette of RGBA values for indexed color
#[derive(Debug, Clone)]
pub struct Palette {
    colors: Vec<RGBA8>,
}

impl Palette {
    /// Get the list of colors from the palette
    pub fn colors(&self) -> &Vec<RGBA8> {
        &self.colors
    }

    /// Consume the palette, returning a list of colors
    pub fn into_colors(self) -> Vec<RGBA8> {
        self.colors
    }

    pub fn len(&self) -> usize {
        self.colors.len()
    }

    pub fn get(&self, index: usize) -> Option<&RGBA8> {
        self.colors.get(index)
    }
}

/// Get a palette from the input stream, beginning where the palette starts.
pub fn get_palette<T: Seek + Read>(input: &mut T, num_colors: usize) -> Result<Palette, CzError> {
    let mut colormap = Vec::with_capacity(num_colors);
    let mut rgba_buf = [0u8; 4];

    for _ in 0..num_colors {
        input.read_exact(&mut rgba_buf)?;
        colormap.push(rgba_buf.into());
    }

    Ok(Palette { colors: colormap })
}

/// Takes an indexed color bitmap and maps a given palette to it, returning an
/// RGBA bitmap.
pub fn indexed_to_rgba(input: &[u8], palette: &Palette) -> Result<Vec<u8>, CzError> {
    let mut output_map = Vec::new();

    for byte in input.iter() {
        let color = palette.get(*byte as usize);
        if let Some(color) = color {
            output_map.extend_from_slice(color.as_slice());
        } else {
            return Err(CzError::PaletteError);
        }
    }

    Ok(output_map)
}

/// Takes an RGBA bitmap and maps the colors in it to indices of an indexed bitmap.
pub fn rgba_to_indexed(input: &[u8], palette: &Palette) -> Result<Vec<u8>, CzError> {
    let mut output_map = Vec::new();
    let mut cache = HashMap::new();

    for rgba in input.windows(4).step_by(4) {
        let value = match cache.get(rgba) {
            Some(val) => *val,
            None => {
                let value = palette
                    .colors()
                    .iter()
                    .position(|e| e.as_slice() == rgba)
                    .unwrap_or_default() as u8;

                cache.insert(rgba, value);
                value
            }
        };

        output_map.push(value)
    }

    Ok(output_map)
}

/// Generate and a bitmap for a given input of RGBA pixels.
pub fn indexed_gen_palette(
    input: &[u8],
    header: &CommonHeader,
) -> Result<(Vec<u8>, Vec<RGBA8>), CzError> {
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

    let mut image = quant
        .new_image(buf, header.width() as usize, header.height() as usize, 0.0)
        .unwrap();

    let mut quant_result = quant.quantize(&mut image).unwrap();

    let (palette, indicies) = quant_result.remapped(&mut image).unwrap();

    let gen_palette: Vec<RGBA8> = palette
        .iter()
        .map(|c| RGBA8::from([c.r, c.g, c.b, c.a]))
        .collect();

    let mut output_palette = vec![RGBA8::from([0, 0, 0, 0]); 256];
    output_palette[0..gen_palette.len()].copy_from_slice(&gen_palette);

    Ok((indicies, output_palette))
}

pub fn _default_palette() -> Vec<RGBA8> {
    let mut colormap = Vec::new();

    for i in 0..=0xFF {
        colormap.push(RGBA8::from([0xFF, 0xFF, 0xFF, i]))
    }

    colormap
}
