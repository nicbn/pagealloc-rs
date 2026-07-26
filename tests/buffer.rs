#![cfg(feature = "buffer")]

use pagealloc::alloc_granularity;

#[test]
fn read_only() {
    let size = 100 * alloc_granularity();
    let buf = pagealloc::buffer::ByteBuffer::new(size, pagealloc::Protection::Read).unwrap();
    for i in 0..size {
        assert_eq!(buf.get()[i], 0);
    }
}

#[test]
fn read_only_misaligned() {
    let size = 100 * alloc_granularity();
    assert!(pagealloc::buffer::ByteBuffer::new(size + 1, pagealloc::Protection::Read).is_err());
}

#[test]
fn read_write() {
    let size = 100 * alloc_granularity();
    let mut buf =
        pagealloc::buffer::ByteBuffer::new(size, pagealloc::Protection::ReadWrite).unwrap();
    for i in 0..size {
        assert_eq!(buf.get()[i], 0);
    }
    for i in 0..size {
        buf.get_mut()[i] = i as u8;
    }
    for i in 0..size {
        assert_eq!(buf.get()[i], i as u8);
    }
}

#[test]
fn protect() {
    let alloc_granularity = alloc_granularity();
    let size = 100 * alloc_granularity;
    let mut buf =
        pagealloc::buffer::ByteBuffer::new(size, pagealloc::Protection::ReadWrite).unwrap();
    for i in 0..size {
        assert_eq!(buf.get()[i], 0);
    }
    for i in 0..size {
        buf.get_mut()[i] = i as u8;
    }
    for i in 0..size {
        assert_eq!(buf.get()[i], i as u8);
    }
    buf.protect(0..alloc_granularity, pagealloc::Protection::Read)
        .unwrap();
    for i in alloc_granularity..size {
        buf.get_mut()[i] = 0;
    }
}

#[test]
fn clear() {
    let alloc_granularity = alloc_granularity();
    let size = 100 * alloc_granularity;
    let mut buf =
        pagealloc::buffer::ByteBuffer::new(size, pagealloc::Protection::ReadWrite).unwrap();
    buf.get_mut().fill(1);
    buf.clear(0..alloc_granularity, pagealloc::Protection::ReadWrite)
        .unwrap();
    for i in 0..alloc_granularity {
        assert_eq!(buf.get()[i], 0);
    }
}

#[test]
fn protect_misaligned() {
    let alloc_granularity = alloc_granularity();
    let size = 100 * alloc_granularity;
    let mut buf =
        pagealloc::buffer::ByteBuffer::new(size, pagealloc::Protection::ReadWrite).unwrap();
    assert!(buf
        .protect(1..alloc_granularity, pagealloc::Protection::Read)
        .is_err());
}

#[test]
fn advise_free() {
    let alloc_granularity = alloc_granularity();
    let size = 100 * alloc_granularity;
    let mut buf =
        pagealloc::buffer::ByteBuffer::new(size, pagealloc::Protection::ReadWrite).unwrap();
    buf.advise_free(0..alloc_granularity).unwrap();
}

#[test]
fn advise_free_misaligned() {
    let alloc_granularity = alloc_granularity();
    let size = 100 * alloc_granularity;
    let mut buf =
        pagealloc::buffer::ByteBuffer::new(size, pagealloc::Protection::ReadWrite).unwrap();
    assert!(buf.advise_free(1..alloc_granularity).is_err());
}

#[test]
fn from_raw_parts() {
    let size = 100 * alloc_granularity();
    let mut buf =
        pagealloc::buffer::ByteBuffer::new(size, pagealloc::Protection::ReadWrite).unwrap();
    for i in 0..size {
        buf.get_mut()[i] = i as u8;
    }
    let (ptr, len) = buf.into_raw_parts();
    let buf = unsafe { pagealloc::buffer::ByteBuffer::from_raw_parts(ptr, len) };
    for i in 0..size {
        assert_eq!(buf.get()[i], i as u8);
    }
}
