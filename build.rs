#![allow(unused)]

use std::convert::AsRef;
use std::env;
use std::ffi::OsString;
use std::path::{PathBuf, Path};
use std::process::Command;
use std::string::ToString;


///////////////////////////////////////////////////////////////////////////////
// UTILS - ENVIROMENT
///////////////////////////////////////////////////////////////////////////////

fn is_release_mode() -> bool {
    let value = std::env::var("PROFILE")
        .expect("missing PROFILE")
        .to_lowercase();
    &value == "release"
}

fn is_debug_mode() -> bool {
    let value = std::env::var("PROFILE")
        .expect("missing PROFILE")
        .to_lowercase();
    &value == "debug"
}

fn out_dir() -> PathBuf {
    PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR env var"))
}

fn source_path() -> PathBuf {
    env::current_dir().unwrap().join("x264-stable")
}

fn install_prefix() -> PathBuf {
    out_dir().join("build")
}

///////////////////////////////////////////////////////////////////////////////
// UTILS - BUILD
///////////////////////////////////////////////////////////////////////////////

pub fn lookup_newest(paths: Vec<PathBuf>) -> Option<PathBuf> {
    use std::time::{SystemTime, Duration};
    let mut newest: Option<(PathBuf, Duration)> = None;
    paths
        .clone()
        .into_iter()
        .filter_map(|x: PathBuf| {
            let timestamp = x
                .metadata()
                .ok()
                .and_then(|y| y.created().ok())
                .and_then(|x| x.duration_since(SystemTime::UNIX_EPOCH).ok());
            match timestamp {
                Some(y) => Some((x, y)),
                _ => None
            }
        })
        .for_each(|(x_path, x_created)| match &newest {
            None => {
                newest = Some((x_path, x_created));
            }
            Some((_, y_created)) => {
                if &x_created > y_created {
                    newest = Some((x_path, x_created));
                }
            }
        });
    // DONE
    newest.map(|(x, _)| x)
}

pub fn files_with_prefix(dir: &PathBuf, pattern: &str) -> Vec<PathBuf> {
    std::fs::read_dir(dir)
        .expect(&format!("get dir contents: {:?}", dir))
        .filter_map(Result::ok)
        .filter_map(|x| {
            let file_name = x
                .file_name()
                .to_str()?
                .to_owned();
            if file_name.starts_with(pattern) {
                Some(x.path())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

fn build_x264() {
    let source_path = source_path();

    let mut prefix_arg = OsString::from("--prefix=");
    prefix_arg.push(&install_prefix());

    let path = env::var("PATH").expect("PATH to be set");

    let result = Command::new("bash")
        .arg("configure")
        .arg(prefix_arg)
        .arg("--disable-cli")
        .arg("--enable-static")
        // need to manually set PATH to make libstd think that the env has
        // changed in order to trigger the right path search logic
        // https://github.com/rust-lang/rust/issues/37519
        .env("PATH", &path)
        .current_dir(&source_path)
        .status()
        .unwrap();

    if !result.success() {
        panic!("failed to configure x264");
    }

    let result = Command::new("make")
        .arg("-j")
        .arg(num_cpus::get().to_string())
        .arg("all")
        .arg("install")
        // need to manually set PATH to make libstd think that the env has
        // changed in order to trigger the right path search logic
        // https://github.com/rust-lang/rust/issues/37519
        .env("PATH", &path)
        .current_dir(&source_path)
        .status()
        .unwrap();

    if !result.success() {
        panic!("Failed to build x264");
    }
}

fn cpy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) {
    std::fs::copy(&from, &to)
        .expect(&format!(
            "unable to cpy from {:?} to {:?}",
            from.as_ref(),
            to.as_ref(),
        ));
}

///////////////////////////////////////////////////////////////////////////////
// BUILD PIPELINE
///////////////////////////////////////////////////////////////////////////////

#[cfg(target_family = "unix")]
const X264_LINK_NAME: &'static str = "x264";

#[cfg(target_family = "windows")]
const X264_LINK_NAME: &'static str = "libx264";

fn build() {
    // SETUP
    assert!(source_path().exists());
    // BUILD
    build_x264();
    // LINK
    println!("cargo:rustc-link-search=native={}", {
        install_prefix().join("lib").to_str().unwrap()
    });


    println!("cargo:rustc-link-lib=static={}", X264_LINK_NAME);

    // CARGO METADATA
    println!("cargo:libs={}", install_prefix().join("lib").to_str().unwrap());
    println!("cargo:pkgconfig={}", install_prefix().join("lib").join("pkgconfig").to_str().unwrap());
}

///////////////////////////////////////////////////////////////////////////////
// MAIN
///////////////////////////////////////////////////////////////////////////////

fn main() {
    build();
}
