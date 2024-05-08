use std::{fs::DirBuilder, time::{Duration, Instant}};

use cz::dynamic::DynamicCz;
use walkdir::WalkDir;

fn main() {
    let _ = DirBuilder::new().create("test");

    let mut total_time = Duration::default();
    let mut num_images = 0;
    for entry in WalkDir::new("../../test_files/") {
        let entry = entry.unwrap();
        if entry.path().is_dir() {
            continue;
        }

        let timer = Instant::now();
        let img = match DynamicCz::open(entry.path()) {
            Ok(img) => img,
            Err(err) => {
                println!("{}: {:?}", entry.path().file_name().unwrap().to_string_lossy(), err);
                continue;
            },
        };
        let elapsed = timer.elapsed();
        total_time += elapsed;
        num_images += 1;

        img.save_as_png(
            &format!("test/{}.png", entry.path().file_name().unwrap().to_string_lossy())
        ).unwrap();
    }

    dbg!(total_time / num_images);
}
