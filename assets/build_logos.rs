use std::fs::read_dir;
use resvg::tiny_skia;

const ASSET_DIR: &str = "../assets";

macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo::warning={}", format!($($tokens)*))
    }
}

fn main() {
    println!("cargo:rerun-if-changed={ASSET_DIR}");

    for image in read_dir(ASSET_DIR).unwrap().filter(|e| e.as_ref().is_ok_and(|e| e.path().is_file())) {
        let svg_path = image.unwrap().path().canonicalize().unwrap();

        if svg_path.extension().is_some_and(|e| e != "svg") {
            continue;
        }

        let mut png_path = svg_path.clone();
        png_path.set_extension("png");

        let tree = {
            let mut opt = usvg::Options {
                // Get file's absolute directory.
                resources_dir: Some(ASSET_DIR.into()),
                ..usvg::Options::default()
            };
            opt.fontdb_mut().load_system_fonts();

            let svg_data = std::fs::read(svg_path).unwrap();
            usvg::Tree::from_data(&svg_data, &opt).unwrap()
        };

        let pixmap_size = tree.size().to_int_size();
        let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
        resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());
        pixmap.save_png(png_path).unwrap();
    }
}
