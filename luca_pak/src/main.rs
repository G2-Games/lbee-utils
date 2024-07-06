use std::{fs::File, io::BufWriter};
use luca_pak::Pak;

fn main() {
    let mut clog = colog::default_builder();
    clog.filter(None, log::LevelFilter::Info);
    clog.init();

    /*
    let paths = std::fs::read_dir(".")
        .unwrap()
        .filter_map(|res| res.ok())
        .map(|dir_entry| dir_entry.path())
        .filter_map(|path| {
            if path.extension().map_or(false, |ext| ext.to_ascii_lowercase() == "pak") {
                Some(path)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let mut pak_files = vec![];
    for path in paths {
        let pak = Pak::open(&path).unwrap();
        pak_files.push(pak)
    }

    pak_files.sort_by_key(|x| x.header().flags().0 & 0xF);

    for pak in pak_files {
        println!(
            "{:#032b} - {} - {:?}",
            pak.header().flags().0,
            pak.unknown_pre_data.len(),
            pak.path(),
        );
    }
    */

    let pak = Pak::open("MANUAL.PAK").unwrap();
    println!("{:#?}", pak.header());
    //println!("{:#032b}", pak.header().flags().0);

    for (i, entry) in pak.entries().iter().enumerate() {
        //println!("{i:03}: {:06.2} kB - {}", entry.len() as f32 / 1_000.0, entry.name().as_ref().unwrap());
        entry.save("./output/").unwrap();
    }

    let mut output = BufWriter::new(File::create("MANUAL-modified.PAK").unwrap());
    pak.encode(&mut output).unwrap();
}
