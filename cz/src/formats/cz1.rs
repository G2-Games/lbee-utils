use byteorder::ReadBytesExt;
use std::{
    fs::File,
    io::{BufWriter, Read, Seek, SeekFrom, Write},
    path::PathBuf,
};

use crate::common::{apply_palette, parse_colormap, CommonHeader, CzError, CzHeader, CzImage};
use crate::compression::{decompress, parse_chunk_info};

#[derive(Debug, Clone)]
pub struct Cz1Image {
    header: CommonHeader,
    raw_bitmap: Option<Vec<u8>>,
    bitmap: Vec<u8>,
    palette: Vec<[u8; 4]>,
}

impl CzImage for Cz1Image {
    type Header = CommonHeader;

    fn decode<T: Seek + ReadBytesExt + Read>(bytes: &mut T) -> Result<Self, CzError> {
        // Get the header from the input
        let mut header = CommonHeader::new(bytes).unwrap();
        bytes.seek(SeekFrom::Start(header.length() as u64))?;

        if header.version() != 1 {
            return Err(CzError::VersionMismatch(1, header.version()));
        }

        // Lock the color depth to 8 if it's over 32
        // This is obviously wrong, but why is it wrong?
        if header.depth() > 32 {
            header.depth = 8
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

    fn header(&self) -> &Self::Header {
        &self.header
    }

    fn set_header(&mut self, header:& Self::Header) {
        self.header = *header
    }

    fn bitmap(&self) -> &Vec<u8> {
        &self.bitmap
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

    fn set_bitmap(&mut self, bitmap: &[u8], width: u16, height: u16) {
        todo!()
    }
}
