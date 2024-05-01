use image::{ImageFormat, RgbaImage};

/// The header of a CZ# file
#[derive(Debug)]
pub struct CZHeader {
    version: u8, // The version from the magic bytes, (eg. CZ3, CZ4)
    length: u8,
    res: (u16, u16), // The width in the header
    depth: u8,       // Bit depth
    crop: (u16, u16),   // Crop dimensions
    bounds: (u16, u16), // Bounding box dimensions
    offset: (u16, u16), // Offset coordinates
}

impl CZHeader {
    pub fn new(bytes: &[u8]) -> Self {
        CZHeader {
            version:  bytes[2] - b'0',
            length: bytes[4],
            res: (
                u16::from_le_bytes(bytes[8..10].try_into().unwrap()),
                u16::from_le_bytes(bytes[10..12].try_into().unwrap())
            ),
            depth: bytes[12],
            crop: (
                u16::from_le_bytes(bytes[20..22].try_into().unwrap()),
                u16::from_le_bytes(bytes[22..24].try_into().unwrap())
            ),
            bounds: (
                u16::from_le_bytes(bytes[24..26].try_into().unwrap()),
                u16::from_le_bytes(bytes[26..28].try_into().unwrap())
            ),
            offset: (
                u16::from_le_bytes(bytes[28..30].try_into().unwrap()),
                u16::from_le_bytes(bytes[30..32].try_into().unwrap())
            ),
        }
    }
}

/// Defines a full file that has a header of type `CZHeader` and a vector bitmap as the body
#[derive(Debug)]
pub struct CZFile {
    header: CZHeader,
    bitmap: Vec<u8>,
}

impl CZFile {
    /// Create and save a PNG of the image data
    /// This errors if the data end up too short
    pub fn to_rgba8(&self) -> RgbaImage {
        let process_bitmap = self.bitmap.clone();

        RgbaImage::from_raw(
            self.header.res.0 as u32,
            self.header.res.1 as u32,
            process_bitmap,
        ).expect("Error encoding the image")
    }

    pub fn to_png(&self, out_name:&str) {
        let image_data = self.to_rgba8();

        match image_data.save_with_format(out_name, ImageFormat::Png) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("ERROR SAVING IMAGE: {}", e);
                eprintln!("You probably have an image with the CZ0 offset bug!")
            }
        }
    }

    /// Pretty-print information about a CZ image
    pub fn info(&self) {
        let mut image_size = self.bitmap.len() as f32 + self.header.length as f32;
        image_size /= 1024.0;

        println!("\n--IMAGE INFORMATION--");
        println!("Image size    : {:.2} KB", image_size);
        println!("Version       : {:?}", self.header.version);
        println!("Header Length : {:?} bytes", self.header.length);
        println!(
            "Resolution    : {}x{}",
            self.header.res.0, self.header.res.1
        );
        println!("Bit Depth     : {} bits", self.header.depth);
        println!(
            "Crop Coords   : {}x{}",
            self.header.crop.0, self.header.crop.1
        );
        println!(
            "Bound Coords  : {}x{}",
            self.header.bounds.0, self.header.bounds.1
        );
        println!(
            "Offset Coords : {}x{}",
            self.header.offset.0, self.header.offset.1
        );
    }
}

/// Utilities for manipulating CZ0 images
pub mod cz0 {
    use std::fs;
    use crate::cz_utils::{CZFile, CZHeader};

    /// Provided a bitstream, extract the header information and the rest of the metadata about a CZ0 file, returning a struct containing the header information and bitmap
    pub fn decode_cz0(input_filename: &str) -> CZFile {
        let mut input = fs::read(input_filename).expect("Error, could not open image");

        // TODO Research the header more!
        let header = CZHeader::new(&input);

        // Chop off the header and keep only the bitmap after it
        input.drain(..header.length as usize);

        // Construct the output CZ0 image
        let final_image = CZFile {
            header,
            bitmap: input,
        };

        println!("Decoded {}", input_filename);
        final_image
    }
}
