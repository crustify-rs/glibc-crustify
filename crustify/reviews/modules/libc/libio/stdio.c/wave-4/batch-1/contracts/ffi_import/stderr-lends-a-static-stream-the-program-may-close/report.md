# `stderr()` lends a `FileRef<'static>` from safe code, on assumptions about the program's C that the crate cannot check

- **item**: `glb::stderr`, bound as `stderr() -> FileRef<'static>`,
  `src/libc/libio/stdio.rs`: the two `// SAFETY:` justifications of its `ffi_import` body
  and the safe signature they let it keep
- **affected commit**: `2a792a8` (wave 4, batch 0)
- **objective**: `contracts`, of the `ffi_import` region. A safe function discharges
  `FileRef::from_ptr`'s contract for the `'static` lifetime by assuming what the whole
  program's C does with a variable glibc lets it set. The contract breaks through safe
  code. The reproducer below shows a heap-use-after-free under ASan.

## What is wrong

`stderr()` is safe and returns `FileRef<'static>`. Its body reads the global `stderr` and
borrows the stream it names with `FileRef::from_ptr`, whose contract for a stream
(`src/libc/libio/genops.rs`, `File`'s documentation, as `6e892ea` corrected it) is:

> the pointer is NULL or an open stream that stays open for the handle's lifetime, `fclose`
> included, and every other access to it while the handle lives, C's or Rust's, on any
> thread, holds its lock.

For `'static`, the function discharges this with assumptions about the program, which its
doc lists. No code assigns `stderr` while another thread may read it. Nothing passes its
stream to `fclose` or to a failing `freopen`. Its locking stays internal. The doc and the
batch's commit message justify these by what glibc's own routines and the C of the two
parent campaigns do. They do not follow from anything the crate or the C contract
guarantees:

- glibc's `stderr` is a variable a program may set. Its manual (`c/manual/stdio.texi:115-122`)
  says so: "In the GNU C Library, `stdin`, `stdout`, and `stderr` are normal variables which
  you can set just like any others." It gives `fclose (stdout); stdout = fopen (…);` as
  the way to redirect a standard stream.
- A stream so opened is heap memory that `fclose` frees. `_IO_new_fclose` ends in
  `_IO_deallocate_file` (`c/libio/iofclose.c:74`), which spares only `_IO_2_1_stdin_`,
  `_IO_2_1_stdout_` and `_IO_2_1_stderr_` and frees every other stream
  (`c/libio/libioP.h:948-958`). Even `_IO_2_1_stderr_` is closed by `fclose`, and C11
  7.21.3p4 leaves a closed stream's pointer indeterminate.
- The handle is `Copy`, `Send` and `'static`, so safe code may keep it anywhere: in a
  `static`, as the batch's own `ub_safe` test `the_handle_outlives_every_scope` does, or on
  a detached thread. C's own `fprintf (stderr, …)` reads the variable at each call. A kept
  Rust handle does not, so a program that redirects its standard error stream is correct C
  and still leaves the handle dangling.

`glibc-bindings` is a library. It is linked into programs whose C it cannot see, and the
parents' C is not all of that C either: libgit2 is itself a library. `binding-symbols`'
Globals section binds a global by its C accesses. A global that is neither immutable after
initialization nor synchronized by C is "otherwise mutable", and gets `unsafe` accessors
that exclude concurrent access. ffibox's goals (`docs/goals.md`, G9 and G10) exclude use
after free through lifetimes, and make adopting a pointer `unsafe`, "an unknown fact
selects[ing] the more restrictive primitive". The safe `'static` signature does neither.

The read of the global has the same flaw: its `// SAFETY:` comment excludes a racing
write by assuming the program makes none. Since glibc lets a program write `stderr`, the
crate cannot vouch for that from safe code.

No safe operation on `FileRef` reads the stream yet in this tree. Wave 3's `FileRef::print`
(`stdio-common/fprintf.c`, on its own branch) will take the handle's stream lock with
`flockfile` and then call `fprintf`. Once both land, `stderr().print(…)` on a kept handle is
a use after free in safe code. The reproducer makes that operation's first read itself,
in Rust, where ASan sees it, since glibc is not instrumented.

## Reproducer

`reproducer/`, a standalone package over the affected crate. It is single-threaded, and
each C step is one glibc's manual offers, written in Rust through extern declarations as C
would make it:

1. C redirects `stderr` to a stream it opens: `stderr = fopen("/dev/null", "w")`.
2. Safe Rust takes `let kept: FileRef<'static> = stderr();`.
3. C rotates it: `fclose(stderr); stderr = <the original stream>`.
4. `kept` is still a live `'static` handle. ASan reports its stream freed
   (`__asan_address_is_poisoned`), and the read that `flockfile`, `FileRef::print`'s first
   step, makes of the stream's `_lock` is a heap-use-after-free.

From a clean checkout of `2a792a8`, with this directory copied in:

```console
$ cd crustify/reviews/modules/libc/libio/stdio.c/wave-4/batch-1/contracts/ffi_import/stderr-lends-a-static-stream-the-program-may-close/reproducer
$ RUSTFLAGS="-Zsanitizer=address -Cforce-frame-pointers=yes" \
    cargo +nightly-2026-10-08 run --target x86_64-unknown-linux-gnu
stderr() lent a FileRef<'static> over 0x748f985e0300, a stream fclose has freed; the next operation on it reads freed memory:
==PID==ERROR: AddressSanitizer: heap-use-after-free on address 0x748f985e0388 at pc 0x5645b1cc4881 bp 0x7ffeae07f150 sp 0x7ffeae07f148
READ of size 8 at 0x748f985e0388 thread T0
    #0 0x5645b1cc4880 in core::ptr::read_volatile::<*mut core::ffi::c_void> …/library/core/src/ptr/mod.rs:2163
    #1 0x5645b1cc4880 in stderr_lends_a_static_stream_the_program_may_close::main …/reproducer/src/main.rs:75

0x748f985e0388 is located 136 bytes inside of 472-byte region [0x748f985e0300,0x748f985e04d8)
freed by thread T0 here:
    #0 0x5645b1c95406 in free ??:?
    #1 0x773f992b940f in _IO_deallocate_file libio/./libio/libioP.h:958
    #2 0x773f992b940f in _IO_new_fclose libio/./libio/iofclose.c:74
    #3 0x5645b1c65f3c in fclose ??:?
    #4 0x5645b1cc4752 in stderr_lends_a_static_stream_the_program_may_close::main …/reproducer/src/main.rs:58

previously allocated by thread T0 here:
    #0 0x5645b1c959d4 in malloc ??:?
    #1 0x773f992b9f5e in __fopen_internal libio/./libio/iofopen.c:65
    #2 0x773f992b9f5e in _IO_new_fopen libio/./libio/iofopen.c:86
    #3 0x5645b1c63dfd in fopen ??:?
    #4 0x5645b1cc42e6 in stderr_lends_a_static_stream_the_program_may_close::c_redirects_stderr …/reproducer/src/main.rs:35
    #5 0x5645b1cc471c in stderr_lends_a_static_stream_the_program_may_close::main …/reproducer/src/main.rs:53

SUMMARY: AddressSanitizer: heap-use-after-free … in stderr_lends_a_static_stream_the_program_may_close::main
==PID==ABORTING
$ echo $?
1
```

The trace was symbolized with `ASAN_OPTIONS=external_symbolizer_path=/usr/bin/addr2line`.
The host's glibc is `Ubuntu GLIBC 2.39-0ubuntu8.9`. The package repeats the affected
workspace's `[patch]` of ffibox, which a path dependency does not carry.

On the fixed tree, the reproducer no longer compiles. Its step 2 calls an `unsafe fn`
outside an `unsafe` block (`E0133`), which is the regression test below.

## Fix

`stderr` is now `pub unsafe fn stderr<'a>() -> FileRef<'a>`, an accessor of an otherwise
mutable global, as `binding-symbols`' Globals section has it. Its `# Safety` section moves
the assumptions the safe function made onto its caller, for the lifetime the caller
chooses:

- no write of `stderr` races with the read;
- the stream it names stays open for `'a`: no `fclose`, and no failing `freopen`, while
  the handle lives;
- every other access to that stream while the handle lives holds its lock: `FileRef`'s
  contract, with its locking internal and `_unlocked` routines only under `flockfile`.

Its doc quotes the manual and states the consequence. A handle taken for one call, as C's
`fprintf (stderr, …)` takes the stream, asks of the program what that C expression asks
of it. A `'static` one asks that nothing ever closes the stream. The two `// SAFETY:`
comments now cite the caller's obligations. The read of the global and the `NULL` panic
are unchanged.

Tests:

- `tests/ub_safe/compile_fail/stdio_stderr_is_lent_by_no_safe_code.rs`, the UB regression
  test: the reproducer's safe step, `let _kept: FileRef<'static> = stderr();`, fails to
  compile (`E0133`).
- `tests/ub_unsafe/stdio.rs`, a new suite: the batch's two `ub_safe` tests (a `'static`
  handle kept in a `static` and sent to a detached thread; 16 threads taking and sharing
  it), now called within the contract.
