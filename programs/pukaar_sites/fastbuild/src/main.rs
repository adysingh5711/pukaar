// Builds ../guest with the local risc0 toolchain (no Docker) and prints the path of the combined .bin.
use risc0_build::{build_package, get_package, GuestOptions};
fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..").canonicalize().unwrap();
    let pkg = get_package(root.join("guest"));
    let out = build_package(&pkg, root.join("target"), GuestOptions::default()).unwrap();
    println!("{}", out[0].path);
}
