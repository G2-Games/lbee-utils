use cz::{Cz3Image, CzImage};
use std::fs;

fn main() {
    let mut input = fs::File::open("../../test_files/GOOD_00009.cz3")
        .expect("Failed to open file");

    let timer = std::time::Instant::now();
    let img_file = Cz3Image::decode(&mut input).expect("Failed to decode image");
    println!("{:?}", timer.elapsed());

    img_file.save_as_png("test1.png").unwrap();
}
