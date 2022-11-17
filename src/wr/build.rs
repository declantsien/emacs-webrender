// build.rs

extern crate cbindgen;

use cbindgen::Config;
use std::path::PathBuf;

use std::env;

fn generate_webrender_ffi() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    println!("{:?}", target_dir());
    let package_name = env::var("CARGO_PKG_NAME").unwrap();
    let output_file = target_dir()
        .join(format!("{}.h.sdf", package_name))
        .display()
        .to_string();

    let config = Config {
        namespace: Some(String::from("ffi")),
        language: cbindgen::Language::C,
        ..Default::default()
    };

    cbindgen::generate_with_config(&crate_dir, config)
        .unwrap()
        .write_to_file(&output_file);
}

fn main() {
    generate_webrender_ffi();
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=cbindgen.toml");
}

/// Find the location of the `target/` directory. Note that this may be
/// overridden by `cmake`, so we also need to check the `CARGO_TARGET_DIR`
/// variable.
fn target_dir() -> PathBuf {
    if let Ok(target) = env::var("CARGO_MANIFEST_DIR") {
        PathBuf::from(target)
    } else {
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("target")
    }
}
