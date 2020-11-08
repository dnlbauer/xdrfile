extern crate cc;

use std::fs;
use std::io::Result;

fn main() -> Result<()> {
    // This builds gromacs' xdrfile library
    let source_files = fs::read_dir("external/xdrfile/src")?
        .map(|r| r.map(|f| f.path()))
        .collect::<Result<Vec<_>>>()?;
    cc::Build::new()
        .files(source_files)
        .include("external/xdrfile/include")
        .warnings(false)
        .compile("libxdrfile.a");
    Ok(())
}
