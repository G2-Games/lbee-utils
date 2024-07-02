use luca_pak::Pak;

fn main() {
    let pak = Pak::open("MANUAL.PAK").unwrap();
    println!("{:#032b}", pak.header().flags());

    for entry in pak.entries() {
        println!("{}", entry.name().as_ref().unwrap());
    }
}
