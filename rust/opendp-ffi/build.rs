extern crate cbindgen;

use std::env;

use cbindgen::Language;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    match cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_language(Language::Cxx)
        .generate() {
        Ok(bindings) => bindings.write_to_file("include/libopendp.h"),
        Err(cbindgen::Error::ParseSyntaxError { .. }) => return, // ignore in favor of cargo's syntax check
        Err(err) => panic!("{:?}", err)
    };
}