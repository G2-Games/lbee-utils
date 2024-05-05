use byteorder::ReadBytesExt;
use image::{ImageFormat, Rgba};
use std::{
    fs::File,
    io::{BufWriter, Read, Seek, SeekFrom, Write},
    path::PathBuf
};

use crate::compression::{decompress, parse_chunk_info};
use crate::common::{apply_palette, parse_colormap, CommonHeader, CzError, CzHeader, CzImage};

#[derive(Debug, Clone)]
pub struct Cz1Image {
    header: CommonHeader,
    raw_bitmap: Option<Vec<u8>>,
    bitmap: Vec<u8>,
    palette: Vec<Rgba<u8>>,
}

impl CzImage for Cz1Image {
    type Header = CommonHeader;

    fn decode<T: Seek + ReadBytesExt + Read>(bytes: &mut T) -> Result<Self, CzError> {
        // Get the header from the input
        let header = CommonHeader::new(bytes).unwrap();
        bytes.seek(SeekFrom::Start(header.length() as u64))?;

        if header.version() != 1 {
            return Err(CzError::VersionMismatch(header.version(), 1));
        }

        // The color palette, gotten for 8 and 4 BPP images
        let mut palette = None;
        if header.depth() == 8 || header.depth() == 4 {
            palette = Some(parse_colormap(bytes, 1 << header.depth())?);
        }

        let chunk_info = parse_chunk_info(bytes)?;

        let mut bitmap = decompress(bytes, &chunk_info).unwrap();
        let mut raw_bitmap = None;

        // Apply the palette if it exists
        if let Some(pal) = &palette {
            if let Some(raw) = &mut raw_bitmap {
                bitmap.clone_into(raw);
            }

            bitmap = apply_palette(&mut bitmap, pal);
        }

        let image = Self {
            header,
            bitmap,
            raw_bitmap,
            palette: palette.unwrap(),
        };

        Ok(image)
    }

    fn save_as_png(&self, name: &str) -> Result<(), image::error::ImageError> {
        image::save_buffer_with_format(
            name,
            &self.bitmap,
            self.header.width() as u32,
            self.header.height() as u32,
            image::ExtendedColorType::Rgba8,
            ImageFormat::Png
        )
    }

    fn header(&self) -> &Self::Header {
        &self.header
    }

    fn set_header(&mut self, header: Self::Header) {
        self.header = header
    }

    fn into_bitmap(self) -> Vec<u8> {
        self.bitmap
    }

    fn save_as_cz<T: Into<PathBuf>>(&self, path: T) -> Result<(), CzError> {
        let mut output_file = BufWriter::new(File::create(path.into())?);

        output_file.write_all(&self.header.to_bytes()?)?;



        output_file.flush()?;
        Ok(())
    }

    fn set_bitmap(&mut self, bitmap: &[u8], header: &Self::Header) {
        todo!()
    }
}
