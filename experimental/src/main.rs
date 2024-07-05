fn main() {
    let mut cz_file = cz::open("test_file.cz3").unwrap();
    cz_file.save_as_png("test.png").unwrap();

    cz_file.header_mut().set_version(4).unwrap();

    cz_file.save_as_cz("test_file.cz4").unwrap();
}
