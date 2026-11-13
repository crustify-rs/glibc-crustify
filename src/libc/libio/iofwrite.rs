//! `libio/iofwrite.c`: `fwrite`, the byte-exact write to a C stream.
//!
//! [`FileRef::write_all`] hands bytes, any bytes, NUL and invalid UTF-8 included, to
//! `fwrite(bytes, 1, len, stream)`, so that they land in the stream's own buffer, in order
//! with what the C that shares the stream writes. It is the write an [`io::Write`] over a C
//! stream makes, one call per `write`, though it implements no such trait itself: a `flush`
//! would need `fflush`, which is not bound.
//!
//! [`FileRef::write_all`]: crate::libc::libio::genops::FileRef::write_all
//! [`io::Write`]: std::io::Write

#[expect(
    unsafe_code,
    reason = "calls ferror and fwrite on a borrowed stream, under its lock"
)]
mod ffi_import {
    use core::ffi::{c_int, c_void};
    use std::io;

    use crate::libc::csu::errno_loc::{errno, set_errno};
    use crate::libc::ffi;
    use crate::libc::libio::genops::{FileRef, StreamLock};

    impl FileRef<'_> {
        /// Wraps `fn::fwrite`
        ///
        /// Writes every byte of `bytes` to the stream, as `fwrite(bytes, 1, bytes.len(), stream)`
        /// does, or fails. The bytes are written as they are: a NUL byte, or bytes that are not
        /// UTF-8, are written like any other.
        ///
        /// The write holds the stream's lock from start to end, taken with `flockfile`, whatever
        /// the stream's locking mode. A stream C set to `FSETLOCKING_BYCALLER` is locked too,
        /// although glibc's own `fwrite` would skip the lock on it. So one call's bytes are never
        /// interleaved with another thread's write to the stream, and the call waits while
        /// another thread holds the lock. The calling thread may already hold it, as the lock is
        /// recursive. A stream that has no lock is refused, as below.
        ///
        /// The bytes land in the stream's buffer, and reach the file when the buffering mode the
        /// C gave the stream says, or at `fflush` or `fclose`. A stream that has no orientation
        /// yet is oriented for bytes, as `fwrite` orients it.
        ///
        /// An empty `bytes` succeeds without touching the stream but for its lock, as `fwrite`
        /// of no bytes returns at once: it orients nothing, and does not fail on a stream that
        /// is wide-oriented or not open for writing, where `print("")`, which is
        /// `fprintf(stream, "%s", "")`, orients the stream or fails.
        ///
        /// # Failure
        ///
        /// `fwrite` reports a failure by returning fewer bytes than it was given, with one
        /// exception: when the bytes fit the buffer of a line-buffered stream and end with a
        /// newline, and the flush that newline calls for fails, it returns the full count.
        /// glibc's `fprintf` reports the same failure as one. A failed write sets the stream's
        /// error indicator, so the call fails when `fwrite` returns a short count, and when it
        /// returns the full count but the stream's error indicator, clear before the call, is
        /// set after it: read with `ferror`, under the same hold of the lock. Two such failures
        /// cannot be told from a success, which is what the call reports, as `fwrite` does,
        /// where `fprintf` fails:
        ///
        /// - one on a stream whose error indicator is already set, by an earlier failure nothing
        ///   has cleared (`clearerr`); and
        /// - one that sets no indicator: a flush that fails in the seek glibc makes before it
        ///   writes, back over bytes read into the buffer and not yet consumed, on a stream C read
        ///   and then wrote without a positioning call, which ISO C leaves undefined, over a file
        ///   that cannot seek, such as a pipe or a socket (`ESPIPE`). The bytes stay in the buffer,
        ///   and `errno` is left as it was.
        ///
        /// A failure is not retried, and the count `fwrite` returned is not reported: part of
        /// `bytes` may have reached the stream's buffer, its storage or its file, and a failed
        /// flush discards what the buffer held.
        ///
        /// # Errors
        ///
        /// - [`io::ErrorKind::Unsupported`], when the stream has no lock for `flockfile` to take:
        ///   the stream glibc hands the handler of a conversion that `register_printf_specifier`
        ///   adds, over printf's own buffer, whose lock is NULL. Nothing is written, and `errno` is
        ///   left as it was.
        /// - `errno`'s error, when `fwrite` fails and sets it: `EBADF` for a stream not open for
        ///   writing, which sets its error indicator and has nothing written to it, and the error
        ///   of the write that flushes the buffer, such as `ENOSPC`, `EPIPE` or `EIO`. `errno`
        ///   keeps that error, as C's does.
        /// - [`io::ErrorKind::Other`], with no OS error, when `fwrite` fails without setting
        ///   `errno`. A stream that is wide-oriented (by `fwide` or a wide-character write) does
        ///   that, and nothing is written to it or to its error indicator. A cookie stream whose
        ///   write function fails without setting `errno` does it too. `errno` is left as it was.
        ///
        /// On success `errno` is left as it was too.
        #[doc(alias = "fwrite")]
        pub fn write_all(&self, bytes: &[u8]) -> io::Result<()> {
            let lock = self.lock()?;
            let saved = errno();
            // `fwrite` reports some failures without setting `errno`: cleared, so that what it
            // holds after the call is what the call set.
            set_errno(0);
            let call = write_locked(&lock, bytes);
            drop(lock);
            let result = outcome(bytes.len(), call);
            if result.is_ok() || call.errno == 0 {
                set_errno(saved);
            }
            result
        }
    }

    /// What one `fwrite` call reported: its return, the `errno` it left after it was cleared,
    /// and the stream's error indicator before and after it.
    #[derive(Clone, Copy)]
    struct Call {
        /// The count `fwrite` returned.
        written: usize,
        /// `errno` after the call, `0` when the call set none.
        errno: c_int,
        /// Whether the stream's error indicator was set before the call.
        error_before: bool,
        /// Whether it was set after the call.
        error_after: bool,
    }

    /// The outcome of a `fwrite` of `len` bytes that reported `call`: a failure when it
    /// returned a short count, or when it set the stream's error indicator, which was clear
    /// before it; the error `errno` reports, or one with no OS error when it reports none.
    fn outcome(len: usize, call: Call) -> io::Result<()> {
        let failed = call.written < len || (call.error_after && !call.error_before);
        if !failed {
            Ok(())
        } else if call.errno == 0 {
            Err(io::Error::other("fwrite failed without setting errno"))
        } else {
            Err(io::Error::from_raw_os_error(call.errno))
        }
    }

    /// Writes `bytes` to the stream `lock` holds with `fwrite(bytes, 1, len, stream)`, reading
    /// the stream's error indicator before and after the call and `errno` right after it, all
    /// under the one hold of the lock.
    fn write_locked(lock: &StreamLock<'_>, bytes: &[u8]) -> Call {
        let p = lock.as_ptr();
        // SAFETY: the stream is open, and locked by this thread, while `lock` lives
        // (`FileRef::lock`). `ferror` only reads its error flag, taking the recursive lock this
        // thread holds.
        let error_before = unsafe { ffi::ferror(p) } != 0;
        // SAFETY: the stream is open and locked by this thread, as above, and its pointer may be
        // written under that lock. `bytes` is valid for reads of `bytes.len()` bytes, elements
        // of size 1, so that the request, their product, cannot overflow; `fwrite` copies from
        // it during the call only, and keeps no pointer to it. An empty `bytes` is a dangling
        // pointer that `fwrite` returns before reading. The stream's lock serializes this write
        // with every other access to it, which `FileRef::from_ptr`'s contract makes hold the
        // lock too.
        let written = unsafe { ffi::fwrite(bytes.as_ptr().cast::<c_void>(), 1, bytes.len(), p) };
        // Read before anything else runs that may change it.
        let errno = errno();
        // SAFETY: as for the first `ferror`.
        let error_after = unsafe { ffi::ferror(p) } != 0;
        Call {
            written,
            errno,
            error_before,
            error_after,
        }
    }

    #[cfg(test)]
    mod unit_priv {
        use std::io;

        use super::{Call, outcome};
        use crate::libc::ffi;

        fn call(written: usize, errno: i32, error_before: bool, error_after: bool) -> Call {
            Call {
                written,
                errno,
                error_before,
                error_after,
            }
        }

        #[test]
        fn a_full_count_with_no_new_error_is_the_success() {
            outcome(0, call(0, 0, false, false)).unwrap();
            outcome(5, call(5, 0, false, false)).unwrap();
            // An `errno` a successful call left is no failure.
            outcome(5, call(5, ffi::EINTR, false, false)).unwrap();
            // An indicator set before the call, and still set, tells nothing of it.
            outcome(5, call(5, 0, true, true)).unwrap();
            outcome(5, call(5, ffi::ENOSPC, true, true)).unwrap();
            outcome(usize::MAX, call(usize::MAX, 0, false, false)).unwrap();
        }

        #[test]
        fn a_short_count_is_a_failure() {
            let e = outcome(5, call(2, ffi::ENOSPC, false, true)).unwrap_err();
            assert_eq!(e.raw_os_error(), Some(ffi::ENOSPC));
            let e = outcome(5, call(0, ffi::EBADF, true, true)).unwrap_err();
            assert_eq!(e.raw_os_error(), Some(ffi::EBADF));
            // A wide stream's: no `errno`, and no indicator.
            let e = outcome(5, call(0, 0, false, false)).unwrap_err();
            assert_eq!(e.kind(), io::ErrorKind::Other);
            assert_eq!(e.raw_os_error(), None);
        }

        #[test]
        fn a_full_count_that_sets_the_indicator_is_a_failure() {
            let e = outcome(5, call(5, ffi::ENOSPC, false, true)).unwrap_err();
            assert_eq!(e.raw_os_error(), Some(ffi::ENOSPC));
            let e = outcome(5, call(5, 0, false, true)).unwrap_err();
            assert_eq!(e.kind(), io::ErrorKind::Other);
            assert_eq!(e.raw_os_error(), None);
            assert_eq!(e.to_string(), "fwrite failed without setting errno");
        }
    }
}
