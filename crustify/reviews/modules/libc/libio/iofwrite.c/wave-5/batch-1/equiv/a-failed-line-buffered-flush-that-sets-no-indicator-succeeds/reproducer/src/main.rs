//! `FileRef::write_all` against `fprintf(stream, "%s", text)`, the write it stands in for,
//! on a line-buffered stream whose flush fails without setting the error indicator.
//!
//! Two streams from one description: each a socket, opened `"r+"` with `fdopen` and made
//! line-buffered, from which C reads one byte of three, leaving two unread in the stream's
//! buffer. Then each takes the same two lines, one stream through the binding and the other
//! through C's `fprintf`. A socket cannot seek, so the flush of each line fails in the seek
//! that would give back the unread bytes (`ESPIPE`), before any write, and the seek's failure
//! sets no error indicator. `fwrite` returns the full count for the second line, which fits
//! the buffer, and the binding reports it as written, where `fprintf` fails.

use core::ffi::{c_char, c_int};
use std::ffi::CString;
use std::os::fd::IntoRawFd;
use std::os::unix::net::UnixStream;
use std::process::ExitCode;

use ffibox::CRefSeam;
use glibc_bindings::libc::csu::errno_loc::{errno, set_errno};
use glibc_bindings::libc::ffi;
use glibc_bindings::libc::libio::genops::FileRef;

unsafe extern "C" {
    /// `<stdio.h>`'s `fdopen`, which the raw bindings do not declare.
    fn fdopen(fd: c_int, mode: *const c_char) -> *mut ffi::FILE;
    /// `<stdio.h>`'s `fgetc`, which the raw bindings do not declare.
    fn fgetc(stream: *mut ffi::FILE) -> c_int;
}

/// A line-buffered `"r+"` stream over one end of a socket pair, from which C has read one
/// byte of the three the other end wrote. The other end is returned, to keep it open.
fn stream_after_a_read() -> (*mut ffi::FILE, UnixStream) {
    let (ours, mut peer) = UnixStream::pair().unwrap();
    std::io::Write::write_all(&mut peer, b"xyz").unwrap();
    let fd = ours.into_raw_fd();
    // SAFETY: an open descriptor this function owns, which the stream takes, and a mode.
    let file = unsafe { fdopen(fd, c"r+".as_ptr()) };
    assert!(!file.is_null(), "fdopen failed");
    let mode = c_int::try_from(ffi::_IOLBF).unwrap();
    // SAFETY: an open stream nothing has read or written yet, and a NULL buffer.
    assert_eq!(unsafe { ffi::setvbuf(file, core::ptr::null_mut(), mode, 8192) }, 0);
    // SAFETY: an open stream; C reads one byte, and buffers the two after it.
    assert_eq!(unsafe { fgetc(file) }, c_int::from(b'x'));
    (file, peer)
}

/// Whether the stream's error indicator is set.
fn has_error(file: *mut ffi::FILE) -> bool {
    // SAFETY: an open stream.
    unsafe { ffi::ferror(file) != 0 }
}

fn main() -> ExitCode {
    let (rust, _rust_peer) = stream_after_a_read();
    let (c, _c_peer) = stream_after_a_read();
    // SAFETY: `rust` stays open until the process exits, and every other access to it is a
    // stdio call that takes its lock.
    let handle = unsafe { FileRef::from_ptr(rust) }.unwrap();

    for (step, line) in ["ab\n", "cd\n"].into_iter().enumerate() {
        let got = handle.write_all(line.as_bytes());
        let text = CString::new(line).unwrap();
        set_errno(0);
        // SAFETY: an open stream, a `"%s"` format and the NUL-terminated string it reads.
        let ret = unsafe { ffi::fprintf(c, c"%s".as_ptr(), text.as_ptr()) };
        let c_errno = errno();
        println!(
            "step {step}: write_all {got:?}, error indicator {}; \
             fprintf {ret}, errno {c_errno}, error indicator {}",
            has_error(rust),
            has_error(c),
        );
        if got.is_ok() != (ret >= 0) {
            eprintln!(
                "step {step}: write_all and fprintf(stream, \"%s\", text) disagree: \
                 the binding reports {line:?} written where fprintf fails"
            );
            return ExitCode::FAILURE;
        }
    }
    println!("write_all and fprintf agree on every step");
    ExitCode::SUCCESS
}
