use std::env;
use std::path::PathBuf;

fn generate_osrm_types() {
    let builder = bindgen::Builder::default()
        .header("src/graph/storage/types.hpp")
        .whitelist_type("EdgeArrayEntry")
        .whitelist_type("NodeArrayEntry")
        .whitelist_type("Metadata")
        .opaque_type("std::.*")
        .clang_arg("-std=c++11");
    let bindings = builder
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings");
}

fn main() {
    generate_osrm_types()
}