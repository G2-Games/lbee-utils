use crate::cz_common::{CommonHeader, CzHeader, CzImage};

struct Cz0Header {
    /// Common CZ# header
    common_header: CommonHeader,

    /// Dimensions of cropped image area
    crop: (u16, u16),

    /// Bounding box dimensions
    bounds: (u16, u16),

    // Offset coordinates
    offset: (u16, u16),
}

struct Cz0Image {
    header: Cz0Header,
    bitmap: Vec<u8>,
}

impl CzHeader for Cz0Header {
    fn new(bytes: &[u8]) -> Self {
        todo!()
    }

    fn version(&self) -> u8 {
        todo!()
    }

    fn header_length(&self) -> u16 {
        todo!()
    }

    fn width(&self) -> u16 {
        todo!()
    }

    fn height(&self) -> u16 {
        todo!()
    }

    fn depth(&self) -> u8 {
        todo!()
    }
}

impl CzImage for Cz0Image {
    fn decode(bytes: &[u8]) -> Self {
        let header = CZH
    }

    fn raw_bitmap(&self) -> &Vec<u8> {
        &self.bitmap
    }
}
