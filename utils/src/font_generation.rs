use std::{
    fs,
    io::Error,
    path::Path
};
use fontdue::{Font, FontSettings};

pub fn load_font<P: ?Sized + AsRef<Path>>(
    path: &P
) -> Result<Font, Error> {
    let font_file: Vec<u8> = fs::read(path)?;

    let font = Font::from_bytes(
        font_file,
        FontSettings {
            scale: 72.0,
            ..Default::default()
        }
    ).unwrap();

    Ok(font)
}
