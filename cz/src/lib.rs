mod binio;
mod color;
mod compression;

pub mod common;
pub mod dynamic;

mod formats {
    pub(crate) mod cz0;
    pub(crate) mod cz1;
    pub(crate) mod cz2;
    pub(crate) mod cz3;
    pub(crate) mod cz4;
}

use common::CzError;
use std::{io::BufReader, path::Path};

/// Open a CZ# file from a path
pub fn open<P: ?Sized + AsRef<Path>>(path: &P) -> Result<CzFile, CzError> {
    let mut img_file = BufReader::new(std::fs::File::open(path)?);

    CzFile::decode(&mut img_file)
}

#[doc(inline)]
pub use dynamic::CzFile;

/*
#[doc(inline)]
pub use formats::cz0::Cz0Image;
#[doc(inline)]
pub use formats::cz1::Cz1Image;
#[doc(inline)]
pub use formats::cz2::Cz2Image;
#[doc(inline)]
pub use formats::cz3::Cz3Image;
#[doc(inline)]
pub use formats::cz4::Cz4Image;
*/
