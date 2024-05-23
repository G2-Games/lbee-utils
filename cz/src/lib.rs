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

#[doc(inline)]
pub use dynamic::DynamicCz;

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
