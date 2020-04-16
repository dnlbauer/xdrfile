extern crate cc;

use std::fs;

fn main() {
    // This builds gromacs' xdrfile library
    let source_files: Vec<_> = fs::read_dir("external/xdrfile/src")
        .unwrap()
        .map(|f| f.unwrap())
        .map(|f| f.path())
        .collect();
    cc::Build::new()
        .files(source_files)
        .include("external/xdrfile/include")
        .warnings(false)
        .compile("libxdrfile.a")
}
