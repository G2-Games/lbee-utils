use std::{fs, io, path::Path};

use bimap::BiMap;
use byteorder::{LittleEndian, ReadBytesExt};

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
