use crate::c_abi;
use crate::FileMode;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
/// The task being attempted when an error was returned
pub enum ErrorTask {
    /// A file was being opened
    OpenFile(PathBuf, FileMode),
    /// The number of atoms was being read from a file
    ReadNumAtoms,
    /// A frame was being read from a file
    Read,
    /// A frame was being written to a file
    Write,
    /// An file was being flushed to disk
    Flush,
    /// A path was being converted to a CString
    ToCString(Option<std::ffi::NulError>),
}

#[derive(Debug, Clone, PartialEq)]
/// Error type for the xdrfile library
pub struct Error {
    code: Option<ErrorCode>,
    task: ErrorTask,
}

impl Error {
    /// Get the task being attempted when the error was returned
    pub fn task(&self) -> &ErrorTask {
        &self.task
    }

    /// Get the error code returned by the C API, if any
    pub fn code(&self) -> &Option<ErrorCode> {
        &self.code
    }

    /// True if the error is an end of file error, false otherwise
    pub fn is_eof(&self) -> bool {
        self.code.as_ref().map_or(false, ErrorCode::is_eof)
    }

    /// Construct a new error during the ToCString task
    pub(crate) fn from_convert() -> Self {
        Self {
            code: None,
            task: ErrorTask::ToCString(None),
        }
    }

    /// Construct a new error during the OpenFile task
    pub(crate) fn from_open(path: impl AsRef<Path>, mode: FileMode) -> Self {
        Self {
            code: None,
            task: ErrorTask::OpenFile(path.as_ref().into(), mode),
        }
    }

    /// Construct a new error during the ReadNumAtoms task from a C error code
    pub(crate) fn from_read_num_atoms(code: impl Into<ErrorCode>) -> Self {
        Self {
            code: Some(code.into()),
            task: ErrorTask::ReadNumAtoms,
        }
    }

    /// Construct a new error during the Read task from a C error code
    pub(crate) fn from_read(code: impl Into<ErrorCode>) -> Self {
        Self {
            code: Some(code.into()),
            task: ErrorTask::Read,
        }
    }

    /// Construct a new error during the Write task from a C error code
    pub(crate) fn from_write(code: impl Into<ErrorCode>) -> Self {
        Self {
            code: Some(code.into()),
            task: ErrorTask::Write,
        }
    }

    /// Construct a new error during the Flush task from a C error code
    pub(crate) fn from_flush(code: impl Into<ErrorCode>) -> Self {
        Self {
            code: Some(code.into()),
            task: ErrorTask::Flush,
        }
    }
}

impl From<std::ffi::NulError> for Error {
    fn from(err: std::ffi::NulError) -> Self {
        Self {
            code: None,
            task: ErrorTask::ToCString(Some(err)),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ErrorTask::*;
        match &self.task {
            OpenFile(path, mode) => write!(
                f,
                "Failed to open file at {path:?} with mode {mode:?}",
                path = path,
                mode = mode
            ),
            ReadNumAtoms => write!(f, "Failed to read atom number from trajectory"),
            Read => write!(f, "Failed to read trajectory"),
            Write => write!(f, "Failed to write trajectory"),
            Flush => write!(f, "Failed to flush trajectory"),
            ToCString(None) => write!(
                f,
                "Path cannot be converted to a C string because it was invalid Unicode"
            ),
            ToCString(Some(_)) => write!(
                f,
                "Path cannot be converted to a C string because it has a null byte"
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        if let Some(e) = &self.code {
            Some(e)
        } else if let ErrorTask::ToCString(Some(e)) = &self.task {
            Some(e)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
/// Error codes returned from the C API
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
        match self {
            Self::ExdrEndOfFile => true,
            _ => false,
        }
    }

    /// Convert an error code and output value from a C call to a Result
    ///
    /// `code` should be an integer return code returned from the C API. `value` should be the
    /// function's output, which is generally either `()` or one of its arguments. If `code`
    /// indicates the function returned successfully, the value is returned; otherwise, the
    /// code is converted into the appropriate `ErrorCode`.
    pub fn check<T>(code: impl Into<Self>, value: T) -> std::result::Result<T, Self> {
        let code = code.into();
        if let Self::ExdrOk = code {
            Ok(value)
        } else {
            Err(code)
        }
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
            write!(f, "C API returned error code {}", i)
        } else {
            write!(f, "C API returned error code {:?}", self)
        }
    }
}

impl std::error::Error for ErrorCode {}

/// `Result` type for errors in the `xdrfile` crate
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_eof() {
        let error = Error {
            code: Some(c_abi::xdrfile::exdrENDOFFILE.into()),
            task: ErrorTask::Read,
        };
        assert!(error.is_eof());

        let error = Error {
            code: Some(ErrorCode::ExdrEndOfFile),
            task: ErrorTask::Read,
        };
        assert!(error.is_eof());

        let error = Error {
            code: Some((c_abi::xdrfile::exdrENDOFFILE + 1).into()),
            task: ErrorTask::Read,
        };
        assert!(!error.is_eof());

        let error = Error {
            code: Some(0.into()),
            task: ErrorTask::Write,
        };
        assert!(!error.is_eof());

        let error = Error {
            code: Some(255.into()),
            task: ErrorTask::Flush,
        };
        assert!(!error.is_eof());

        let error = Error {
            code: None,
            task: ErrorTask::OpenFile(PathBuf::from("not/a/file"), FileMode::Read),
        };
        assert!(!error.is_eof());
    }
}
