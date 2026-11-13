//! `libio/stdio.c`: the standard streams `stdin`, `stdout` and `stderr`.
//!
//! [`stderr`] is C's standard error stream, lent for the whole program as a
//! [`FileRef<'static>`](crate::libc::libio::genops::FileRef). `stdin` and `stdout` are not
//! bound yet.

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
    /// C's `stderr` expression does, lent for the rest of the program.
    ///
    /// glibc defines `stderr` with its stream's address, `_IO_2_1_stderr_`, a stream in
    /// static storage, unbuffered and open on descriptor 2, which is never freed: even
    /// `fclose` keeps the storage of a standard stream. No glibc routine assigns `stderr`
    /// (only its tests and its programs do, and a compatibility constructor for binaries
    /// linked against glibc 2.0, which runs before `main`), so every call returns that
    /// stream. It is shared: every C stdio routine and every operation bound on
    /// [`FileRef`] holds its lock, as [`FileRef`]'s contract for a borrowed stream says.
    ///
    /// The `'static` borrow assumes what that contract asks of a stream for the whole
    /// program, which neither the C of `crustify-rs/libgit2-bindings` nor that of
    /// `crustify-rs/ffmpeg-bindings` breaks:
    ///
    /// - no code assigns `stderr` while another thread may read it, and whatever stream it names
    ///   stays open until the program ends: nothing passes it to `fclose`, or to a `freopen` that
    ///   fails and leaves it closed. A `freopen` that succeeds keeps the stream open at its
    ///   address, under its lock throughout;
    /// - its locking stays internal, glibc's default: nothing sets it to `FSETLOCKING_BYCALLER`
    ///   with `__fsetlocking`, and an `_unlocked` routine reaches it only under `flockfile`, as
    ///   glibc's own `error` does.
    ///
    /// A handle taken before C assigns `stderr` keeps the stream it was taken from.
    ///
    /// # Panics
    ///
    /// If C has set `stderr` to NULL, which no C of glibc or of either parent does.
    #[must_use]
    pub fn stderr() -> FileRef<'static> {
        // SAFETY: `stderr` is a pointer-sized global glibc initialises statically, before
        // any Rust code runs, and no write races with this read of it: glibc never writes
        // it, and the assumptions above exclude any other write while another thread may
        // read it.
        let stream = unsafe { ffi::stderr };
        // SAFETY: `stream` is NULL, which `from_ptr` refuses, or an open stream, glibc's
        // `_IO_2_1_stderr_` unless C assigned another, which stays open until the program
        // ends and whose every access holds its lock, as the assumptions above state: the
        // contract of `FileRef::from_ptr` for a stream, for the `'static` lifetime.
        unsafe { FileRef::from_ptr(stream) }.expect("glibc's `stderr` is never NULL")
    }
}
