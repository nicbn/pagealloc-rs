use pagealloc::alloc_granularity;
use std::ptr;

#[test]
fn alloc_dealloc() {
    for page_count in [1, 10, 100, 1000] {
        let size = page_count * alloc_granularity();
        let ptr = unsafe { pagealloc::alloc(ptr::null_mut(), size, pagealloc::Protection::None) }
            .unwrap();
        unsafe { pagealloc::dealloc(ptr, size).unwrap() };
    }
}

#[test]
fn alloc_write_dealloc() {
    for page_count in [1, 10, 100, 1000] {
        let size = page_count * alloc_granularity();
        let ptr =
            unsafe { pagealloc::alloc(ptr::null_mut(), size, pagealloc::Protection::ReadWrite) }
                .unwrap();
        for i in 0..size {
            unsafe { *ptr.as_ptr().add(i) = i as u8 };
        }
        for i in 0..size {
            assert_eq!(unsafe { *ptr.as_ptr().add(i) }, i as u8);
        }
        unsafe { pagealloc::dealloc(ptr, size).unwrap() };
    }
}

#[test]
fn alloc_write_clear_dealloc() {
    for page_count in [1, 10, 100, 1000] {
        let size = page_count * alloc_granularity();
        let ptr =
            unsafe { pagealloc::alloc(ptr::null_mut(), size, pagealloc::Protection::ReadWrite) }
                .unwrap();
        for i in 0..size {
            unsafe { *ptr.as_ptr().add(i) = i as u8 };
        }
        for i in 0..size {
            assert_eq!(unsafe { *ptr.as_ptr().add(i) }, i as u8);
        }
        unsafe { pagealloc::clear(ptr, size, pagealloc::Protection::ReadWrite).unwrap() };
        for i in 0..size {
            assert_eq!(unsafe { *ptr.as_ptr().add(i) }, 0);
        }
        unsafe { pagealloc::dealloc(ptr, size).unwrap() };
    }
}

#[test]
fn alloc_write_protect_dealloc() {
    for page_count in [1, 10, 100, 1000] {
        let size = page_count * alloc_granularity();
        let ptr =
            unsafe { pagealloc::alloc(ptr::null_mut(), size, pagealloc::Protection::ReadWrite) }
                .unwrap();
        for i in 0..size {
            unsafe { *ptr.as_ptr().add(i) = i as u8 };
        }
        unsafe { pagealloc::protect(ptr, size, pagealloc::Protection::Read) }.unwrap();
        for i in 0..size {
            assert_eq!(unsafe { *ptr.as_ptr().add(i) }, i as u8);
        }
        unsafe { pagealloc::dealloc(ptr, size).unwrap() };
    }
}

#[test]
fn alloc_write_advise_free_dealloc() {
    for page_count in [1, 10, 100, 1000] {
        let size = page_count * alloc_granularity();
        let ptr =
            unsafe { pagealloc::alloc(ptr::null_mut(), size, pagealloc::Protection::ReadWrite) }
                .unwrap();
        for i in 0..size {
            unsafe { *ptr.as_ptr().add(i) = i as u8 };
        }
        unsafe { pagealloc::advise_free(ptr, size) }.unwrap();
        unsafe { pagealloc::dealloc(ptr, size).unwrap() };
    }
}

#[test]
fn clear_buffer() {
    #[repr(align(4096))]
    struct Buffer([u8; 8192]);

    if pagealloc::page_size() != 4096 {
        eprintln!("page size is not 4096, skip test");
        return;
    }

    let mut buffer = Buffer([1; 8192]);
    unsafe {
        pagealloc::clear(
            ptr::NonNull::new(buffer.0.as_mut_ptr()).unwrap(),
            4096,
            pagealloc::Protection::ReadWrite,
        )
    }
    .unwrap();
    for i in 0..4096 {
        assert_eq!(buffer.0[i], 0);
    }
    for i in 4096..8192 {
        assert_eq!(buffer.0[i], 1);
    }
}

#[test]
fn clear_buffer_misaligned() {
    #[repr(align(4096))]
    struct Buffer([u8; 8192]);

    if pagealloc::page_size() != 4096 {
        eprintln!("page size is not 4096, skip test");
        return;
    }

    let mut buffer = Buffer([1; 8192]);
    assert!(
        unsafe {
            pagealloc::clear(
                ptr::NonNull::new(buffer.0.as_mut_ptr()).unwrap().add(10),
                4096,
                pagealloc::Protection::ReadWrite,
            )
        }
        .is_err()
    );
}

#[test]
fn advise_free_buffer() {
    #[repr(align(4096))]
    struct Buffer([u8; 8192]);

    if pagealloc::page_size() != 4096 {
        eprintln!("page size is not 4096, skip test");
        return;
    }

    let mut buffer = Buffer([1; 8192]);
    unsafe { pagealloc::advise_free(ptr::NonNull::new(buffer.0.as_mut_ptr()).unwrap(), 4096) }
        .unwrap();
    for i in 0..4096 {
        assert!(buffer.0[i] == 0 || buffer.0[i] == 1);
    }
    for i in 4096..8192 {
        assert_eq!(buffer.0[i], 1);
    }
}

#[test]
fn advise_free_buffer_misaligned() {
    #[repr(align(4096))]
    struct Buffer([u8; 8192]);

    if pagealloc::page_size() != 4096 {
        eprintln!("page size is not 4096, skip test");
        return;
    }

    let mut buffer = Buffer([1; 8192]);
    assert!(
        unsafe {
            pagealloc::advise_free(
                ptr::NonNull::new(buffer.0.as_mut_ptr()).unwrap().add(10),
                4096,
            )
        }
        .is_err()
    );
}
