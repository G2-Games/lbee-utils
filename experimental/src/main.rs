fn main() {
    let cz_file = cz::open("test_file.cz3").unwrap();

    cz_file.save_as_png("test_file.png").unwrap();

    cz_file.save_as_cz("test_file.cz").unwrap();
}
