//! `stderr` against C's `stderr`: the handle is the stream C's expression reads, on every
//! call, before and after C points `stderr` at another stream.

use std::sync::{Mutex, MutexGuard, PoisonError};
use std::thread;

use ffibox::CRefSeam;
use glibc_bindings::libc::ffi;
use glibc_bindings::libc::libio::stdio::stderr;

/// Held by every test of this file while it reads or writes C's `stderr`: one test writes
/// it, and no write may race with a read, C's expression's or `stderr()`'s. No other test of
/// this binary reads it.
static STDERR: Mutex<()> = Mutex::new(());

fn only_this_test() -> MutexGuard<'static, ()> {
    STDERR.lock().unwrap_or_else(PoisonError::into_inner)
}

/// C's `stderr` expression: a read of the global, made under [`STDERR`].
fn c_stderr() -> *mut ffi::FILE {
    // SAFETY: `stderr` is initialised before any Rust code runs, and its every write in this
    // binary is made under `STDERR`, which the caller holds, so the read races with none.
    unsafe { ffi::stderr }
}

/// C's assignment `stderr = stream`, made under [`STDERR`].
fn c_sets_stderr(stream: *mut ffi::FILE) {
    // SAFETY: as in `c_stderr`: the caller holds `STDERR`, so no read or write races with it.
    unsafe { ffi::stderr = stream };
}

/// Puts `stderr` back as it was, should an assertion fail while it names another stream.
struct Restore(*mut ffi::FILE);

impl Drop for Restore {
    fn drop(&mut self) {
        c_sets_stderr(self.0);
    }
}

#[test]
#[cfg_attr(miri, ignore = "reads a C global")]
fn stderr_is_the_stream_c_names() {
    let _only = only_this_test();
    for _ in 0..3 {
        let c = c_stderr();
        assert!(!c.is_null());
        // SAFETY: no write of `stderr` races with the read (`STDERR` is held), and its stream,
        // which nothing in this binary closes or unlocks, outlives the handle.
        assert_eq!(unsafe { stderr() }.as_ptr(), c.cast_const());
        assert_eq!(c_stderr(), c);
    }
    let there = thread::spawn(|| {
        // SAFETY: as above: the spawning test holds `STDERR` until this thread is joined.
        let rust = unsafe { stderr() }.as_ptr().addr();
        (rust, c_stderr().addr())
    })
    .join()
    .unwrap();
    assert_eq!(there.0, there.1);
    assert_eq!(there.0, c_stderr().addr());
}

#[test]
#[cfg_attr(miri, ignore = "reads a C global")]
fn stderr_follows_c_pointing_it_at_another_stream() {
    let _only = only_this_test();
    let original = c_stderr();
    // SAFETY: two NUL-terminated literals.
    let other = unsafe { ffi::fopen(c"/dev/null".as_ptr(), c"w".as_ptr()) };
    assert!(!other.is_null(), "fopen(\"/dev/null\", \"w\") failed");
    let restore = Restore(original);

    // C redirects its standard error stream, as glibc's manual offers.
    c_sets_stderr(other);
    // SAFETY: no write of `stderr` races with the read (`STDERR` is held); `other` stays open,
    // and nothing else accesses it, until the `fclose` below, after the handle's last use.
    let redirected = unsafe { stderr() };
    assert_eq!(redirected.as_ptr(), other.cast_const());
    assert_eq!(redirected.as_ptr(), c_stderr().cast_const());

    // C points it back: a later call reads the stream it names now, and the handle taken
    // before keeps the stream it was taken from.
    drop(restore);
    assert_eq!(c_stderr(), original);
    // SAFETY: as in `stderr_is_the_stream_c_names`, the original stream.
    let restored = unsafe { stderr() };
    assert_eq!(restored.as_ptr(), original.cast_const());
    assert_eq!(redirected.as_ptr(), other.cast_const());

    // SAFETY: `other` is the stream `fopen` opened, which no handle reaches past here and
    // which `stderr` no longer names.
    assert_eq!(unsafe { ffi::fclose(other) }, 0);
}
