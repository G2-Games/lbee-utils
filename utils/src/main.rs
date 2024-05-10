use std::time::Instant;

use cz::{
    common::{CzVersion, ExtendedHeader},
    dynamic::DynamicCz,
};

fn main() {
    let timer = Instant::now();
    let mio = image::open("mio_inverted.png").unwrap().to_rgba8();
    println!("Opening PNG took {:?}", timer.elapsed());

    let timer = Instant::now();
    let cz_mio = DynamicCz::from_raw(
        CzVersion::CZ3,
        mio.width() as u16,
        mio.height() as u16,
        mio.into_raw(),
    )
    .with_extended_header(
        ExtendedHeader::new()
            .with_crop(1280, 960)
            .with_bounds(1280, 960),
    );
    println!("Constructing CZ3 took {:?}", timer.elapsed());

    let timer = Instant::now();
    cz_mio.save_as_png("test_save.png").unwrap();
    println!("Saving CZ3 as PNG took {:?}", timer.elapsed());

    let timer = Instant::now();
    cz_mio.save_as_cz("test1.cz3").unwrap();
    println!("Saving CZ3 as CZ3 took {:?}", timer.elapsed());

    let timer = Instant::now();
    let img = DynamicCz::open("test1.cz3").unwrap();
    println!("Opening saved CZ3 took {:?}", timer.elapsed());
}
