//! Higher-level safe wrapper for managing buffers.

use crate::{advise_free, alloc, clear, dealloc, protect, Error, Protection, Result};
use core::{
    borrow::{Borrow, BorrowMut},
    mem::forget,
    ops::{Bound, RangeBounds},
    ptr::{self, NonNull},
    slice,
};

/// A safe wrapper around multiple pages.
///
/// # Examples
///
/// ```
/// # (|| -> pagealloc::Result<()> {
/// # use pagealloc::buffer::ByteBuffer;
/// #
/// let mut buf = ByteBuffer::new(
///     pagealloc::alloc_granularity(),
///     pagealloc::Protection::ReadWrite,
/// )?;
/// buf.get_mut()[0] = 1;
/// buf.get_mut()[1] = 2;
/// buf.get_mut()[2] = 3;
/// buf.clear(.., pagealloc::Protection::ReadWrite)?;
/// assert_eq!(buf.get()[0], 0);
/// # Ok(())
/// # })().unwrap()
/// ```
pub struct ByteBuffer {
    ptr: NonNull<u8>,
    size: usize,
}

unsafe impl Send for ByteBuffer {}

unsafe impl Sync for ByteBuffer {}

impl ByteBuffer {
    /// Allocate pages.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidInput`] if:
    /// - `size` is zero, or
    /// - `size` is not a multiple of [allocation granularity](crate::alloc_granularity).
    ///
    /// # Panics
    ///
    /// Panics if `size` is zero or if `size` overflows when rounded up to the page size.
    /// 
    /// # Examples
    ///
    /// ```
    /// # (|| -> pagealloc::Result<()> {
    /// # use pagealloc::buffer::ByteBuffer;
    /// #
    /// let buf = ByteBuffer::new(
    ///     pagealloc::alloc_granularity(),
    ///     pagealloc::Protection::ReadWrite,
    /// )?;
    /// # Ok(())
    /// # })().unwrap()
    /// ```
    pub fn new(size: usize, protection: Protection) -> Result<Self> {
        let ptr = unsafe { alloc(ptr::null_mut(), size, protection)? };
        Ok(Self { ptr, size })
    }

    /// Instantiate a [`ByteBuffer`] from an allocated range of memory.
    ///
    /// # Safety
    ///
    /// For portability, `ptr` and `size` must **exactly** match the memory
    /// range reserved via [`alloc`] or returned by [`into_raw_parts`](Self::into_raw_parts).
    /// 
    /// # Examples
    ///
    /// ```
    /// # (|| -> pagealloc::Result<()> {
    /// # use pagealloc::buffer::ByteBuffer;
    /// let buf = ByteBuffer::new(
    ///     pagealloc::alloc_granularity(),
    ///     pagealloc::Protection::ReadWrite,
    /// )?;
    /// let (ptr, len) = buf.into_raw_parts();
    /// let buf = unsafe { ByteBuffer::from_raw_parts(ptr, len) };
    /// # Ok(())
    /// # })().unwrap()
    /// ```
    #[inline(always)]
    pub unsafe fn from_raw_parts(ptr: NonNull<u8>, size: usize) -> Self {
        Self { ptr, size }
    }

    /// Clear part of the buffer, such that it remains allocated and returns zero when accessed.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidInput`] if either lower or upper bound
    /// is not a multiple of [page size](crate::page_size), or if out of bounds.
    /// 
    /// # Examples
    ///
    /// ```
    /// # (|| -> pagealloc::Result<()> {
    /// # use pagealloc::buffer::ByteBuffer;
    /// # let mut buf = ByteBuffer::new(
    /// #     pagealloc::alloc_granularity(),
    /// #     pagealloc::Protection::ReadWrite,
    /// # )?;
    /// buf.clear(.., pagealloc::Protection::ReadWrite)?;
    /// assert_eq!(buf.get()[0], 0);
    /// # Ok(())
    /// # })().unwrap()
    /// ```
    pub fn clear(&mut self, range: impl RangeBounds<usize>, protection: Protection) -> Result<()> {
        let start = match range.start_bound() {
            Bound::Included(start) => *start,
            Bound::Excluded(start) => start + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(end) => end + 1,
            Bound::Excluded(end) => *end,
            Bound::Unbounded => self.size,
        };

        if start == end {
            return Ok(());
        }

        if end > self.size {
            return Err(Error::InvalidInput);
        }

        unsafe { clear(self.ptr.add(start), end - start, protection) }
    }

    /// Change protection of part of the buffer.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidInput`] if either lower or upper bound
    /// is not a multiple of [page size](crate::page_size), or if out of bounds.
    /// 
    /// # Examples
    ///
    /// ```
    /// # (|| -> pagealloc::Result<()> {
    /// # use pagealloc::buffer::ByteBuffer;
    /// # let mut buf = ByteBuffer::new(
    /// #     pagealloc::alloc_granularity(),
    /// #     pagealloc::Protection::ReadWrite,
    /// # )?;
    /// buf.protect(.., pagealloc::Protection::Read)?;
    /// # Ok(())
    /// # })().unwrap()
    /// ```
    pub fn protect(
        &mut self,
        range: impl RangeBounds<usize>,
        protection: Protection,
    ) -> Result<()> {
        let start = match range.start_bound() {
            Bound::Included(start) => *start,
            Bound::Excluded(start) => start + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(end) => end + 1,
            Bound::Excluded(end) => *end,
            Bound::Unbounded => self.size,
        };

        if start == end {
            return Ok(());
        }

        if end > self.size {
            return Err(Error::InvalidInput);
        }

        unsafe { protect(self.ptr.add(start), end - start, protection) }
    }

