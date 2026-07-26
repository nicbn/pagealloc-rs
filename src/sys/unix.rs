use crate::{Error, OsError, Protection, Result};
use core::ptr::NonNull;
use errno::errno;

pub type RawOsError = i32;

pub unsafe fn alloc(location: *mut u8, size: usize, protection: Protection) -> Result<NonNull<u8>> {
    let mut flags = libc::MAP_PRIVATE | libc::MAP_ANONYMOUS;
    if !location.is_null() {
        flags |= libc::MAP_FIXED;
    }
    let r = unsafe {
        libc::mmap(
            location as *mut libc::c_void,
            size,
            protection_libc(protection),
            flags,
            -1,
            0,
        )
    };
    if r == libc::MAP_FAILED || r.is_null() {
        return Err(error());
    }
    Ok(unsafe { NonNull::new_unchecked(r as *mut u8) })
}

pub unsafe fn dealloc(start: NonNull<u8>, size: usize) -> Result<()> {
    cvt(unsafe { libc::munmap(start.as_ptr() as *mut libc::c_void, size) })
}

cfg_if::cfg_if! {
    if #[cfg(any(
        target_os = "android",
        target_os = "linux",
    ))] {
        pub unsafe fn clear(start: NonNull<u8>, size: usize, protection: Protection) -> Result<()> {
            cvt(unsafe {
                libc::madvise(
                    start.as_ptr() as *mut libc::c_void,
                    size,
                    libc::MADV_DONTNEED,
                )
            })?;
            unsafe { protect(start, size, protection) }
        }
    } else {
        pub unsafe fn clear(start: NonNull<u8>, size: usize, protection: Protection) -> Result<()> {
            unsafe { alloc(start.as_ptr(), size, protection)? };
            Ok(())
        }
    }
}

pub unsafe fn protect(start: NonNull<u8>, size: usize, protection: Protection) -> Result<()> {
    cvt(unsafe {
        libc::mprotect(
            start.as_ptr() as *mut libc::c_void,
            size,
            protection_libc(protection),
        )
    })
}

cfg_if::cfg_if! {
    if #[cfg(any(
        target_os = "android",
        target_os = "linux",
    ))] {
        // Use MADV_DONTNEED.

        // Do not use MADV_FREE because it creates problems.
        // See also:
        // https://github.com/JuliaLang/julia/issues/51086
        // https://github.com/golang/go/commit/05e6d28849293266028c0bc9e9b0f8d0da38a2e2

        pub unsafe fn advise_free(start: NonNull<u8>, size: usize) -> Result<()> {
            cvt(unsafe { libc::madvise(start.as_ptr() as *mut libc::c_void, size, libc::MADV_DONTNEED) })
        }
    } else if #[cfg(any(
        target_os = "ios",
        target_os = "solaris",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "fuchsia",
        target_os = "haiku",
        target_os = "illumos",
        target_os = "l4re",
        target_os = "netbsd",
        target_os = "openbsd",
    ))] {
        // Use MADV_FREE

        pub unsafe fn advise_free(start: NonNull<u8>, size: usize) -> Result<()> {
            cvt(unsafe { libc::madvise(start.as_ptr() as *mut libc::c_void, size, libc::MADV_FREE) })
        }
    } else if #[cfg(target_os = "macos")] {
        // Use MADV_FREE_REUSABLE

        pub unsafe fn advise_free(start: NonNull<u8>, size: usize) -> Result<()> {
            cvt(unsafe { libc::madvise(start.as_ptr() as *mut libc::c_void, size, libc::MADV_FREE_REUSABLE) })
        }
    } else {
        // Use POSIX_MADV_DONTNEED

        pub unsafe fn advise_free(start: NonNull<u8>, size: usize) -> Result<()> {
            cvt(unsafe { libc::posix_madvise(start.as_ptr() as *mut libc::c_void, size, libc::POSIX_MADV_DONTNEED) })
        }
    }
}

pub fn page_size() -> usize {
    unsafe { libc::sysconf(libc::_SC_PAGESIZE) as _ }
}

#[inline(always)]
pub fn alloc_granularity() -> usize {
    page_size()
}

fn protection_libc(protection: Protection) -> libc::c_int {
    match protection {
        Protection::None => libc::PROT_NONE,
        Protection::Read => libc::PROT_READ,
        Protection::ReadWrite => libc::PROT_READ | libc::PROT_WRITE,
        Protection::Exec => libc::PROT_EXEC,
        Protection::ExecRead => libc::PROT_EXEC | libc::PROT_READ,
        Protection::ExecReadWrite => libc::PROT_EXEC | libc::PROT_READ | libc::PROT_WRITE,
    }
}

#[inline(always)]
fn cvt(r: i32) -> Result<()> {
    if r == 0 { Ok(()) } else { Err(error()) }
}

#[cold]
fn error() -> Error {
    Error::Os(OsError(errno().0))
}
