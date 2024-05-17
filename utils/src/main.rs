mod font_generation;

use cz::{
    common::{CzHeader, CzVersion},
    dynamic::DynamicCz
};

fn main() {
    // Open the desired PNG
    let new_bitmap = image::open("CGGALLERY_CH01_003.png")
        .unwrap()
        .to_rgba8();

    let mut gallery_cz = DynamicCz::open("CGGALLERY_CH01_003").unwrap();

    gallery_cz.set_bitmap(new_bitmap.into_vec());
    gallery_cz.header_mut().set_depth(32);
    gallery_cz.header_mut().set_version(CzVersion::CZ0);
    gallery_cz.save_as_cz("CGGALLERY_CH01_003-MODIFIED").unwrap();

    // Open that same CZ3 again to test decoding
    let cz_image_test = DynamicCz::open("CGGALLERY_CH01_003-MODIFIED").unwrap();

    // Save the newly decoded CZ3 as another PNG as a test
    cz_image_test.save_as_png("CGGALLERY_CH01_003-MODIFIED.png").unwrap();
}
