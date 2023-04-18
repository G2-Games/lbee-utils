use sdl2::pixels::Color;
use sdl2::rect::Point;
//use sdl2::render::Canvas;
//use sdl2::video::Window;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use image::{RgbaImage, ImageFormat, open};

use std::io;
use std::fs;
use std::thread;
use std::time::Duration;

struct Header {
    magic: [u8; 3],     // The magic bytes, can be CZ0, CZ1, CZ2
    res: (i16, i16),    // The width in the header
    depth: i16,         // Bit depth
    crop: (i16, i16),   // Crop dimensions
    bounds: (i16, i16), // Bounding box dimensions
}

struct Image {
    header: Header,
    bitmap: Vec<u8>,
}

// Converts 8 bit bytes to 16 bit words in little endian
fn bytes_to_word(first:u8, second:u8) -> i16 {
    let final_value = ((second as i16) << 8) | (first as i16);

    return final_value;
}

fn extract_header(header_vec: Vec<u8>) -> Header {
    // Get the magic bytes
    let magic: [u8; 3] = header_vec[0..3].try_into().unwrap();

    // Convert the width and height to i16 values
    let width = bytes_to_word(header_vec[8], header_vec[9]);
    let height = bytes_to_word(header_vec[10], header_vec[11]);

    // Get the bit depth
    let depth = bytes_to_word(header_vec[12], header_vec[13]);

    // Get the crop resolution
    let crop_width = bytes_to_word(header_vec[20], header_vec[21]);
    let crop_height = bytes_to_word(header_vec[22], header_vec[23]);

    // Get bounding box
    let bound_width = bytes_to_word(header_vec[24], header_vec[25]);
    let bound_height = bytes_to_word(header_vec[26], header_vec[27]);

    println!("Magic Bytes:  {:?}", magic);
    println!("Resolution:   {}x{}", width, height);
    println!("Bit Depth:    {} bits", depth);
    println!("Crop Coords:  {}x{}", crop_width, crop_height);
    println!("Bound Coords: {}x{}", bound_width, bound_height);

    let image_header = Header {
        magic,
        res: (width, height),
        depth,
        crop: (crop_width, crop_height),
        bounds: (bound_width, bound_height),
    };

    return image_header;
}

// Provided a bitstream, extract the header information and
// the rest of the metadata about a CZ file
fn decode_cz(mut input:Vec<u8>) -> Image {

    // Get the header from the first 28 (?) bytes
    // TODO Research the header more!
    let header_bytes: Vec<u8> = input.drain(0..28).collect();
    let header = extract_header(header_bytes);

    // Construct the image struct
    let final_image = Image {
        header,
        bitmap: input,
    };

    return final_image;
}

/*
fn encode_cz() -> Image {
    let on_top = open("path/to/some.png").unwrap().into_rgb8();

    // Construct the image struct
    let final_image = Image {
        header,
        bitmap: input,
    };

    return Image;
}*/

fn main() -> io::Result<()> {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    // Read all bytes of the CZ image to an array
    println!("Reading image...");
    let image_raw = fs::read("782s.cz0")?;

    let image = decode_cz(image_raw);

    let width = image.header.res.0 as i32;
    let height = image.header.res.1 as i32;

    // Build the window
    let window = video_subsystem.window("SDL2 Rust", width as u32, height as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_draw_color(Color::RGBA(255, 255, 255, 255));
    canvas.clear();

    thread::sleep(Duration::from_millis(40)); // Sleep to allow the window to display

    // Draw the bitmap to the canvas
    for (i, chunk) in image.bitmap.chunks(4).enumerate() {
        let a = chunk[3] / 255;

        // Get the pixel colors
        let r = chunk[0] * a;
        let g = chunk[1] * a;
        let b = chunk[2] * a;
        canvas.set_draw_color(Color::RGBA(r, g, b, a));

        // Get the pixel location
        let y = i as i32/width;
        let x = i as i32%width;
        canvas.draw_point(Point::new(x, y)).unwrap();
    }

    canvas.present(); // Display the image on the SDL2 canvas

    // Create and save a PNG of the image data
    let tmp = RgbaImage::from_raw(
        width as u32,
        height as u32,
        image.bitmap
    ).unwrap();

    match tmp.save_with_format("tmp.png", ImageFormat::Png) {
        Ok(()) => {
            println!("Image saved successfully");
        }
        Err(e) => {
            eprintln!("Error saving image: {}", e);
        }
    }

    // Wait for the user to press escape
    let mut event_pump = sdl_context.event_pump().unwrap();
    loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    println!("Exiting");
                    return Ok(());
                }

                _ => {
                    continue;
                }
            }
        }
    }
}
