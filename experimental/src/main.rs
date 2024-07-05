use std::time::Instant;

fn main() {
    let mut cz_file = cz::open("test_file.cz3").unwrap();
    cz_file.save_as_png("test.png").unwrap();

    cz_file.header_mut().set_version(1).unwrap();

    let timer = Instant::now();
    cz_file.save_as_cz("unfixed.cz1").unwrap();
    println!("Saving CZ took: {}ms", timer.elapsed().as_micros() as f32 / 1000.0);
}
