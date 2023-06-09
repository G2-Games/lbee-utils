use image::{ImageFormat, RgbaImage};

/// The header of a CZ* file
#[derive(Debug)]
struct CZHeader {
    magic: [u8; 3], // The magic bytes, can be CZ0, (CZ1, CZ2?)
    length: u8,
    res: (i16, i16), // The width in the header
    depth: u8,       // Bit depth
    mystery: Vec<u8>,
    crop: (i16, i16),   // Crop dimensions
    bounds: (i16, i16), // Bounding box dimensions
    offset: (i16, i16), // Offset coordinates
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

        let image_data = RgbaImage::from_raw(
            self.header.res.0 as u32,
            self.header.res.1 as u32,
            process_bitmap,
        ).expect("Error encoding the image");

        return image_data;
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
        println!("Magic Bytes   : {:?}", self.header.magic);
        println!("Header Length : {:?} bytes", self.header.length);
        println!(
            "Resolution    : {}x{}",
            self.header.res.0, self.header.res.1
        );
        println!("Bit Depth     : {} bits", self.header.depth);
        println!("Mystery Bytes : {:?}", self.header.mystery);
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
    use std::fs::File;
    use std::io;
    use std::io::Write;

    use crate::cz_utils::{CZFile, CZHeader};
    use crate::utils::*;
    use image::DynamicImage;

    /// Extract all the header information from a CZ0 file
    fn extract_header_cz0(header_vec: &Vec<u8>) -> (CZHeader, usize) {
        // Get the magic bytes
        let magic: [u8; 3] = header_vec[0..3].try_into().unwrap();

        // Get the length of the header
        let length = header_vec[4];

        // Convert the width and height to i16 values
        let width = bytes_to_word(header_vec[8], header_vec[9]);
        let height = bytes_to_word(header_vec[10], header_vec[11]);

        // Get the bit depth
        let depth = header_vec[12];

        // Get the mystery bytes
        let mystery = header_vec[13..20].to_vec();

        // Get the crop resolution
        let crop_width = bytes_to_word(header_vec[20], header_vec[21]);
        let crop_height = bytes_to_word(header_vec[22], header_vec[23]);

        // Get bounding box
        let bound_width = bytes_to_word(header_vec[24], header_vec[25]);
        let bound_height = bytes_to_word(header_vec[26], header_vec[27]);

        // Get offset coordinates
        let offset_x = bytes_to_word(header_vec[28], header_vec[29]);
        let offset_y = bytes_to_word(header_vec[30], header_vec[31]);

        let image_header = CZHeader {
            magic,
            length,
            res: (width, height),
            depth,
            mystery,
            crop: (crop_width, crop_height),
            bounds: (bound_width, bound_height),
            offset: (offset_x, offset_y),
        };

        return (image_header, length as usize);
    }

    /// Provided a bitstream, extract the header information and the rest of the metadata about a CZ0 file, returning a struct containing the header information and bitmap
    pub fn decode_cz0(input_filename: &str) -> CZFile {
        let mut input = fs::read(input_filename).expect("Error, could not open image");

        // TODO Research the header more!
        let (header, header_length) = extract_header_cz0(&input);

        // Chop off the header and keep only the bitmap after it
        input.drain(..header_length);

        // Construct the output CZ0 image
        let final_image = CZFile {
            header,
            bitmap: input,
        };

        println!("Decoded {}", input_filename);
        return final_image;
    }

    /// Provided an image, extract the bitstream and create the header information using a previous CZ0 file in order to replace it
    pub fn encode_cz0(original_file: CZFile, input_image: DynamicImage, out_name: &str) -> io::Result<()> {

        let original_image = input_image.to_rgba8();
        let (input_width, input_height) = original_image.dimensions();

        // Construct the header
        let header = CZHeader {
            magic: [67, 90, 48],
            length: original_file.header.length,
            res: (input_width as i16, input_height as i16),
            depth: original_file.header.depth,
            mystery: original_file.header.mystery,
            crop: (input_width as i16, input_height as i16),
            bounds: (original_file.header.bounds.0, original_file.header.bounds.1),
            offset: (original_file.header.offset.0, original_file.header.offset.1),
        };

        let mut file = File::create(out_name)?;

        // Assemble the header
        let mut header_assembled = [
            &header.magic[..],
            &[0],
            &[header.length],
            &vec![0u8; 3],
            &word_to_bytes(header.res.0),
            &word_to_bytes(header.res.1),
            &[header.depth],
            &header.mystery,
            &word_to_bytes(header.crop.0),
            &word_to_bytes(header.crop.1),
            &word_to_bytes(header.bounds.0),
            &word_to_bytes(header.bounds.1),
            &word_to_bytes(header.offset.0),
            &word_to_bytes(header.offset.1),
            &vec![0u8; 4],
        ].concat();

        // Cut off unnecessary information from the header
        // past the length defined in the header itself
        header_assembled.drain(header.length as usize..);

        // Write the header to the file
        file.write_all(&header_assembled)?;

        // Turn the image data into a vector
        let bitmap = original_image.to_vec();

        // Write the actual image data
        file.write_all(&bitmap)?;

        let actual_size = input_width * input_height;

        if actual_size > bitmap.len() as u32 {
            let size_diff = bitmap.len() as u32 - actual_size;
            file.write_all(&vec![0u8; size_diff as usize])?;
        }

        println!("Encoded {}", out_name);
        return Ok(());
    }
}
