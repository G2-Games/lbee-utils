use image::{RgbaImage, ImageFormat, open};

use std::io;
use std::io::Write;
use std::fs;
use std::fs::File;

#[derive(Copy, Clone)]
struct HeaderCZ0 {
    magic: [u8; 3],     // The magic bytes, can be CZ0, (CZ1, CZ2?)
    length: u8,
    res: (i16, i16),    // The width in the header
    depth: u8,         // Bit depth
    crop: (i16, i16),   // Crop dimensions
    bounds: (i16, i16), // Bounding box dimensions
    offset: (i16, i16), // Offset coordinates
}

struct CZ0File {
    header: HeaderCZ0,
    bitmap: Vec<u8>,
}

// Converts 8 bit bytes to a 16 bit little endian word or
pub fn bytes_to_word(first:u8, second:u8) -> i16 {
    let final_value = ((second as i16) << 8) | (first as i16);

    return final_value;
}

// Converts a 16 bit little endian word to 8 bit bytes
pub fn word_to_bytes(word:i16) -> [u8; 2] {
    let first: u8 = (word & 0xFF) as u8; // Extract the first byte
    let second: u8 = ((word >> 8) & 0xFF) as u8; // Extract the second byte

    return [first, second];
}

/// Extract all the header information from a CZ0 file
fn extract_header_cz0(header_vec: &Vec<u8>) -> (HeaderCZ0, usize) {
    // Get the magic bytes
    let magic: [u8; 3] = header_vec[0..3].try_into().unwrap();
    println!("Magic Bytes  : {:?}", magic);

    // Get the length of the header
    let length = header_vec[4];
    println!("Header Length: {:?}", length);

    // Convert the width and height to i16 values
    let width = bytes_to_word(header_vec[8], header_vec[9]);
    let height = bytes_to_word(header_vec[10], header_vec[11]);
    println!("Resolution   : {}x{}", width, height);

    // Get the bit depth
    let depth = header_vec[12];
    println!("Bit Depth    : {} bits", depth);

    // Get the crop resolution
    let crop_width = bytes_to_word(header_vec[20], header_vec[21]);
    let crop_height = bytes_to_word(header_vec[22], header_vec[23]);
    println!("Crop Coords  : {}x{}", crop_width, crop_height);

    // Get bounding box
    let bound_width = bytes_to_word(header_vec[24], header_vec[25]);
    let bound_height = bytes_to_word(header_vec[26], header_vec[27]);
    println!("Bound Coords : {}x{}", bound_width, bound_height);

    // Get offset coordinates
    let offset_x = bytes_to_word(header_vec[28], header_vec[29]);
    let offset_y = bytes_to_word(header_vec[30], header_vec[31]);
    println!("Offset Coords: {}x{}", offset_x, offset_y);

    let image_header = HeaderCZ0 {
        magic,
        length,
        res: (width, height),
        depth,
        crop: (crop_width, crop_height),
        bounds: (bound_width, bound_height),
        offset: (offset_x, offset_y),
    };

    return (image_header, length as usize);
}

/// Provided a bitstream, extract the header information and
/// the rest of the metadata about a CZ0 file, returning a
/// struct containing the header information and bitmap
fn decode_cz0(input_filename:&str) -> CZ0File {
    println!("Reading input file...");
    let mut input = fs::read(input_filename).unwrap();

    println!("Decoding input...");

    // TODO Research the header more!
    let (header, header_length) = extract_header_cz0(&input);
    input.drain(..header_length);

    // Construct the output CZ0 image
    let final_image = CZ0File {
        header,
        bitmap: input,
    };

    return final_image;
}

/// Provided an image, extract the bitstream and create the
/// header information from a previous CZ0 file in order to
/// replace it
fn encode_cz0(original_file:CZ0File, in_name:&str, out_name:&str) -> io::Result<()> {
    let input_image = open(in_name).unwrap().into_rgba8();
    let (input_width, input_height) = input_image.dimensions();

    let width_diff = input_width as i16 - original_file.header.res.0;
    let height_diff = input_height as i16 - original_file.header.res.1;

    // Crop
    let crop_width = input_width as i16;
    let crop_height = input_height as i16;

    // Offset
    let offset_x = original_file.header.offset.0 + width_diff;
    let offset_y = original_file.header.offset.1 + height_diff;

    // Construct the header
    let header = HeaderCZ0 {
        magic: [67, 90, 48],
        length: 36,
        res: (input_width as i16, input_height as i16),
        depth: 32,
        crop: (crop_width, crop_height),
        bounds: (original_file.header.bounds.0, original_file.header.bounds.1),
        offset: (offset_x, offset_y),
    };

    let bitmap = input_image.to_vec();
    let mut file = File::create(out_name)?;

    // Write magic and length
    file.write_all(&header.magic)?;
    file.write_all(&[0])?;
    file.write_all(&[header.length])?;
    file.write_all(&vec![0u8; 3])?;

    // Write width and height
    file.write_all(&word_to_bytes(header.res.0))?;
    file.write_all(&word_to_bytes(header.res.1))?;

    // Write bit depth
    file.write_all(&[header.depth])?;
    file.write_all(&vec![0u8; 7])?;

    // Write crop width and height
    file.write_all(&word_to_bytes(header.crop.0))?;
    file.write_all(&word_to_bytes(header.crop.1))?;

    // Write bound width and height
    file.write_all(&word_to_bytes(header.bounds.0))?;
    file.write_all(&word_to_bytes(header.bounds.1))?;

    // Write offset width and height
    file.write_all(&word_to_bytes(header.offset.0))?;
    file.write_all(&word_to_bytes(header.offset.1))?;

    // Write unknown padding bytes
    file.write_all(&vec![0u8; 4])?;

    // Write the actual image data
    file.write_all(&bitmap)?;

    return Ok(());
}

fn main() -> io::Result<()> {
    let image = decode_cz0("775.cz0");

    /* Create and save a PNG of the image data
    // This errors if the image data is too short
    let tmp = match RgbaImage::from_raw(
        image.header.res.0 as u32,
        image.header.res.1 as u32,
        image.bitmap,
    ) {
        Some(img) => img,
        None => {
            RgbaImage::new(0, 0)
        }
    };

    match tmp.save_with_format("tmp.png", ImageFormat::Png) {
        Ok(()) => {
            println!("Image saved successfully");
        }
        Err(e) => {
            eprintln!("ERROR SAVING IMAGE: {}", e);
            eprintln!("You probably have an image with the CZ0 offset bug!")
        }
    }*/

    encode_cz0(image, "melon_test.png", "test.cz0").unwrap();
    decode_cz0("test.cz0");

    return Ok(());
}
