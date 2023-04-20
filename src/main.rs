use image::{RgbaImage, Rgba, ImageFormat, open};

use std::env;
use std::io;
use std::io::{Write, BufReader, Read};
use std::fs;
use std::fs::File;

pub mod utils;
use utils::util::*;

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

struct CZFile {
    header: HeaderCZ0,
    bitmap: Vec<u8>,
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
fn decode_cz0(input_filename:&str) -> CZFile {
    println!("Reading input file...");
    let mut input = fs::read(input_filename).unwrap();

    println!("Decoding input...");
    // TODO Research the header more!
    let (header, header_length) = extract_header_cz0(&input);
    input.drain(..header_length);

    // Construct the output CZ0 image
    let final_image = CZFile {
        header,
        bitmap: input,
    };

    return final_image;
}

/// Provided an image, extract the bitstream and create the
/// header information from a previous CZ0 file in order to
/// replace it
fn encode_cz0(original_file:CZFile, input_image:RgbaImage, out_name:&str) -> io::Result<()> {
    println!("Reading input file...");
    let (input_width, input_height) = input_image.dimensions();

    // Construct the header
    let header = HeaderCZ0 {
        magic: [67, 90, 48],
        length: original_file.header.length,
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

    println!("{:?}", header_assembled);

    // Cut off unnecessary information from the header
    header_assembled.drain(header.length as usize..);

    // Write the header to the image
    file.write_all(&header_assembled)?;

    // Write the actual image data
    file.write_all(&bitmap)?;

    let actual_size = input_width * input_height;

    if actual_size > bitmap.len() as u32 {
        let size_diff = bitmap.len() as u32 - actual_size;
        file.write_all(&vec![0u8; size_diff as usize])?;
    }

    return Ok(());
}

// Create and save a PNG of the image data
// This errors if the image data is too short
fn create_png(image:CZFile, out_name:&str) {
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
    }
}

fn main() {

    /*
    match encode_cz0(img, img2, "test.cz0") {
        Ok(file) => file,
        Err(error) => panic!("Problem opening the file: {:?}", error),
    };*/

    // Running this function standalone simply prints information about the image's header
    let image = decode_cz0("test.cz0");
    create_png(image, "tmp.png");
}
