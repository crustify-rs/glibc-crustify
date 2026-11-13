//! `libio/iofwrite.c`: `FileRef::write_all` against `fwrite(bytes, 1, len, stream)`, and
//! against the `fprintf(stream, "%s", text)` it stands in for where C writes text.
//!
//! Each test opens two streams from one description, writes one through the binding and
//! the other through the C, with the same bytes in the same order, and compares after each
//! write: the outcome, as the binding maps C's, `errno`'s error, the stream's error
//! indicator, its orientation, its lock, which the binding leaves released, and what it
//! holds, read with an `fflush`. A test whose failure a flush between its writes would
//! change compares what the streams hold once, after the last.
//!
//! Against `fprintf`, no text is empty: `fwrite` of no bytes does nothing, where
//! `fprintf(stream, "%s", "")` orients the stream, and fails on a stream not open for
//! writing or wide-oriented. The parent's `io::Write` never hands its `write` an empty
//! buffer through `write_all`.

use core::ffi::{c_int, c_void};
use std::ffi::CString;
use std::io;

use glibc_bindings::libc::csu::errno_loc::{errno, set_errno};
use glibc_bindings::libc::ffi;

use crate::stream::Stream;

/// What a C write reported: whether it failed, and the `errno` it left after it was cleared.
struct Reported {
    failed: bool,
    errno: c_int,
}

/// A C write of bytes to a stream, and what it reported.
type CWrite = fn(&Stream, &[u8]) -> Reported;

/// C's write: `fwrite(bytes, 1, len, stream)`, failed when it returns a short count or sets
/// the stream's error indicator, clear before it, as `FileRef::write_all` documents it.
fn c_fwrite(stream: &Stream, bytes: &[u8]) -> Reported {
    let error_before = stream.has_error();
    set_errno(0);
    // SAFETY: an open stream the fixture keeps, and `bytes`, valid for `bytes.len()` reads.
    let written = unsafe {
        ffi::fwrite(
            bytes.as_ptr().cast::<c_void>(),
            1,
            bytes.len(),
            stream.as_ptr(),
        )
    };
    let errno = errno();
    Reported {
        failed: written < bytes.len() || (stream.has_error() && !error_before),
        errno,
    }
}

/// C's write of text: `fprintf(stream, "%s", text)`, failed when it returns a negative count.
fn c_fprintf(stream: &Stream, bytes: &[u8]) -> Reported {
    let text = CString::new(bytes).unwrap();
    set_errno(0);
    // SAFETY: an open stream the fixture keeps, a `"%s"` format and the NUL-terminated
    // string its conversion reads.
    let ret = unsafe { ffi::fprintf(stream.as_ptr(), c"%s".as_ptr(), text.as_ptr()) };
    Reported {
        failed: ret < 0,
        errno: errno(),
    }
}

/// Whether `compare` reads what the streams hold after each write, flushing them, or once,
/// after the last.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Flush {
    EachStep,
    AtEnd,
}

/// Writes `writes` to `rust` through the binding and to `c` through `c_write`, comparing
/// after each.
fn compare(rust: &Stream, c: &Stream, c_write: CWrite, writes: &[&[u8]]) {
    compare_flushing(rust, c, c_write, writes, Flush::EachStep);
}

/// As `compare`, reading what the streams hold as `flush` says.
fn compare_flushing(rust: &Stream, c: &Stream, c_write: CWrite, writes: &[&[u8]], flush: Flush) {
    for (step, bytes) in writes.iter().enumerate() {
        let got: io::Result<()> = rust.handle().write_all(bytes);
        let want = c_write(c, bytes);
        match got {
            Ok(()) => assert!(!want.failed, "step {step}: Rust succeeded, C failed"),
            Err(e) => {
                assert!(
                    want.failed,
                    "step {step}: Rust failed with {e}, C succeeded"
                );
                assert_eq!(
                    e.raw_os_error(),
                    (want.errno != 0).then_some(want.errno),
                    "step {step}"
                );
            }
        }
        assert_eq!(
            rust.has_error(),
            c.has_error(),
            "step {step}: error indicator"
        );
        assert_eq!(
            rust.orientation(),
            c.orientation(),
            "step {step}: orientation"
        );
        assert!(
            rust.is_unlocked(),
            "step {step}: the binding left the stream locked"
        );
        if flush == Flush::EachStep {
            assert_eq!(rust.contents(), c.contents(), "step {step}: contents");
        }
    }
    if flush == Flush::AtEnd {
        assert_eq!(rust.contents(), c.contents(), "contents");
    }
}

