#![feature(slice_patterns)]
#![feature(advanced_slice_patterns)]
#![allow(dead_code)]

#![feature(slice_patterns)]
#![feature(advanced_slice_patterns)]
#![allow(dead_code)]

/// The sub-directory font.
const SPEC_SUBD_NCF: &'static str = "fonts";

const SPEC_ROOT: &'static str = "assets";

use std::env;
use std::fs;
use std::ffi::OsStr;
use std::path::PathBuf;

fn copy<S: AsRef<OsStr>>(
    mut source: PathBuf,
    mut destination: PathBuf,
    sub: S
) -> Option<()> {
    source.push(sub.as_ref());
    destination.push(sub.as_ref());
    fs::create_dir_all(&destination).ok()
       .and_then(|()|
                 fs::read_dir(&source).ok()
                 .and_then(|entry|
                           entry.filter_map(|is| is.ok())
                           .filter_map(|source| {
                                    fs::copy(
                                        source.path(),
                                        destination.join(source.file_name())
                                    ).err()
                           }).next().and_then(|_| None)
                                   .unwrap_or(Some(()))
                 )
       )
}

fn main() {
    env::var("CARGO_MANIFEST_DIR").ok()
        .and_then(|path: String| {
            let mut source: PathBuf = PathBuf::from(path);
            env::var_os("NEKO_PATH").and_then(|path| {
                  let destination = PathBuf::from(path);
                  source.push(SPEC_ROOT);
                  copy(source.clone(), destination.clone(), SPEC_SUBD_NCF)
                      .and(copy(source.clone(), destination.clone(), SPEC_SUBD_NCF))
                  })});
}
