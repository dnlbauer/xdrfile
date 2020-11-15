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
//!
//! fn main() -> Result<()> {
//!     // get a handle to the file
//!     let mut trj = XTCTrajectory::open_read("tests/1l2y.xtc")?;
//!
//!     // find number of atoms in the file
//!     let num_atoms = trj.get_num_atoms()?;
//!
//!     // a frame object is used to get to read or write from a trajectory
//!     // without instantiating data arrays for every step
//!     let mut frame = Frame::with_len(num_atoms);
//!
//!     // read the first frame of the trajectory
//!     trj.read(&mut frame)?;
//!
//!     assert_eq!(frame.step, 1);
//!     assert_eq!(frame.len(), num_atoms);
//!
//!     let first_atom_coords = frame.coords[0];
//!     assert_eq!(first_atom_coords, [-0.8901, 0.4127, -0.055499997]);
//!
//!     Ok(())
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
//!
//! fn main() -> Result<()> {
//!     // get a handle to the file
//!     let trj = XTCTrajectory::open_read("tests/1l2y.xtc")?;
//!
//!     // iterate over all frames
//!     for (idx, result) in trj.into_iter().enumerate() {
//!         let frame = result?;
//!         println!("{}", frame.time);
//!         assert_eq!(idx+1, frame.step);
//!     }
//!     Ok(())
//! }
//! ```

#[cfg(test)]
#[macro_use]
extern crate assert_approx_eq;
extern crate lazy_init;

pub mod c_abi;
mod errors;
mod frame;
mod iterator;
pub use errors::*;
pub use frame::Frame;
pub use iterator::*;

use c_abi::xdr_seek;
use c_abi::xdrfile;
use c_abi::xdrfile::XDRFILE;
use c_abi::xdrfile_trr;
use c_abi::xdrfile_xtc;

use lazy_init::Lazy;
use std::cell::Cell;
use std::convert::{TryFrom, TryInto};
use std::ffi::CString;
use std::io;
use std::io::SeekFrom;
use std::os::raw::{c_float, c_int};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub enum FileMode {
    Write,
    Append,
    Read,
}

impl FileMode {
    /// Get a CStr slice corresponding to the file mode
    fn to_cstr(&self) -> &'static std::ffi::CStr {
        let bytes: &[u8; 2] = match *self {
            FileMode::Write => b"w\0",
            FileMode::Append => b"a\0",
            FileMode::Read => b"r\0",
        };

        std::ffi::CStr::from_bytes_with_nul(bytes).expect("CStr::from_bytes_with_nul failed")
    }
}

fn path_to_cstring(path: impl AsRef<Path>) -> Result<CString> {
    let s = path.as_ref().to_str().ok_or(Error::InvalidOsStr)?;
    Ok(CString::new(s)?)
}

fn to_c_int(value: usize, task: ErrorTask) -> Result<c_int> {
    value.try_into().map_err(|e| Error::CastToCintFailed {
        source: e,
        value,
        task,
    })
}

/// Convert an error code from a C call to an Error
///
/// `code` should be an integer return code returned from the C API.
/// If `code` indicates the function returned successfully, None is returned;
/// otherwise, the code is converted into the appropriate `Error`.
pub fn check_code(code: impl Into<ErrorCode>, task: ErrorTask) -> Option<Error> {
    let code: ErrorCode = code.into();
    if let ErrorCode::ExdrOk = code {
        None
    } else {
        Some(Error::from((code, task)))
    }
}

/// A safe wrapper around the c implementation of an XDRFile
struct XDRFile {
    xdrfile: *mut XDRFILE,
    #[allow(dead_code)]
    filemode: FileMode,
    path: PathBuf,
}

