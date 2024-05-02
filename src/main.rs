pub mod cz_common;
pub mod formats{
    pub mod cz0;
    pub mod cz1;
}

// Generic tools
use std::fs;

use crate::{cz_common::CzImage, formats::cz1::Cz1Image};

fn main() {
    let input = fs::read("../test_files/x5a3bvy.cz1").expect("Error, could not open image");
    let cz1_file = Cz1Image::decode(&input).unwrap();
    println!("{:#?}", cz1_file.header());

    cz1_file.save_as_png("test.png")
}
