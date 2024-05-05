use std::fs;
use cz::{Cz2Image, CzImage};

fn main() {
    let mut input = fs::File::open("../../test_files/font_files/24.cz2")
            .expect("Failed to open file");

    let img_file = Cz2Image::decode(&mut input)
        .expect("Failed to decode image");

    img_file.save_as_png("test1.png").unwrap();
}
