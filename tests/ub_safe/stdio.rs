//! `stderr` through the safe API: a `'static` shared handle, kept past every scope and used
//! from many threads at once.

use std::sync::OnceLock;
use std::thread;

use ffibox::CRefSeam;
use glibc_bindings::libc::libio::genops::FileRef;
use glibc_bindings::libc::libio::stdio::stderr;

/// A handle kept in a `static`, which outlives every scope that took it.
static KEPT: OnceLock<FileRef<'static>> = OnceLock::new();

#[test]
#[cfg_attr(miri, ignore = "reads a C global")]
fn the_handle_outlives_every_scope() {
    let kept = *KEPT.get_or_init(stderr);
    let detached = thread::spawn(move || kept.as_ptr().addr());
    assert_eq!(detached.join().unwrap(), stderr().as_ptr().addr());
    assert_eq!(KEPT.get().unwrap().as_ptr(), stderr().as_ptr());
}

#[test]
#[cfg_attr(miri, ignore = "reads a C global")]
fn threads_take_and_share_it_at_once() {
    let mine = stderr();
    let addrs: Vec<_> = thread::scope(|s| {
        let threads: Vec<_> = (0..16)
            .map(|_| {
                s.spawn(move || {
                    let mut taken = stderr().as_ptr().addr();
                    for _ in 0..1000 {
                        let again = stderr().as_ptr().addr();
                        assert_eq!(again, taken);
                        assert_eq!(mine.as_ptr().addr(), taken);
                        taken = again;
                    }
                    taken
                })
            })
            .collect();
        threads.into_iter().map(|t| t.join().unwrap()).collect()
    });
    assert!(addrs.iter().all(|&a| a == mine.as_ptr().addr()));
}
