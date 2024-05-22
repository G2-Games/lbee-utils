mod font_generation;

use cz::{
    common::{CzHeader, CzVersion},
    dynamic::DynamicCz
};

fn main() {
    // Open the desired PNG
    let new_bitmap = image::open("mio.png")
        .unwrap()
        .to_rgba8();

    let mut gallery_cz = DynamicCz::open("166.cz3").unwrap();

    gallery_cz.set_bitmap(new_bitmap.into_vec());
    gallery_cz.header_mut().set_depth(8);
    gallery_cz.header_mut().set_version(CzVersion::CZ3);
    gallery_cz.save_as_cz("mio_modified-smallchunks.cz3").unwrap();

    // Open that same CZ3 again to test decoding
    let cz_image_test = DynamicCz::open("mio_modified-smallchunks.cz3").unwrap();

    // Save the newly decoded CZ3 as another PNG as a test
    cz_image_test.save_as_png("mio_modified.png").unwrap();
}
