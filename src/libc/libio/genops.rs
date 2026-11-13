//! `libio/genops.c`: the generic stream operations, home of `FILE` (`struct _IO_FILE`).
//!
//! [`File`] is C's stream, bound opaque and borrowed: the routines that open, write and
//! close a stream are C's, and Rust holds a stream only through the [`FileRef`] that
//! whatever keeps it open lends.

pub use ffi_import::{File, FileMut, FileRef};

#[expect(unsafe_code, reason = "asserts the thread safety of glibc's streams")]
mod ffi_import {
    use ffibox::define_ctype;

    use crate::libc::ffi;

    define_ctype!(
        /// Wraps `st::_IO_FILE`
        ///
        /// Wraps `tdf::FILE`
        ///
        /// C's stream, `FILE`: a file, a buffer or a cookie behind glibc's stdio buffer,
        /// which C's stdio routines read, write and close. Opaque: the safe API reads and
        /// writes none of its fields.
        ///
        /// Borrowed only. C allocates every stream (`fopen`, `fdopen`, `tmpfile`,
        /// `open_memstream` and their kin, or the static storage of `stdin`, `stdout` and
        /// `stderr`) and releases it with `fclose`, and glibc keeps the address of every
        /// file-backed stream in its list of open streams, which `exit` flushes, from its
        /// opening to its close. So no `File` value exists in Rust, and no owner: a stream
        /// is reached through a [`FileRef`] borrowed from what keeps it open, the C caller
        /// that hands it for a call, or the program, for the standard streams.
        ///
        /// Shared. C writes a stream's buffer and state from every stdio call, under the
        /// stream's lock: a recursive lock of its own, which each stdio routine takes,
        /// unless the stream's locking is the caller's (`__fsetlocking` with
        /// `FSETLOCKING_BYCALLER`) or the routine is an `_unlocked` one, and which
        /// `flockfile` takes whatever the stream's locking. [`FileRef`] is `Send` and
        /// `Sync` on that lock: every operation bound on it holds the lock, taken with
        /// `flockfile`, for the whole of its access to the stream, and reads nothing of
        /// the stream without it. No producer yields a [`FileMut`]: nothing holds a stream
        /// exclusively.
        ///
        /// A stream C hands Rust is borrowed with `FileRef::from_ptr`, whose contract
        /// reads, for a stream: the pointer is NULL or an open stream that stays open for
        /// the handle's lifetime, `fclose` included, and every other access to it while
        /// the handle lives, C's or Rust's, on any thread, holds its lock. glibc's stdio
        /// routines take that lock themselves only while the stream's locking is internal,
        /// its default. On a stream set to `FSETLOCKING_BYCALLER`, every routine skips it,
        /// `fflush(NULL)` and `exit`'s flush of every open stream included, and the
        /// `_unlocked` routines skip it on every stream: the C that uses them holds the
        /// lock itself, with `flockfile`, or the stream is not borrowed while it runs.
        #[doc(alias = "_IO_FILE")]
        #[doc(alias = "FILE")]
        File,
        FileRef,
        FileMut,
        ffi::_IO_FILE
    );

    // SAFETY: a stream is tied to no thread: glibc opens, writes and closes it from any
    // thread, its lock recording an owner only while a call holds it. No `File` value
    // exists in Rust (the field is private and this region builds none), so a `File`
    // crosses threads only behind a pointer: an ffibox owner or view over it, which
    // inherits this assertion, and which a later region may add, its stream then closed
    // or reached on whichever thread holds it; the handles assert their own below.
    unsafe impl Send for File {}
    // SAFETY: as for `Send for File`: the exclusive handle holds no lock and no state of
    // its own, only the pointer.
    unsafe impl Send for FileMut<'_> {}
    // SAFETY: nothing reachable through a shared handle touches the stream but under the
    // stream's lock: `FileRef` reads no field, and every operation a region adds on it
    // holds the lock, taken with `flockfile`, which locks whatever the stream's locking
    // mode, for the whole of its access, so that two threads' operations exclude each
    // other, and every other access, which `FileRef::from_ptr`'s contract for a stream
    // makes hold the lock too.
    unsafe impl Sync for File {}
    // SAFETY: as for `Sync for File`: copies of a shared handle reach the stream only
    // through its locked operations.
    unsafe impl Send for FileRef<'_> {}
    // SAFETY: as above.
    unsafe impl Sync for FileRef<'_> {}
    // SAFETY: `&FileMut` reaches the stream only through `as_ref`, a shared handle.
    unsafe impl Sync for FileMut<'_> {}

    #[cfg(test)]
    #[expect(unsafe_code, reason = "borrows stand-in streams through the seams")]
    mod unit_priv {
        use core::ptr;
        use std::thread;

        use ffibox::{CMutSeam, CRefSeam};

        use super::{File, FileMut, FileRef};
        use crate::libc::ffi;

        /// The fixture of these tests, which hand the stream to no C routine and read
        /// nothing of it: zeroed storage of a stream's layout, which stands in for an open
        /// stream as far as a handle, which carries only its pointer, can tell.
        fn storage() -> ffi::_IO_FILE {
            ffi::_IO_FILE::default()
        }

        fn send_sync<T: Send + Sync>() {}

        #[test]
        fn a_null_stream_is_no_handle() {
            // SAFETY: NULL is the input `from_ptr` reads as no handle.
            assert!(unsafe { FileRef::from_ptr(ptr::null_mut()) }.is_none());
            // SAFETY: as above.
            assert!(unsafe { FileMut::from_ptr(ptr::null_mut()) }.is_none());
        }

        #[test]
        fn the_handles_keep_the_stream_address() {
            let mut stream = storage();
            let p = &raw mut stream;
            // SAFETY: `stream` is live and initialised for the handle's lifetime, and
            // nothing else writes it.
            let shared = unsafe { FileRef::from_ptr(p) }.unwrap();
            assert_eq!(shared.as_ptr(), p.cast_const());
            let copy = shared;
            assert_eq!(copy.as_ptr(), shared.as_ptr());

            // SAFETY: as above, from a pointer with write provenance; `shared` and `copy`
            // are not used while the exclusive handle lives.
            let mut exclusive = unsafe { FileMut::from_ptr(p) }.unwrap();
            assert_eq!(exclusive.as_mut_ptr(), p);
            assert_eq!(exclusive.as_ref().as_ptr(), p.cast_const());
            let mut reborrow = exclusive.as_mut();
            assert_eq!(reborrow.as_mut_ptr(), p);
            assert_eq!(exclusive.as_mut_ptr(), p);
        }

        #[test]
        fn a_shared_handle_crosses_threads() {
            send_sync::<File>();
            send_sync::<FileRef<'static>>();
            send_sync::<FileMut<'static>>();

            let mut stream = storage();
            let p = &raw mut stream;
            // SAFETY: `stream` outlives the scope every thread runs in, and nothing writes
            // it.
            let shared = unsafe { FileRef::from_ptr(p) }.unwrap();
            let addr = p.addr();
            thread::scope(|s| {
                for _ in 0..8 {
                    s.spawn(move || assert_eq!(shared.as_ptr().addr(), addr));
                }
            });

            // SAFETY: as above, from a pointer with write provenance, and no other handle is
            // used while it lives.
            let exclusive = unsafe { FileMut::from_ptr(p) }.unwrap();
            let moved =
                thread::scope(|s| s.spawn(move || exclusive.as_ref().as_ptr().addr()).join());
            assert_eq!(moved.unwrap(), addr);
        }
    }
}
