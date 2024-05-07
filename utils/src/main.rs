use cz::{dynamic::DynamicCz, Cz0Image, CzImage};
use std::fs;

fn main() {
    let mut file = fs::File::open("TEXTBOX.CZ3").unwrap();
    let img = DynamicCz::decode(&mut file).unwrap();

    img.save_as_png("test.png").unwrap();
}
