pub mod common;
pub mod compression;
pub mod formats {
    pub mod cz0;
    pub mod cz1;
    pub mod cz3;
}

use common::CzImage;
pub use formats::cz0::Cz0Image;
pub use formats::cz1::Cz1Image;
pub use formats::cz3::Cz3Image;

// Generic tools
use std::fs;

fn main() {
    let mut input = fs::File::open("../test_files/font_files/24-style1.cz1").unwrap();
    let img_file = Cz1Image::decode(&mut input).unwrap();

    img_file.save_as_cz("test1.cz1").unwrap();
    img_file.save_as_png("test1.png").unwrap();
}
