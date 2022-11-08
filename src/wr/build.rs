// build.rs

extern crate cbindgen;

use std::path::PathBuf;
use cbindgen::Config;

use cargo_toml::Dependency;
use cargo_toml::Manifest;
use std::fs::read;

use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

const WEBRENDER_DEP_NAME: &str = "webrender";

fn generate_webrender_revision() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let revision_file_path = Path::new(&out_dir).join("webrender_revision.rs");
    let manifest = Manifest::from_slice(&read("Cargo.toml").unwrap()).unwrap();
    let webrender_head_rev = {
        if !manifest.dependencies.contains_key(WEBRENDER_DEP_NAME) {
            "unknown"
        } else {
            let webrender_dep = &manifest.dependencies[WEBRENDER_DEP_NAME];
            match webrender_dep {
                Dependency::Detailed(detail) => detail.rev.as_ref().unwrap(),
                Dependency::Simple(version) => version,
                _ => "unknown",
            }
        }
    };

    let mut revision_file = fs::File::create(&revision_file_path).unwrap();

    write!(
        &mut revision_file,
        "{}",
        format!(
            "static WEBRENDER_HEAD_REV: Option<&'static str> = Some(\"{}\");",
            webrender_head_rev
        )
    )
    .unwrap();
}

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
    // generate_webrender_ffi();
    generate_webrender_revision();
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
