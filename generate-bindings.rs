// cargo-deps: bindgen="0.50"
//
// run with cargo script

use std::env;
use std::path::PathBuf;

pub const HEADERS: &[&str] = &[
    "x264.h",
];

fn main() {
    let current_dir = env::current_dir().unwrap();
    let x264_src = current_dir.join("x264-stable");
    let our_src = current_dir.join("src");

    let codegen = |file_name: &str, headers: &[&str]| {
        let codegen = bindgen::Builder::default();
        let codegen = codegen.header("include/prelude.h");
        let codegen = headers
            .iter()
            .fold(codegen, |codegen: bindgen::Builder, path: &&str| -> bindgen::Builder {
                let path: &str = path.clone();
                let path: PathBuf = x264_src.join(path);
                let path: &str = path.to_str().expect("PathBuf to str");
                assert!(PathBuf::from(path).exists());
                codegen.header(path)
            });
        codegen
            .generate_comments(true)
            .generate()
            .expect("Unable to generate bindings")
            .write_to_file(our_src.join(file_name))
            .expect("Couldn't write bindings!");
    };
    codegen("bindings_x264.rs", HEADERS);
}
