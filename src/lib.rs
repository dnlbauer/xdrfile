//! # xdrfile
//! Read and write xdr trajectory files in .xtc and .trr file format
//!
//! This crate is mainly intended to be a wrapper around the GROMACS libxdrfile
//! XTC library and provides basic functionality to read and write xtc and trr
//! files with a safe api.
//!
//! # Basic usage example
//! ```rust
//! use xdrfile::*;
//! use std::path::Path;
//!
//! let mut path = Path::new("tests/1l2y.xtc");
//! // get a handle to the file
//! let mut trj = XTCTrajectory::open(path, FileMode::Read).unwrap();
//!
//! // find number of atoms in the file
//! let num_atoms = trj.get_num_atoms().unwrap();
//!
//! // a frame object is used to get to read or write from a trajectory
//! // without instantiating data arrays for every step
//! let mut frame = Frame::with_capacity(num_atoms);
//!
//! // read the first frame of the trajectory
//! let result = trj.read(&mut frame);
//! match result {
//!    Ok(_) => {
//!        assert_eq!(frame.step, 1);
//!        assert_eq!(frame.num_atoms, num_atoms);
//!
//!        let first_atom_coords = frame.coords[0];
//!        assert_eq!(first_atom_coords, [-0.8901, 0.4127, -0.055499997]);
//!    }
//!    Err(msg) => {
//!        panic!("Something went wrong: {}", msg);
//!    }
//! }
//! ```
//!
//! # Frame iteration
//! For convenience, the trajectory implementations provide "into_iter" to
//! be turned into an iterator that yields Rc<Frame>. If a frame is not kept
//! during iteration, the Iterator reuses it for better performance (and hence,
//! Rc is required)
//!
//! ```rust
//! use xdrfile::*;
//! use std::path::Path;
//!
//! let mut path = Path::new("tests/1l2y.xtc");
//! // get a handle to the file
//! let trj = XTCTrajectory::open(path, FileMode::Read).unwrap();
//!
//! // iterate over all frames
//! for (idx, frame) in trj.into_iter().filter_map(Result::ok).enumerate() {
//!     println!("{}", frame.time);
//!     assert_eq!(idx+1, frame.step as usize);
//! }
//! ```

#[cfg(test)]
#[macro_use]
extern crate assert_approx_eq;
extern crate lazy_init;

pub mod c_abi;
mod frame;
mod iterator;
pub use frame::Frame;
pub use iterator::*;

use c_abi::xdr_seek;
use c_abi::xdrfile;
use c_abi::xdrfile::XDRFILE;
use c_abi::xdrfile_trr;
use c_abi::xdrfile_xtc;

use lazy_init::Lazy;
use std::cell::Cell;
use std::ffi::CString;
use std::path::Path;

#[derive(Debug, Clone)]
pub enum Error {
    CouldNotOpenFile(std::path::PathBuf, FileMode),
    CouldNotReadAtomNumber(u32),
    CouldNotRead(u32),
    CouldNotWrite(u32),
    CouldNotFlush(u32),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            CouldNotOpenFile(path, mode) => write!(
                f,
                "Failed to open file at {path:?} with mode {mode:?}",
                path = path,
                mode = mode
            ),
            CouldNotReadAtomNumber(code) => write!(
                f,
                "Failed to read atom number from trajectory: C API returned error code {}",
                code
            ),
            CouldNotRead(code) => write!(
                f,
                "Failed to read trajectory: C API returned error code {}",
                code
            ),
            CouldNotWrite(code) => write!(
                f,
                "Failed to write trajectory: C API returned error code {}",
                code
            ),
            CouldNotFlush(code) => write!(
                f,
                "Failed to flush trajectory: C API returned error code {}",
                code
            ),
        }
    }
}

impl std::error::Error for Error {}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq)]
pub enum FileMode {
    Write,
    Append,
    Read,
}

impl FileMode {
    pub fn value(&self) -> &str {
        match *self {
            FileMode::Write => "w",
            FileMode::Append => "a",
            FileMode::Read => "r",
        }
    }
}

