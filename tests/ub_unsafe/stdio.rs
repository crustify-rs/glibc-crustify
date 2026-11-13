//! `stderr` within its contract: a handle the caller vouches for, kept past every scope and
//! used from many threads at once.
//!
//! Nothing in this test binary assigns `stderr`, closes the stream it names or changes its
//! locking, so every test may vouch for any lifetime, `'static` included.

use std::sync::OnceLock;
use std::thread;

use ffibox::CRefSeam;
use glibc_bindings::libc::libio::genops::FileRef;
use glibc_bindings::libc::libio::stdio::stderr;

/// `stderr` for the rest of the program.
fn for_good() -> FileRef<'static> {
    // SAFETY: nothing in this binary writes `stderr`, closes its stream or unlocks it, as
    // the module's documentation states, so the stream stays open, and every access to it
    // locked, until the program ends.
    unsafe { stderr() }
}

/// A handle kept in a `static`, which outlives every scope that took it.
static KEPT: OnceLock<FileRef<'static>> = OnceLock::new();

#[test]
#[cfg_attr(miri, ignore = "reads a C global")]
fn a_static_handle_outlives_every_scope() {
    let kept = *KEPT.get_or_init(for_good);
    let detached = thread::spawn(move || kept.as_ptr().addr());
    assert_eq!(detached.join().unwrap(), for_good().as_ptr().addr());
    assert_eq!(KEPT.get().unwrap().as_ptr(), for_good().as_ptr());
}

#[test]
#[cfg_attr(miri, ignore = "reads a C global")]
fn threads_take_and_share_it_at_once() {
    let mine = for_good();
    let addrs: Vec<_> = thread::scope(|s| {
        let threads: Vec<_> = (0..16)
            .map(|_| {
                s.spawn(move || {
                    // SAFETY: as in `for_good`, for the scope this handle lives in.
                    let mut taken = unsafe { stderr() }.as_ptr().addr();
                    for _ in 0..1000 {
                        let again = for_good().as_ptr().addr();
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
