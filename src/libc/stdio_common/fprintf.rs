//! `stdio-common/fprintf.c`: `fprintf`, the formatted write to a C stream.
//!
//! Rust formats, and C writes: [`FileRef::print`] hands text Rust has already formatted
//! to `fprintf(stream, "%s", text)`, so that it lands in the stream's own buffer, in order
//! with what the C that shares the stream writes, and [`FileRef::write_fmt`] lets `write!`
//! format into it. printf's conversions are not reached: the text is never a format, so
//! a `%` in it is written as it is.
//!
//! [`FileRef::print`]: crate::libc::libio::genops::FileRef::print
//! [`FileRef::write_fmt`]: crate::libc::libio::genops::FileRef::write_fmt

#[expect(
    unsafe_code,
    reason = "calls flockfile, fprintf and funlockfile on a borrowed stream"
)]
mod ffi_import {
    use core::ffi::{CStr, c_int};
    use core::fmt;
    use std::ffi::CString;
    use std::io;

    use ffibox::CRefSeam;

    use crate::libc::csu::errno_loc::{errno, set_errno};
    use crate::libc::ffi;
    use crate::libc::libio::genops::FileRef;

    impl FileRef<'_> {
        /// Wraps `fn::fprintf`
        ///
        /// Writes `text` to the stream, as `fprintf(stream, "%s", text)` does, and returns the
        /// number of bytes written, `text.len()`.
        ///
        /// The write holds the stream's lock from start to end, taken with `flockfile`, whatever
        /// the stream's locking mode. A stream C set to `FSETLOCKING_BYCALLER` is locked too,
        /// although glibc's own `fprintf` would skip the lock on it. So one call's text is
        /// never interleaved with another thread's write to the stream, and the call waits
        /// while another thread holds the lock. The calling thread may already hold it, as
        /// the lock is recursive. A stream that has no lock is refused, as below.
        ///
        /// The text lands in the stream's buffer, and reaches the file when the buffering mode
        /// the C gave the stream says, or at `fflush` or `fclose`.
        ///
        /// # Errors
        ///
        /// - [`io::ErrorKind::InvalidInput`], when `text` holds a NUL byte, which a C string cannot
        ///   carry. Nothing is written, and `errno` is left as it was.
        /// - [`io::ErrorKind::Unsupported`], when the stream has no lock for `flockfile` to take:
        ///   the stream glibc hands the handler of a conversion that `register_printf_specifier`
        ///   adds, over printf's own buffer, whose lock is NULL. Nothing is written, and `errno` is
        ///   left as it was.
        /// - `errno`'s error, when `fprintf` fails and sets it: `EBADF` for a stream not open for
        ///   writing, which sets its error indicator and has nothing written to it; the error of
        ///   the write that flushes the buffer, such as `ENOSPC`, `EPIPE` or `EIO`, after part of
        ///   `text` may have reached the stream; and `EOVERFLOW` for a `text` longer than `INT_MAX`
        ///   bytes, whose count `fprintf` cannot return, though the text is written. `errno` keeps
        ///   that error, as C's does.
        /// - [`io::ErrorKind::Other`], with no OS error, when `fprintf` fails without setting
        ///   `errno`. A stream that is wide-oriented (by `fwide` or a wide-character write) does
        ///   that, and nothing is written to it. A cookie stream whose write function fails without
        ///   setting `errno` does it too. `errno` is left as it was.
        ///
        /// On success `errno` is left as it was too.
        #[doc(alias = "fprintf")]
        pub fn print(&self, text: &str) -> io::Result<usize> {
            let text =
                CString::new(text).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            if !self.has_lock() {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "the stream has no lock",
                ));
            }
            let saved = errno();
            // `fprintf` reports some failures without setting `errno`: cleared, so that what it
            // holds after the call is what the call set.
            set_errno(0);
            let (ret, err) = print_locked(*self, &text);
            let result = outcome(ret, err);
            if result.is_ok() || err == 0 {
                set_errno(saved);
            }
            result
        }

        /// Writes `args`, formatted by Rust, to the stream in one [`print`](Self::print). It is
        /// the method `write!` calls, so that `write!(stream, "{name}: ")` writes as
        /// `fprintf(stream, "%s: ", name)` does, with the same count, errors and lock. Unlike
        /// [`io::Write::write_fmt`], each `write!` is one write under one hold of the lock, and
        /// returns its count.
        ///
        /// # Errors
        ///
        /// Those of [`print`](Self::print), for the formatted text, and
        /// [`io::ErrorKind::Other`] when a `Display` or other formatting impl of an argument
        /// fails, as [`io::Write::write_fmt`] reports it, with nothing written.
        pub fn write_fmt(&self, args: fmt::Arguments<'_>) -> io::Result<usize> {
            if let Some(text) = args.as_str() {
                return self.print(text);
            }
            let mut text = String::new();
            fmt::Write::write_fmt(&mut text, args)
                .map_err(|fmt::Error| io::Error::other("formatter error"))?;
            self.print(&text)
        }

        /// Whether the stream has a lock for `flockfile` to take, which `flockfile` reaches
        /// without checking: every stream C opens has one, and the stream glibc hands a
        /// `register_printf_specifier` handler, over printf's own buffer, has none.
        fn has_lock(&self) -> bool {
            let p = self.as_ptr();
            // SAFETY: `p` is an open stream for the handle's lifetime (`FileRef::from_ptr`'s
            // contract), valid and aligned for a read of its layout. The routine that opens a
            // stream sets its `_lock` before handing it out, and nothing writes it while the
            // stream is open, so this read, without the lock, races with no write: the stdio
            // routines write only the stream's other fields, under its lock.
            !unsafe { (*p)._lock }.is_null()
        }
    }

    /// `fprintf`'s outcome: `ret`, its return, as a count, or, when negative, the failure that
    /// `err`, the `errno` it left after it was cleared, reports. An `err` of `0` is a failure
    /// that set no `errno`.
    fn outcome(ret: c_int, err: c_int) -> io::Result<usize> {
        match usize::try_from(ret) {
            Ok(count) => Ok(count),
            Err(_) if err == 0 => Err(io::Error::other("fprintf failed without setting errno")),
            Err(_) => Err(io::Error::from_raw_os_error(err)),
        }
    }

    /// Writes `text` to `stream` with `fprintf(stream, "%s", text)`, holding the stream's
    /// lock, taken with `flockfile`, from before the call to after `errno` is read. Returns
    /// what `fprintf` returned, with the `errno` it left. `stream` has a lock, as
    /// `FileRef::has_lock` checks.
    fn print_locked(stream: FileRef<'_>, text: &CStr) -> (c_int, c_int) {
        // The stream's pointer, with the write provenance of the one C handed
        // `FileRef::from_ptr`: `fprintf`, `flockfile` and `funlockfile` write the stream
        // under its lock, which `FileRef`'s contract allows (`File`'s `Sync`).
        let p = stream.as_ptr().cast_mut();
        // SAFETY: `p` is an open stream for the handle's lifetime, which the call does not
        // outlast (`FileRef::from_ptr`'s contract for a stream), and it has a lock, which
        // `print` checked first (`has_lock`): `flockfile` dereferences the stream's `_lock`
        // unchecked. It takes that recursive lock with no other precondition, waiting while
        // another thread holds it, whatever the stream's locking mode.
        unsafe { ffi::flockfile(p) };
        // SAFETY: `p` is an open stream, locked by this thread, as above. The format is a
        // NUL-terminated `"%s"` literal, whose one conversion reads one `char *`: `text`, a
        // NUL-terminated string that outlives the call and that `fprintf` only reads. The
        // stream's lock serializes this write with every other access to it, on any thread,
        // which `FileRef::from_ptr`'s contract makes hold the lock too.
        let ret = unsafe { ffi::fprintf(p, c"%s".as_ptr(), text.as_ptr()) };
        // Read before anything else runs that may change it.
        let err = errno();
        // SAFETY: this thread took the lock with the `flockfile` above, and nothing between
        // that and this point can unwind (`fprintf` is C, and `errno` reads a C `int`), so
        // the lock is released on every path, once, by the thread that holds it.
        unsafe { ffi::funlockfile(p) };
        (ret, err)
    }

    #[cfg(test)]
    mod unit_priv {
        use std::io;

        use super::outcome;
        use crate::libc::ffi;

        #[test]
        fn a_count_is_the_success() {
            assert_eq!(outcome(0, 0).unwrap(), 0);
            assert_eq!(outcome(12, ffi::EINTR).unwrap(), 12);
            assert_eq!(outcome(i32::MAX, 0).unwrap(), 2_147_483_647);
        }

        #[test]
        fn a_negative_return_reports_errno() {
            let e = outcome(-1, ffi::EACCES).unwrap_err();
            assert_eq!(e.raw_os_error(), Some(ffi::EACCES));
            let e = outcome(i32::MIN, ffi::ENOENT).unwrap_err();
            assert_eq!(e.raw_os_error(), Some(ffi::ENOENT));
        }

        #[test]
        fn a_failure_without_errno_is_no_os_error() {
            let e = outcome(-1, 0).unwrap_err();
            assert_eq!(e.kind(), io::ErrorKind::Other);
            assert_eq!(e.raw_os_error(), None);
            assert_eq!(e.to_string(), "fprintf failed without setting errno");
        }
    }
}
