pub mod cz_common;
pub mod formats{
    pub mod cz0;
    pub mod cz1;
    pub mod cz3;
}

// Generic tools
use std::fs;

use crate::{cz_common::CzImage, formats::cz3::Cz3Image};

fn main() {
    let input = fs::read("../test_files/Old_TestFiles/129.CZ3").expect("Error, could not open image");
    let cz3_file = Cz3Image::decode(&input).unwrap();
    println!("{:#?}", cz3_file.header());

    cz3_file.save_as_png("test.png")
}
