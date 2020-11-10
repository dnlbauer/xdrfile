use crate::c_abi;
use crate::FileMode;
use crate::Frame;
use std::path::{Path, PathBuf};
use std::error::Error as StdError;

/// Error type for the xdrfile library
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// An error code from the C API
    CApiError {
        code: ErrorCode,
        task: ErrorTask,
    },
    /// Passed in a frame of the wrong size
    WrongSizeFrame {
        expected: usize,
        found: usize,
    },
    /// C API failed to open a file (No return code provided)
    CouldNotOpen {
        path: PathBuf,
        mode: FileMode,
    },
    /// A path could not be converted to &OsStr
    InvalidOsStr,
    /// A path could not be converted to &CStr because it had a null byte
    NullInStr(std::ffi::NulError),
    CouldNotCheckNAtoms(Box<Error>),
}

impl Error {
    /// Get the error code returned by the C API, if any
    pub fn code(&self) -> Option<ErrorCode> {
        // use std::error::Error as _;
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
        // use std::error::Error as _;
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

    /// Convert an error code and output value from a C call to a Result
    ///
    /// `code` should be an integer return code returned from the C API. `value` should be the
    /// function's output, which is generally either `()` or one of its arguments. If `code`
    /// indicates the function returned successfully, the value is returned; otherwise, the
    /// code is converted into the appropriate `Error`.
    pub fn check_code<T>(code: impl Into<ErrorCode>, value: T, task: ErrorTask) -> Result<T, Self> {
        let code: ErrorCode = code.into();
        if let ErrorCode::ExdrOk = code {
            Ok(value)
        } else {
            Err(Self::CApiError { code, task })
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Error::NullInStr(err) => Some(err),
            Error::CouldNotCheckNAtoms(err) => Some(err.as_ref()),
            _ => None,
        }
    }
}

impl From<std::ffi::NulError> for Error {
    fn from(err: std::ffi::NulError) -> Self {
        Self::NullInStr(err)
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
        use Error::*;
        match &self {
            CApiError { code, task } => write!(
                f,
                "Error while {task}: C API returned error code {code}",
                task = task,
                code = code
            ),
            WrongSizeFrame { expected, found } => write!(
                f,
                "Expected frame of size {:?}, found {:?}",
                expected, found
            ),
            CouldNotOpen { path, mode } => {
                write!(f, "Could not open file at {:?} in mode {:?}", path, mode)
            }
            InvalidOsStr => write!(f, "Paths must be valid unicode on this platform"),
            NullInStr(_err) => write!(f, "Paths cannot include null bytes"),
            CouldNotCheckNAtoms(_err) => write!(
                f,
                "Failed to check number of atoms in trajectory while reading a frame"
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
    /// An file was being flushed to disk
    Flush,
}

impl std::fmt::Display for ErrorTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ErrorTask::*;
        match &self {
            ReadNumAtoms => write!(f, "reading atom number from trajectory"),
            Read => write!(f, "reading trajectory"),
            Write => write!(f, "writing trajectory"),
            Flush => write!(f, "flushing trajectory"),
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
    UnmatchedCode(c_abi::xdrfile::BindgenTy1),
}

impl ErrorCode {
    /// True if the error is an end of file error, false otherwise
    pub fn is_eof(&self) -> bool {
        matches!(self, Self::ExdrEndOfFile)
    }
}

impl From<c_abi::xdrfile::BindgenTy1> for ErrorCode {
    fn from(code: c_abi::xdrfile::BindgenTy1) -> Self {
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
        use Error::CApiError;
        let error = CApiError {
            code: c_abi::xdrfile::exdrENDOFFILE.into(),
            task: ErrorTask::Read,
        };
        assert!(error.is_eof());

        let error = CApiError {
            code: ErrorCode::ExdrEndOfFile,
            task: ErrorTask::Read,
        };
        assert!(error.is_eof());

        let error = CApiError {
            code: (c_abi::xdrfile::exdrENDOFFILE + 1).into(),
            task: ErrorTask::Read,
        };
        assert!(!error.is_eof());

        let error = CApiError {
            code: 0.into(),
            task: ErrorTask::Read,
        };
        assert!(!error.is_eof());

        let error = CApiError {
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
}