fn path_to_cstring(path: &Path) -> CString {
    CString::new(path.to_str().unwrap()).unwrap()
}

/// A safe wrapper around the c implementation of an XDRFile
struct XDRFile {
    xdrfile: *mut XDRFILE,
    #[allow(dead_code)]
    filemode: FileMode,
    path: String,
}

impl XDRFile {
    pub fn open(path: &Path, filemode: FileMode) -> Result<XDRFile> {
        let path_p = path_to_cstring(path).into_raw();
        let mode_p = CString::new(filemode.value()).unwrap().into_raw();

        unsafe {
            let xdrfile = xdrfile::xdrfile_open(path_p, mode_p);

            if !xdrfile.is_null() {
                let path = String::from(path.to_str().unwrap());
                Ok(XDRFile {
                    xdrfile,
                    filemode,
                    path,
                })
            } else {
                // Something went wrong. But the C api does not tell us what
                Err(Error::CouldNotOpenFile(path.into(), filemode))
            }
        }
    }
}

impl Drop for XDRFile {
    /// Close the underlying xdr file on drop
    fn drop(&mut self) {
        unsafe {
            xdrfile::xdrfile_close(self.xdrfile);
        }
    }
}

/// The trajectory trait defines shared methods for xtc and trr trajectories
pub trait Trajectory {
    /// Read the next step of the trajectory into the frame object
    fn read(&mut self, frame: &mut Frame) -> Result<()>;

    /// Write the frame to the trajectory file
    fn write(&mut self, frame: &Frame) -> Result<()>;

    /// Flush the trajectory file
    fn flush(&mut self) -> Result<()>;

    /// Get the number of atoms from the give trajectory
    fn get_num_atoms(&mut self) -> Result<u32>;
}

/// Read/Write XTC Trajectories
pub struct XTCTrajectory {
    handle: XDRFile,
    precision: Cell<f32>, // internal mutability required for read method
    num_atoms: Lazy<Result<u32>>,
}

impl XTCTrajectory {
    pub fn open(path: &Path, filemode: FileMode) -> Result<XTCTrajectory> {
        let xdr = XDRFile::open(path, filemode)?;
        Ok(XTCTrajectory {
            handle: xdr,
            precision: Cell::new(1000.0),
            num_atoms: Lazy::new(),
        })
    }
}

impl Trajectory for XTCTrajectory {
    fn read(&mut self, frame: &mut Frame) -> Result<()> {
        unsafe {
            // C lib requires an i32 to be passed, but step is exposed it as u32
            // (A step cannot be negative, can it?). So we need to create a step
            // variable to pass to read_xtc and cast it afterwards to u32
            let mut step: i32 = 0;
            let code = xdrfile_xtc::read_xtc(
                self.handle.xdrfile,
                frame.num_atoms as i32,
                &mut step,
                &mut frame.time,
                &mut frame.box_vector,
                frame.coords.as_ptr() as *mut [f32; 3],
                &mut self.precision.get(),
            ) as u32;
            frame.step = step as u32;
            match code {
                xdrfile::exdrOK => Ok(()),
                _ => Err(Error::CouldNotRead(code)),
            }
        }
    }

    fn write(&mut self, frame: &Frame) -> Result<()> {
        unsafe {
            let code = xdrfile_xtc::write_xtc(
                self.handle.xdrfile,
                frame.num_atoms as i32,
                frame.step as i32,
                frame.time,
                frame.box_vector.as_ptr() as *mut [[f32; 3]; 3],
                frame.coords[..].as_ptr() as *mut [f32; 3],
                1000.0,
            ) as u32;
            match code {
                xdrfile::exdrOK => Ok(()),
                _ => Err(Error::CouldNotWrite(code)),
            }
        }
    }

    fn flush(&mut self) -> Result<()> {
        unsafe {
            let code = xdr_seek::xdr_flush(self.handle.xdrfile) as u32;
            match code {
                xdrfile::exdrOK => Ok(()),
                _ => Err(Error::CouldNotFlush(code)),
            }
        }
    }

