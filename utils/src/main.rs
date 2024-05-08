use cz::dynamic::DynamicCz;

fn main() {
    let img = DynamicCz::open("../../test_files/x5a3bvy.cz1").unwrap();

    img.save_as_cz("test.cz1").unwrap();
}
