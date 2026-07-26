#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]

//! Low-level, cross-platform page allocation library.
//!
//! This crate provides a cross-platform interface to the operating system's virtual memory
//! allocation, such as `mmap` on Unix and `VirtualAlloc` on Windows.
//!
//! # Crate features
//!
//! - `std` (default): Enables `std` compatibility.
//! - `buffer`: Enables the `buffer` module, providing a safer API around allocation.
//!
//! # Examples
//!
//! ```
//! # (|| -> pagealloc::Result<()> {
//! unsafe {
//!     let ptr = pagealloc::alloc(
//!         std::ptr::null_mut(),
//!         pagealloc::alloc_granularity(),
//!         pagealloc::Protection::ReadWrite,
//!     )?;
//!     // ...
//!     pagealloc::dealloc(ptr, pagealloc::alloc_granularity())?;
//!     # Ok(())
//! }
//! # })().unwrap()
//! ```

use core::{
    ptr::NonNull,
    sync::atomic::{AtomicUsize, Ordering},
};

mod error;
mod sys;

#[cfg_attr(docsrs, doc(cfg(feature = "buffer")))]
#[cfg(feature = "buffer")]
pub mod buffer;

pub use self::error::*;

/// The rights the user will have for a page.
#[derive(Clone, Copy, Debug, Hash, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum Protection {
    /// The page cannot be used.
    #[default]
    None,

    /// The page can be read.
    Read,

    /// The page can be read, or written to.
    ReadWrite,

    /// The page can be executed.
    Exec,

    /// The page can be executed, or read.
    ExecRead,

    /// The page can be executed, read, or written to.
    ExecReadWrite,
}

/// Returns the page size.
///
/// This method caches the value returned by the system, and therefore may
/// be called repeatedly with reduced performance impact.
///
/// # Examples
///
/// ```
/// let page_size = pagealloc::page_size();
/// println!("page size is {} bytes", page_size);
/// ```
#[inline]
pub fn page_size() -> usize {
    static PAGE_SIZE: AtomicUsize = AtomicUsize::new(0);

    #[cold]
    fn page_size_slow() -> usize {
        let v = sys::page_size();
        PAGE_SIZE.store(v, Ordering::Relaxed);
        v
    }

    match PAGE_SIZE.load(Ordering::Relaxed) {
        0 => page_size_slow(),
        v => v,
    }
}

/// Returns the allocation granularity.
///
/// This method caches the value returned by the system, and therefore may
/// be called repeatedly with reduced performance impact.
///
/// # Examples
///
/// ```
/// let alloc_granularity = pagealloc::alloc_granularity();
/// println!("allocation granularity is {} bytes", alloc_granularity);
/// ```
#[inline]
pub fn alloc_granularity() -> usize {
    static ALLOC_GRANULARITY: AtomicUsize = AtomicUsize::new(0);

    #[cold]
    fn alloc_granularity_slow() -> usize {
        let v = sys::alloc_granularity();
        ALLOC_GRANULARITY.store(v, Ordering::Relaxed);
        v
    }

    match ALLOC_GRANULARITY.load(Ordering::Relaxed) {
        0 => alloc_granularity_slow(),
        v => v,
    }
}

/// Allocate some virtual pages from the OS.
///
/// This operation is also called "mapping" or "committing". When first accessed, the memory is
/// all zeros. The OS defers actually allocating physical memory until the first access.
///
/// If `location` is [`null`](core::ptr::null_mut), the OS automatically chooses a location.
///
/// # Errors
///
/// Returns [`Error::InvalidInput`] if:
/// - `location` is non-null and not aligned to the [allocation granularity](alloc_granularity), or
/// - `size` is zero, or
/// - `size` is not a multiple of [allocation granularity](alloc_granularity).
///
/// # Safety
///
/// Undefined behavior if `location + size` overflows.
///
/// # Examples
///
/// ```
/// # (|| -> pagealloc::Result<()> {
/// let ptr = unsafe {
///     pagealloc::alloc(
///         std::ptr::null_mut(),
///         pagealloc::alloc_granularity(),
///         pagealloc::Protection::ReadWrite,
///     )?
/// };
/// // ...
/// unsafe { pagealloc::dealloc(ptr, pagealloc::alloc_granularity())? };
/// # Ok(())
/// # })().unwrap()
/// ```
pub unsafe fn alloc(location: *mut u8, size: usize, protection: Protection) -> Result<NonNull<u8>> {
    let alloc_granularity = alloc_granularity();
    if (!location.is_null() && location.align_offset(alloc_granularity) != 0)
        || size == 0
        || !size.is_multiple_of(alloc_granularity)
    {
        return Err(Error::InvalidInput);
    }

    unsafe { sys::alloc(location, size, protection) }
}

