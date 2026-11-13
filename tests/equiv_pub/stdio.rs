//! `stderr` against C's `stderr`: the handle is the stream C's expression reads.

use std::thread;

use ffibox::CRefSeam;
use glibc_bindings::libc::ffi;
use glibc_bindings::libc::libio::stdio::stderr;

/// C's `stderr` expression: a read of the global.
fn c_stderr() -> *const ffi::FILE {
    // SAFETY: `stderr` is initialised before any Rust code runs, and no test writes it, so
    // the read races with no write.
    unsafe { ffi::stderr }.cast_const()
}

#[test]
#[cfg_attr(miri, ignore = "reads a C global")]
fn stderr_is_the_stream_c_names() {
    for _ in 0..3 {
        let c = c_stderr();
        assert!(!c.is_null());
        assert_eq!(stderr().as_ptr(), c);
        assert_eq!(c_stderr(), c);
    }
    let there = thread::spawn(|| (stderr().as_ptr().addr(), c_stderr().addr()))
        .join()
        .unwrap();
    assert_eq!(there.0, there.1);
    assert_eq!(there.0, c_stderr().addr());
}
