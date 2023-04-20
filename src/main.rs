pub mod cz_utils;
pub mod utils;
use cz_utils::create_png;
use cz_utils::cz0::*;

fn main() {
    let original_image = decode_cz0("../test_files/775.cz0");
    let replacement_image =
        image::open("../test_files/blue.png").expect("Failed to open image file");

    encode_cz0(original_image, replacement_image, "test.cz0").expect("Error encoding the image");

    let image = decode_cz0("test.cz0");
    display_info(&image);
    create_png(&image, "tmp.png");
}
