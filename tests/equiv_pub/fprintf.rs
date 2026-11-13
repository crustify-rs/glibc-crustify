//! `stdio-common/fprintf.c`: `FileRef::print` against `fprintf(stream, "%s", text)`.
//!
//! Each test opens two streams from one description, writes one through the binding and
//! the other through the raw `fprintf`, with the same texts in the same order, and compares
//! after each write: the count or the error, `errno`'s error as the binding maps it, the
//! stream's error indicator, its lock, which the binding leaves released, and what it holds.

use core::ffi::{CStr, c_int};
use std::ffi::CString;

use glibc_bindings::libc::csu::errno_loc::{errno, set_errno};
use glibc_bindings::libc::ffi;

use crate::stream::Stream;

/// C's write: `fprintf(stream, "%s", text)`, with the `errno` it left after it was cleared.
fn c_print(stream: &Stream, text: &CStr) -> (c_int, c_int) {
    set_errno(0);
    // SAFETY: an open stream the fixture keeps, a `"%s"` format and the NUL-terminated
    // string its conversion reads.
    let ret = unsafe { ffi::fprintf(stream.as_ptr(), c"%s".as_ptr(), text.as_ptr()) };
    (ret, errno())
}

/// The stream's error indicator, as `ferror` reads it.
fn has_error(stream: &Stream) -> bool {
    // SAFETY: an open stream.
    unsafe { ffi::ferror(stream.as_ptr()) != 0 }
}

/// Writes `texts` to `rust` through the binding and to `c` through C, comparing after each.
fn compare(rust: &Stream, c: &Stream, texts: &[&str]) {
    for (step, text) in texts.iter().enumerate() {
        let got = rust.handle().print(text);
        let want = c_print(c, &CString::new(*text).unwrap());
        match got {
            Ok(count) => assert_eq!(Ok(count), usize::try_from(want.0), "step {step}"),
            Err(e) => {
                assert!(
                    want.0 < 0,
                    "step {step}: Rust failed with {e}, C returned {}",
                    want.0
                );
                assert_eq!(
                    e.raw_os_error(),
                    (want.1 != 0).then_some(want.1),
                    "step {step}"
                );
            }
        }
        assert_eq!(
            has_error(rust),
            has_error(c),
            "step {step}: error indicator"
        );
        assert!(
            rust.is_unlocked(),
            "step {step}: the binding left the stream locked"
        );
        assert_eq!(rust.contents(), c.contents(), "step {step}: contents");
    }
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_growing_stream_takes_the_same_bytes() {
    let long = "0123456789abcdef".repeat(1024);
    let texts = [
        "",
        "unknown option: ",
        "--bare",
        " 100% %s %n %%\n",
        "é→✓",
        &long,
        "\n",
    ];
    compare(&Stream::memory(), &Stream::memory(), &texts);
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_stream_locked_by_its_caller_takes_the_same_bytes() {
    let (rust, c) = (Stream::memory(), Stream::memory());
    rust.lock_by_caller();
    c.lock_by_caller();
    let long = "x".repeat(20_000);
    compare(
        &rust,
        &c,
        &["argument '", "--output", "' requires a value.\n", &long],
    );
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_stream_not_open_for_writing_fails_alike() {
    compare(
        &Stream::fixed(16, c"r"),
        &Stream::fixed(16, c"r"),
        &["text", "", "more"],
    );
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_full_stream_fails_alike() {
    let long = "y".repeat(64 * 1024);
    let texts = ["fits", &long, "after", ""];
    compare(&Stream::fixed(16, c"w"), &Stream::fixed(16, c"w"), &texts);
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_failing_device_fails_alike() {
    let long = "z".repeat(64 * 1024);
    compare(&Stream::full(), &Stream::full(), &["fits", &long, "after"]);
}

#[test]
#[cfg_attr(miri, ignore = "calls C")]
fn a_wide_stream_fails_alike() {
    let (rust, c) = (Stream::file(), Stream::file());
    rust.orient_wide();
    c.orient_wide();
    compare(&rust, &c, &["text", ""]);
}