- `tests/equiv_pub/stdio.rs`: the existing comparison with C's expression, and a new one,
  `stderr_follows_c_pointing_it_at_another_stream`, which fills a `test-adequacy` gap: no
  test changed `stderr`, so none could tell a read on every call from a cached one. C points
  `stderr` at a stream it opens with `fopen`. `stderr()` then lends that stream, and after
  C points it back, lends the original, while the handle taken before keeps its own. The
  file's tests hold a lock while they read or write the global, as the contract asks.
  `fopen` and `fclose` join the raw bindings for it, under the allowlist's test-only
  heading.
- The batch's two `unit_pub` tests are removed with their suite. They called the accessor
  and asserted the same stream on every call and thread, which `equiv_pub` asserts against C.
  `unit_pub` forbids `unsafe_code`, so they cannot call it there.

The public API changes: `ffimap api diff` classifies `glb::stderr` `changed` (`fn() ->
FileRef<'_>` to `unsafe fn<'_>() -> FileRef<'_>`), kept for soundness. `st::_IO_FILE` and
`tdf::FILE` are `kept`. No consumer in this crate calls `stderr`. The parent's
`llhttp__debug` port, which plans to write to it, can now vouch for a handle held for its
one write, as its C does. A safe writer that reads the global and writes in one locked
call, as C's `fprintf (stderr, …)` does, can be added once `fprintf` lands; it is beyond
this fix, since `FileRef::print` is not on this branch.
