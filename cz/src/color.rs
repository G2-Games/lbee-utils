use std::{
    collections::HashMap,
    io::{Read, Seek},
};

use byteorder::ReadBytesExt;
use image::Rgba;
use quantizr::Image;

use crate::common::{CommonHeader, CzError};

pub fn get_palette<T: Seek + ReadBytesExt + Read>(
    input: &mut T,
    num_colors: usize,
) -> Result<Vec<Rgba<u8>>, CzError> {
    let mut colormap = Vec::with_capacity(num_colors);
    let mut rgba_buf = [0u8; 4];

    for _ in 0..num_colors {
        input.read_exact(&mut rgba_buf)?;
        colormap.push(rgba_buf.into());
    }

    Ok(colormap)
}

/// Take a bitmap of indicies, and map a given palette to it, returning a new
/// RGBA bitmap
pub fn apply_palette(input: &[u8], palette: &[Rgba<u8>]) -> Result<Vec<u8>, CzError> {
    let mut output_map = Vec::new();

    for byte in input.iter() {
        let color = palette.get(*byte as usize);
        if let Some(color) = color {
            output_map.extend_from_slice(&color.0);
        } else {
            return Err(CzError::PaletteError);
        }
    }

    Ok(output_map)
}

pub fn rgba_to_indexed(input: &[u8], palette: &[Rgba<u8>]) -> Result<Vec<u8>, CzError> {
    let mut output_map = Vec::new();
    let mut cache = HashMap::new();

    for rgba in input.windows(4).step_by(4) {
        let value = match cache.get(rgba) {
            Some(val) => *val,
            None => {
                let value = palette.iter().position(|e| e.0 == rgba).unwrap_or_default() as u8;
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
) -> Result<(Vec<u8>, Vec<image::Rgba<u8>>), CzError> {
    let size = (header.width() as u32 * header.height() as u32) * 4;

    let mut buf = vec![0; size as usize];
    buf[..input.len()].copy_from_slice(input);

    let image = Image::new(&buf, header.width() as usize, header.height() as usize).unwrap();

    let mut opts = quantizr::Options::default();
    opts.set_max_colors(1 << header.depth()).unwrap();

    let mut result = quantizr::QuantizeResult::quantize(&image, &opts);
    result.set_dithering_level(0.5).unwrap();

    let mut indicies = vec![0u8; header.width() as usize * header.height() as usize];
    result.remap_image(&image, indicies.as_mut_slice()).unwrap();

    let palette = result.get_palette();

    let gen_palette = palette
        .entries
        .as_slice()
        .iter()
        .map(|c| Rgba([c.r, c.g, c.b, c.a]))
        .collect();

    Ok((indicies, gen_palette))
}

pub fn _default_palette() -> Vec<Rgba<u8>> {
    let mut colormap = Vec::new();

    for i in 0..=0xFF {
        colormap.push(Rgba([0xFF, 0xFF, 0xFF, i]))
    }

    colormap
}