impl XDRFile {
    pub fn open(path: impl AsRef<Path>, filemode: FileMode) -> Result<XDRFile> {
        let path = path.as_ref();
        unsafe {
            let path_p = path_to_cstring(path)?.into_raw();
            // SAFETY: mode_p must not be mutated by the C code
            let mode_p = filemode.to_cstr().as_ptr();

            let xdrfile = xdrfile::xdrfile_open(path_p, mode_p);

            // Reconstitute the CString so it is deallocated correctly
            let _ = CString::from_raw(path_p);

            if !xdrfile.is_null() {
                let path = path.to_owned();
                Ok(XDRFile {
                    xdrfile,
                    filemode,
                    path,
                })
            } else {
                // Something went wrong. But the C api does not tell us what
                Err((path, filemode))?
            }
        }
    }

    /// Get the current position in the file
    pub fn tell(&self) -> u64 {
        unsafe {
            xdr_seek::xdr_tell(self.xdrfile)
                .try_into()
                .expect("i64 could not be converted to u64")
        }
    }
}

impl io::Seek for XDRFile {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        let (whence, pos) = match pos {
            SeekFrom::Start(u) => (
                0,
                i64::try_from(u).expect("Seek position did not fit in i64"),
            ),
            SeekFrom::Current(i) => (1, i),
            SeekFrom::End(i) => (2, i),
        };
        unsafe {
            let code = xdr_seek::xdr_seek(self.xdrfile, pos, whence);
            match check_code(code, ErrorTask::Seek) {
                None => Ok(self.tell()),
                Some(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
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
    fn get_num_atoms(&mut self) -> Result<usize>;
}

/// Read/Write XTC Trajectories
pub struct XTCTrajectory {
    handle: XDRFile,
    precision: Cell<c_float>, // internal mutability required for read method
    num_atoms: Lazy<Result<usize>>,
}

impl XTCTrajectory {
    pub fn open(path: impl AsRef<Path>, filemode: FileMode) -> Result<XTCTrajectory> {
        let xdr = XDRFile::open(path, filemode)?;
        Ok(XTCTrajectory {
            handle: xdr,
            precision: Cell::new(1000.0),
            num_atoms: Lazy::new(),
        })
    }

    /// Open a file in read mode
    pub fn open_read(path: impl AsRef<Path>) -> Result<Self> {
        Self::open(path, FileMode::Read)
    }

    /// Open a file in append mode
    pub fn open_append(path: impl AsRef<Path>) -> Result<Self> {
        Self::open(path, FileMode::Append)
    }

    /// Open a file in write mode
    pub fn open_write(path: impl AsRef<Path>) -> Result<Self> {
        Self::open(path, FileMode::Write)
    }
}

impl Trajectory for XTCTrajectory {
    fn read(&mut self, frame: &mut Frame) -> Result<()> {
        let mut step: c_int = 0;

        let num_atoms = self
            .get_num_atoms()
            .map_err(|e| Error::CouldNotCheckNAtoms(Box::new(e)))?;
        if num_atoms != frame.coords.len() {
            Err((&*frame, num_atoms))?;
        };

        unsafe {
            let code = xdrfile_xtc::read_xtc(
                self.handle.xdrfile,
                to_c_int(num_atoms, ErrorTask::Read)?,
                &mut step,
                &mut frame.time,
                &mut frame.box_vector,
                frame.coords.as_mut_ptr(),
                &mut self.precision.get(),
            );
            if let Some(err) = check_code(code, ErrorTask::Read) {
                return Err(err);
            }
            frame.step = usize::try_from(step).map_err(|_| Error::StepSizeOutOfRange(step))?;
            Ok(())
        }
    }

    fn write(&mut self, frame: &Frame) -> Result<()> {
        unsafe {
            let code = xdrfile_xtc::write_xtc(
                self.handle.xdrfile,
                to_c_int(frame.len(), ErrorTask::Write)?,
                to_c_int(frame.step, ErrorTask::Write)?,
                frame.time,
                &frame.box_vector,
                frame.coords.as_ptr(),
                1000.0,
            );
            if let Some(err) = check_code(code, ErrorTask::Write) {
                Err(err)
            } else {
                Ok(())
            }
        }
    }

    fn flush(&mut self) -> Result<()> {
        unsafe {
            let code = xdr_seek::xdr_flush(self.handle.xdrfile);
            if let Some(err) = check_code(code, ErrorTask::Read) {
                Err(err)
            } else {
                Ok(())
            }
        }
    }

    fn get_num_atoms(&mut self) -> Result<usize> {
        self.num_atoms
            .get_or_create(|| {
                let mut num_atoms: c_int = 0;

                unsafe {
                    let path = path_to_cstring(&self.handle.path)?;
                    let path_p = path.into_raw();
                    let code = xdrfile_xtc::read_xtc_natoms(path_p, &mut num_atoms);
                    // Reconstitute the CString so it is deallocated correctly
                    let _ = CString::from_raw(path_p);

                    if let Some(err) = check_code(code, ErrorTask::ReadNumAtoms) {
                        Err(err)
                    } else {
                        Ok(usize::try_from(num_atoms)
                            .expect("Number of atoms in file does not fit in usize"))
                    }
                }
            })
            .clone()
    }
}

impl XTCTrajectory {
    /// Get the current position in the file
    pub fn tell(&self) -> u64 {
        self.handle.tell()
    }
}

impl io::Seek for XTCTrajectory {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.handle.seek(pos)
    }
}

/// Read/Write TRR Trajectories
pub struct TRRTrajectory {
    handle: XDRFile,
    num_atoms: Lazy<Result<usize>>,
}

impl TRRTrajectory {
    pub fn open(path: impl AsRef<Path>, filemode: FileMode) -> Result<TRRTrajectory> {
        let xdr = XDRFile::open(path, filemode)?;
        Ok(TRRTrajectory {
            handle: xdr,
            num_atoms: Lazy::new(),
        })
    }

    /// Open a file in read mode
    pub fn open_read(path: impl AsRef<Path>) -> Result<Self> {
        Self::open(path, FileMode::Read)
    }

    /// Open a file in append mode
    pub fn open_append(path: impl AsRef<Path>) -> Result<Self> {
        Self::open(path, FileMode::Append)
    }

    /// Open a file in write mode
    pub fn open_write(path: impl AsRef<Path>) -> Result<Self> {
        Self::open(path, FileMode::Write)
    }
}

impl Trajectory for TRRTrajectory {
    fn read(&mut self, frame: &mut Frame) -> Result<()> {
        let mut step: c_int = 0;
        let mut lambda: c_float = 0.0;

        let num_atoms = self
            .get_num_atoms()
            .map_err(|e| Error::CouldNotCheckNAtoms(Box::new(e)))?;
        if num_atoms != frame.coords.len() {
            Err((&*frame, num_atoms))?;
        }

        unsafe {
            let code = xdrfile_trr::read_trr(
                self.handle.xdrfile,
                to_c_int(num_atoms, ErrorTask::Read)?,
                &mut step,
                &mut frame.time,
                &mut lambda,
                &mut frame.box_vector,
                frame.coords.as_mut_ptr(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            if let Some(err) = check_code(code, ErrorTask::Read) {
                return Err(err);
            }
            frame.step = usize::try_from(step).map_err(|_| Error::StepSizeOutOfRange(step))?;
            Ok(())
        }
    }

    fn write(&mut self, frame: &Frame) -> Result<()> {
        unsafe {
            let code = xdrfile_trr::write_trr(
                self.handle.xdrfile,
                to_c_int(frame.len(), ErrorTask::Write)?,
                to_c_int(frame.step, ErrorTask::Write)?,
                frame.time,
                0.0,
                &frame.box_vector,
                frame.coords[..].as_ptr(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            if let Some(err) = check_code(code, ErrorTask::Write) {
                Err(err)
            } else {
                Ok(())
            }
        }
    }

    fn flush(&mut self) -> Result<()> {
        unsafe {
            let code = xdr_seek::xdr_flush(self.handle.xdrfile);
            if let Some(err) = check_code(code, ErrorTask::Flush) {
                Err(err)
            } else {
                Ok(())
            }
        }
    }

    fn get_num_atoms(&mut self) -> Result<usize> {
        self.num_atoms
            .get_or_create(|| {
                let mut num_atoms: c_int = 0;
                unsafe {
                    let path = path_to_cstring(&self.handle.path)?;
                    let path_p = path.into_raw();
                    let code = xdrfile_trr::read_trr_natoms(path_p, &mut num_atoms);
                    // Reconstitute the CString so it is deallocated correctly
                    let _ = CString::from_raw(path_p);

                    if let Some(err) = check_code(code, ErrorTask::ReadNumAtoms) {
                        Err(err)
                    } else {
                        Ok(usize::try_from(num_atoms)
                            .expect("Number of atoms in file does not fit in usize"))
                    }
                }
            })
            .clone()
    }
}

impl TRRTrajectory {
    /// Get the current position in the file
    pub fn tell(&self) -> u64 {
        self.handle.tell()
    }
}

impl io::Seek for TRRTrajectory {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.handle.seek(pos)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::io::Seek;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_read_write_xtc() -> Result<()> {
        let tempfile = NamedTempFile::new().expect("Could not create temporary file");
        let tmp_path = tempfile.path();

        let natoms = 2;
        let frame = Frame {
            step: 5,
            time: 2.0,
            box_vector: [[1.0, 2.0, 3.0], [2.0, 1.0, 3.0], [3.0, 2.0, 1.0]],
            coords: vec![[1.0, 1.0, 1.0], [1.0, 1.0, 1.0]],
        };
        let mut f = XTCTrajectory::open_write(&tmp_path)?;
        let write_status = f.write(&frame);
        match write_status {
            Err(_) => panic!("Failed"),
            Ok(()) => {}
        }
        f.flush()?;

        let mut new_frame = Frame::with_len(natoms);
        let mut f = XTCTrajectory::open_read(tmp_path)?;
        let num_atoms = f.get_num_atoms()?;
        assert_eq!(num_atoms, natoms);

        let read_status = f.read(&mut new_frame);
        match read_status {
            Err(e) => assert!(false, "{:?}", e),
            Ok(()) => {}
        }

        assert_eq!(new_frame.len(), frame.len());
        assert_eq!(new_frame.step, frame.step);
        assert_approx_eq!(new_frame.time, frame.time);
        assert_eq!(new_frame.box_vector, frame.box_vector);
        assert_eq!(new_frame.coords, frame.coords);
        Ok(())
    }

    #[test]
    fn test_read_write_trr() -> Result<()> {
        let tempfile = NamedTempFile::new().expect("Could not create temporary file");
        let tmp_path = tempfile.path();

        let natoms = 2;
        let frame = Frame {
            step: 5,
            time: 2.0,
            box_vector: [[1.0, 2.0, 3.0], [2.0, 1.0, 3.0], [3.0, 2.0, 1.0]],
            coords: vec![[1.0, 1.0, 1.0], [1.0, 1.0, 1.0]],
        };
        let mut f = TRRTrajectory::open_write(tmp_path)?;
        let write_status = f.write(&frame);
        match write_status {
            Err(_) => panic!("Failed"),
            Ok(()) => {}
        }
        f.flush()?;

        let mut new_frame = Frame::with_len(natoms);
        let mut f = TRRTrajectory::open_read(tmp_path)?;
        // let num_atoms = f.get_num_atoms()?;
        // assert_eq!(num_atoms, natoms);

        let read_status = f.read(&mut new_frame);
        match read_status {
            Err(e) => assert!(false, "{:?}", e),
            Ok(()) => {}
        }

        assert_eq!(new_frame.len(), frame.len());
        assert_eq!(new_frame.step, frame.step);
        assert_eq!(new_frame.time, frame.time);
        assert_eq!(new_frame.box_vector, frame.box_vector);
        assert_eq!(new_frame.coords, frame.coords);
        Ok(())
    }

    #[test]
    pub fn test_manual_loop() -> Result<(), Box<dyn std::error::Error>> {
        let mut xtc_frames = Vec::new();
        let mut xtc_traj = XTCTrajectory::open_read("tests/1l2y.xtc")?;
        let mut frame = Frame::with_len(xtc_traj.get_num_atoms()?);

        while let Ok(()) = xtc_traj.read(&mut frame) {
            xtc_frames.push(frame.clone());
        }

        let mut trr_frames = Vec::new();
        let mut trr_traj = TRRTrajectory::open_read("tests/1l2y.trr")?;

        while let Ok(()) = trr_traj.read(&mut frame) {
            trr_frames.push(frame.clone());
        }

        for (xtc, trr) in xtc_frames.into_iter().zip(trr_frames) {
            assert_eq!(xtc.len(), trr.len());
            assert_eq!(xtc.step, trr.step);
            assert_eq!(xtc.time, trr.time);
            assert_eq!(xtc.box_vector, trr.box_vector);
            for (xtc_xyz, trr_xyz) in xtc.coords.into_iter().zip(trr.coords) {
                assert!(xtc_xyz[0] - trr_xyz[0] <= 1e-5);
                assert!(xtc_xyz[1] - trr_xyz[1] <= 1e-5);
                assert!(xtc_xyz[2] - trr_xyz[2] <= 1e-5);
            }
        }
        Ok(())
    }

    #[test]
    pub fn test_wrong_size_frame() -> Result<(), Box<dyn std::error::Error>> {
        let mut xtc_traj = XTCTrajectory::open_read("tests/1l2y.xtc")?;
        let mut frame = Frame::new();

        let result = xtc_traj.read(&mut frame);
        if let Err(e) = result {
            assert!(matches!(e, Error::WrongSizeFrame { .. }));
        } else {
            panic!("A read with an incorrectly sized frame should not succeed")
        }
        Ok(())
    }

    #[test]
    fn test_path_to_cstring() -> Result<(), Box<dyn std::error::Error>> {
        let result_invalid = path_to_cstring(PathBuf::from("invalid/\0path"));

        if let Err(err) = result_invalid {
            match err {
                Error::NullInStr(_) => (),
                Error::InvalidOsStr => (),
                _ => panic!("Improper error type for path_to_cstring"),
            }
        } else {
            panic!("path_to_cstring should return Err if there are null bytes");
        }

        let result_valid = path_to_cstring("valid/path");
        assert_eq!(result_valid, Ok(CString::new("valid/path")?));

        Ok(())
    }

    #[test]
    fn test_tell() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let tempfile = NamedTempFile::new()?;
        let tmp_path = tempfile.path();

        let natoms: usize = 2;
        let frame = Frame {
            step: 5,
            time: 2.0,
            box_vector: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            coords: vec![[0.0, 0.0, 0.0], [0.5, 0.5, 0.5]],
        };
        let mut f = TRRTrajectory::open_write(tmp_path)?;
        assert_eq!(f.tell(), 0);
        f.write(&frame)?;
        assert_eq!(f.tell(), 144);
        f.flush()?;

        let mut new_frame = Frame::with_len(natoms);
        let mut f = TRRTrajectory::open_read(tmp_path)?;
        assert_eq!(f.tell(), 0);

        f.read(&mut new_frame)?;
        assert_eq!(f.tell(), 144);

        Ok(())
    }

    #[test]
    fn test_seek() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let tempfile = NamedTempFile::new()?;
        let tmp_path = tempfile.path();

        let natoms: usize = 2;
        let mut frame = Frame {
            step: 0,
            time: 0.0,
            box_vector: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            coords: vec![[0.0, 0.0, 0.0], [0.5, 0.5, 0.5]],
        };
        let mut f = TRRTrajectory::open_write(tmp_path)?;
        f.write(&frame)?;
        let after_first_frame = f.tell();
        frame.step += 1;
        frame.time += 10.0;
        f.write(&frame)?;
        let after_second_frame = f.tell();
        f.flush()?;

        let mut new_frame = Frame::with_len(natoms);
        let mut f = TRRTrajectory::open_read(tmp_path)?;
        let pos = f.seek(std::io::SeekFrom::Current(144))?;
        assert_eq!(pos, after_first_frame);

        f.read(&mut new_frame)?;
        assert_eq!(f.tell(), after_second_frame);

        assert_eq!(new_frame.len(), frame.len());
        assert_eq!(new_frame.step, frame.step);
        assert_eq!(new_frame.time, frame.time);
        assert_eq!(new_frame.box_vector, frame.box_vector);
        assert_eq!(new_frame.coords, frame.coords);

        Ok(())
    }

    #[test]
    fn test_err_could_not_open() {
        let file_name = "non-existent.xtc";

        let path = Path::new(&file_name);
        if let Err(e) = XDRFile::open(file_name, FileMode::Read) {
            if let Error::CouldNotOpen {
                path: err_path,
                mode: err_mode,
            } = e
            {
                assert_eq!(path, err_path);
                assert_eq!(FileMode::Read, err_mode)
            } else {
                panic!("Wrong Error type")
            }
        }
    }

    #[test]
    fn test_err_could_not_read_atom_nr() -> Result<()> {
        let file_name = "README.md"; // not a trajectory
        let mut trr = TRRTrajectory::open_read(file_name)?;
        if let Err(e) = trr.get_num_atoms() {
            assert_eq!(Some(ErrorCode::ExdrMagic), e.code());
        } else {
            panic!("Should not be able to read number of atoms from readme");
        }
        Ok(())
    }

    #[test]
    fn test_err_could_not_read() -> Result<()> {
        let file_name = "README.md"; // not a trajectory
        let mut frame = Frame::with_len(1);
        let mut trr = TRRTrajectory::open_read(file_name)?;
        if let Err(e) = trr.read(&mut frame) {
            assert_eq!(Some(ErrorCode::ExdrMagic), e.code());
        } else {
            panic!("Should not be able to read number of atoms from readme");
        }
        Ok(())
    }

    #[test]
    fn test_err_file_eof() -> Result<(), Box<dyn std::error::Error>> {
        let tempfile = NamedTempFile::new()?;
        let tmp_path = tempfile.path();

        let natoms = 2;
        let frame = Frame {
            step: 5,
            time: 2.0,
            box_vector: [[1.0, 2.0, 3.0], [2.0, 1.0, 3.0], [3.0, 2.0, 1.0]],
            coords: vec![[1.0, 1.0, 1.0], [1.0, 1.0, 1.0]],
        };
        let mut f = XTCTrajectory::open_write(&tmp_path)?;
        f.write(&frame)?;
        f.flush()?;

        let mut new_frame = Frame::with_len(natoms);
        let mut f = XTCTrajectory::open_read(tmp_path)?;

        f.read(&mut new_frame)?;

        let result = f.read(&mut new_frame); // Should be eof as we only wrote one frame
        if let Err(e) = result {
            assert!(e.is_eof());
        } else {
            panic!("read two frames after writing one");
        }

        let mut file = std::fs::File::create(tmp_path)?;
        file.write_all(&[0; 999])?;
        file.flush()?;

        let mut f = XTCTrajectory::open_read(tmp_path)?;
        let result = f.read(&mut new_frame); // Should be an invalid XTC file
        if let Err(e) = result {
            assert!(!e.is_eof());
        } else {
            panic!("999 zero bytes was read as a valid XTC file");
        }

        Ok(())
    }

    #[test]
    fn test_check_code() {
        let code: ErrorCode = 0.into();
        assert!(!check_code(code, ErrorTask::Read).is_some());

        for i in vec![1, 10, 100, 1000] {
            let code: ErrorCode = i.into();
            assert!(check_code(code, ErrorTask::Read).is_some());
        }
    }

    #[test]
    fn test_to_c_int() -> Result<()> {
        assert_eq!(24234 as c_int, to_c_int(24234_usize, ErrorTask::Read)?);

        let try_from_int_err = match u8::try_from(-1) {
            Err(e) => e,
            _ => panic!("Conversion from -1 to u8 succeeded"),
        };
        let expected = Error::CastToCintFailed {
            source: try_from_int_err,
            task: ErrorTask::Write,
            value: 3_294_967_295_usize,
        };
        assert_eq!(
            Err(expected),
            to_c_int(3_294_967_295_usize, ErrorTask::Write)
        );

        Ok(())
    }
}
