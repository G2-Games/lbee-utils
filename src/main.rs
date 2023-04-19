use image::{RgbaImage, ImageFormat, open};

use std::io;
use std::io::Write;
use std::fs;
use std::fs::File;

struct HeaderCZ0 {
    magic: [u8; 3],     // The magic bytes, can be CZ0, (CZ1, CZ2?)
    length: u8,
    res: (i16, i16),    // The width in the header
    depth: u8,         // Bit depth
    mystery: Vec<u8>,
    crop: (i16, i16),   // Crop dimensions
    bounds: (i16, i16), // Bounding box dimensions
    offset: (i16, i16), // Offset coordinates
}

struct CZ0File {
    header: HeaderCZ0,
    bitmap: Vec<u8>,
}

// Converts 8 bit bytes to a 16 bit little endian word or
fn bytes_to_word(first:u8, second:u8) -> i16 {
    let final_value = ((second as i16) << 8) | (first as i16);

    return final_value;
}

// Converts a 16 bit little endian word to 8 bit bytes
fn word_to_bytes(word:i16) -> [u8; 2] {
    let first: u8 = (word & 0xFF) as u8; // Extract the first byte
    let second: u8 = ((word >> 8) & 0xFF) as u8; // Extract the second byte

    return [first, second];
}

/// Extract all the header information from a CZ0 file
fn extract_header_cz0(header_vec: &Vec<u8>) -> (HeaderCZ0, usize) {
    println!("");

    // Get the magic bytes
    let magic: [u8; 3] = header_vec[0..3].try_into().unwrap();
    println!("Magic Bytes   : {:?}", magic);

    // Get the length of the header
    let length = header_vec[4];
    println!("Header Length : {:?} bytes", length);

    // Convert the width and height to i16 values
    let width = bytes_to_word(header_vec[8], header_vec[9]);
    let height = bytes_to_word(header_vec[10], header_vec[11]);
    println!("Resolution    : {}x{}", width, height);

    // Get the bit depth
    let depth = header_vec[12];
    println!("Bit Depth     : {} bits", depth);

    // Get the mystery bytes
    let mystery = header_vec[13..20].to_vec();
    println!("Header Length : {:?} bytes", length);

    // Get the crop resolution
    let crop_width = bytes_to_word(header_vec[20], header_vec[21]);
    let crop_height = bytes_to_word(header_vec[22], header_vec[23]);
    println!("Crop Coords   : {}x{}", crop_width, crop_height);

    // Get bounding box
    let bound_width = bytes_to_word(header_vec[24], header_vec[25]);
    let bound_height = bytes_to_word(header_vec[26], header_vec[27]);
    println!("Bound Coords  : {}x{}", bound_width, bound_height);

    // Get offset coordinates
    let offset_x = bytes_to_word(header_vec[28], header_vec[29]);
    let offset_y = bytes_to_word(header_vec[30], header_vec[31]);
    println!("Offset Coords : {}x{}", offset_x, offset_y);

    let image_header = HeaderCZ0 {
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
    println!("Reading input file...");
    let input_image = open(in_name).unwrap().into_rgba8();
    let (input_width, input_height) = input_image.dimensions();

    let width_diff = input_width as i16 - original_file.header.res.0;
    let height_diff = input_height as i16 - original_file.header.res.1;

    // Offset
    let offset_x = original_file.header.offset.0 + width_diff;
    let offset_y = original_file.header.offset.1 + height_diff;

    let mut length = 28;
    if offset_y > 0 || offset_x > 0 {
        length = 36;
    }

    // Construct the header
    let header = HeaderCZ0 {
        magic: [67, 90, 48],
        length,
        res: (input_width as i16, input_height as i16),
        depth: original_file.header.depth,
        mystery: original_file.header.mystery,
        crop: (input_width as i16, input_height as i16),
        bounds: (original_file.header.bounds.0, original_file.header.bounds.1),
        offset: (original_file.header.offset.0, original_file.header.offset.1),
    };

    let bitmap = input_image.to_vec();

    println!("Writing to output file...");
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

    header_assembled.drain(length as usize..);

    file.write_all(&header_assembled)?;

    // Write the actual image data
    file.write_all(&bitmap)?;

    return Ok(());
}

fn main() -> io::Result<()> {
    let image = decode_cz0("775.cz0");

    /*
    // Create and save a PNG of the image data
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


    match encode_cz0(image, "melon_test.png", "test.cz0") {
        Ok(file) => file,
        Err(error) => panic!("Problem opening the file: {:?}", error),
    };

    decode_cz0("test.cz0"); // Running this function standalone simply prints information about the image's header

    return Ok(());
}
