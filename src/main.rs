pub mod cz_common;
pub mod formats {
    pub mod cz0;
    pub mod cz1;
    pub mod cz3;
}

// Generic tools
use std::fs;

use crate::{cz_common::CzImage, formats::cz3::Cz3Image};

fn main() {
    let input = fs::read("../test_files/BAD_BG_011_10.cz4").expect("Error, could not open image");
    let img_file = Cz3Image::decode(&input).unwrap();
    println!("{:#?}", img_file.header());

    img_file.save_as_png("test.png")
}
