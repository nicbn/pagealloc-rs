use crate::sys;
use core::fmt;

/// An error raised by the library.
///
/// This can be cast into [`std::io::Error`] if the `std` feature is enabled.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Error {
    /// Input requirements not met. For example, a pointer is not aligned to the page size, or
    /// a size is not a multiple of the page size.
    InvalidInput,
    /// Error returned by the OS.
    Os(OsError),
}

impl Error {
    /// Transform this error into [`std::io::Error`].
    #[inline]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    #[cfg(feature = "std")]
    pub fn into_io(self) -> std::io::Error {
        match self {
            Self::InvalidInput => std::io::Error::from(std::io::ErrorKind::InvalidInput),
            Self::Os(e) => e.into_io(),
        }
    }
}

impl From<OsError> for Error {
    fn from(e: OsError) -> Self {
        Self::Os(e)
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
impl From<Error> for std::io::Error {
    #[inline]
    fn from(origin: Error) -> Self {
        origin.into_io()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput => write!(f, "invalid input"),
            Self::Os(e) => fmt::Display::fmt(e, f),
        }
    }
}

impl core::error::Error for Error {}

/// An error raised by the OS.
///
/// This can be cast into [`std::io::Error`] if the `std` feature is enabled.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct OsError(pub(crate) sys::RawOsError);

impl OsError {
    /// The system error code.
    #[inline]
    pub fn code(self) -> sys::RawOsError {
        self.0
    }

    /// Transform this error into [`std::io::Error`].
    #[inline]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    #[cfg(feature = "std")]
    pub fn into_io(self) -> std::io::Error {
        std::io::Error::from_raw_os_error(self.code())
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
#[cfg(feature = "std")]
impl From<OsError> for std::io::Error {
    #[inline]
    fn from(origin: OsError) -> Self {
        origin.into_io()
    }
}

impl fmt::Display for OsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "system error code 0x{:X}", self.code())
    }
}

impl fmt::Debug for OsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("OsError")
            .field(&format_args!("0x{:X}", self.code()))
            .finish()
    }
}

impl core::error::Error for OsError {}

/// Alias for `Result<T, Error>`.
pub type Result<T> = core::result::Result<T, Error>;
