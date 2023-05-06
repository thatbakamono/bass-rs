use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=src/bass.h");

    let bindings = bindgen::Builder::default()
        .header("src/bass.h")
        .allowlist_type("BASS.*")
        .allowlist_function("BASS_.*")
        .allowlist_var("BASS_.*")
        .dynamic_library_name("Bass")
        .dynamic_link_require_all(true)
        .layout_tests(false)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
