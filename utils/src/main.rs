mod font_generation;

use std::{fs::{self, File}, io::{self, Write}, path::Path};
use bimap::BiMap;
use byteorder::{LittleEndian, ReadBytesExt};
use cz::{common::default_palette, dynamic::DynamicCz};
use font_generation::load_font;
use image::{ColorType, DynamicImage, GenericImage, GenericImageView};

fn main() {
    DynamicCz::open("24-style1.cz1").unwrap().save_as_png("24-style1.png").unwrap();

    parse_info("info24-lbee").unwrap();

    let font = load_font("RodinNTLG Pro M.otf").unwrap();
    let fallback = load_font("NotoSans-Regular.ttf").unwrap();

    let characters = fs::read_to_string("character_list").unwrap();

    const FONT_SIZE: f32 = 24.0;
    const BASELINE: f32 = FONT_BOX * 0.84;
    const FONT_BOX: f32 = 25.0;

    let mut font_grid = DynamicImage::new(2504, 1800, ColorType::L8);

    let mut x_offset = 0;
    let mut y_offset = 0;
    for (_i, character) in characters.chars().enumerate() {
        if character == '\n' {
            continue
        }

        let (metrics, char_bitmap) = match font.has_glyph(character) {
            true => font.rasterize(character, FONT_SIZE),
            false => fallback.rasterize(character, FONT_SIZE),
        };

        let char_image: DynamicImage = image::GrayImage::from_raw(
            metrics.width as u32,
            metrics.height as u32,
            char_bitmap
        ).unwrap().into();

        let char_x_offset = metrics.xmin as i32;
        let char_y_offset = ((BASELINE as isize - metrics.height as isize) - metrics.ymin as isize) as i32;

        for y in 0..char_image.height() as i32 {
            for x in 0..char_image.width() as i32 {
                let x_pos = x + x_offset + char_x_offset;
                let y_pos = y + y_offset + char_y_offset;

                if !font_grid.in_bounds(
                    x_pos as u32,
                    y_pos as u32
                ) {
                    continue
                }

                if x_pos > x_offset + FONT_BOX as i32 || x_pos < x_offset {
                    continue
                } else if y_pos > y_offset + FONT_BOX as i32 || y_pos < y_offset {
                    continue
                }

                font_grid.put_pixel(
                    x_pos as u32,
                    y_pos as u32,
                    char_image.get_pixel(x as u32, y as u32)
                );
            }
        }

        x_offset += FONT_BOX as i32;
        if x_offset + FONT_BOX as i32 >= font_grid.width() as i32 {
            x_offset = 0;
            y_offset += FONT_BOX as i32;
        }
    }

    let result_image = cz::common::apply_palette(font_grid.as_bytes(), &default_palette()).unwrap();
    let cz1_font = DynamicCz::from_raw(
        cz::common::CzVersion::CZ1,
        font_grid.width() as u16,
        font_grid.height() as u16,
        result_image
    );

    cz1_font.save_as_cz("replacement_24.cz1").unwrap();

    cz1_font.save_as_png("grid.png").unwrap();
}

#[derive(Debug)]
struct FontInfo {
    font_size: u16,
    font_box: u16,
    character_count: u16,
    character_count2: u16,
    position_map: BiMap<char, u16>,
    draw_sizes: Vec<DrawSize>,
    char_sizes: Vec<CharSize>,
}

#[derive(Debug)]
struct DrawSize {
    x: u8, // x offset
    w: u8, // width
    y: u8, // y offset
}

#[derive(Debug)]
struct CharSize {
    x: u8, // x offset
    w: u8, // width
}

fn parse_info<P: ?Sized + AsRef<Path>>(
    path: &P
) -> Result<FontInfo, io::Error> {
    let mut file = fs::File::open(path).unwrap();

    let font_size = file.read_u16::<LittleEndian>().unwrap();
    let font_box = file.read_u16::<LittleEndian>().unwrap();

    let character_count = file.read_u16::<LittleEndian>().unwrap();
    let character_count2 = file.read_u16::<LittleEndian>().unwrap();

    // If the character count is 100, the other character count is correct?
    let real_char_count = if character_count == 100 {
        character_count2
    } else {
        character_count
    };

    let mut draw_sizes = Vec::new();
    for _ in 0..real_char_count {
        draw_sizes.push(DrawSize {
            x: file.read_u8().unwrap(),
            w: file.read_u8().unwrap(),
            y: file.read_u8().unwrap(),
        })
    }

    let mut utf16_index = BiMap::new();
    utf16_index.insert(' ', 0);
    let mut list = vec![];
    for index in 0..65535 {
        let map_position = file.read_u16::<LittleEndian>().unwrap();
        if map_position == 0 {
            continue
        }

        list.push((char::from_u32(index).unwrap(), map_position));
        utf16_index.insert(char::from_u32(index).unwrap(), map_position);
    }
    dbg!(utf16_index.get_by_left(&'!'));

    let mut char_sizes = vec![];
    for _ in 0..65535 {
        char_sizes.push(CharSize {
            x: file.read_u8().unwrap(),
            w: file.read_u8().unwrap(),
        })
    }

    Ok(FontInfo {
        font_size,
        font_box,
        character_count,
        character_count2,
        position_map: utf16_index,
        draw_sizes,
        char_sizes,
    })
}
