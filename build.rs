extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let bindings = bindgen::Builder::default()
        .header_contents("ffi.h", "#include <ucontext.h>")
        .whitelisted_function("getcontext")
        .whitelisted_var("REG_.*")
        .clang_arg("-D_XOPEN_SOURCE")
        .generate()
        .expect("Should generate FFI bindings OK");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("ffi.rs"))
        .expect("Should write ffi.rs OK");
}
