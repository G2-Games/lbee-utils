use std::{fs::File, io::{BufWriter, Read}};
use luca_pak::Pak;

fn main() {
    let mut clog = colog::default_builder();
    clog.filter(None, log::LevelFilter::Info);
    clog.init();

    let mut pak = Pak::open("MANUAL.PAK").unwrap();

    let rep_cz_data: Vec<u8> = std::fs::read("en_manual01_Linkto_2_6.cz1").unwrap();
    pak.replace(4, &rep_cz_data).unwrap();

    let mut output = BufWriter::new(File::create("MANUAL-modified.PAK").unwrap());
    pak.encode(&mut output).unwrap();
}
