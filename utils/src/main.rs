use cz::{dynamic::DynamicCz, CzImage};
use std::fs;
use walkdir::WalkDir;

fn main() {
    if let Err(err) = fs::DirBuilder::new().create("test/") {
        println!("{}", err);
    }

    let mut success = 0;
    let mut failure = 0;
    for entry in WalkDir::new("../../test_files") {
        let entry = entry.unwrap();

        if entry.path().is_dir() {
            continue;
        }

        let mut input = match fs::File::open(entry.path()) {
            Ok(file) => file,
            Err(_) => continue,
        };

        let img_file = match DynamicCz::decode(&mut input) {
            Ok(file) => file,
            Err(err) => {
                println!(
                    "{}: {}",
                    entry.path().file_name().unwrap().to_string_lossy(),
                    err,
                );
                failure += 1;
                continue;
            },
        };

        success += 1;

        img_file.save_as_png(
            &format!("test/z-{}.png", entry.path().file_stem().unwrap().to_string_lossy())
        ).unwrap();
    }
}
