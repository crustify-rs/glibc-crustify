//! `stdio-common/fprintf.c`: `FileRef::print` and `write!` on a stream.

use std::sync::Barrier;
use std::time::Duration;
use std::{fmt, io, thread};

use glibc_bindings::libc::csu::errno_loc::{errno, set_errno};
use glibc_bindings::libc::ffi;

use crate::stream::Stream;
use crate::stream_unit::in_printf_handler;

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn print_writes_the_text_and_returns_its_length() {
    let stream = Stream::memory();
    let file = stream.handle();
    assert_eq!(file.print("unknown option: ").unwrap(), 16);
    assert_eq!(file.print("--bare").unwrap(), 6);
    assert_eq!(file.print("").unwrap(), 0);
    // Not a format: a conversion in the text is written as it is.
    assert_eq!(file.print(" 100% %s %n\n").unwrap(), 12);
    // UTF-8 is written byte for byte.
    assert_eq!(file.print("é→✓").unwrap(), "é→✓".len());
    assert_eq!(
        stream.contents(),
        "unknown option: --bare 100% %s %n\né→✓".as_bytes()
    );
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_text_longer_than_the_buffer_is_written_whole() {
    let stream = Stream::memory();
    let text = "0123456789abcdef".repeat(4096);
    assert_eq!(stream.handle().print(&text).unwrap(), text.len());
    assert_eq!(stream.contents(), text.as_bytes());
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn write_formats_in_rust_and_returns_the_count() {
    let stream = Stream::memory();
    let file = stream.handle();
    let (name, status) = ("cli", 7);
    assert_eq!(write!(file, "{name}: ").unwrap(), 5);
    assert_eq!(writeln!(file, "unknown status: {status}").unwrap(), 18);
    assert_eq!(
        writeln!(file, "flags={:02x} next=null", 0x0A_u8).unwrap(),
        19
    );
    // A literal with no argument is written without formatting.
    assert_eq!(writeln!(file, "no error").unwrap(), 9);
    assert_eq!(
        stream.contents(),
        b"cli: unknown status: 7\nflags=0a next=null\nno error\n"
    );
}

/// An argument whose formatting fails, as a `Display` impl may.
struct Failing;

impl fmt::Display for Failing {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        Err(fmt::Error)
    }
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_failing_argument_writes_nothing() {
    let stream = Stream::memory();
    let file = stream.handle();
    let e = write!(file, "before {Failing} after").unwrap_err();
    assert_eq!(e.kind(), io::ErrorKind::Other);
    assert_eq!(e.raw_os_error(), None);
    assert_eq!(stream.contents(), b"");
    assert!(stream.is_unlocked());
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_nul_in_the_text_is_refused_before_the_call() {
    let stream = Stream::memory();
    let file = stream.handle();
    file.print("kept").unwrap();
    set_errno(ffi::EINTR);
    let e = file.print("cut\0here").unwrap_err();
    assert_eq!(e.kind(), io::ErrorKind::InvalidInput);
    assert_eq!(e.raw_os_error(), None);
    assert_eq!(errno(), ffi::EINTR);
    let text = "a\0b";
    let e = write!(file, "{text}").unwrap_err();
    assert_eq!(e.kind(), io::ErrorKind::InvalidInput);
    assert_eq!(stream.contents(), b"kept");
    assert!(stream.is_unlocked());
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_success_leaves_errno_as_it_was() {
    let stream = Stream::memory();
    set_errno(ffi::ENOENT);
    stream.handle().print("text").unwrap();
    assert_eq!(errno(), ffi::ENOENT);
    set_errno(0);
    stream.handle().print("more").unwrap();
    assert_eq!(errno(), 0);
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_stream_not_open_for_writing_fails_with_ebadf() {
    let stream = Stream::fixed(16, c"r");
    set_errno(0);
    let e = stream.handle().print("text").unwrap_err();
    assert_eq!(e.raw_os_error(), Some(ffi::EBADF));
    // `errno` keeps the error, as C's does.
    assert_eq!(errno(), ffi::EBADF);
    assert!(stream.has_error());
    assert!(stream.is_unlocked());
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_failed_flush_reports_the_write_error() {
    // A text longer than the stream's buffer flushes it, into a device that takes nothing.
    let stream = Stream::full();
    let text = "x".repeat(64 * 1024);
    set_errno(0);
    let e = stream.handle().print(&text).unwrap_err();
    assert_eq!(e.raw_os_error(), Some(ffi::ENOSPC));
    assert_eq!(errno(), ffi::ENOSPC);
    assert!(stream.is_unlocked());
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_write_that_sets_no_errno_fails_without_it() {
    // 16 bytes of `fmemopen` storage: its write function takes what fits and then fails
    // without setting `errno`, so the flush of a longer text fails without it too.
    let stream = Stream::fixed(16, c"w");
    let text = "x".repeat(64 * 1024);
    set_errno(ffi::EINTR);
    let e = stream.handle().print(&text).unwrap_err();
    assert_eq!(e.kind(), io::ErrorKind::Other);
    assert_eq!(e.raw_os_error(), None);
    assert_eq!(errno(), ffi::EINTR);
    assert!(stream.is_unlocked());
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_wide_stream_fails_without_errno() {
    let stream = Stream::file();
    stream.orient_wide();
    set_errno(ffi::EINTR);
    let e = stream.handle().print("text").unwrap_err();
    assert_eq!(e.kind(), io::ErrorKind::Other);
    assert_eq!(e.raw_os_error(), None);
    assert_eq!(errno(), ffi::EINTR);
    assert_eq!(stream.contents(), b"");
    assert!(stream.is_unlocked());
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn the_lock_is_released_after_every_outcome() {
    let stream = Stream::memory();
    stream.handle().print("ok").unwrap();
    assert!(stream.is_unlocked());
    write!(stream.handle(), "{}", 1).unwrap();
    assert!(stream.is_unlocked());
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn print_waits_for_the_lock_on_a_stream_locked_by_its_caller() {
    // Under `FSETLOCKING_BYCALLER`, glibc's own `fprintf` would not wait: the binding's
    // `flockfile` does.
    let stream = Stream::memory();
    stream.lock_by_caller();
    let file = stream.handle();
    let hold = stream.hold_lock();
    let started = Barrier::new(2);
    thread::scope(|s| {
        let waiter = s.spawn(|| {
            started.wait();
            file.print("second").unwrap()
        });
        started.wait();
        thread::sleep(Duration::from_millis(100));
        // The lock is recursive: this thread, which holds it, writes through it.
        file.print("first ").unwrap();
        drop(hold);
        assert_eq!(waiter.join().unwrap(), 6);
    });
    assert_eq!(stream.contents(), b"first second");
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn writes_from_threads_never_interleave() {
    // Every thread shares one handle to a stream whose locking is the caller's, so that
    // only the binding's lock orders the writes. Each line is long enough that writes flush
    // the stream's buffer mid-line. A file stream allocates its buffer once: an
    // `open_memstream` one reallocates it with `malloc` and `free`, which TSan sees, under
    // glibc's stream lock, which it does not, and reports as a race.
    const THREADS: usize = 8;
    const LINES: usize = 200;
    let stream = Stream::file();
    stream.lock_by_caller();
    let file = stream.handle();
    let pad = "=".repeat(3000);
    thread::scope(|s| {
        for t in 0..THREADS {
            let pad = &pad;
            s.spawn(move || {
                for i in 0..LINES {
                    let line = format!("{t}:{i}:{pad}\n");
                    assert_eq!(write!(file, "{line}").unwrap(), line.len());
                }
            });
        }
    });
    let contents = String::from_utf8(stream.contents()).unwrap();
    let mut seen = vec![vec![false; LINES]; THREADS];
    for line in contents.lines() {
        let mut parts = line.splitn(3, ':');
        let t: usize = parts.next().unwrap().parse().unwrap();
        let i: usize = parts.next().unwrap().parse().unwrap();
        assert_eq!(parts.next().unwrap(), pad, "line {t}:{i} interleaved");
        assert!(!seen[t][i], "line {t}:{i} written twice");
        seen[t][i] = true;
    }
    assert!(seen.iter().flatten().all(|&s| s), "a line is missing");
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_stream_without_a_lock_is_refused() {
    // The stream glibc hands a printf handler has no lock, which `flockfile` would
    // dereference: `print` refuses it before any call, and leaves `errno` alone.
    let (print, write, errno_after) = in_printf_handler(|file| {
        set_errno(ffi::EINTR);
        let print = file.print("from Rust").unwrap_err();
        let write = write!(file, "{}", 1).unwrap_err();
        (print, write, errno())
    });
    for e in [print, write] {
        assert_eq!(e.kind(), io::ErrorKind::Unsupported);
        assert_eq!(e.raw_os_error(), None);
        assert_eq!(e.to_string(), "the stream has no lock");
    }
    assert_eq!(errno_after, ffi::EINTR);
}
