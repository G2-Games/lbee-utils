use byteorder::ReadBytesExt;
use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    path::Path,
};

use crate::{
    color::{apply_palette, get_palette, indexed_gen_palette, rgba_to_indexed, Palette},
    common::{CommonHeader, CzError, CzVersion, ExtendedHeader},
    formats::{cz0, cz1, cz2, cz3, cz4},
};

/// A CZ# interface which abstracts the CZ# generic file interface for
/// convenience.
#[derive(Debug)]
pub struct DynamicCz {
    header_common: CommonHeader,
    header_extended: Option<ExtendedHeader>,
    palette: Option<Palette>,
    bitmap: Vec<u8>,
}

impl DynamicCz {
    /// Open a CZ# file from a path
    pub fn open<P: ?Sized + AsRef<Path>>(path: &P) -> Result<Self, CzError> {
        let mut img_file = BufReader::new(std::fs::File::open(path)?);

        Self::decode(&mut img_file)
    }

    /// Decode a CZ# file from anything which implements [`Read`] and [`Seek`]
    ///
    /// The input must begin with the
    /// [magic bytes](https://en.wikipedia.org/wiki/File_format#Magic_number)
    /// of the file
    pub fn decode<T: Seek + ReadBytesExt + Read>(input: &mut T) -> Result<Self, CzError> {
        // Get the header common to all CZ images
        let header_common = CommonHeader::from_bytes(input)?;
        let mut header_extended = None;
        if header_common.length() > 15 && header_common.version() != CzVersion::CZ2 {
            header_extended = Some(ExtendedHeader::from_bytes(input, &header_common)?);
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
            eprintln!(
                "Image is wrong, length is {}, expected {}",
                bitmap.len(),
                image_size * (header_common.depth() >> 3) as usize
            );
            return Err(CzError::Corrupt(String::from("Bitmap size incorrect")));
        }

        match header_common.depth() {
            4 => {
                eprintln!("Files with a bit depth of 4 are not yet supported");
                todo!()
            }
            8 => {
                if let Some(palette) = &palette {
                    bitmap = apply_palette(&bitmap, palette)?;
                } else {
                    return Err(CzError::PaletteError);
                }
            }
            24 => {
                bitmap = bitmap
                    .windows(3)
                    .step_by(3)
                    .flat_map(|p| [p[0], p[1], p[2], 0xFF])
                    .collect();
            }
            32 => (),
            _ => {
                return Err(CzError::Corrupt(format!(
                    "Invalid bit depth: {}",
                    header_common.depth()
                )))
            }
        }

        Ok(Self {
            header_common,
            header_extended,
            palette,
            bitmap,
        })
    }

    /// Save the `DynamicCz` as a CZ# file. The format saved in is determined
    /// from the format in the header. Check [`CommonHeader::set_version()`]
    /// to change the CZ# version.
    pub fn save_as_cz<T: Into<std::path::PathBuf>>(&self, path: T) -> Result<(), CzError> {
        let mut out_file = BufWriter::new(File::create(path.into())?);
        let mut header = self.header().clone();

        if header.version() == CzVersion::CZ2 {
            header.set_length(0x12)
        }
        header.write_into(&mut out_file)?;

        if header.version() == CzVersion::CZ2 {
            // CZ2 files have this odd section instead of an extended header...?
            out_file.write_all(&[0, 0, 0])?;
        } else if let Some(ext) = self.header_extended {
            ext.write_into(&mut out_file)?;
        }

        let output_bitmap;
        match header.depth() {
            4 => {
                eprintln!("Files with a bit depth of 4 are not yet supported");
                todo!()
            }
            8 => {
                // Do things with palettes
                if let Some(pal) = &self.palette {
                    // Use the existing palette to palette the image
                    output_bitmap = rgba_to_indexed(self.bitmap(), pal)?;

                    for rgba in &pal.colors {
                        out_file.write_all(&rgba.0)?;
                    }
                } else {
                    // Generate a palette and corresponding indexed bitmap if there is none
                    let result = indexed_gen_palette(self.bitmap(), self.header())?;

                    output_bitmap = result.0;
                    let palette = result.1;

                    for rgba in palette {
                        let mut rgba_clone = rgba.0;
                        if false {
                            // TODO: Make a toggle for this
                            rgba_clone[0..3].reverse();
                        }
                        out_file.write_all(&rgba_clone)?;
                    }
                }
            }
            24 => {
                // Convert from RGBA to RGB
                output_bitmap = self
                    .bitmap
                    .windows(4)
                    .step_by(4)
                    .flat_map(|p| &p[0..3])
                    .copied()
                    .collect();
            }
            32 => output_bitmap = self.bitmap.clone(),
            _ => {
                return Err(CzError::Corrupt(format!(
                    "Invalid bit depth: {}",
                    self.header_common.depth()
                )))
            }
        }

        match self.header_common.version() {
            CzVersion::CZ0 => cz0::encode(&mut out_file, &output_bitmap)?,
            CzVersion::CZ1 => cz1::encode(&mut out_file, &output_bitmap)?,
            CzVersion::CZ2 => cz2::encode(&mut out_file, &output_bitmap)?,
            CzVersion::CZ3 => cz3::encode(&mut out_file, &output_bitmap, &self.header_common)?,
            CzVersion::CZ4 => cz4::encode(&mut out_file, &output_bitmap, &self.header_common)?,
            CzVersion::CZ5 => todo!(),
        }

        Ok(())
    }

