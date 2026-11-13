//! `libio/stdio.c`: the standard streams `stdin`, `stdout` and `stderr`.
//!
//! [`stderr`] is C's standard error stream, lent as a
//! [`FileRef`](crate::libc::libio::genops::FileRef) for as long as its caller vouches that the
//! program keeps it open: glibc's `stderr` is a variable the program's C may set, and the
//! stream it names one that C may close. `stdin` and `stdout` are not bound yet.

pub use ffi_import::stderr;

#[expect(
    unsafe_code,
    reason = "reads glibc's `stderr` and borrows the stream it names"
)]
mod ffi_import {
    use ffibox::CRefSeam;

    use crate::libc::ffi;
    use crate::libc::libio::genops::FileRef;

    /// Wraps `glb::stderr`
    ///
    /// The standard error stream: the stream C's `stderr` names when the call reads it, as
    /// C's `stderr` expression does, lent for `'a`, the lifetime the caller chooses.
    ///
    /// glibc defines `stderr` with its stream's address, `_IO_2_1_stderr_`, a stream in
    /// static storage, unbuffered and open on descriptor 2, which is never freed: even
    /// `fclose` keeps the storage of a standard stream. No glibc routine assigns `stderr`
    /// (only its tests and its programs do, and a compatibility constructor for binaries
    /// linked against glibc 2.0, which runs before `main`). It is shared: every C stdio
    /// routine and every operation bound on [`FileRef`] holds its lock, as [`FileRef`]'s
    /// contract for a borrowed stream says.
    ///
    /// But glibc's `stderr` is a variable which a program "can set just like any others",
    /// as its manual says (*Standard Streams*), giving `fclose (stdout); stdout = fopen
    /// (…);` as the way to redirect a standard stream. A stream so opened is freed by the
    /// `fclose` that closes it. What the program's C does with `stderr` is beyond the
    /// crate's sight, so the caller vouches for it, for the handle's lifetime. A handle
    /// taken for one call, as C's `fprintf (stderr, …)` takes the stream, asks of the program
    /// what that C expression asks of it. One kept for the rest of the program, `'static`,
    /// asks that nothing ever closes it.
    ///
    /// A handle taken before C assigns `stderr` keeps the stream it was taken from.
    ///
    /// # Safety
    ///
    /// - No write of `stderr` races with this read: C assigns it only while no other thread may
    ///   read it.
    /// - The stream `stderr` names stays open for `'a`: nothing passes it to `fclose`, or to a
    ///   `freopen` that fails and leaves it closed, while the handle lives. A `freopen` that
    ///   succeeds keeps the stream open at its address, under its lock throughout.
    /// - Every other access to that stream while the handle lives, C's or Rust's, on any thread,
    ///   holds its lock: [`FileRef`]'s contract for a borrowed stream. Its locking stays internal,
    ///   glibc's default, so that nothing sets it to `FSETLOCKING_BYCALLER` with `__fsetlocking`,
    ///   and an `_unlocked` routine reaches it only under `flockfile`, as glibc's own `error` does.
    ///
    /// # Panics
    ///
    /// If C has set `stderr` to NULL, which no C of glibc or of either parent does.
    #[must_use]
    pub unsafe fn stderr<'a>() -> FileRef<'a> {
        // SAFETY: `stderr` is a pointer-sized global glibc initialises statically, before
        // any Rust code runs. glibc never writes it, and the caller ensures that no other
        // write races with this read.
        let stream = unsafe { ffi::stderr };
        // SAFETY: `stream` is NULL, which `from_ptr` refuses, or the stream `stderr` names,
        // which the caller keeps open for `'a`, every other access to it holding its lock:
        // the contract of `FileRef::from_ptr` for a stream.
        unsafe { FileRef::from_ptr(stream) }.expect("glibc's `stderr` is never NULL")
    }
}
