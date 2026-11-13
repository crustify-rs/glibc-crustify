//! `FileRef::print` on the stream glibc hands a `register_printf_specifier` handler.
//!
//! glibc 2.37 and later hand a custom conversion's handler a `FILE` of their own over the
//! printf buffer, which has no lock: its `_lock` is NULL and its locking is the caller's
//! (`c/stdio-common/printf_buffer_as_file.c`, `__printf_buffer_as_file_init`). glibc's own
//! stdio routines write it, skipping the lock. The handler borrows it as `FileRef::from_ptr`'s
//! contract asks: the stream is open while the handler runs, which the handle does not
//! outlast, and nothing else accesses it meanwhile, the printf that called the handler
//! waiting on this thread. `print` took the stream's lock with `flockfile`, which
//! dereferences `_lock`: on the affected commit this exits on a SEGV, which ASan reports. Once
//! `print` refuses a stream without a lock, it exits 0.

use core::ffi::{c_char, c_int, c_void};

use ffibox::CRefSeam;
use glibc_bindings::libc::ffi;
use glibc_bindings::libc::libio::genops::FileRef;

unsafe extern "C" {
    /// `<printf.h>`'s, which the raw bindings do not declare.
    fn register_printf_specifier(
        spec: c_int,
        func: unsafe extern "C" fn(*mut ffi::FILE, *const c_void, *const *const c_void) -> c_int,
        arginfo: unsafe extern "C" fn(*const c_void, usize, *mut c_int, *mut c_int) -> c_int,
    ) -> c_int;
}

/// `%W` takes no argument.
unsafe extern "C" fn arginfo(_: *const c_void, _: usize, _: *mut c_int, _: *mut c_int) -> c_int {
    0
}

/// `%W` writes its text through the safe binding, as a Rust handler would.
unsafe extern "C" fn handler(
    stream: *mut ffi::FILE,
    _: *const c_void,
    _: *const *const c_void,
) -> c_int {
    eprintln!("handler: stream {stream:p}, _lock {:p}", unsafe { (*stream)._lock });
    // SAFETY: glibc's open stream for the handler's call, which the handle does not outlast;
    // nothing else accesses it while the handler runs.
    let Some(file) = (unsafe { FileRef::from_ptr(stream) }) else {
        return -1;
    };
    // No panic may unwind into glibc: the outcome is reported instead.
    match file.print("from Rust") {
        Ok(count) => c_int::try_from(count).unwrap_or(-1),
        Err(e) => {
            eprintln!("print refused the stream: {e} ({:?})", e.kind());
            0
        }
    }
}

fn main() {
    // SAFETY: a conversion letter glibc lets a program register, with two functions of the
    // signatures `<printf.h>` gives.
    let r = unsafe { register_printf_specifier(c_int::from(b'W'), handler, arginfo) };
    assert_eq!(r, 0, "register_printf_specifier failed");
    // SAFETY: two NUL-terminated strings.
    let out = unsafe { ffi::fopen(c"/dev/null".as_ptr(), c"w".as_ptr()) };
    assert!(!out.is_null());
    // SAFETY: an open stream and a format whose one conversion takes no argument.
    let n = unsafe { ffi::fprintf(out, c"[%W]".as_ptr().cast::<c_char>()) };
    // SAFETY: the stream opened above, closed once.
    unsafe { ffi::fclose(out) };
    // Reached only when `print` returned: the handler wrote `from Rust`, or `print` refused
    // the stream and the handler wrote nothing.
    println!("fprintf returned {n}");
    assert!(n == 2 || n == 11, "fprintf failed: {n}");
}
