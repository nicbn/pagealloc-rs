use pagealloc::page_size;
use std::{
    env,
    hint::black_box,
    mem,
    process::{Command, ExitStatus},
    ptr,
};

fn assert_segfault(status: ExitStatus) {
    cfg_if::cfg_if! {
        if #[cfg(unix)] {
            use std::os::unix::process::ExitStatusExt;
            let signal = status.signal().unwrap();
            assert!(
                signal == libc::SIGSEGV || signal == libc::SIGBUS,
                "expected sigsegv ({}) or sigbus ({}), got {}",
                libc::SIGSEGV,
                libc::SIGBUS,
                signal,
            );
        } else if #[cfg(windows)] {
            assert_eq!(status.code(), Some(windows_sys::Win32::Foundation::STATUS_ACCESS_VIOLATION));
        }
    }
}

macro_rules! test_should_crash {
    ($name:ident, $implementation:expr) => {
        test_should_crash!($name, $implementation, |_| true);
    };

    ($name:ident, $implementation:expr, $filter:expr) => {
        #[test]
        fn $name() {
            if env::var("PAGEALLOC_SHOULD_CRASH").is_ok() {
                ($implementation)();
                return;
            }
            let output = Command::new(env::current_exe().unwrap())
                .arg(stringify!($name))
                .arg("--nocapture")
                .arg("--exact")
                .env("PAGEALLOC_SHOULD_CRASH", "")
                .output()
                .unwrap();
            if output.status.success() {
                panic!(
                    "expected test to crash, got success.\nstdout: {}\nstderr: {}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr),
                );
            }
            ($filter)(output.status);
        }
    };
}

test_should_crash!(
    read_without_protection,
    read_without_protection_impl,
    assert_segfault
);
fn read_without_protection_impl() {
    let ptr =
        unsafe { pagealloc::alloc(ptr::null_mut(), page_size(), pagealloc::Protection::None) }
            .unwrap();
    black_box(unsafe { *ptr.as_ptr() });
}

test_should_crash!(
    write_without_protection,
    write_without_protection_impl,
    assert_segfault
);
fn write_without_protection_impl() {
    let ptr =
        unsafe { pagealloc::alloc(ptr::null_mut(), page_size(), pagealloc::Protection::Read) }
            .unwrap();
    unsafe { *ptr.as_ptr() = 0 };
}

test_should_crash!(
    write_then_read_without_protection,
    write_then_read_without_protection_impl,
    assert_segfault
);
fn write_then_read_without_protection_impl() {
    let ptr = unsafe {
        pagealloc::alloc(
            ptr::null_mut(),
            page_size(),
            pagealloc::Protection::ReadWrite,
        )
    }
    .unwrap();
    unsafe { *ptr.as_ptr() = 0 };
    unsafe { pagealloc::protect(ptr, page_size(), pagealloc::Protection::None).unwrap() };
    black_box(unsafe { *ptr.as_ptr() });
}

test_should_crash!(
    write_then_write_without_protection,
    write_then_write_without_protection_impl,
    assert_segfault
);
fn write_then_write_without_protection_impl() {
    let ptr = unsafe {
        pagealloc::alloc(
            ptr::null_mut(),
            page_size(),
            pagealloc::Protection::ReadWrite,
        )
    }
    .unwrap();
    unsafe { *ptr.as_ptr() = 0 };
    unsafe { pagealloc::protect(ptr, page_size(), pagealloc::Protection::Read).unwrap() };
    unsafe { *ptr.as_ptr() = 0 };
}

test_should_crash!(
    exec_without_protection,
    exec_without_protection_impl,
    assert_segfault
);
fn exec_without_protection_impl() {
    fn template_func() {}

    let ptr = unsafe {
        pagealloc::alloc(
            ptr::null_mut(),
            page_size(),
            pagealloc::Protection::ReadWrite,
        )
    }
    .unwrap();

    // Copy template_func to ptr
    unsafe { ptr::copy_nonoverlapping(template_func as *const u8, ptr.as_ptr(), page_size()) };

    unsafe { mem::transmute::<*mut u8, fn()>(ptr.as_ptr())() };
}
