use crate::errors::ErrorFileMode;

mod private {
    /// Prevent users from implementing FileMode on their own types
    ///
    /// https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed
    pub trait Sealed {}

    impl Sealed for super::Read {}
    impl Sealed for super::Write {}
    impl Sealed for super::Append {}
}

/// Trait for all file modes
pub trait FileMode: Default + Into<ErrorFileMode> + private::Sealed {
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
