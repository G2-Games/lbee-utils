pub mod common;
mod binio;
mod compression;
pub mod formats {
    pub mod cz0;
    pub mod cz1;
    pub mod cz2;
    pub mod cz3;
}

#[doc(inline)]
pub use formats::cz0::Cz0Image;
#[doc(inline)]
pub use formats::cz1::Cz1Image;
#[doc(inline)]
pub use formats::cz2::Cz2Image;
#[doc(inline)]
pub use formats::cz3::Cz3Image;

/// Traits for CZ# images
#[doc(inline)]
pub use common::CzImage;
