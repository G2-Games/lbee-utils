use cz::dynamic::DynamicCz;

fn main() {
    let img = DynamicCz::open("font72.cz1").unwrap();

    img.save_as_cz("test.cz1").unwrap();
    img.save_as_png("test1.png").unwrap();

    let img2 = DynamicCz::open("test.cz1").unwrap();
    img2.save_as_png("test2.png").unwrap();
}
