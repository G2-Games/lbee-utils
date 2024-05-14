mod font_generation;

use font_generation::load_font;
use image::{ColorType, DynamicImage, GenericImage, GenericImageView};

fn main() {
    let font = load_font("NotoSans-Regular.ttf").unwrap();

    let mut characters = vec![];
    for ascii_char in 32..2048 {
        characters.push(char::from_u32(ascii_char).unwrap())
    }

    const FONT_SIZE: f32 = 24.0;
    const FONT_BOX: f32 = 25.0;

    let mut font_grid = DynamicImage::new(2560, 1800, ColorType::L8);

    let mut x_offset = 0.0;
    let mut y_offset = 0.0;
    for (_i, character) in characters.iter().enumerate() {
        let (metrics, char_bitmap) = font.rasterize(*character, FONT_SIZE);

        let mut char_image: DynamicImage = image::GrayImage::from_raw(
            metrics.width as u32,
            metrics.height as u32,
            char_bitmap
        ).unwrap().into();
        let char_image = char_image.crop(0, 0, FONT_BOX as u32, FONT_BOX as u32);

        let char_x_offset = (((FONT_BOX / 2.0) - metrics.advance_width) + metrics.bounds.xmin).ceil() as u32;
        let char_y_offset = (((FONT_BOX / 1.5) - metrics.bounds.height) - metrics.bounds.ymin).ceil() as u32;

        for y in 0..char_image.height() {
            for x in 0..char_image.width() {
                font_grid.put_pixel(
                    x + char_x_offset + x_offset as u32,
                    y + char_y_offset + y_offset as u32,
                    char_image.get_pixel(x, y)
                );
            }
        }

        x_offset += FONT_BOX;
        if x_offset + FONT_BOX >= font_grid.width() as f32 {
            x_offset = 0.0;
            y_offset += FONT_BOX;
        }
    }

    font_grid.save("grid.png").unwrap();
}
