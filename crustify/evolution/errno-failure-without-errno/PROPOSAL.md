# A failure that sets no `errno` is not reported as the stale one

Status: `open`. Filed by glibc-crustify `wave-3/batch-0/run-1` (2026-11-13), under the
default `repair` autonomy (`crustify/CRUSTIFY.md` gives no `evolution` answer). It is a gap
in the spec and in a tool's documentation. It did not block the batch, so it is proposed.

## Signal

A rule worked out by hand that a skill should have stated (`evolution` §2), and one that
the tool's documentation leads a binder wrong on. `binding-symbols`' Functions section reads
a failure from "error state the library keeps beside the call, read immediately after it".
ffibox's `docs/patterns.md` maps "`-1` for failure, the error in `errno`" to
`status::errno`, which returns `io::Error::last_os_error()` on a `-1`
(`src/status.rs:84`, at the revision this campaign patches in). Neither covers a routine
that can return its failure without setting `errno`. A binding that follows them reports
that failure as whatever error an earlier call left in `errno`, or as `Os { code: 0,
"Success" }`.

glibc's `fprintf` (wave 3's item) does this on two ordinary paths:

- a wide-oriented stream: `vfprintf`'s orientation check returns `EOF` with no `errno`
  (`c/stdio-common/vfprintf-internal.c:134`, `ORIENT`);
- a cookie stream whose write function fails without setting it, as `fmemopen`'s does
  once its storage is full: it takes what fits and returns that count, and the short
  write fails `fprintf` (`c/libio/fmemopen.c`, `fmemopen_write`).

The batch's equivalence tests compare the binding with the raw `fprintf` on identical
streams (`tests/equiv_pub/fprintf.rs`: `a_full_stream_fails_alike`,
`a_wide_stream_fails_alike`). C's `errno` after both failures is the value it held
before the call. The unit tests `a_wide_stream_fails_without_errno` and
`a_write_that_sets_no_errno_fails_without_it` check the binding's answer. The binding
clears `errno` before the call. A failure that leaves it at 0 becomes an error with no OS
code, `io::ErrorKind::Other`, and `errno` is restored, as C leaves it
(`src/libc/stdio_common/fprintf.rs`, `print` and `outcome`).

## Destination

- The spec: `binding-symbols`' Functions section, its **Errors** bullet.
- A tool: ffibox's `docs/patterns.md`, the `status::errno` row, and `status::errno`'s
  doc comment.

## Change

`binding-symbols`, Errors, after "read immediately after it, before another call into the
library can replace it": "Where the analysis finds a failure the convention reports
without its error state, such as a `-1` that sets no `errno`, the state is cleared before
the call, and a failure that leaves it clear is an error of its own. That error carries no
code, and the caller's state is restored, as the C leaves it."

ffibox, `status::errno`'s documentation: "It reads `errno` as it stands. For a routine
that may fail without setting it, clear `errno` before the call, and read an error whose
`raw_os_error()` is `Some(0)` as a failure the routine did not describe."

## Validation

The binding written this way passes its equivalence tests against C on every stream
kind a C caller can hand it, and its unit tests (`tests/unit_pub/fprintf.rs`), plain and
under ASan and TSan. A binding that read `errno` without clearing it would report the
`EINTR` that those unit tests set before the call. No rerun of another item is needed. No
other item of this campaign uses the `errno` convention yet.

## Review

Not reviewed.
