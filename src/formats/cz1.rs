use std::io::Cursor;
use image::{ImageFormat, Rgba};
use crate::cz_common::{decompress, parse_chunk_info, parse_colormap, CommonHeader, CzError, CzImage};

#[derive(Debug, Clone)]
pub struct Cz1Image {
    header: CommonHeader,
    bitmap: Vec<u8>,
    palette: Vec<Rgba<u8>>,
}

impl CzImage for Cz1Image {
    type Header = CommonHeader;

    fn decode(bytes: &[u8]) -> Result<Self, CzError> {
        let mut input = Cursor::new(bytes);

        // Get the header from the input
        let header = CommonHeader::new(&mut input).unwrap();

        // The color palette, gotten for 8 and 4 BPP images
        let mut palette = None;
        if header.depth == 8 || header.depth == 4 {
            palette = Some(parse_colormap(&mut input, 1 << header.depth)?);
        }

        let chunk_info = parse_chunk_info(&mut input)?;

        if chunk_info.total_size_compressed as usize > bytes.len() {
            return Err(CzError::InvalidFormat{
                expected: chunk_info.total_size_compressed,
                got: bytes.len(),
            })
        }

        let mut bitmap = decompress(&mut input, chunk_info).unwrap();

        // Apply the palette if it exists
        if let Some(pal) = &palette {
            apply_palette(&mut bitmap, pal);
        }

        let image = Self {
            header,
            bitmap,
            palette: palette.unwrap(),
        };

        Ok(image)
    }

    fn save_as_png(&self, name: &str) {
        let img = image::RgbaImage::from_raw(
            self.header.width as u32,
            self.header.height as u32,
            self.bitmap.clone()
        ).unwrap();

        img.save_with_format(name, ImageFormat::Png).unwrap();
    }

    fn header(&self) -> &Self::Header {
        &self.header
    }

    fn into_bitmap(self) -> Vec<u8> {
        self.bitmap
    }
}

fn apply_palette(input: &mut Vec<u8>, palette: &[Rgba<u8>]) {
    let mut output_map = Vec::new();

    for byte in input.iter() {
        let color = palette[*byte as usize].0;
        output_map.extend_from_slice(&color);
    }

    *input = output_map
}
