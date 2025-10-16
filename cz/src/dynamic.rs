use byteorder_lite::ReadBytesExt;
use rgb::ComponentSlice;
use std::{
    fs::File,
    io::{BufWriter, Read, Seek, SeekFrom, Write},
};

use crate::{
    color::{get_palette, indexed_gen_palette, indexed_to_rgba, rgba_to_indexed, Palette},
    common::{CommonHeader, CzError, CzVersion, ExtendedHeader},
    formats::{cz0, cz1, cz2, cz3, cz4},
};

/// A CZ# interface which can open and save any CZ file type.
#[derive(Debug, Clone)]
pub struct CzFile {
    header_common: CommonHeader,
    header_extended: Option<ExtendedHeader>,

    /// A palette of RGBA values for indexed color
    palette: Option<Palette>,

    /// 32bpp RGBA bitmap representation of the file contents
    bitmap: Vec<u8>,
}

impl CzFile {
    /// Decode a CZ# file from anything that implements [`Read`] and [`Seek`]
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
            CzVersion::CZ5 => unimplemented!("CZ5 files are not implemented! Please contact the application developers about this file."),
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
                todo!("Files with a bit depth of 4 are not yet supported")
            }
            8 => {
                if let Some(palette) = &palette {
                    bitmap = indexed_to_rgba(&bitmap, palette)?;
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
    pub fn save_as_cz<P: ?Sized + AsRef<std::path::Path>>(&self, path: &P) -> Result<(), CzError> {
        let mut out_file = BufWriter::new(File::create(path.as_ref())?);

        self.encode(&mut out_file)?;

        Ok(())
    }

    /// Encode a CZ# file into anything that implements [`Write`] and [`Seek`]
    ///
    /// This encodes everything based on options the header which have been
    /// set by the user. For example, to change the version of file to be
    /// saved, use [`CommonHeader::set_version()`]
    pub fn encode<T: Write>(&self, mut output: &mut T) -> Result<(), CzError> {
        let mut header = *self.header();

        if header.version() == CzVersion::CZ2 {
            header.set_length(0x12)
        }
        header.write_into(&mut output)?;

        if header.version() == CzVersion::CZ2 {
            // TODO: CZ2 files have this odd section instead of an extended header...?
            output.write_all(&[0, 0, 0])?;
        } else if let Some(ext) = self.header_extended {
            ext.write_into(&mut output)?;
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
                    output_bitmap = rgba_to_indexed(self.as_raw(), pal)?;

                    for rgba in pal.colors() {
                        output.write_all(rgba.as_slice())?;
                    }
                } else {
                    // Generate a palette and corresponding indexed bitmap if there is none
                    let result = indexed_gen_palette(self.as_raw(), self.header())?;

                    output_bitmap = result.0;
                    let palette = result.1;

                    for rgba in palette {
                        output.write_all(rgba.as_slice())?;
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
            CzVersion::CZ0 => cz0::encode(&mut output, &output_bitmap)?,
            CzVersion::CZ1 => cz1::encode(&mut output, &output_bitmap)?,
            CzVersion::CZ2 => cz2::encode(&mut output, &output_bitmap)?,
            CzVersion::CZ3 => cz3::encode(&mut output, &output_bitmap, &self.header_common)?,
            CzVersion::CZ4 => cz4::encode(&mut output, &output_bitmap, &self.header_common)?,
            CzVersion::CZ5 => todo!(),
        }

        Ok(())
    }

    /// Create a CZ# image from RGBA bytes. The bytes *must* be RGBA, as that
    /// is the only format that is used internally.
    pub fn from_raw(version: CzVersion, width: u16, height: u16, bitmap: Vec<u8>) -> Self {
        let header_common = CommonHeader::new(version, width, height);

        Self {
            header_common,
            header_extended: None,
            palette: None,
            bitmap,
        }
    }

    /// Set a specific header for the image.
    pub fn with_header(mut self, header: CommonHeader) -> Self {
        self.header_common = header;

        self
    }

    /// Set an [`ExtendedHeader`] to be used for the image. This header
    /// controls things like cropping and offsets in the game engine.
    pub fn with_extended_header(mut self, ext_header: ExtendedHeader) -> Self {
        if ext_header.unknown_width.is_some() {
            self.header_common.set_length(36)
        } else {
            self.header_common.set_length(28)
        }

        self.header_extended = Some(ext_header);

        self
    }

    /// Remove an extended header if the image has one, else this does nothing.
    pub fn clear_extended_header(&mut self) {
        self.header_extended = None
    }

    /// Returns a reference to the palette if it exists.
    pub fn palette(&self) -> &Option<Palette> {
        &self.palette
    }

    /// Returns a mutable reference to the palette if it exists.
    pub fn palette_mut(&mut self) -> &mut Option<Palette> {
        &mut self.palette
    }

    /// Remove the image palette, which forces palette regeneration on save
    /// for images with a bit depth of 8.
    pub fn clear_palette(&mut self) {
        *self.palette_mut() = None
    }

    /// Returns a reference to the [`CommonHeader`] of the image.
    pub fn header(&self) -> &CommonHeader {
        &self.header_common
    }

    pub fn header_mut(&mut self) -> &mut CommonHeader {
        &mut self.header_common
    }

    pub fn extended_header(&self) -> &Option<ExtendedHeader> {
        &self.header_extended
    }

    pub fn extended_header_mut(&mut self) -> &mut Option<ExtendedHeader> {
        &mut self.header_extended
    }

    pub fn set_header(&mut self, header: &CommonHeader) {
        header.clone_into(&mut self.header_common)
    }

    /// Returns the underlying raw buffer.
    pub fn as_raw(&self) -> &Vec<u8> {
        &self.bitmap
    }

    pub fn into_raw(self) -> Vec<u8> {
        self.bitmap
    }

    pub fn set_bitmap(&mut self, bitmap: Vec<u8>) {
        self.bitmap = bitmap
    }
}
