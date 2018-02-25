extern crate bindgen;
extern crate cbindgen;
extern crate pkg_config;

use std::env;
use std::path::PathBuf;

fn main() {
    let exempi = pkg_config::Config::new()
        .print_system_libs(false)
        .probe("exempi-2.0")
        .unwrap();
    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let mut builder = bindgen::Builder::default()
        .enable_cxx_namespaces()
        .generate_comments(false)
        .generate_inline_functions(true)
        // The input header we would like to generate
        // bindings for.
        .whitelisted_type("eng::NiepceProperties")
        .whitelisted_type("eng::QueriedContent")
        .opaque_type("eng::QueriedContent")
        .whitelisted_type("fwk::FileList")
        .whitelisted_type("eng::IndexToXmp")
        .whitelisted_function("eng::property_index_to_xmp")
        .header("src/engine/db/bindings.hpp")
        .clang_arg("--std=c++11")
        .clang_arg("-DRUST_BINDGEN=1")
        .clang_arg("-I./src");

    for include in exempi.include_paths.into_iter().map(|path| {
        format!("-I{}", path.to_str().unwrap_or("."))
    }) {
        builder = builder.clang_arg(include);
    }

    // Finish the builder and generate the bindings.
    let bindings = builder.generate()
    // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Use cbindgen to generate C bindings.
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let target_dir = env::var("CARGO_TARGET_DIR").unwrap_or(String::from("./target"));
    let mut target_file = PathBuf::from(target_dir);
    target_file.push("bindings.h");
    let cbuilder = cbindgen::Builder::new()
        .with_include_guard("niepce_rust_bindings_h")
        .with_namespace("ffi")
        .with_language(cbindgen::Language::Cxx)
        .with_parse_deps(true)
        .with_parse_exclude(&["exempi", "chrono"])
        .exclude_item("GtkWindow")
        .with_crate(&crate_dir);

    if let Ok(bindings) = cbuilder.generate() {
        bindings.write_to_file(&*target_file.to_string_lossy());
    } else {
        println!("Couldn't generate bindings");
    }
}