    fn get_num_atoms(&mut self) -> Result<u32> {
        self.num_atoms
            .get_or_create(|| {
                let mut num_atoms: i32 = 0;
                unsafe {
                    let path = CString::new(self.handle.path.as_str()).unwrap();
                    let path_p = path.into_raw();
                    let code =
                        xdrfile_xtc::read_xtc_natoms(path_p, &mut num_atoms as *const i32) as u32;
                    match code {
                        xdrfile::exdrOK => Ok(num_atoms as u32),
                        _ => Err(Error::CouldNotReadAtomNumber(code)),
                    }
                }
            })
            .clone()
    }
}

/// Read/Write TRR Trajectories
pub struct TRRTrajectory {
    handle: XDRFile,
    num_atoms: Lazy<Result<u32>>,
}

impl TRRTrajectory {
    pub fn open(path: &Path, filemode: FileMode) -> Result<TRRTrajectory> {
        let xdr = XDRFile::open(path, filemode)?;
        Ok(TRRTrajectory {
            handle: xdr,
            num_atoms: Lazy::new(),
        })
    }
}

impl Trajectory for TRRTrajectory {
    fn read(&mut self, frame: &mut Frame) -> Result<()> {
        unsafe {
            // C lib requires an i32 to be passed, but step is exposed it as u32
            // (A step cannot be negative, can it?). So we need to create a step
            // variable to pass to read_trr and cast it afterwards to u32.
            // Similar for lambda.
            let mut step: i32 = 0;
            let mut lambda: f32 = 0.0;
            let code = xdrfile_trr::read_trr(
                self.handle.xdrfile,
                frame.num_atoms as i32,
                &mut step,
                &mut frame.time,
                &mut lambda,
                &mut frame.box_vector,
                frame.coords.as_ptr() as *mut [f32; 3],
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            ) as u32;
            frame.step = step as u32;
            match code {
                xdrfile::exdrOK => Ok(()),
                _ => Err(Error::CouldNotRead(code)),
            }
        }
    }

    fn write(&mut self, frame: &Frame) -> Result<()> {
        unsafe {
            let code = xdrfile_trr::write_trr(
                self.handle.xdrfile,
                frame.num_atoms as i32,
                frame.step as i32,
                frame.time,
                0.0,
                frame.box_vector.as_ptr() as *mut [[f32; 3]; 3],
                frame.coords[..].as_ptr() as *mut [f32; 3],
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            ) as u32;
            match code {
                xdrfile::exdrOK => Ok(()),
                _ => Err(Error::CouldNotWrite(code)),
            }
        }
    }

    fn flush(&mut self) -> Result<()> {
        unsafe {
            let code = xdr_seek::xdr_flush(self.handle.xdrfile) as u32;
            match code {
                xdrfile::exdrOK => Ok(()),
                _ => Err(Error::CouldNotFlush(code)),
            }
        }
    }