    /// Save the CZ# image as a lossless PNG file.
    ///
    /// Internally, the [`DynamicCz`] struct operates on 32-bit RGBA values,
    /// which is the highest encountered in CZ# files, therefore saving them
    /// as a PNG of the same or better quality is lossless.
    pub fn save_as_png<P: ?Sized + AsRef<Path>>(
        &self,
        path: &P,
    ) -> Result<(), image::error::EncodingError> {
        let size = (self.header_common.width() as u32 * self.header_common.height() as u32) * 4;

        let mut buf = vec![0; size as usize];
        buf[..self.bitmap.len()].copy_from_slice(&self.bitmap);

        let image = image::RgbaImage::from_raw(
            self.header_common.width() as u32,
            self.header_common.height() as u32,
            buf.clone(),
        )
        .unwrap();

        image
            .save_with_format(path, image::ImageFormat::Png)
            .unwrap();

        Ok(())
    }

    /// Create a CZ# image from RGBA bytes. The bytes *must* be RGBA, as it is
    /// used internally for operations
    pub fn from_raw(
        version: CzVersion,
        depth: u16,
        width: u16,
        height: u16,
        bitmap: Vec<u8>,
    ) -> Self {
        let mut header_common = CommonHeader::new(version, width, height);
        header_common.set_depth(depth);

        Self {
            header_common,
            header_extended: None,
            palette: None,
            bitmap,
        }
    }

    /// Set a specific header for the image, this basica
    pub fn with_header(mut self, header: CommonHeader) -> Self {
        self.header_common = header;

        self
    }

    /// Add an [`ExtendedHeader`] to the image. This header controls things like
    /// cropping and offsets in the game engine
    pub fn with_extended_header(mut self, ext_header: ExtendedHeader) -> Self {
        if ext_header.offset_width.is_some() {
            self.header_common.set_length(36)
        } else {
            self.header_common.set_length(28)
        }

        self.header_extended = Some(ext_header);

        self
    }

    /// Retrieve a reference to the palette if it exists, otherwise [`None`]
    /// is returned
    pub fn palette(&self) -> &Option<Palette> {
        &self.palette
    }

    /// Retrieve a mutable reference to the palette if it exists, otherwise
    /// [`None`] is returned
    pub fn palette_mut(&mut self) -> &mut Option<Palette> {
        &mut self.palette
    }

    /// Remove the image palette, which forces palette regeneration on save
    /// for bit depths 8 or less
    pub fn remove_palette(&mut self) {
        *self.palette_mut() = None
    }

    pub fn header(&self) -> &CommonHeader {
        &self.header_common
    }

    pub fn header_mut(&mut self) -> &mut CommonHeader {
        &mut self.header_common
    }

    pub fn set_header(&mut self, header: &CommonHeader) {
        header.clone_into(&mut self.header_common)
    }

    pub fn bitmap(&self) -> &Vec<u8> {
        &self.bitmap
    }

    pub fn into_bitmap(self) -> Vec<u8> {
        self.bitmap
    }

    pub fn set_bitmap(&mut self, bitmap: Vec<u8>) {
        self.bitmap = bitmap
    }
}
