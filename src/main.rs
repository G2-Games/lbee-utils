// Create the modules
pub mod cz_utils; // CZ file tools
pub mod utils;    // Generic tools
use cz_utils::cz0::*;

fn main() {
    let original_image = decode_cz0("../test_files/782.cz0");
    let replacement_image = image::open("../test_files/melon_test.png").expect("Failed to open image file");

    encode_cz0(original_image, replacement_image, "test.cz0").expect("Error encoding the image");

    let image = decode_cz0("test.cz0");
    image.info();
    image.to_png("tmp.png");
}
