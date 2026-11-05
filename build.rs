// The bundled MinGW toolchain has no libshlwapi.a. `linklibs/libshlwapi.a` is a one-line
// GNU ld script that points at the Windows import library shipped with the `windows` crates.
fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.ends_with("windows-gnu") {
        let dir = std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("linklibs");
        println!("cargo:rustc-link-search=native={}", dir.display());
    }
    println!("cargo:rerun-if-changed=build.rs");
}
