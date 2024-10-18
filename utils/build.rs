use vergen_git2::{BuildBuilder, CargoBuilder, Emitter, Git2Builder, RustcBuilder, SysinfoBuilder};

fn main() {
    let build = BuildBuilder::all_build().unwrap();
    let cargo = CargoBuilder::all_cargo().unwrap();
    let git2 = Git2Builder::all_git().unwrap();
    let rustc = RustcBuilder::all_rustc().unwrap();
    let si = SysinfoBuilder::all_sysinfo().unwrap();

    Emitter::default()
        .add_instructions(&build).unwrap()
        .add_instructions(&cargo).unwrap()
        .add_instructions(&git2).unwrap()
        .add_instructions(&rustc).unwrap()
        .add_instructions(&si).unwrap()
        .emit().unwrap();
}
