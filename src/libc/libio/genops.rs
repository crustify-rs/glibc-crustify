//! `libio/genops.c`: the generic stream operations, home of `FILE` (`struct _IO_FILE`).
//!
//! [`File`] is C's stream, bound opaque and borrowed: the routines that open, write and
//! close a stream are C's, and Rust holds a stream only through the [`FileRef`] that
//! whatever keeps it open lends.

pub(crate) use ffi_import::StreamLock;
pub use ffi_import::{File, FileMut, FileRef};

#[expect(
    unsafe_code,
    reason = "asserts the thread safety of glibc's streams, and takes and releases their locks"
)]
mod ffi_import {
    use core::marker::PhantomData;
    use std::io;

    use ffibox::{CRefSeam, define_ctype};

    use crate::libc::ffi;

    define_ctype!(
        /// Wraps `st::_IO_FILE`
        ///
        /// Wraps `tdf::FILE`
        ///
        /// C's stream, `FILE`: a file, a buffer or a cookie behind glibc's stdio buffer,
        /// which C's stdio routines read, write and close. Opaque: the safe API writes none
        /// of its fields, and reads only whether it has a lock (`_lock`), which the routine
        /// that opens a stream sets and nothing changes while it is open.
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
        /// the stream without it but whether it has a lock: a stream without one, the one
        /// glibc hands a `register_printf_specifier` handler, is refused. No producer
        /// yields a [`FileMut`]: nothing holds a stream exclusively.
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
    // stream's lock: `FileRef` reads no field but `_lock`, which nothing writes while the
    // stream is open, and every operation a region adds on it holds the lock, taken with
    // `flockfile`, which locks whatever the stream's locking mode, for the whole of its
    // access, so that two threads' operations exclude each other, and every other access,
    // which `FileRef::from_ptr`'s contract for a stream makes hold the lock too.
    unsafe impl Sync for File {}
    // SAFETY: as for `Sync for File`: copies of a shared handle reach the stream only
    // through its locked operations.
    unsafe impl Send for FileRef<'_> {}
    // SAFETY: as above.
    unsafe impl Sync for FileRef<'_> {}
    // SAFETY: `&FileMut` reaches the stream only through `as_ref`, a shared handle.
    unsafe impl Sync for FileMut<'_> {}

    impl<'a> FileRef<'a> {
        /// Takes the stream's lock with `flockfile`, for an operation bound on [`FileRef`] to
        /// hold for the whole of its access to the stream, as `File`'s `Sync` requires: the
        /// lock is taken whatever the stream's locking mode, waiting while another thread
        /// holds it, and recursively when this thread does. It is released when the hold is
        /// dropped, by the thread that took it.
        ///
        /// A stream that has no lock, as [`has_lock`](Self::has_lock) tells, is refused with
        /// [`io::ErrorKind::Unsupported`], before anything of it but its `_lock` is read and
        /// without touching `errno`: `flockfile` would dereference its NULL `_lock`.
        pub(crate) fn lock(self) -> io::Result<StreamLock<'a>> {
            if !self.has_lock() {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "the stream has no lock",
                ));
            }
            // SAFETY: the stream is open for the handle's lifetime, which the hold does not
            // outlast (`FileRef::from_ptr`'s contract for a stream), and it has a lock, checked
            // just above: `flockfile` dereferences the stream's `_lock` unchecked. It takes that
            // recursive lock with no other precondition, whatever the stream's locking mode.
            // The pointer has the write provenance of the one C handed `FileRef::from_ptr`, and
            // `flockfile` writes only the lock, which that contract allows (`File`'s `Sync`).
            unsafe { ffi::flockfile(self.as_ptr().cast_mut()) };
            Ok(StreamLock {
                stream: self,
                _thread: PhantomData,
            })
        }

        /// Whether the stream has a lock for `flockfile` to take, which `flockfile` reaches
        /// without checking: every stream C opens has one, and the stream glibc hands a
        /// `register_printf_specifier` handler, over printf's own buffer, has none.
        pub(crate) fn has_lock(self) -> bool {
            let p = self.as_ptr();
            // SAFETY: `p` is an open stream for the handle's lifetime (`FileRef::from_ptr`'s
            // contract), valid and aligned for a read of its layout. The routine that opens a
            // stream sets its `_lock` before handing it out, and nothing writes it while the
            // stream is open, so this read, without the lock, races with no write: the stdio
            // routines write only the stream's other fields, under its lock.
            !unsafe { (*p)._lock }.is_null()
        }
    }

    /// A hold of a stream's lock, which [`FileRef::lock`] takes with `flockfile`, released
    /// with `funlockfile` when dropped, unwinding included. It is neither `Send` nor `Sync`:
    /// the lock records the thread that took it, which alone may release it.
    pub(crate) struct StreamLock<'a> {
        /// The locked stream.
        stream: FileRef<'a>,
        /// Ties the hold to the thread that took the lock.
        _thread: PhantomData<*const ()>,
    }

    impl StreamLock<'_> {
        /// The locked stream's pointer, for the stdio calls its holder makes under the lock:
        /// it has the write provenance of the one C handed `FileRef::from_ptr`, and a stdio
        /// routine writes the stream under its lock, which that contract allows (`File`'s
        /// `Sync`). The stream is open while the hold lives, and locked by this thread.
        pub(crate) fn as_ptr(&self) -> *mut ffi::FILE {
            self.stream.as_ptr().cast_mut()
        }
    }

    impl Drop for StreamLock<'_> {
        fn drop(&mut self) {
            // SAFETY: this thread took the lock in `FileRef::lock`, the hold being neither
            // `Send` nor `Sync`, on a stream that is still open, the hold not outliving its
            // handle; and it is released once, here, as the hold is dropped once.
            unsafe { ffi::funlockfile(self.as_ptr()) };
        }
    }

    #[cfg(test)]
    #[expect(unsafe_code, reason = "borrows and locks streams through the seams")]
    mod unit_priv {
        use core::ffi::c_char;
        use core::ptr;
        use std::panic::{self, AssertUnwindSafe};
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

        /// Whether another thread may take the stream's lock now: `ftrylockfile` from a
        /// thread of its own, released again at once.
        fn is_unlocked(file: FileRef<'_>) -> bool {
            thread::scope(|s| {
                s.spawn(move || {
                    let p = file.as_ptr().cast_mut();
                    // SAFETY: an open stream for the scope, which the handle's lifetime
                    // covers.
                    let taken = unsafe { ffi::ftrylockfile(p) } == 0;
                    if taken {
                        // SAFETY: this thread took the lock just above.
                        unsafe { ffi::funlockfile(p) };
                    }
                    taken
                })
                .join()
                .unwrap()
            })
        }

        #[test]
        #[cfg_attr(miri, ignore = "calls C")]
        fn a_hold_keeps_the_lock_until_dropped_unwinding_included() {
            let mut buf: *mut c_char = ptr::null_mut();
            let mut size = 0_usize;
            // SAFETY: both out-pointers address locals that outlive the stream, which the
            // `fclose` below closes.
            let p = unsafe { ffi::open_memstream(&raw mut buf, &raw mut size) };
            assert!(!p.is_null(), "open_memstream failed");
            // SAFETY: an open stream until the `fclose` below, after the handle's last use;
            // nothing but the holds and `is_unlocked`, which take its lock, reaches it.
            let file = unsafe { FileRef::from_ptr(p) }.unwrap();

            assert!(is_unlocked(file));
            let hold = file.lock().unwrap();
            assert_eq!(hold.as_ptr(), p);
            assert!(!is_unlocked(file));
            // Recursive: a second hold on this thread is taken at once, and the stream stays
            // locked until the first is dropped too.
            let inner = file.lock().unwrap();
            drop(inner);
            assert!(!is_unlocked(file));
            drop(hold);
            assert!(is_unlocked(file));

            let unwound = panic::catch_unwind(AssertUnwindSafe(|| {
                let _hold = file.lock().unwrap();
                panic!("a panic while the stream's lock is held");
            }));
            assert!(unwound.is_err());
            assert!(is_unlocked(file));

            // SAFETY: an open stream, closed once, with no handle used after it.
            assert_eq!(unsafe { ffi::fclose(p) }, 0);
            // SAFETY: `fclose` left the stream's buffer to the caller, to free with `free`.
            unsafe { ffi::free(buf.cast()) };
        }
    }
}
