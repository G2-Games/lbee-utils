use cz::{common::{CzVersion, ExtendedHeader}, dynamic::DynamicCz};

fn main() {
    let mio = image::open("mio_inverted.png").unwrap();
    let mio = mio.to_rgba8();

    let cz_mio =
        DynamicCz::from_raw(
            CzVersion::CZ3,
            mio.width() as u16,
            mio.height() as u16,
            mio.into_raw()
        )
        .with_extended_header(
            ExtendedHeader::new(1280, 960, 1280, 960)
        );

    cz_mio.save_as_cz("test1.cz3").unwrap();

    let img = DynamicCz::open("test1.cz3").unwrap();

    img.save_as_png("test.png").unwrap();
}
