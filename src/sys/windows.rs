use crate::{Error, OsError, Protection, Result};
use core::ffi::c_void;
use core::mem::MaybeUninit;
use core::ptr::NonNull;
use windows_sys::Win32;

pub type RawOsError = i32;

pub unsafe fn alloc(location: *mut u8, size: usize, protection: Protection) -> Result<NonNull<u8>> {
    unsafe {
        let r = Win32::System::Memory::VirtualAlloc(
            location as *mut c_void,
            size,
            Win32::System::Memory::MEM_RESERVE | Win32::System::Memory::MEM_COMMIT,
            protection_flag(protection),
        );
        NonNull::new(r as *mut u8).ok_or_else(error)
    }
}

pub unsafe fn dealloc(start: NonNull<u8>, _: usize) -> Result<()> {
    unsafe {
        if Win32::System::Memory::VirtualFree(
            start.as_ptr() as *mut c_void,
            0,
            Win32::System::Memory::MEM_RELEASE,
        ) != 0
        {
            Ok(())
        } else {
            Err(error())
        }
    }
}

pub unsafe fn clear(start: NonNull<u8>, size: usize, protection: Protection) -> Result<()> {
    unsafe {
        if Win32::System::Memory::VirtualFree(
            start.as_ptr() as *mut c_void,
            0,
            Win32::System::Memory::MEM_DECOMMIT,
        ) != 0
            && !Win32::System::Memory::VirtualAlloc(
                start.as_ptr() as *mut c_void,
                size,
                Win32::System::Memory::MEM_COMMIT,
                protection_flag(protection),
            )
            .is_null()
        {
            Ok(())
        } else {
            Err(error())
        }
    }
}

pub unsafe fn protect(start: NonNull<u8>, size: usize, protection: Protection) -> Result<()> {
    unsafe {
        if Win32::System::Memory::VirtualProtect(
            start.as_ptr() as *mut c_void,
            size,
            protection_flag(protection),
            &mut 0,
        ) != 0
        {
            Ok(())
        } else {
            Err(error())
        }
    }
}

pub unsafe fn advise_free(start: NonNull<u8>, size: usize) -> Result<()> {
    unsafe {
        if !Win32::System::Memory::VirtualAlloc(
            start.as_ptr() as *mut c_void,
            size,
            Win32::System::Memory::MEM_RESET,
            Win32::System::Memory::PAGE_NOACCESS,
        )
        .is_null()
        {
            Ok(())
        } else {
            Err(error())
        }
    }
}

pub fn page_size() -> usize {
    let mut info = MaybeUninit::uninit();
    let info = unsafe {
        Win32::System::SystemInformation::GetSystemInfo(info.as_mut_ptr());
        info.assume_init()
    };

    info.dwPageSize as usize
}

pub fn alloc_granularity() -> usize {
    let mut info = MaybeUninit::uninit();
    let info = unsafe {
        Win32::System::SystemInformation::GetSystemInfo(info.as_mut_ptr());
        info.assume_init()
    };

    info.dwAllocationGranularity as usize
}

fn protection_flag(protection: Protection) -> Win32::System::Memory::PAGE_PROTECTION_FLAGS {
    match protection {
        Protection::None => Win32::System::Memory::PAGE_NOACCESS,
        Protection::Read => Win32::System::Memory::PAGE_READONLY,
        Protection::ReadWrite => Win32::System::Memory::PAGE_READWRITE,
        Protection::Exec => Win32::System::Memory::PAGE_EXECUTE,
        Protection::ExecRead => Win32::System::Memory::PAGE_EXECUTE_READ,
        Protection::ExecReadWrite => Win32::System::Memory::PAGE_EXECUTE_READWRITE,
    }
}

#[cold]
fn error() -> Error {
    unsafe { Error::Os(OsError(Win32::Foundation::GetLastError() as i32)) }
}
