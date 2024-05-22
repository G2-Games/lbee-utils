use std::path::PathBuf;

use cz::{
    common::CzVersion,
    dynamic::DynamicCz
};

fn main() {
    // Open the desired PNG
    let new_bitmap = image::open("mio.png")
        .unwrap()
        .to_rgba8();

    let mut gallery_cz = DynamicCz::open("24.cz2").unwrap();
    gallery_cz.save_as_png("24.png").unwrap();

    //gallery_cz.header_mut().set_depth(8);
    gallery_cz.remove_palette();
    gallery_cz.header_mut().set_version(CzVersion::CZ2);
    gallery_cz.save_as_cz("24-modified.cz2").unwrap();

    // Open that same CZ3 again to test decoding
    let cz_image_test = DynamicCz::open("24-modified.cz2").unwrap();

    // Save the newly decoded CZ3 as another PNG as a test
    cz_image_test.save_as_png("24-modified.png").unwrap();
}
