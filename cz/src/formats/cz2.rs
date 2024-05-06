use byteorder::{ReadBytesExt, WriteBytesExt};
use image::{ImageFormat, Rgba};
use std::{
    fs::File,
    io::{BufWriter, Read, Seek, SeekFrom, Write},
    path::PathBuf,
};

use crate::common::{apply_palette, parse_colormap, CommonHeader, CzError, CzHeader, CzImage};
use crate::compression::{decompress, decompress_2, parse_chunk_info};

#[derive(Debug, Clone, Copy)]
pub struct Cz2Header {
    common: CommonHeader,
    unknown_1: u8,
    unknown_2: u8,
    unknown_3: u8,
}

impl CzHeader for Cz2Header {
    fn new<T: Seek + ReadBytesExt + Read>(bytes: &mut T) -> Result<Self, CzError>
    where
        Self: Sized,
    {
        let common = CommonHeader::new(bytes)?;

        if common.version() != 2 {
            return Err(CzError::VersionMismatch(common.version(), 2));
        }

        Ok(Self {
            common,
            unknown_1: bytes.read_u8()?,
            unknown_2: bytes.read_u8()?,
            unknown_3: bytes.read_u8()?,
        })
    }

    fn to_bytes(&self) -> Result<Vec<u8>, std::io::Error> {
        let mut buf = vec![];

        buf.write_all(&self.common.to_bytes()?)?;
        buf.write_u8(self.unknown_1)?;
        buf.write_u8(self.unknown_2)?;
        buf.write_u8(self.unknown_3)?;

        Ok(buf)
    }

    fn version(&self) -> u8 {
        self.common.version()
    }

    fn length(&self) -> usize {
        self.common.length()
    }

    fn width(&self) -> u16 {
        self.common.width()
    }

    fn height(&self) -> u16 {
        self.common.height()
    }

    fn depth(&self) -> u16 {
        self.common.depth()
    }

    fn color_block(&self) -> u8 {
        self.common.color_block()
    }
}

#[derive(Debug, Clone)]
pub struct Cz2Image {
    header: Cz2Header,
    bitmap: Vec<u8>,
    palette: Vec<Rgba<u8>>,
}

impl CzImage for Cz2Image {
    type Header = Cz2Header;

    fn decode<T: Seek + ReadBytesExt + Read>(bytes: &mut T) -> Result<Self, CzError> {
        let header = Cz2Header::new(bytes).unwrap();
        bytes.seek(SeekFrom::Start(header.length() as u64))?;

        if header.version() != 2 {
            return Err(CzError::VersionMismatch(header.version(), 2));
        }

        dbg!(header);

        // The color palette, gotten for 8 and 4 BPP images
        let mut palette = None;
        if header.depth() == 8 || header.depth() == 4 {
            palette = Some(parse_colormap(bytes, 1 << header.depth())?);
        }

        let chunk_info = parse_chunk_info(bytes)?;
        bytes.seek(SeekFrom::Start(chunk_info.length as u64))?;

        let mut bitmap = decompress_2(bytes, &chunk_info).unwrap();

        // Apply the palette if it exists
        if let Some(pal) = &palette {
            bitmap = apply_palette(&mut bitmap, pal);
        }

        let image = Self {
            header,
            bitmap,
            palette: palette.unwrap(),
        };

        Ok(image)
    }

    fn save_as_png(&self, name: &str) -> Result<(), image::error::ImageError> {
        let img = image::RgbaImage::from_raw(
            self.header.width() as u32,
            self.header.height() as u32,
            self.bitmap.clone(),
        )
        .unwrap();

        img.save(name)?;

        Ok(())
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
