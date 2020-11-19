use crate::errors::ErrorFileMode;

/// Trait for all file modes
pub trait FileMode: Default + Into<ErrorFileMode> {
    /// Get a CStr slice corresponding to the file mode
    fn to_cstr() -> &'static std::ffi::CStr;
}

/// The read-only mode
#[derive(Debug, Clone, PartialEq, Copy, Default)]
pub struct Read;
/// The write-only mode
#[derive(Debug, Clone, PartialEq, Copy, Default)]
pub struct Write;
/// The read/write append mode
#[derive(Debug, Clone, PartialEq, Copy, Default)]
pub struct Append;

impl FileMode for Read {
    fn to_cstr() -> &'static std::ffi::CStr {
        std::ffi::CStr::from_bytes_with_nul(b"r\0").expect("CStr::from_bytes_with_nul failed")
    }
}
impl FileMode for Write {
    fn to_cstr() -> &'static std::ffi::CStr {
        std::ffi::CStr::from_bytes_with_nul(b"w\0").expect("CStr::from_bytes_with_nul failed")
    }
}
impl FileMode for Append {
    fn to_cstr() -> &'static std::ffi::CStr {
        std::ffi::CStr::from_bytes_with_nul(b"a\0").expect("CStr::from_bytes_with_nul failed")
    }
}

/// Trait for modes that can read (read, append)
pub trait ReaderMode: FileMode {}
/// Trait for modes that can write (write, append)
pub trait WriterMode: FileMode {}
impl ReaderMode for Read {}
impl ReaderMode for Append {}
impl WriterMode for Append {}
impl WriterMode for Write {}

impl From<Read> for ErrorFileMode {
    fn from(_: Read) -> Self {
        Self::Read
    }
}

impl From<Write> for ErrorFileMode {
    fn from(_: Write) -> Self {
        Self::Write
    }
}

impl From<Append> for ErrorFileMode {
    fn from(_: Append) -> Self {
        Self::Append
    }
}