    fn get_num_atoms(&mut self) -> Result<u32> {
        self.num_atoms
            .get_or_create(|| {
                let mut num_atoms: i32 = 0;
                unsafe {
                    let path = CString::new(self.handle.path.as_str()).unwrap();
                    let path_p = path.into_raw();
                    let code =
                        xdrfile_trr::read_trr_natoms(path_p, &mut num_atoms as *const i32) as u32;
                    match code {
                        xdrfile::exdrOK => Ok(num_atoms as u32),
                        _ => Err(Error::CouldNotReadAtomNumber(code)),
                    }
                }
            })
            .clone()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_read_write_xtc() {
        let tempfile = NamedTempFile::new().unwrap();
        let tmp_path = tempfile.path();

        let natoms: u32 = 2;
        let frame = Frame {
            num_atoms: natoms,
            step: 5,
            time: 2.0,
            box_vector: [[1.0, 2.0, 3.0], [2.0, 1.0, 3.0], [3.0, 2.0, 1.0]],
            coords: vec![[1.0, 1.0, 1.0], [1.0, 1.0, 1.0]],
        };
        let mut f = XTCTrajectory::open(tmp_path, FileMode::Write).unwrap();
        let write_status = f.write(&frame);
        match write_status {
            Err(_) => panic!("Failed"),
            Ok(()) => {}
        }
        f.flush().unwrap();

        let mut new_frame = Frame::with_capacity(natoms);
        let mut f = XTCTrajectory::open(tmp_path, FileMode::Read).unwrap();
        let num_atoms = f.get_num_atoms().unwrap();
        assert_eq!(num_atoms, natoms);

        let read_status = f.read(&mut new_frame);
        match read_status {
            Err(e) => assert!(false, "{:?}", e),
            Ok(()) => {}
        }

        assert_eq!(new_frame.num_atoms, frame.num_atoms);
        assert_eq!(new_frame.step, frame.step);
        assert_approx_eq!(new_frame.time, frame.time);
        assert_eq!(new_frame.box_vector, frame.box_vector);
        assert_eq!(new_frame.coords, frame.coords);
    }

    #[test]
    fn test_read_write_trr() {
        let tempfile = NamedTempFile::new().unwrap();
        let tmp_path = tempfile.path();

        let natoms: u32 = 2;
        let frame = Frame {
            num_atoms: natoms,
            step: 5,
            time: 2.0,
            box_vector: [[1.0, 2.0, 3.0], [2.0, 1.0, 3.0], [3.0, 2.0, 1.0]],
            coords: vec![[1.0, 1.0, 1.0], [1.0, 1.0, 1.0]],
        };
        let mut f = TRRTrajectory::open(tmp_path, FileMode::Write).unwrap();
        let write_status = f.write(&frame);
        match write_status {
            Err(_) => panic!("Failed"),
            Ok(()) => {}
        }
        f.flush().unwrap();

        let mut new_frame = Frame::with_capacity(natoms);
        let mut f = TRRTrajectory::open(tmp_path, FileMode::Read).unwrap();
        // let num_atoms = f.get_num_atoms().unwrap();
        // assert_eq!(num_atoms, natoms);

        let read_status = f.read(&mut new_frame);
        match read_status {
            Err(e) => assert!(false, "{:?}", e),
            Ok(()) => {}
        }

        assert_eq!(new_frame.num_atoms, frame.num_atoms);
        assert_eq!(new_frame.step, frame.step);
        assert_eq!(new_frame.time, frame.time);
        assert_eq!(new_frame.box_vector, frame.box_vector);
        assert_eq!(new_frame.coords, frame.coords);
    }

    #[test]
    fn test_err_could_not_open() {
        let file_name = "non-existent.xtc";
        let path = Path::new(&file_name);
        if let Err(e) = XDRFile::open(path, FileMode::Read) {
            match e {
                Error::CouldNotOpenFile(err_path, err_mode) => {
                    assert_eq!(path, err_path);
                    assert!(FileMode::Read == err_mode)
                }
                _ => panic!("Wrong Error type"),
            }
        }
    }

    #[test]
    fn test_err_could_not_read_atom_nr() {
        let file_name = "README.md"; // not a trajectory
        let path = Path::new(&file_name);
        let mut trr = TRRTrajectory::open(path, FileMode::Read).unwrap();
        if let Err(e) = trr.get_num_atoms() {
            match e {
                Error::CouldNotReadAtomNumber(code) => {
                    assert_eq!(xdrfile::exdrMAGIC, code);
                }
                _ => panic!("Wrong Error type"),
            }
        }
    }

    #[test]
    fn test_err_could_not_read() {
        let file_name = "README.md"; // not a trajectory
        let path = Path::new(&file_name);
        let mut frame = Frame::with_capacity(1);
        let mut trr = TRRTrajectory::open(path, FileMode::Read).unwrap();
        if let Err(e) = trr.read(&mut frame) {
            match e {
                Error::CouldNotRead(code) => {
                    assert_eq!(xdrfile::exdrMAGIC, code);
                }
                _ => panic!("Wrong Error type"),
            }
        }
    }
}
