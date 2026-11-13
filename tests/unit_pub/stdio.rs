//! `stderr`: one stream, the same on every call and every thread.

use std::thread;

use ffibox::CRefSeam;
use glibc_bindings::libc::libio::stdio::stderr;

#[test]
#[cfg_attr(miri, ignore = "reads a C global")]
fn every_call_lends_the_same_stream() {
    let first = stderr();
    assert!(!first.as_ptr().is_null());
    for _ in 0..3 {
        assert_eq!(stderr().as_ptr(), first.as_ptr());
    }
}

#[test]
#[cfg_attr(miri, ignore = "reads a C global")]
fn every_thread_is_lent_the_same_stream() {
    let here = stderr().as_ptr().addr();
    let there = thread::spawn(|| stderr().as_ptr().addr()).join().unwrap();
    assert_eq!(there, here);
}