/// Texts the C's status messages are made of, `fprintf`'s `"%s"` among them: no NUL, and
/// none empty.
fn texts(long: &[u8]) -> Vec<&[u8]> {
    vec![
        b"unknown option: ",
        b"--caf\xe9",
        b" 100% %s %n %%\n",
        "é→✓".as_bytes(),
        long,
        b"\n",
    ]
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_growing_stream_takes_the_same_bytes_as_fwrite() {
    let long: Vec<u8> = (0..=255_u8).cycle().take(16 * 1024).collect();
    let writes: [&[u8]; 6] = [b"", b"a\0b", b"--caf\xe9", &long, b"\0", b"\n"];
    compare(&Stream::memory(), &Stream::memory(), c_fwrite, &writes);
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_growing_stream_takes_the_same_bytes_as_fprintf() {
    let long = b"0123456789abcdef".repeat(1024);
    compare(
        &Stream::memory(),
        &Stream::memory(),
        c_fprintf,
        &texts(&long),
    );
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_stream_locked_by_its_caller_takes_the_same_bytes() {
    let long = vec![b'x'; 20_000];
    let writes: [&[u8]; 4] = [b"argument '", b"--output", b"' requires a value.\n", &long];
    for c_write in [c_fwrite, c_fprintf] {
        let (rust, c) = (Stream::memory(), Stream::memory());
        rust.lock_by_caller();
        c.lock_by_caller();
        compare(&rust, &c, c_write, &writes);
    }
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_stream_not_open_for_writing_fails_alike() {
    let writes: [&[u8]; 3] = [b"text", b"", b"more"];
    compare(
        &Stream::fixed(16, c"r"),
        &Stream::fixed(16, c"r"),
        c_fwrite,
        &writes,
    );
    compare(
        &Stream::fixed(16, c"r"),
        &Stream::fixed(16, c"r"),
        c_fprintf,
        &[b"text", b"more"],
    );
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_full_stream_fails_alike() {
    let long = vec![b'y'; 64 * 1024];
    let writes: [&[u8]; 4] = [b"fits", &long, b"after", b""];
    compare(
        &Stream::fixed(16, c"w"),
        &Stream::fixed(16, c"w"),
        c_fwrite,
        &writes,
    );
    compare(
        &Stream::fixed(16, c"w"),
        &Stream::fixed(16, c"w"),
        c_fprintf,
        &writes[..3],
    );
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_failing_device_fails_alike() {
    let long = vec![b'z'; 64 * 1024];
    let writes: [&[u8]; 3] = [b"fits", &long, b"after"];
    for c_write in [c_fwrite, c_fprintf] {
        compare(&Stream::full(), &Stream::full(), c_write, &writes);
    }
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_wide_stream_fails_alike() {
    let all: [&[u8]; 3] = [b"text", b"", b"more"];
    let texts: [&[u8]; 2] = [b"text", b"more"];
    let runs: [(CWrite, &[&[u8]]); 2] = [(c_fwrite, &all), (c_fprintf, &texts)];
    for (c_write, writes) in runs {
        let (rust, c) = (Stream::file(), Stream::file());
        rust.orient_wide();
        c.orient_wide();
        compare(&rust, &c, c_write, writes);
    }
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_line_buffered_stream_takes_the_same_lines() {
    let long = b"0123456789abcdef".repeat(1024);
    for c_write in [c_fwrite, c_fprintf] {
        compare(
            &Stream::file().line_buffered(),
            &Stream::file().line_buffered(),
            c_write,
            &texts(&long),
        );
    }
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_failed_line_buffered_flush_fails_as_fprintf_does() {
    // The flush of a line that fits the buffer fails: `fwrite` returns the full count, and
    // `fprintf` fails, as the binding does. Once the indicator is set, the binding cannot tell
    // that failure, which `fprintf` reports: the comparison stops before.
    let raw = Stream::full().line_buffered();
    for (bytes, failed) in [(&b"status: "[..], false), (b"failed\n", true)] {
        // SAFETY: an open stream the fixture keeps, and `bytes`, valid for its length.
        let written = unsafe {
            ffi::fwrite(
                bytes.as_ptr().cast::<c_void>(),
                1,
                bytes.len(),
                raw.as_ptr(),
            )
        };
        assert_eq!(written, bytes.len(), "fwrite's count");
        assert_eq!(raw.has_error(), failed, "fwrite's error indicator");
    }
    let writes: [&[u8]; 2] = [b"status: ", b"failed\n"];
    compare_flushing(
        &Stream::full().line_buffered(),
        &Stream::full().line_buffered(),
        c_fprintf,
        &writes,
        Flush::AtEnd,
    );
    let writes: [&[u8]; 2] = [b"abcdef", b"\n"];
    compare_flushing(
        &Stream::fixed(4, c"w").line_buffered(),
        &Stream::fixed(4, c"w").line_buffered(),
        c_fprintf,
        &writes,
        Flush::AtEnd,
    );
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_failed_line_buffered_flush_fails_alike_after_an_earlier_failure() {
    // Against `fwrite`, whose failures the binding reads the same way, the comparison goes
    // on past the first failure: the hidden one included, and after `clearerr`.
    let writes: [&[u8]; 4] = [b"x\n", b"status: ", b"failed\n", b"more"];
    let (rust, c) = (
        Stream::full().line_buffered(),
        Stream::full().line_buffered(),
    );
    compare_flushing(&rust, &c, c_fwrite, &writes, Flush::AtEnd);
    rust.clear_error();
    c.clear_error();
    compare_flushing(&rust, &c, c_fwrite, &writes[1..], Flush::AtEnd);
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn an_unbuffered_stream_takes_and_fails_alike() {
    // Unbuffered, as glibc opens `stderr`, which the parent's C writes: each write goes to the
    // file during the call, and a failure fails the call that meets it.
    let long = b"0123456789abcdef".repeat(1024);
    for c_write in [c_fwrite, c_fprintf] {
        compare(
            &Stream::file().unbuffered(),
            &Stream::file().unbuffered(),
            c_write,
            &texts(&long),
        );
        compare_flushing(
            &Stream::full().unbuffered(),
            &Stream::full().unbuffered(),
            c_write,
            &[b"x", b"status: ", b"failed\n", &long],
            Flush::AtEnd,
        );
    }
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_flush_that_fails_in_its_seek_is_reported_as_fwrite_reports_it() {
    // A line-buffered stream C read from and then writes: each flush seeks back over the two
    // unread bytes, which a socket cannot, and fails with `ESPIPE`, setting no indicator. The
    // first line fails short; the second fits the buffer, and `fwrite` returns its full count.
    let writes: [&[u8]; 3] = [b"ab\n", b"cd\n", b"ef"];
    compare_flushing(
        &Stream::line_buffered_socket_after_a_read(),
        &Stream::line_buffered_socket_after_a_read(),
        c_fwrite,
        &writes,
        Flush::AtEnd,
    );
    // The departure `FileRef::write_all` documents: `fprintf` fails the second line, which
    // the binding, seeing no indicator set, reports written.
    let (rust, c) = (
        Stream::line_buffered_socket_after_a_read(),
        Stream::line_buffered_socket_after_a_read(),
    );
    for (bytes, rust_ok) in [(&b"ab\n"[..], false), (b"cd\n", true)] {
        assert_eq!(rust.handle().write_all(bytes).is_ok(), rust_ok);
        let want = c_fprintf(&c, bytes);
        assert!(want.failed, "fprintf of {bytes:?}");
        assert_eq!(want.errno, ffi::ESPIPE);
        assert!(!rust.has_error() && !c.has_error(), "no indicator is set");
    }
    assert_eq!(rust.contents(), c.contents());
}
