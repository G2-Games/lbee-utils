use std::{
    io::{BufReader, Read, Seek},
    path::Path
};
use byteorder::ReadBytesExt;

use crate::{
    common::{CommonHeader, CzError, CzVersion, CzHeader},
    Cz0Image,
    Cz1Image,
    Cz2Image,
    Cz3Image,
    Cz4Image,
    CzImage
};

struct DynamicCz {
    cz_type: CzVersion,
    bitmap: Vec<u8>,
    header_common: CommonHeader,
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

impl CzImage for DynamicCz {
    type Header = CommonHeader;

    fn decode<T: Seek + ReadBytesExt + Read>(input: &mut T)
        -> Result<DynamicCz, crate::common::CzError>
    {
        let common_header = CommonHeader::new(input)?;
        input.seek(std::io::SeekFrom::Start(0))?;

        Ok(match common_header.version() {
            0 => DynamicCz::CZ0(Cz0Image::decode(input)?),
            1 => DynamicCz::CZ1(Cz1Image::decode(input)?),
            2 => DynamicCz::CZ2(Cz2Image::decode(input)?),
            3 => DynamicCz::CZ3(Cz3Image::decode(input)?),
            4 => DynamicCz::CZ4(Cz4Image::decode(input)?),
            _ => return Err(CzError::NotCzFile),
        })
    }

    fn save_as_cz<T: Into<std::path::PathBuf>>(&self, path: T) -> Result<(), CzError> {
        match self {
            DynamicCz::CZ0(img) => img.save_as_cz(path),
            DynamicCz::CZ1(_) => unimplemented!(),
            DynamicCz::CZ2(_) => unimplemented!(),
            DynamicCz::CZ3(_) => unimplemented!(),
            DynamicCz::CZ4(_) => unimplemented!(),
        }
    }

    fn header(&self) -> &Self::Header {
        match self {
            DynamicCz::CZ0(img) => img.header().common(),
            DynamicCz::CZ1(img) => img.header().common(),
            DynamicCz::CZ2(img) => img.header().common(),
            DynamicCz::CZ3(img) => img.header().common(),
            DynamicCz::CZ4(img) => img.header().common(),
        }
    }

    fn header_mut(&mut self) -> &mut Self::Header {
        todo!()
    }

    fn set_header(&mut self, header: &Self::Header) {
        todo!()
    }

    fn bitmap(&self) -> &Vec<u8> {
        match self {
            DynamicCz::CZ0(img) => img.bitmap(),
            DynamicCz::CZ1(img) => img.bitmap(),
            DynamicCz::CZ2(img) => img.bitmap(),
            DynamicCz::CZ3(img) => img.bitmap(),
            DynamicCz::CZ4(img) => img.bitmap(),
        }
    }

    fn into_bitmap(self) -> Vec<u8> {
        match self {
            DynamicCz::CZ0(img) => img.into_bitmap(),
            DynamicCz::CZ1(img) => img.into_bitmap(),
            DynamicCz::CZ2(img) => img.into_bitmap(),
            DynamicCz::CZ3(img) => img.into_bitmap(),
            DynamicCz::CZ4(img) => img.into_bitmap(),
        }
    }

    fn set_bitmap(&mut self, bitmap: &[u8], width: u16, height: u16) {
        unimplemented!()
    }
}
