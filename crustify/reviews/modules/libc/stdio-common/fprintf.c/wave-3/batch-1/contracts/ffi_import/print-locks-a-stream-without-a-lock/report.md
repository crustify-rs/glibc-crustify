# `FileRef::print` takes the lock of a stream that has none: `flockfile` dereferences its NULL `_lock`

- **item**: `fn::fprintf`, bound as `FileRef::print` and `FileRef::write_fmt`,
  `src/libc/stdio_common/fprintf.rs`: the `// SAFETY:` justification of the `flockfile`
  call in `print_locked`, which `print` reaches for every stream `FileRef::from_ptr`'s
  contract admits
- **affected commit**: `53dd776` (wave 3, batch 0)
- **objective**: `contracts`, of the `ffi_import` region. The justification says
  `flockfile` "takes its recursive lock with no other precondition". It has one: the stream
  must have a lock. A stream that honours `FileRef::from_ptr`'s contract can lack one, and
  `print` then reads through a NULL pointer inside glibc.

## What is wrong

glibc's `flockfile` is `_IO_lock_lock (*stream->_lock)` with no check
(`c/stdio-common/flockfile.c:25`). glibc's own stdio routines never reach it on a stream
without a lock. They lock through the `_IO_flockfile` macro, which skips any stream
carrying `_IO_USER_LOCK` (`c/libio/libio.h:282-285`).

Every stream a program opens has a lock. The opening routines set `_lock` before they
hand the stream out, and nothing writes it again while the stream is open (`fopen`,
`fdopen`, `fopencookie`, `popen`, `open_memstream`, `open_wmemstream`, and `fmemopen`
through `fopencookie`; `stdin`, `stdout` and `stderr` have static locks). glibc also hands
user code a stream of its own that has no lock. Since glibc 2.37, the handler of a
conversion that `register_printf_specifier` adds gets a `FILE` over printf's own buffer,
which `__printf_buffer_as_file_init` creates with `_lock = NULL` and `_IO_USER_LOCK`
(`c/stdio-common/printf_buffer_as_file.c:97-101`,
`c/stdio-common/Xprintf_function_invoke.c:43-47`; `wprintf_buffer_as_file.c` likewise).
The handler writes it with stdio routines, such as `fputs` and `fprintf`, which skip the
lock.

A handler that hands that stream to Rust honours `FileRef::from_ptr`'s contract for a
stream (`src/libc/libio/genops.rs`, `File`'s documentation). The stream is open for the
handle's lifetime, the handler's call, and no other access happens while the handle lives,
since the printf that called the handler waits on the same thread. `print` then calls
`flockfile`, which reads `_lock->owner` at address `0x8`. This is reachable from a C
printf handler that the parent campaigns' remaining C could hold, or from a Rust handler
that borrows its stream.

## Reproducer

`reproducer/` is a standalone package. It registers `%W`, whose handler borrows its stream
with `FileRef::from_ptr`, under the contract above, and writes to it with `print`. `main`
writes `[%W]` with the raw `fprintf`. On the affected commit it dies with a SEGV inside
`flockfile`. Once `print` refuses the stream, it prints the refusal and exits 0.

```console
$ cd crustify/reviews/modules/libc/stdio-common/fprintf.c/wave-3/batch-1/contracts/ffi_import/print-locks-a-stream-without-a-lock/reproducer
$ CRUSTIFY_BUILD_VARIANT=plain RUSTFLAGS="-Zsanitizer=address -Cforce-frame-pointers=yes" \
    ASAN_SYMBOLIZER_PATH=/usr/lib/llvm-18/bin/llvm-symbolizer \
    cargo +nightly-2026-10-08 run --offline --target x86_64-unknown-linux-gnu; echo exit=$?
handler: stream 0x7ffc8315ced0, _lock 0x0
AddressSanitizer:DEADLYSIGNAL
=================================================================
==1279356==ERROR: AddressSanitizer: SEGV on unknown address 0x000000000008 (pc 0x77a8cd7ee4f5 bp 0x7ffc8315ca50 sp 0x7ffc8315ca40 T0)
==1279356==The signal is caused by a READ memory access.
==1279356==HINT: address points to the zero page.
    #0 0x77a8cd7ee4f5 in flockfile stdio-common/flockfile.c:25:3
    #1 0x57107cf33d88 in glibc_bindings::libc::stdio_common::fprintf::ffi_import::print_locked src/libc/stdio_common/fprintf.rs:121:18
    #2 0x57107cf319b9 in <glibc_bindings::libc::libio::genops::ffi_import::FileRef>::print src/libc/stdio_common/fprintf.rs:68:30
    #3 0x57107cf30b60 in print_locks_a_stream_without_a_lock::handler .../reproducer/src/main.rs:42:22
    #4 0x77a8cd7f382c in __printf_function_invoke stdio-common/Xprintf_function_invoke.c:48:14
    #5 0x77a8cd7f67ed in printf_positional stdio-common/vfprintf-internal.c:1346:5
    #6 0x77a8cd7f8244 in __printf_buffer stdio-common/vfprintf-internal.c:1041:3
    #7 0x77a8cd7fa73a in __vfprintf_internal stdio-common/vfprintf-internal.c:1544:7
exit=1
```

This was observed on the host's glibc, `Ubuntu GLIBC 2.39-0ubuntu8.9`. The trace's line
numbers are the affected commit's. ASan also prints `unexpected format specifier in printf
interceptor: %W` once; that warning is unrelated. The plain build (`cargo run`, without
`RUSTFLAGS`) dies with `SIGSEGV`.

## Fix

The fix refuses the stream in Rust, where Rust can test it, before any call. That is the
code that the broken contract guards.

- `FileRef::has_lock` (private, in `fprintf.rs`'s region) reads the stream's `_lock` and
  reports whether it is non-null. The read takes no lock. It races with no write, since
  only the opening routine writes `_lock`, before any handle to the stream exists.
- `print` returns `io::ErrorKind::Unsupported`, "the stream has no lock", for such a
  stream, after the NUL check and before `errno` is touched. Nothing is written, and
  `errno` is left as it was. `write_fmt` inherits the refusal. The divergence from C,
  whose `fprintf` writes such a stream unlocked, is documented in `print`'s Errors.
  Writing it unlocked would break `FileRef`'s `Sync`: the handle can be copied to another
  thread for the handler's duration.
- `flockfile`'s justification now states the precondition, and names the check that
  meets it.
- `File`'s documentation, and `Sync for File`'s justification, said `FileRef` reads no
  field of the stream. They now say it reads `_lock` alone, and why that read needs no
  lock.

The reproducer became the unit test `a_stream_without_a_lock_is_refused`
(`tests/unit_pub/fprintf.rs`). Its stream comes from `in_printf_handler`, a new fixture in
`tests/fixtures/stream_unit.rs`. `unit_pub` is the suite: ffibox's `testing` section makes
a test through the `from_ptr` seam a unit test. The test passes plain and under ASan and
TSan. With the check removed, it dies with `SIGSEGV`. It stops under Miri at
`register_printf_specifier`, which carries its `calls C` mark.
