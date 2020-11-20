//! Mode types for opening trajectory files

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
///
/// FileMode is object safe:
/// ```
/// # use xdrfile::filemode::{FileMode, Read, Write, Append};
/// let trait_obj: Box<dyn FileMode> = Box::new(Read);
/// ```
pub trait FileMode: private::Sealed {
    fn as_bytes_with_null() -> &'static [u8]
    where
        Self: Sized;

    /// Get a CStr slice corresponding to the file mode
    fn to_cstr() -> &'static std::ffi::CStr
    where
        Self: Sized,
    {
        std::ffi::CStr::from_bytes_with_nul(Self::as_bytes_with_null())
            .expect("CStr::from_bytes_with_nul failed")
    }
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
    fn as_bytes_with_null() -> &'static [u8] {
        b"r\0"
    }
}
impl FileMode for Write {
    fn as_bytes_with_null() -> &'static [u8] {
        b"w\0"
    }
}
impl FileMode for Append {
    fn as_bytes_with_null() -> &'static [u8] {
        b"a\0"
    }
}
