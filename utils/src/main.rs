use std::fs;

use cz::{Cz3Image, CzImage};

fn main() {
    let mut input = fs::File::open("../../test_files/Old_TestFiles/129.CZ3").unwrap();
    let img_file = Cz3Image::decode(&mut input).unwrap();

    img_file.save_as_png("test1.png").unwrap();
}
