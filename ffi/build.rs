extern crate cbindgen;

use std::io::Write;
use std::path::PathBuf;
use std::{env, fs};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    let target_path = PathBuf::from("target");

    // TODO: check for target, only needed on macOS (and iOS?)
    let libs_priv = "-framework Security -framework Foundation";

    let pkg_config = format!(
        include_str!("pgp.pc.in"),
        name = env::var("CARGO_PKG_NAME").unwrap(),
        description = env::var("CARGO_PKG_DESCRIPTION").unwrap(),
        url = env::var("CARGO_PKG_HOMEPAGE").unwrap_or("".to_string()),
        version = env::var("CARGO_PKG_VERSION").unwrap(),
        libs_priv = libs_priv,
        prefix = env::var("PREFIX").unwrap_or("/usr/local".to_string()),
    );

    fs::create_dir_all(target_path.join("pkgconfig")).unwrap();
    fs::File::create(target_path.join("pkgconfig").join("distinst.pc.stub"))
        .unwrap()
        .write_all(&pkg_config.as_bytes())
        .unwrap();

    let cfg = cbindgen::Config::from_file(&format!("{}/cbindgen.toml", &crate_dir))
        .expect("invalid config");

    let c = cbindgen::Builder::new()
        .with_config(cfg)
        .with_crate(crate_dir)
        .with_header(format!("/* libpgp Header Version {} */", VERSION))
        .with_language(cbindgen::Language::C)
        .generate();

    // This is needed to ensure we don't panic if there are errors in the crates code
    // but rather just tell the rest of the system we can't proceed.
    match c {
        Ok(res) => {
            res.write_to_file("libpgp.h");
        }
        Err(err) => {
            eprintln!("unable to generate bindings: {:#?}", err);
            std::process::exit(1);
        }
    }
}