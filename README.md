[![Build Status](https://travis-ci.com/danijoo/xdrfile.svg?branch=master)](https://travis-ci.com/danijoo/xdrfile) [![crates](https://img.shields.io/badge/crates.io-v0.1.0-orange.svg?longCache=true)](https://crates.io/crates/xdrfile)

# xdrfile
Read and write xdr trajectory files in .xtc and .trr file format

This crate is mainly intended to be a wrapper around the GROMACS libxdrfile
XTC library and provides basic functionality to read and write xtc and trr
files with a safe api.

## Examples
### Basic usage
```rust
use xdrfile::*;

// get a handle to the file
let mut trj = XTCTrajectory::open_read("tests/1l2y.xtc").unwrap();

// find number of atoms in the file
let num_atoms = trj.get_num_atoms().unwrap();

// a frame object is used to get to read or write from a trajectory
// without instantiating data arrays for every step
let mut frame = Frame::with_capacity(num_atoms);

// read the first frame of the trajectory
let result = trj.read(&mut frame);
match result {
   Ok(_) => {
       assert_eq!(frame.step, 1);
       assert_eq!(frame.num_atoms, num_atoms);

       let first_atom_coords = frame.coords[0];
       assert_eq!(first_atom_coords, [-0.8901, 0.4127, -0.055499997]);
   }
   Err(msg) => {
       panic!("Something went wrong: {}", msg);    
   }
}
```

### Frame iteration
For convenience, the trajectory implementations provide "into_iter" to
be turned into an iterator that yields Rc<Frame>. If a frame is not kept
during iteration, the Iterator reuses it for better performance (and hence,
Rc is required)

```rust
use xdrfile::*;

// get a handle to the file
let trj = XTCTrajectory::open_read("tests/1l2y.xtc").unwrap();

// iterate over all frames
for (idx, frame) in trj.into_iter().filter_map(Result::ok).enumerate() {
    println!("{}", frame.time);
    assert_eq!(idx+1, frame.step as usize);
}
```

## xdrfile
Uses the lowlevel xdrfile c library version 1.1.4 with some minor fixes and additions copied from [mdtraj](https://github.com/mdtraj/mdtraj).



## License
Licensed under LGPL because this is what the underlying c library (xdrfile) uses