    /// Indicate to the system that a memory region will temporarily not be of
    /// interest. This way the system can use the memory if needed.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidInput`] if either lower or upper bound
    /// is not a multiple of [page size](crate::page_size), or if out of bounds.
    /// 
    /// # Examples
    ///
    /// ```
    /// # (|| -> pagealloc::Result<()> {
    /// # use pagealloc::buffer::ByteBuffer;
    /// # let mut buf = ByteBuffer::new(
    /// #     pagealloc::alloc_granularity(),
    /// #     pagealloc::Protection::ReadWrite,
    /// # )?;
    /// buf.get_mut()[0] = 1;
    /// buf.advise_free(..)?;
    /// assert!(buf.get()[0] == 0 || buf.get()[0] == 1);
    /// # Ok(())
    /// # })().unwrap()
    /// ```
    pub fn advise_free(&mut self, range: impl RangeBounds<usize>) -> Result<()> {
        let start = match range.start_bound() {
            Bound::Included(start) => *start,
            Bound::Excluded(start) => start + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(end) => end + 1,
            Bound::Excluded(end) => *end,
            Bound::Unbounded => self.size,
        };

        if start == end {
            return Ok(());
        }

        if end > self.size {
            return Err(Error::InvalidInput);
        }

        unsafe { advise_free(self.ptr.add(start), end - start) }
    }

    /// Consume the buffer, returning the underlying allocation.
    ///
    /// 
    /// # Examples
    ///
    /// ```
    /// # (|| -> pagealloc::Result<()> {
    /// # use pagealloc::buffer::ByteBuffer;
    /// let buf = ByteBuffer::new(
    ///     pagealloc::alloc_granularity(),
    ///     pagealloc::Protection::ReadWrite,
    /// )?;
    /// let (ptr, len) = buf.into_raw_parts();
    /// # let buf = unsafe { ByteBuffer::from_raw_parts(ptr, len) };
    /// # Ok(())
    /// # })().unwrap()
    /// ```
    #[inline(always)]
    pub fn into_raw_parts(self) -> (NonNull<u8>, usize) {
        let ptr = self.ptr;
        let size = self.size;
        forget(self);
        (ptr, size)
    }

    /// Get a reference to the underlying allocation.
    /// 
    /// # Examples
    ///
    /// ```
    /// # (|| -> pagealloc::Result<()> {
    /// # use pagealloc::buffer::ByteBuffer;
    /// # let mut buf = ByteBuffer::new(
    /// #     pagealloc::alloc_granularity(),
    /// #     pagealloc::Protection::ReadWrite,
    /// # )?;
    /// let data: &[u8] = buf.get();
    /// # Ok(())
    /// # })().unwrap()
    /// ```
    #[inline(always)]
    pub fn get(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.size) }
    }

    /// Get a mutable reference to the underlying allocation.
    /// 
    /// # Examples
    ///
    /// ```
    /// # (|| -> pagealloc::Result<()> {
    /// # use pagealloc::buffer::ByteBuffer;
    /// # let mut buf = ByteBuffer::new(
    /// #     pagealloc::alloc_granularity(),
    /// #     pagealloc::Protection::ReadWrite,
    /// # )?;
    /// let data: &mut [u8] = buf.get_mut();
    /// # Ok(())
    /// # })().unwrap()
    /// ```
    #[inline(always)]
    pub fn get_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.size) }
    }

    /// Get a reference to the underlying allocation as a [`NonNull`] pointer.
    /// 
    /// # Examples
    ///
    /// ```
    /// # (|| -> pagealloc::Result<()> {
    /// # use pagealloc::buffer::ByteBuffer;
    /// # use std::ptr::NonNull;
    /// # let mut buf = ByteBuffer::new(
    /// #     pagealloc::alloc_granularity(),
    /// #     pagealloc::Protection::ReadWrite,
    /// # )?;
    /// let data: NonNull<[u8]> = buf.get_raw();
    /// # Ok(())
    /// # })().unwrap()
    /// ```
    #[inline(always)]
    pub fn get_raw(&self) -> NonNull<[u8]> {
        NonNull::slice_from_raw_parts(self.ptr, self.size)
    }
}

impl AsRef<[u8]> for ByteBuffer {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        self.get()
    }
}

impl Borrow<[u8]> for ByteBuffer {
    #[inline(always)]
    fn borrow(&self) -> &[u8] {
        self.get()
    }
}

impl AsMut<[u8]> for ByteBuffer {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut [u8] {
        self.get_mut()
    }
}

impl BorrowMut<[u8]> for ByteBuffer {
    #[inline(always)]
    fn borrow_mut(&mut self) -> &mut [u8] {
        self.get_mut()
    }
}

impl Drop for ByteBuffer {
    fn drop(&mut self) {
        unsafe {
            let _ = dealloc(self.ptr, self.size);
        }
    }
}
