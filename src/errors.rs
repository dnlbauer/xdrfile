use crate::c_abi;
use crate::FileMode;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorTask {
    OpenFile(PathBuf, FileMode),
    ReadNumAtoms,
    Read,
    Write,
    Flush,
    ToCString(Option<std::ffi::NulError>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Error {
    code: Option<ErrorCode>,
    task: ErrorTask,
}

impl Error {
    pub fn task(&self) -> &ErrorTask {
        &self.task
    }

    pub fn code(&self) -> &Option<ErrorCode> {
        &self.code
    }

    pub fn is_eof(&self) -> bool {
        self.code.as_ref().map_or(false, ErrorCode::is_eof)
    }

    pub fn from_convert() -> Self {
        Self {
            code: None,
            task: ErrorTask::ToCString(None),
        }
    }

    pub fn from_open(path: impl AsRef<Path>, mode: FileMode) -> Self {
        Self {
            code: None,
            task: ErrorTask::OpenFile(path.as_ref().into(), mode),
        }
    }

    pub fn from_read_num_atoms(code: impl Into<ErrorCode>) -> Self {
        Self {
            code: Some(code.into()),
            task: ErrorTask::ReadNumAtoms,
        }
    }

    pub fn from_read(code: impl Into<ErrorCode>) -> Self {
        Self {
            code: Some(code.into()),
            task: ErrorTask::Read,
        }
    }

    pub fn from_write(code: impl Into<ErrorCode>) -> Self {
        Self {
            code: Some(code.into()),
            task: ErrorTask::Write,
        }
    }

    pub fn from_flush(code: impl Into<ErrorCode>) -> Self {
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
            ReadNumAtoms => write!(
                f,
                "Failed to read atom number from trajectory: C API returned error code {:?}",
                self.code
            ),
            Read => write!(
                f,
                "Failed to read trajectory: C API returned error code {:?}",
                self.code
            ),
            Write => write!(
                f,
                "Failed to write trajectory: C API returned error code {:?}",
                self.code
            ),
            Flush => write!(
                f,
                "Failed to flush trajectory: C API returned error code {:?}",
                self.code
            ),
            ToCString(_) => write!(
                f,
                "Path cannot be converted to a C string because it has a null byte"
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorCode {
    ExdrOk,
    ExdrHeader,
    ExdrString,
    ExdrDouble,
    ExdrInt,
    ExdrFloat,
    ExdrUint,
    Exdr3dx,
    ExdrClose,
    ExdrMagic,
    ExdrNoMem,
    ExdrEndOfFile,
    ExdrFileNotFound,
    ExdrNr,
    UnmatchedCode(c_abi::xdrfile::BindgenTy1),
}

impl ErrorCode {
    pub fn is_eof(&self) -> bool {
        match self {
            Self::ExdrEndOfFile => true,
            _ => false,
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

impl std::error::Error for Error {}

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
