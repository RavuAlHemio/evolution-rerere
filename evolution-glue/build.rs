use std::env;
use std::path::PathBuf;

use pkg_config::Library;


fn pkg_conf_lib(name: &str) -> Library {
    match pkg_config::probe_library(name) {
        Ok(l) => l,
        Err(e) => panic!("failed to find {} using pkg-config: {}", name, e),
    }
}

fn libs_to_inc_flags(libs: &[Library]) -> Vec<String> {
    let mut flags = Vec::new();
    for lib in libs {
        for inc_path in &lib.include_paths {
            flags.push(format!("-I{}", inc_path.to_string_lossy()));
        }
    }
    flags
}

fn main() {
    let libs = [
        pkg_conf_lib("libedataserver-1.2"),
        pkg_conf_lib("gtk+-3.0"),
        pkg_conf_lib("webkit2gtk-4.1"),
    ];
    let clang_args = libs_to_inc_flags(&libs);

    let bindings = bindgen::Builder::default()
        .clang_arg("-I../thirdparty/evolution/src")
        .clang_arg("-I/usr/include/evolution")
        .clang_args(&clang_args)
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .blocklist_item("IPPORT_RESERVED")
        .generate()
        .expect("unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
