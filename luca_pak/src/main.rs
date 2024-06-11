use luca_pak::Pak;

fn main() {
    let pak = Pak::open("PARAM.PAK").unwrap();

    let file = pak.get_file(0).unwrap();

    dbg!(pak.files());

    file.save("test").unwrap();
}
