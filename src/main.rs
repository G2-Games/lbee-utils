// Create the modules
pub mod cz_utils;
pub mod utils;

// Generic tools
use std::fs;

use crate::cz_utils::CZHeader;


fn main() {
    let input = fs::read(
        "/home/g2/Documents/projects/lbee-utils/test_files/GOOD_extra_bg.cz3"
    ).expect("Error, could not open image");

    let header = CZHeader::new(&input);

    dbg!(header);
}
