extern crate bindgen;

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("rerun-if-changed=build.rs");
    println!("rerun-if-changed=snappy");

    let root_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR")
            .expect("should have CARGO_MANIFEST_DIR var"));
    assert!(root_dir.is_dir());

    let snappy_dir = {
        let mut dir = root_dir.clone();
        dir.push("snappy");
        dir
    };
    assert!(snappy_dir.is_dir());

    let status = Command::new("./autogen.sh")
        .current_dir(&snappy_dir)
        .status()
        .expect("should spawn autogen.sh ok");
    assert!(status.success(),
            "autogen.sh should finish ok");

    let status = Command::new("./configure")
        .current_dir(&snappy_dir)
        .status()
        .expect("should spawn configure ok");
    assert!(status.success(),
            "configure should finish ok");

    let status = Command::new("make")
        .args(&["-f", "MyMakefile"])
        .current_dir(&snappy_dir)
        .status()
        .expect("should spawn make ok");
    assert!(status.success(),
            "make should finish ok");

    let libsnappy = {
        let mut lib = snappy_dir.clone();
        lib.push("libsnappy.a");
        lib
    };
    assert!(libsnappy.is_file());

    let bindings = bindgen::Builder::default()
        .no_unstable_rust()
        .header("snappy/snappy-c.h")
        .raw_line("#![allow(non_upper_case_globals)]")
        .raw_line("#![allow(non_camel_case_types)]")
        .raw_line("#![allow(non_snake_case)]")
        .raw_line("#![allow(dead_code)]")
        .generate()
        .expect("should automaically generate FFI bindings");

    bindings
        .write_to_file(root_dir.join("src/bindings.rs"))
        .expect("should write bindings!");


    println!("cargo:rustc-link-lib=static=snappy");
    println!("cargo:rustc-link-lib=c++abi");
    println!("cargo:rustc-link-search={}",
             snappy_dir.to_str().expect("dir should be utf8"));
}
