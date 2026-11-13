//! `stderr()` lends a `FileRef<'static>` from safe code, so a safe caller may keep it. Then
//! C does what glibc's manual offers for redirecting a standard stream, "normal variables
//! which you can set just like any others" (`c/manual/stdio.texi:115-122`): it closes the
//! stream `stderr` names and points `stderr` at another. The kept handle is still a live
//! `'static` borrow, of a stream `fclose` freed.
//!
//! The program is single-threaded, and every C step is legal C under glibc's contract for
//! the variable. The C steps are written in Rust through extern declarations, as C would
//! make them; the safe crate is reached through its safe API alone. Run under ASan, which
//! the affected commit's `wave-4` tests also build with:
//!
//! RUSTFLAGS="-Zsanitizer=address -Cforce-frame-pointers=yes" \
//!   cargo +nightly-2026-10-08 run --target x86_64-unknown-linux-gnu

use core::ffi::{c_char, c_int, c_void};
use std::process::ExitCode;

use ffibox::CRefSeam;
use glibc_bindings::libc::ffi;
use glibc_bindings::libc::libio::genops::FileRef;
use glibc_bindings::libc::libio::stdio::stderr;

unsafe extern "C" {
    fn fopen(path: *const c_char, mode: *const c_char) -> *mut ffi::FILE;
    fn fclose(stream: *mut ffi::FILE) -> c_int;
    /// ASan's interface: 1 when `addr` is poisoned, as freed memory in quarantine is.
    fn __asan_address_is_poisoned(addr: *const c_void) -> c_int;
}

/// What C does before the Rust runs: the manual's redirection, `stderr = fopen(...)`.
fn c_redirects_stderr() -> *mut ffi::FILE {
    // SAFETY: nothing else runs; `stderr` is read and written on this one thread.
    let original = unsafe { ffi::stderr };
    // SAFETY: NUL-terminated literals.
    let log = unsafe { fopen(c"/dev/null".as_ptr(), c"w".as_ptr()) };
    assert!(!log.is_null(), "fopen(/dev/null) failed");
    // SAFETY: as above.
    unsafe { ffi::stderr = log };
    original
}

/// What C does later, its log rotated: `fclose(stderr); stderr = <another stream>`.
fn c_closes_and_reassigns_stderr(next: *mut ffi::FILE) {
    // SAFETY: `stderr` names the stream `c_redirects_stderr` opened, which nothing else
    // uses; this thread is the only one.
    let closed = unsafe { fclose(ffi::stderr) };
    assert_eq!(closed, 0, "fclose failed");
    // SAFETY: as above.
    unsafe { ffi::stderr = next };
}

fn main() -> ExitCode {
    let original = c_redirects_stderr();

    // Safe Rust: a 'static handle, which the API lets a caller keep for the program's life.
    let kept: FileRef<'static> = stderr();

    c_closes_and_reassigns_stderr(original);

    // `kept` is still a valid 'static `FileRef` by its type. Its stream has been freed.
    let p = kept.as_ptr();
    // SAFETY: an ASan interface query; it reads only ASan's shadow memory.
    let freed = unsafe { __asan_address_is_poisoned(p.cast()) } == 1;
    if !freed {
        println!("the kept handle's stream is not freed: no finding");
        return ExitCode::SUCCESS;
    }
    eprintln!(
        "stderr() lent a FileRef<'static> over {p:p}, a stream fclose has freed; \
         the next operation on it reads freed memory:"
    );
    // What wave 3's `FileRef::print` does first, `flockfile(stream)`, reads the stream's
    // `_lock`. glibc is not instrumented, so the read is made here, where ASan sees it.
    // SAFETY: none: this read is the use after free the 'static borrow permits.
    let lock = unsafe { core::ptr::read_volatile(&raw const (*p)._lock) };
    println!("read the freed stream's lock pointer: {lock:p}");
    ExitCode::FAILURE
}
