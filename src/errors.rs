use crate::c_abi;
use crate::FileMode;
use crate::Frame;
use std::error::Error as StdError;
use std::path::{Path, PathBuf};

/// Error type for the xdrfile library
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// An error code from the C API
    CApiError { code: ErrorCode, task: ErrorTask },
    /// Passed in a frame of the wrong size
    WrongSizeFrame { expected: usize, found: usize },
    /// C API failed to open a file (No return code provided)
    CouldNotOpen { path: PathBuf, mode: FileMode },
    /// A path could not be converted to &OsStr
    InvalidOsStr(Option<std::ffi::NulError>),
    /// Checking the number of atoms failed while reading a frame
    CouldNotCheckNAtoms(Box<Error>),
    /// Error for an out-of-range numeric conversion
    OutOfRange {
        name: &'static str,
        task: ErrorTask,
        value: String,
        target: &'static str,
    },
    /// Attempted to perform an unsupported operation given the file mode
    WrongMode { mode: FileMode, task: ErrorTask },
}

impl Error {
    /// Get the error code returned by the C API, if any
    pub fn code(&self) -> Option<ErrorCode> {
        if let Error::CApiError { code, .. } = self {
            Some(*code)
        } else if let Some(e) = self.source() {
            e.downcast_ref::<Self>().and_then(Self::code)
        } else {
            None
        }
    }

    /// Get the task being attempted when the C API returned an error, if any
    pub fn task(&self) -> Option<ErrorTask> {
        if let Error::CApiError { task, .. } = self {
            Some(*task)
        } else if let Some(e) = self.source() {
            e.downcast_ref::<Self>().and_then(Self::task)
        } else {
            None
        }
    }

    /// True if the error is an end of file error, false otherwise
    pub fn is_eof(&self) -> bool {
        self.code().map_or(false, |e| e.is_eof())
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Error::InvalidOsStr(err) => {
                if let Some(err) = err {
                    Some(err)
                } else {
                    None
                }
            }
            Error::CouldNotCheckNAtoms(err) => Some(err.as_ref()),
            _ => None,
        }
    }
}

impl From<(ErrorCode, ErrorTask)> for Error {
    fn from(value: (ErrorCode, ErrorTask)) -> Self {
        let (code, task) = value;
        Self::CApiError { code, task }
    }
}

impl From<(&Path, FileMode)> for Error {
    fn from(value: (&Path, FileMode)) -> Self {
        let (path, mode) = value;
        Error::CouldNotOpen {
            path: path.to_owned(),
            mode,
        }
    }
}

impl From<(&Frame, usize)> for Error {
    fn from(value: (&Frame, usize)) -> Self {
        let (frame, num_atoms) = value;
        Error::WrongSizeFrame {
            expected: num_atoms,
            found: frame.coords.len(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::CApiError { code, task } => write!(
                f,
                "Error while {task}: C API returned error code {code}",
                task = task,
                code = code
            ),
            Error::WrongSizeFrame { expected, found } => write!(
                f,
                "Expected frame of size {:?}, found {:?}",
                expected, found
            ),
            Error::CouldNotOpen { path, mode } => {
                write!(f, "Could not open file at {:?} in mode {:?}", path, mode)
            }
            Error::InvalidOsStr(_) => write!(f, "Cannot convert path to CString."),
            Error::CouldNotCheckNAtoms(_) => {
                write!(f, "Failed to read number of atoms in trajectory file")
            }
            Error::OutOfRange {
                name,
                task,
                value,
                target,
            } => write!(
                f,
                "Illegal {name} while {task}: Failed to cast {value} to {target}",
                name = name,
                task = task,
                value = value,
                target = target
            ),
            Error::WrongMode { mode, task } => write!(
                f,
                "{task} is impossible with file mode {mode:?}",
                mode = mode,
                task = task.uppercase_first(),
            ),
        }
    }
}

/// The task being attempted when the C API returns an error
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorTask {
    /// The number of atoms was being read from a file
    ReadNumAtoms,
    /// A frame was being read from a file
    Read,
    /// A frame was being written to a file
    Write,
    /// A file was being flushed to disk
    Flush,
    /// A seek operation was being run on a file
    Seek,
}

impl ErrorTask {
    fn uppercase_first(&self) -> String {
        let mut s = format!("{}", self);
        &mut s[0..1].make_ascii_uppercase();
        s
    }
}

impl std::fmt::Display for ErrorTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            ErrorTask::ReadNumAtoms => write!(f, "reading atom number from trajectory"),
            ErrorTask::Read => write!(f, "reading trajectory"),
            ErrorTask::Write => write!(f, "writing trajectory"),
            ErrorTask::Flush => write!(f, "flushing trajectory"),
            ErrorTask::Seek => write!(f, "seeking in trajectory"),
        }
    }
}

