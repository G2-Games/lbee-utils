pub mod cz_common;
pub mod formats{
    pub mod cz0;
}

// Generic tools
use std::fs;
use crate::cz_common::CommonHeader;

fn main() {
    let input = fs::read("../test_files/x5a3bvy.cz1").expect("Error, could not open image");
    let header = CommonHeader::new(&input);
    println!("{:?}", header);
}
