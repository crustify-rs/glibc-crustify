//! `libio/iofwrite.c`: `FileRef::write_all` on a stream.

use std::sync::Barrier;
use std::time::Duration;
use std::{io, thread};

use glibc_bindings::libc::csu::errno_loc::{errno, set_errno};
use glibc_bindings::libc::ffi;

use crate::stream::Stream;
use crate::stream_unit::in_printf_handler;

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn write_all_writes_the_bytes_exactly() {
    let stream = Stream::memory();
    let file = stream.handle();
    file.write_all(b"unknown option: ").unwrap();
    // Not UTF-8: written as it is, where `print` could not take it.
    file.write_all(b"--caf\xe9").unwrap();
    file.write_all(b"").unwrap();
    // A NUL byte is written like any other.
    file.write_all(b" a\0b").unwrap();
    // Not a format: a conversion in the bytes is written as it is.
    file.write_all(b" 100% %s %n\n").unwrap();
    assert_eq!(
        stream.contents(),
        b"unknown option: --caf\xe9 a\0b 100% %s %n\n"
    );
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn every_byte_value_is_written_through_a_buffer_flush() {
    // Longer than the stream's buffer, so that the write flushes it mid-call.
    let bytes: Vec<u8> = (0..=255_u8).cycle().take(64 * 1024 + 7).collect();
    let stream = Stream::file();
    stream.handle().write_all(&bytes).unwrap();
    assert_eq!(stream.contents(), bytes);
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn writes_land_in_order_with_print() {
    let stream = Stream::memory();
    let file = stream.handle();
    file.print("error: ").unwrap();
    file.write_all(b"\xff\xfe").unwrap();
    writeln!(file, " {}", 7).unwrap();
    file.write_all(b"done").unwrap();
    assert_eq!(stream.contents(), b"error: \xff\xfe 7\ndone");
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_success_leaves_errno_as_it_was() {
    let stream = Stream::memory();
    set_errno(ffi::ENOENT);
    stream.handle().write_all(b"bytes").unwrap();
    assert_eq!(errno(), ffi::ENOENT);
    set_errno(0);
    stream.handle().write_all(b"more").unwrap();
    assert_eq!(errno(), 0);
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_stream_not_open_for_writing_fails_with_ebadf() {
    let stream = Stream::fixed(16, c"r");
    set_errno(0);
    let e = stream.handle().write_all(b"bytes").unwrap_err();
    assert_eq!(e.raw_os_error(), Some(ffi::EBADF));
    // `errno` keeps the error, as C's does.
    assert_eq!(errno(), ffi::EBADF);
    assert!(stream.has_error());
    assert!(stream.is_unlocked());
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_failed_flush_reports_the_write_error() {
    // Bytes longer than the stream's buffer flush it, into a device that takes nothing.
    let stream = Stream::full();
    set_errno(0);
    let e = stream
        .handle()
        .write_all(&vec![b'x'; 64 * 1024])
        .unwrap_err();
    assert_eq!(e.raw_os_error(), Some(ffi::ENOSPC));
    assert_eq!(errno(), ffi::ENOSPC);
    assert!(stream.has_error());
    assert!(stream.is_unlocked());
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_write_that_sets_no_errno_fails_without_it() {
    // 16 bytes of `fmemopen` storage: its write function takes what fits and then fails
    // without setting `errno`, so the flush of longer bytes fails without it too.
    let stream = Stream::fixed(16, c"w");
    set_errno(ffi::EINTR);
    let e = stream
        .handle()
        .write_all(&vec![b'x'; 64 * 1024])
        .unwrap_err();
    assert_eq!(e.kind(), io::ErrorKind::Other);
    assert_eq!(e.raw_os_error(), None);
    assert_eq!(e.to_string(), "fwrite failed without setting errno");
    assert_eq!(errno(), ffi::EINTR);
    assert!(stream.has_error());
    assert!(stream.is_unlocked());
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_wide_stream_fails_without_errno_or_the_indicator() {
    let stream = Stream::file();
    stream.orient_wide();
    set_errno(ffi::EINTR);
    let e = stream.handle().write_all(b"bytes").unwrap_err();
    assert_eq!(e.kind(), io::ErrorKind::Other);
    assert_eq!(e.raw_os_error(), None);
    assert_eq!(errno(), ffi::EINTR);
    assert!(!stream.has_error());
    assert!(stream.orientation() > 0);
    assert_eq!(stream.contents(), b"");
    assert!(stream.is_unlocked());
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_stream_is_oriented_for_bytes_by_its_first_write() {
    let stream = Stream::file();
    assert_eq!(stream.orientation(), 0);
    // Nothing written, nothing oriented, as `fwrite` of no bytes does.
    stream.handle().write_all(b"").unwrap();
    assert_eq!(stream.orientation(), 0);
    stream.handle().write_all(b"x").unwrap();
    assert!(stream.orientation() < 0);
    assert_eq!(stream.contents(), b"x");
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_failed_line_buffered_flush_is_a_failure_despite_the_full_count() {
    // The second write fits the buffer and ends with a newline, whose flush fails: `fwrite`
    // returns its full count, and only the error indicator tells the failure.
    let stream = Stream::full().line_buffered();
    let file = stream.handle();
    file.write_all(b"status: ").unwrap();
    assert!(!stream.has_error());
    set_errno(0);
    let e = file.write_all(b"failed\n").unwrap_err();
    assert_eq!(e.raw_os_error(), Some(ffi::ENOSPC));
    assert_eq!(errno(), ffi::ENOSPC);
    assert!(stream.has_error());
    assert!(stream.is_unlocked());
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_failed_line_buffered_flush_without_errno_is_a_failure() {
    // 4 bytes of `fmemopen` storage take 4 of the line's 7, and its write function sets no
    // `errno`.
    let stream = Stream::fixed(4, c"w").line_buffered();
    let file = stream.handle();
    file.write_all(b"abcdef").unwrap();
    set_errno(ffi::EINTR);
    let e = file.write_all(b"\n").unwrap_err();
    assert_eq!(e.kind(), io::ErrorKind::Other);
    assert_eq!(e.raw_os_error(), None);
    assert_eq!(errno(), ffi::EINTR);
    assert!(stream.has_error());
    // The storage took 4 bytes, the last of which `fmemopen` gives to its NUL.
    assert_eq!(stream.contents(), b"abc");
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_stream_already_in_error_cannot_tell_a_failed_line_buffered_flush() {
    // The documented limit: an error indicator set before the call hides the one failure
    // `fwrite` returns the full count for.
    let stream = Stream::full().line_buffered();
    let file = stream.handle();
    // A first write that ends with a newline flushes byte by byte and fails short.
    let e = file.write_all(b"x\n").unwrap_err();
    assert_eq!(e.raw_os_error(), Some(ffi::ENOSPC));
    assert!(stream.has_error());
    file.write_all(b"status: ").unwrap();
    file.write_all(b"failed\n").unwrap();
    // Cleared, the indicator tells the next one again.
    stream.clear_error();
    file.write_all(b"status: ").unwrap();
    let e = file.write_all(b"failed\n").unwrap_err();
    assert_eq!(e.raw_os_error(), Some(ffi::ENOSPC));
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_line_buffered_stream_takes_its_lines() {
    let stream = Stream::file().line_buffered();
    let file = stream.handle();
    file.write_all(b"one\ntw").unwrap();
    file.write_all(b"o\n").unwrap();
    file.write_all(b"three").unwrap();
    assert!(!stream.has_error());
    assert_eq!(stream.contents(), b"one\ntwo\nthree");
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn the_lock_is_released_after_every_outcome() {
    let stream = Stream::memory();
    stream.handle().write_all(b"ok").unwrap();
    assert!(stream.is_unlocked());
    stream.handle().write_all(b"").unwrap();
    assert!(stream.is_unlocked());
    let read_only = Stream::fixed(16, c"r");
    read_only.handle().write_all(b"no").unwrap_err();
    assert!(read_only.is_unlocked());
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn write_all_waits_for_the_lock_on_a_stream_locked_by_its_caller() {
    // Under `FSETLOCKING_BYCALLER`, glibc's own `fwrite` would not wait: the binding's
    // `flockfile` does.
    let stream = Stream::memory();
    stream.lock_by_caller();
    let file = stream.handle();
    let hold = stream.hold_lock();
    let started = Barrier::new(2);
    thread::scope(|s| {
        let waiter = s.spawn(|| {
            started.wait();
            file.write_all(b"second").unwrap();
        });
        started.wait();
        thread::sleep(Duration::from_millis(100));
        // The lock is recursive: this thread, which holds it, writes through it.
        file.write_all(b"first ").unwrap();
        drop(hold);
        waiter.join().unwrap();
    });
    assert_eq!(stream.contents(), b"first second");
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn writes_from_threads_never_interleave() {
    // Every thread shares one handle to a file stream whose locking is the caller's, so that
    // only the binding's lock orders the writes, each long enough to flush the buffer
    // mid-line. A file stream, not an `open_memstream` one, under TSan (`crustify/LESSONS.md`).
    const THREADS: usize = 8;
    const LINES: usize = 200;
    let stream = Stream::file();
    stream.lock_by_caller();
    let file = stream.handle();
    let pad = vec![0xA5_u8; 3000];
    thread::scope(|s| {
        for t in 0..THREADS {
            let pad = &pad;
            s.spawn(move || {
                for i in 0..LINES {
                    let mut line = format!("{t}:{i}:").into_bytes();
                    line.extend_from_slice(pad);
                    line.push(b'\n');
                    file.write_all(&line).unwrap();
                }
            });
        }
    });
    let contents = stream.contents();
    let mut seen = vec![vec![false; LINES]; THREADS];
    let mut lines = contents.split(|&b| b == b'\n');
    assert_eq!(lines.next_back(), Some(&b""[..]));
    for line in lines {
        let mut parts = line.splitn(3, |&b| b == b':');
        let t: usize = str::from_utf8(parts.next().unwrap())
            .unwrap()
            .parse()
            .unwrap();
        let i: usize = str::from_utf8(parts.next().unwrap())
            .unwrap()
            .parse()
            .unwrap();
        assert_eq!(parts.next().unwrap(), &pad[..], "line {t}:{i} interleaved");
        assert!(!seen[t][i], "line {t}:{i} written twice");
        seen[t][i] = true;
    }
    assert!(seen.iter().flatten().all(|&s| s), "a line is missing");
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_stream_without_a_lock_is_refused() {
    // The stream glibc hands a printf handler has no lock, which `flockfile` would
    // dereference: `write_all` refuses it before any call, even with nothing to write, and
    // leaves `errno` alone.
    let (bytes, empty, errno_after) = in_printf_handler(|file| {
        set_errno(ffi::EINTR);
        let bytes = file.write_all(b"from Rust").unwrap_err();
        let empty = file.write_all(b"").unwrap_err();
        (bytes, empty, errno())
    });
    for e in [bytes, empty] {
        assert_eq!(e.kind(), io::ErrorKind::Unsupported);
        assert_eq!(e.raw_os_error(), None);
        assert_eq!(e.to_string(), "the stream has no lock");
    }
    assert_eq!(errno_after, ffi::EINTR);
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_flush_that_fails_in_its_seek_is_reported_as_written() {
    // The second failure the doc names that the binding cannot see: a line-buffered stream C
    // read from and then writes, over a socket, whose flush fails in its seek back over the
    // unread bytes, and sets no indicator.
    let stream = Stream::line_buffered_socket_after_a_read();
    let file = stream.handle();
    // A first line goes to the buffer byte by byte, and the flush at its newline fails short.
    set_errno(0);
    let e = file.write_all(b"ab\n").unwrap_err();
    assert_eq!(e.raw_os_error(), Some(ffi::ESPIPE));
    assert_eq!(errno(), ffi::ESPIPE);
    assert!(!stream.has_error());
    // A second fits the buffer: `fwrite` returns its full count, and the call succeeds,
    // leaving `errno` as it was.
    set_errno(ffi::EINTR);
    file.write_all(b"cd\n").unwrap();
    assert_eq!(errno(), ffi::EINTR);
    assert!(!stream.has_error());
    assert!(stream.is_unlocked());
    // Nothing reached the socket.
    assert_eq!(stream.contents(), b"");
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn an_unbuffered_stream_takes_each_write_at_once() {
    // As glibc opens `stderr`: what fits a buffer goes out during the call, so a device that
    // takes nothing fails the first write, however short.
    let stream = Stream::file().unbuffered();
    let file = stream.handle();
    file.write_all(b"status: ").unwrap();
    file.write_all(b"ok\n").unwrap();
    assert_eq!(stream.contents(), b"status: ok\n");

    let full = Stream::full().unbuffered();
    set_errno(0);
    let e = full.handle().write_all(b"x").unwrap_err();
    assert_eq!(e.raw_os_error(), Some(ffi::ENOSPC));
    assert_eq!(errno(), ffi::ENOSPC);
    assert!(full.has_error());
    assert!(full.is_unlocked());
    // Buffered, the same write succeeds, its failure left to the flush.
    Stream::full().handle().write_all(b"x").unwrap();
}
