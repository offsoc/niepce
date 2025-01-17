extern crate bindgen;
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
        .whitelist_type("eng::NiepceProperties")
        .constified_enum("eng::NiepceProperties")
        .prepend_enum_name(false)
        .opaque_type("std::.*")
        .whitelist_type("eng::IndexToXmp")
        .whitelist_function("eng::property_index_to_xmp")
        .header("../../src/engine/db/bindings.hpp")
        // bindgen 0.38+ now require to set the language so that it can find
        // the proper headers for C++
        .clang_arg("-xc++")
        .clang_arg("--std=c++11")
        .clang_arg("-DRUST_BINDGEN=1")
        .clang_arg("-I../../src");

    for include in exempi
        .include_paths
        .into_iter()
        .map(|path| format!("-I{}", path.to_str().unwrap_or(".")))
    {
        builder = builder.clang_arg(include);
    }

    builder
        .dump_preprocessed_input()
        .expect("Unable to dump preprocessed input");
    // Finish the builder and generate the bindings.
    let bindings = builder
        .rustfmt_bindings(true)
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    if env::var("SKIP_CBINDINGS").is_err() {
        // Use cbindgen to generate C bindings.
        let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let target_dir = env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| String::from("./target"));
        let mut target_file = PathBuf::from(target_dir);
        target_file.push("eng_bindings.h");
        cbindgen::Builder::new()
            .with_include_guard("niepce_rust_eng_bindings_h")
            .with_namespace("ffi")
            .with_language(cbindgen::Language::Cxx)
            .with_parse_deps(true)
            .with_parse_exclude(&[
                "exempi", "chrono", "multimap", "clap", "nom", "winapi", "glib", "rusqlite",
                "strum",
            ])
            .include_item("Managed")
            .exclude_item("CUSTOM_START")
            .exclude_item("INTERNAL_START")
            .exclude_item("GdkPixbuf")
            .exclude_item("GtkWindow")
            .exclude_item("GtkToolbar")
            .exclude_item("GFileInfo")
            .exclude_item("RgbColour")
            .with_crate(&crate_dir)
            .generate()
            .expect("Couldn't generate bindings")
            .write_to_file(&*target_file.to_string_lossy());
    }
}