/// Deallocate previously allocated virtual pages, releasing them to the OS.
///
/// This operation is also called "unmapping" or "releasing".
///
/// # Errors
///
/// Returns [`Error::InvalidInput`] if:
/// - `location` is not aligned to the [page size](page_size), or
/// - `size` is not a multiple of [page size](page_size).
///
/// # Safety
///
/// For portability, `location` and `size` must **exactly** match the memory
/// range reserved via [`alloc`].
///
/// Platform specific:
/// - On Windows, only `location` must exactly match the start of the memory
///   range reserved via [`alloc`], and `size` is ignored.
/// - On Unix, you can unreserve a subrange, i.e. `location` and `size` must
///   be currently reserved, but do not need to match exactly match the
///   memory range reserved via [`alloc`].
///
/// Undefined behavior if `location + size` overflows.
/// 
/// # Examples
/// 
/// ```
/// # (|| -> pagealloc::Result<()> {
/// let ptr = unsafe {
///     pagealloc::alloc(
///         std::ptr::null_mut(),
///         pagealloc::alloc_granularity(),
///         pagealloc::Protection::ReadWrite,
///     )?
/// };
/// // ...
/// unsafe { pagealloc::dealloc(ptr, pagealloc::alloc_granularity())? };
/// # Ok(())
/// # })().unwrap()
/// ```
pub unsafe fn dealloc(location: NonNull<u8>, size: usize) -> Result<()> {
    let alloc_granularity = alloc_granularity();
    if location.align_offset(alloc_granularity) != 0 || !size.is_multiple_of(alloc_granularity) {
        return Err(Error::InvalidInput);
    }

    unsafe { sys::dealloc(location, size) }
}

/// Recommit a range of pages, such that they remain allocated and return zero when accessed.
///
/// It is only portable to call this on ranges of memory that were allocated with [`alloc`].
/// 
/// # Platform-specific behavior
/// 
/// On Windows, this function will return an error if the memory range was not allocated with
/// [`alloc`].
/// 
/// # Errors
///
/// Returns [`Error::InvalidInput`] if:
/// - `location` is not aligned to the [page size](page_size), or
/// - `size` is not a multiple of [page size](page_size).
///
/// # Safety
///
/// This must be a valid range of memory.
///
/// Undefined behavior if `location + size` overflows.
/// 
/// # Examples
/// 
/// ```
/// # (|| -> pagealloc::Result<()> {
/// # let ptr = unsafe {
/// #     pagealloc::alloc(
/// #         std::ptr::null_mut(),
/// #         pagealloc::alloc_granularity(),
/// #         pagealloc::Protection::ReadWrite,
/// #     )?
/// # };
/// unsafe {
///     *ptr.as_ptr() = 1;
///     pagealloc::clear(ptr, pagealloc::page_size(), pagealloc::Protection::ReadWrite)?;
///     assert_eq!(*ptr.as_ptr(), 0);
/// }
/// # unsafe { pagealloc::dealloc(ptr, pagealloc::alloc_granularity())? };
/// # Ok(())
/// # })().unwrap()
/// ```
pub unsafe fn clear(location: NonNull<u8>, size: usize, protection: Protection) -> Result<()> {
    let page_size = page_size();
    if location.align_offset(page_size) != 0 || !size.is_multiple_of(page_size) {
        return Err(Error::InvalidInput);
    }

    unsafe { sys::clear(location, size, protection) }
}

/// Change the protection of a range of pages.
/// 
/// # Errors
///
/// Returns [`Error::InvalidInput`] if:
/// - `location` is not aligned to the [page size](page_size), or
/// - `size` is not a multiple of [page size](page_size).
///
/// # Safety
///
/// This must be a valid range of memory.
///
/// Undefined behavior if `location + size` overflows.
///
/// # Examples
///
/// ```
/// # (|| -> pagealloc::Result<()> {
/// unsafe {
///     let ptr = pagealloc::alloc(
///         std::ptr::null_mut(),
///         pagealloc::alloc_granularity(),
///         pagealloc::Protection::ReadWrite,
///     )?;
///     *ptr.as_ptr() = 1;
///     pagealloc::protect(ptr, pagealloc::page_size(), pagealloc::Protection::Read)?;
///     // *ptr.as_ptr() = 2; // would cause a segfault
///     pagealloc::dealloc(ptr, pagealloc::alloc_granularity())?;
/// }
/// # Ok(())
/// # })().unwrap()
/// ```
#[inline]
pub unsafe fn protect(location: NonNull<u8>, size: usize, protection: Protection) -> Result<()> {
    let page_size = page_size();
    if location.align_offset(page_size) != 0 || !size.is_multiple_of(page_size) {
        return Err(Error::InvalidInput);
    }

    unsafe { sys::protect(location, size, protection) }
}

/// Indicate to the system that a memory region will temporarily not be of
/// interest. This way the system can use the memory if needed.
///
/// This may clear some or all of the memory.
///
/// Simply access the memory to cancel the advise.
///
/// # Errors
///
/// Returns [`Error::InvalidInput`] if:
/// - `location` is not aligned to the [page size](page_size), or
/// - `size` is not a multiple of [page size](page_size).
///
/// # Safety
///
/// This must be a valid range of memory.
///
/// Undefined behavior if `location + size` overflows.
/// 
/// # Examples
///
/// ```
/// # (|| -> pagealloc::Result<()> {
/// unsafe {
///     let ptr = pagealloc::alloc(
///         std::ptr::null_mut(),
///         pagealloc::alloc_granularity(),
///         pagealloc::Protection::ReadWrite,
///     )?;
///     *ptr.as_ptr() = 1;
///     pagealloc::advise_free(ptr, pagealloc::page_size())?;
///     assert!(*ptr.as_ptr() == 0 || *ptr.as_ptr() == 1);
///     pagealloc::dealloc(ptr, pagealloc::alloc_granularity())?;
/// }
/// # Ok(())
/// # })().unwrap()
/// ```
#[inline]
pub unsafe fn advise_free(location: NonNull<u8>, size: usize) -> Result<()> {
    let page_size = page_size();
    if location.align_offset(page_size) != 0 || !size.is_multiple_of(page_size) {
        return Err(Error::InvalidInput);
    }

    unsafe { sys::advise_free(location, size) }
}
