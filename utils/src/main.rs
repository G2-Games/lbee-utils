use std::fs;
use cz::{Cz4Image, CzImage};

fn main() {
    let mut input = fs::File::open("../../test_files/BAD_BG_011_10.cz4")
            .expect("Failed to open file");

    let timer = std::time::Instant::now();
    let img_file = Cz4Image::decode(&mut input)
        .expect("Failed to decode image");
    println!("{:?}", timer.elapsed());

    img_file.save_as_png("test1.png").unwrap();
}
