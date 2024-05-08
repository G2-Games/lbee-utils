use std::{
    io::{BufReader, Read, Seek, SeekFrom},
    path::Path
};
use byteorder::ReadBytesExt;

use crate::{
    common::{apply_palette, get_palette, CommonHeader, CzError, CzHeader, CzVersion, ExtendedHeader},
    formats::{cz0, cz1, cz2, cz3, cz4},
};

pub struct DynamicCz {
    header_common: CommonHeader,
    header_extended: Option<ExtendedHeader>,
    palette: Option<Vec<[u8; 4]>>,
    bitmap: Vec<u8>,
}

impl DynamicCz {
    pub fn open<P: ?Sized + AsRef<Path>>(path: &P) -> Result<Self, CzError> {
        let mut img_file = BufReader::new(std::fs::File::open(path)?);

        Self::decode(&mut img_file)
    }

    pub fn save_as_png<P: ?Sized + AsRef<Path>>(&self, path: &P) -> Result<(), png::EncodingError> {
        let file = std::fs::File::create(path).unwrap();
        let writer = std::io::BufWriter::new(file);

        let mut encoder = png::Encoder::new(
            writer,
            self.header().width() as u32,
            self.header().height() as u32,
        );
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;

        writer.write_image_data(self.bitmap())?; // Save

        Ok(())
    }
}

impl DynamicCz {
    pub fn decode<T: Seek + ReadBytesExt + Read>(input: &mut T) -> Result<Self, CzError> {
        // Get the header common to all CZ images
        let header_common = CommonHeader::new(input)?;
        let mut header_extended = None;
        if header_common.length() > 15 && header_common.version() != CzVersion::CZ2 {
            header_extended = Some(ExtendedHeader::new(input, &header_common)?);
        }
        input.seek(SeekFrom::Start(header_common.length() as u64))?;

        // Get the color palette if the bit depth is 8 or less
        let palette = if header_common.depth() <= 8 {
            let color_count = 1 << header_common.depth();
            Some(get_palette(input, color_count)?)
        } else {
            None
        };

        // Get the image data as a bitmap
        let mut bitmap = match header_common.version() {
            CzVersion::CZ0 => cz0::decode(input)?,
            CzVersion::CZ1 => cz1::decode(input)?,
            CzVersion::CZ2 => cz2::decode(input)?,
            CzVersion::CZ3 => cz3::decode(input, &header_common)?,
            CzVersion::CZ4 => cz4::decode(input, &header_common)?,
            CzVersion::CZ5 => unimplemented!(),
        };

        let image_size = header_common.width() as usize * header_common.height() as usize;
        if bitmap.len() != image_size * (header_common.depth() >> 3) as usize {
            // If the bitmap is smaller or larger than the image size, it is likely wrong
            return Err(CzError::Corrupt)
        }

        if let Some(palette) = &palette {
            bitmap = apply_palette(&bitmap, palette)?;
        }

        Ok(Self {
            header_common,
            header_extended,
            palette,
            bitmap,
        })
    }

    pub fn save_as_cz<T: Into<std::path::PathBuf>>(&self, path: T) -> Result<(), CzError> {
        todo!()
    }

    pub fn header(&self) -> &CommonHeader {
        &self.header_common
    }

    pub fn header_mut(&mut self) -> &mut CommonHeader {
        &mut self.header_common
    }

    pub fn set_header(&mut self, header: &CommonHeader) {
        self.header_common = header.to_owned()
    }

    pub fn bitmap(&self) -> &Vec<u8> {
        &self.bitmap
    }

    pub fn into_bitmap(self) -> Vec<u8> {
        self.bitmap
    }

    pub fn set_bitmap(&mut self, bitmap: Vec<u8>, width: u16, height: u16) -> Result<(), CzError> {
        if bitmap.len() != width as usize * height as usize {
            return Err(CzError::BitmapFormat)
        }

        self.bitmap = bitmap;

        self.header_mut().set_width(width);
        self.header_mut().set_height(height);

        Ok(())
    }
}