/// Error codes returned from the C API
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum ErrorCode {
    /// No error, C API returned successfully
    ExdrOk,
    /// TRR file had corrupt Header
    ExdrHeader,
    /// Failed to read a string where expected
    ExdrString,
    /// Failed to read a double where expected
    ExdrDouble,
    /// Failed to read an int where expected
    ExdrInt,
    /// Failed to read a float where expected
    ExdrFloat,
    /// Failed to read a uint where expected
    ExdrUint,
    /// Failed to read compressed XTC coordinates
    Exdr3dx,
    /// Error encountered while closing file
    ExdrClose,
    /// File had incorrect "magic number" (not an XTC file)
    ExdrMagic,
    /// Failed to allocate memory for TRR file
    ExdrNoMem,
    /// End of file was reached while trying to read
    ExdrEndOfFile,
    /// File was not found when trying to open
    ExdrFileNotFound,
    /// Failed to seek within file
    ExdrNr,
    /// Something unexpected happened
    UnmatchedCode(i32),
}

impl ErrorCode {
    /// True if the error is an end of file error, false otherwise
    pub fn is_eof(&self) -> bool {
        matches!(self, Self::ExdrEndOfFile)
    }
}

impl From<i32> for ErrorCode {
    fn from(code: i32) -> Self {
        match code {
            c_abi::xdrfile::exdrOK => Self::ExdrOk,
            c_abi::xdrfile::exdrHEADER => Self::ExdrHeader,
            c_abi::xdrfile::exdrSTRING => Self::ExdrString,
            c_abi::xdrfile::exdrDOUBLE => Self::ExdrDouble,
            c_abi::xdrfile::exdrINT => Self::ExdrInt,
            c_abi::xdrfile::exdrFLOAT => Self::ExdrFloat,
            c_abi::xdrfile::exdrUINT => Self::ExdrUint,
            c_abi::xdrfile::exdr3DX => Self::Exdr3dx,
            c_abi::xdrfile::exdrCLOSE => Self::ExdrClose,
            c_abi::xdrfile::exdrMAGIC => Self::ExdrMagic,
            c_abi::xdrfile::exdrNOMEM => Self::ExdrNoMem,
            c_abi::xdrfile::exdrENDOFFILE => Self::ExdrEndOfFile,
            c_abi::xdrfile::exdrFILENOTFOUND => Self::ExdrFileNotFound,
            c_abi::xdrfile::exdrNR => Self::ExdrNr,
            code => Self::UnmatchedCode(code),
        }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Self::UnmatchedCode(i) = self {
            write!(f, "{}", i)
        } else {
            write!(f, "{:?}", self)
        }
    }
}

/// `Result` type for errors in the `xdrfile` crate
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_eof() {
        let error = Error::CApiError {
            code: c_abi::xdrfile::exdrENDOFFILE.into(),
            task: ErrorTask::Read,
        };
        assert!(error.is_eof());

        let error = Error::CApiError {
            code: ErrorCode::ExdrEndOfFile,
            task: ErrorTask::Read,
        };
        assert!(error.is_eof());

        let error = Error::CApiError {
            code: (c_abi::xdrfile::exdrENDOFFILE + 1).into(),
            task: ErrorTask::Read,
        };
        assert!(!error.is_eof());

        let error = Error::CApiError {
            code: 0.into(),
            task: ErrorTask::Read,
        };
        assert!(!error.is_eof());

        let error = Error::CApiError {
            code: 255.into(),
            task: ErrorTask::Read,
        };
        assert!(!error.is_eof());

        let error = Error::CouldNotOpen {
            path: PathBuf::from("not/a/file"),
            mode: FileMode::Read,
        };
        assert!(!error.is_eof());
    }

    #[test]
    fn test_from_correct_type() {
        let code = 3.into();
        let task = ErrorTask::Read;
        let expected = Error::CApiError { code, task };
        let err = Error::from((code, task));
        assert_eq!(expected, err);

        let path = Path::new(".");
        let mode = FileMode::Read;
        let expected = Error::CouldNotOpen {
            path: path.to_path_buf(),
            mode: mode.to_owned(),
        };
        let err = Error::from((path, mode));
        assert_eq!(expected, err);

        let frame = Frame::with_len(0);
        let expected = Error::WrongSizeFrame {
            expected: 10,
            found: 0,
        };
        let err = Error::from((&frame, 10));
        assert_eq!(expected, err);
    }
}
