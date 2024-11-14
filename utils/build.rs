use vergen_gix::{BuildBuilder, CargoBuilder, Emitter, GixBuilder, RustcBuilder, SysinfoBuilder};

fn main() {
    let build = BuildBuilder::all_build().unwrap();
    let cargo = CargoBuilder::all_cargo().unwrap();
    let gitcl = GixBuilder::all_git().unwrap();
    let rustc = RustcBuilder::all_rustc().unwrap();
    let si = SysinfoBuilder::all_sysinfo().unwrap();

    Emitter::default()
        .add_instructions(&build)
        .unwrap()
        .add_instructions(&cargo)
        .unwrap()
        .add_instructions(&gitcl)
        .unwrap()
        .add_instructions(&rustc)
        .unwrap()
        .add_instructions(&si)
        .unwrap()
        .emit()
        .unwrap();
}
