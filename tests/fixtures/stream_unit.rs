//! The parts of the stream fixture that only `unit_pub` uses: a hold of a stream's lock, as
//! C that locks a stream itself holds it, and the stream glibc hands a printf handler.

use core::cell::RefCell;
use core::ffi::{c_int, c_void};
use std::panic::{self, AssertUnwindSafe};
use std::rc::Rc;
use std::sync::Once;

use ffibox::CRefSeam;
use glibc_bindings::libc::ffi;
use glibc_bindings::libc::libio::genops::FileRef;

use crate::stream::Stream;

/// A hold of a stream's lock, taken with `flockfile` and released with `funlockfile` when
/// dropped, on the thread that took it: it borrows the stream, which is not `Sync`, so it
/// cannot cross threads.
pub(crate) struct LockHold<'a>(&'a Stream);

impl Stream {
    /// Takes the stream's lock, as C does with `flockfile`, until the hold is dropped.
    pub(crate) fn hold_lock(&self) -> LockHold<'_> {
        // SAFETY: an open stream, which the borrow keeps open until the hold releases it.
        unsafe { ffi::flockfile(self.as_ptr()) };
        LockHold(self)
    }
}

impl Drop for LockHold<'_> {
    fn drop(&mut self) {
        // SAFETY: this thread took the lock in `hold_lock`, and releases it once, here.
        unsafe { ffi::funlockfile(self.0.as_ptr()) };
    }
}

/// A printf handler, as `<printf.h>`'s `printf_function` gives it: the stream, the
/// conversion's `struct printf_info`, and its arguments.
type Handler = unsafe extern "C" fn(*mut ffi::FILE, *const c_void, *const *const c_void) -> c_int;

/// A conversion's argument types, as `<printf.h>`'s `printf_arginfo_size_function` gives it.
type ArgInfo = unsafe extern "C" fn(*const c_void, usize, *mut c_int, *mut c_int) -> c_int;

unsafe extern "C" {
    /// `<printf.h>`'s `register_printf_specifier`, declared here rather than in the raw
    /// bindings: its function types take `struct printf_info`, whose bit-fields would add
    /// some 1300 generated lines to `libc-sys` for this fixture alone, which passes the
    /// struct on by pointer only, as `*const c_void`.
    fn register_printf_specifier(spec: c_int, func: Handler, arginfo: ArgInfo) -> c_int;
}

/// What `%W`'s handler runs on glibc's stream.
type Run = Box<dyn FnOnce(FileRef<'_>)>;

thread_local! {
    /// What `%W`'s handler runs, once, on the calling thread.
    static RUN: RefCell<Option<Run>> = const { RefCell::new(None) };
}

/// `%W` takes no argument.
unsafe extern "C" fn arginfo(_: *const c_void, _: usize, _: *mut c_int, _: *mut c_int) -> c_int {
    0
}

/// `%W`'s handler: runs what `in_printf_handler` set on glibc's stream, writing nothing of
/// its own.
unsafe extern "C" fn handler(
    stream: *mut ffi::FILE,
    _: *const c_void,
    _: *const *const c_void,
) -> c_int {
    let Some(run) = RUN.take() else {
        return 0;
    };
    // SAFETY: glibc's stream for the handler's call, open until the handler returns, which
    // the handle does not outlast; nothing else accesses it meanwhile, the printf that
    // called the handler waiting on this thread.
    let Some(file) = (unsafe { FileRef::from_ptr(stream) }) else {
        return -1;
    };
    // A panic, such as a failed assertion, must not unwind into glibc: caught, it leaves no
    // result, which `in_printf_handler` reports.
    if panic::catch_unwind(AssertUnwindSafe(|| run(file))).is_err() {
        return -1;
    }
    0
}

/// Runs `f` on the stream glibc 2.37 and later hand the handler of a conversion a program
/// registers, `%W` here, and returns what it returned. That stream writes into printf's own
/// buffer and has no lock: its `_lock` is NULL (`c/stdio-common/printf_buffer_as_file.c`).
/// It is borrowed for the handler's call, as a C handler that hands it to Rust lends it.
///
/// Registering a conversion makes every later printf of the process take glibc's
/// positional path, which writes a `"%s"` alike.
pub(crate) fn in_printf_handler<R: 'static>(f: impl FnOnce(FileRef<'_>) -> R + 'static) -> R {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        // SAFETY: a conversion letter glibc leaves to programs, with two functions of the
        // signatures `<printf.h>` gives them.
        let r = unsafe { register_printf_specifier(c_int::from(b'W'), handler, arginfo) };
        assert_eq!(r, 0, "register_printf_specifier failed");
    });
    let out = Rc::new(RefCell::new(None));
    let slot = Rc::clone(&out);
    RUN.set(Some(Box::new(move |file| {
        *slot.borrow_mut() = Some(f(file));
    })));
    let stream = Stream::memory();
    // SAFETY: an open stream and a format whose one conversion takes no argument.
    let ret = unsafe { ffi::fprintf(stream.as_ptr(), c"%W".as_ptr()) };
    RUN.take();
    assert_eq!(ret, 0, "printf failed, or the handler failed or panicked");
    out.take().expect("the printf handler did not run")
}
