# pagealloc

[![crates.io](https://img.shields.io/crates/v/pagealloc.svg)](https://crates.io/crates/pagealloc)
[![docs.rs](https://docs.rs/pagealloc/badge.svg)](https://docs.rs/pagealloc)
[![CI](https://github.com/nicbn/pagealloc-rs/actions/workflows/build_and_test.yaml/badge.svg)](https://github.com/nicbn/pagealloc-rs/actions/workflows/build_and_test.yaml)

Low-level, cross-platform page allocation library.

This crate provides a cross-platform interface to the operating system's virtual memory
allocation, such as `mmap` on Unix and `VirtualAlloc` on Windows. It allows allocating pages of
memory, clearing them, marking them as free, and changing memory protections.

The API is mostly unsafe, and an optional `buffer` feature is provided to enable a safe wrapper
around page allocation.

Read more in the official [crate documentation](https://docs.rs/pagealloc).

## Platform support

UNIX-like systems and Windows are currently supported.

The following platforms are tested in CI:

* Windows
* Linux
* macOS

Contributions to support other platforms are welcome.

## Minimum Supported Rust Version

Currently the Minimum Supported Rust Version (MSRV) is 1.85. This version may
be increased in the future with a minor release bump.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
