pub mod cz_common;
pub mod formats{
    pub mod cz0;
}

// Generic tools
use std::fs;

use crate::{cz_common::CzImage, formats::cz0::Cz0Image};

fn main() {
    let input = fs::read("../test_files/Old_TestFiles/EX_PT.CZ0").expect("Error, could not open image");
    let cz0_file = Cz0Image::decode(&input).unwrap();
    println!("{:#?}", cz0_file.header());

    cz0_file.save_as_png("test.png")
}
