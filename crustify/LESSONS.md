# Lessons of glibc-crustify

Rules that hold for this campaign alone, as `evolution`'s Lessons section describes. Every
run reads them before its first step.

## A batch's gates run with no build variant

**Rule.** libc is the system's, so `build.json` has no variants, and this tree has no
`cargo-variant` script. A batch runs its gates directly, each build in a target dir of its
own under `crustify/tmp/`:

- campaign mode: `CRUSTIFY_BUILD_VARIANT=<plain | asan-ubsan | tsan>` set on every
  `cargo` command that builds the tests, which then regenerate the bindings and check them
  against the committed ones;
- format: `cargo +<the workspace's nightly> fmt --all -- --check`, since `rustfmt.toml`
  uses nightly-only options;
- ASan: on that nightly, `--target x86_64-unknown-linux-gnu`, with
  `RUSTFLAGS="-Zsanitizer=address -Cforce-frame-pointers=yes"`, and the same flags in
  `RUSTDOCFLAGS`, without which the doctests fail to link against the ASan runtime; TSan:
  the same with `-Zsanitizer=thread` and `-Zbuild-std`. They instrument the Rust alone,
  since the system's glibc is not instrumented, so TSan sees no race inside glibc. The host
  has no `llvm-symbolizer` on its `PATH`, so a sanitizer trace a report carries is
  symbolized with `ASAN_OPTIONS=external_symbolizer_path=/usr/bin/addr2line`;
- coverage: Rust only, measured as the parent's `crustify/scripts/coverage/baseline.sh`
  does, over a scratch copy without `c/`. Its `report.py` rejects the scope, since
  `llvm-cov` prints this tree's file names relative to `src/`, so the `TOTAL` line is read
  from `llvm-cov report` itself. The compile-fail suite fails in the marked copy, which
  only a nightly builds, and it covers nothing. Its `mark.py` misses a test item whose
  attributes span lines, so a test module's `#[expect]` is kept on one line, and the copy
  holds the tree's package, or the script measures the workspace that encloses it
  (`crustify/evolution/coverage-marking-misses-multiline-attributes`);
- a wave's and a module's end gates: `testing-gates`' `end-gates`, with `--stages
  plain,asan-ubsan,tsan,miri` and with `--runner` set to a script in `crustify/tmp/`. Given
  `<variant> <cargo args>`, the script runs `cargo` with the flags above, adding
  `--target-dir` under the worktree's `crustify/tmp/` so that every snapshot reuses one
  build per variant. Miri takes `--stage-env miri:CRUSTIFY_BUILD_VARIANT=plain` and
  `--stage-env miri:CARGO_TARGET_DIR=<worktree>/crustify/tmp/target-miri`.

**Evidence.** wave-2/batch-0/run-1 (`libio/genops.c`) worked these out by hand.
wave-2/batch-1/run-1, its module pass, wrote the `end-gates` runner by hand too.
wave-3/batch-0/run-1 (`stdio-common/fprintf.c`) found that the `struct_tm_h` doctest fails to
link under ASan without `RUSTDOCFLAGS`, and passes with it.
wave-4/batch-0/run-1 (`libio/stdio.c`) found the doctests failing to link under ASan with
`RUSTFLAGS` alone (`undefined symbol: __asan_init`), and passing with `RUSTDOCFLAGS`.
wave-4/batch-1/run-1, its module pass, found its reproducer's ASan trace printed as bare
addresses, and symbolized through `addr2line`.
wave-5/batch-1/run-1, its module pass, measured `genops.rs`' test module as covered code
under a multi-line `#[expect]`, and kept it on one line.

**Scope.** Every batch until the tree gets a variant script.

## A threaded stream test writes a file stream, not an `open_memstream` one

**Rule.** A test that writes one stream from several threads, to check that its writes
hold the stream's lock, opens a file stream (`fopen`), whose buffer glibc allocates once.
An `open_memstream` stream grows its buffer with `malloc` and `free` inside each write,
under the stream's lock. TSan intercepts those calls but not glibc's lock, which is an
uninstrumented futex, so it reports every two threads' reallocations as a data race in
`free` (`_IO_str_overflow`), and the suite fails under TSan though the writes are
serialized. A test with one writing thread may use either.

**Evidence.** wave-3/batch-0/run-1 (`stdio-common/fprintf.c`):
`writes_from_threads_never_interleave` over `open_memstream` was reported under TSan, and was
clean over `fopen` in five runs. It still fails without the binding's `flockfile`, with
interleaved lines over `fopen` and a glibc abort over `open_memstream`.

**Scope.** Every TSan run over tests that share a C stream between threads, `stderr`'s
(wave 4) included.

## An operation that takes a stream's lock refuses a stream without one

**Rule.** An operation bound on `FileRef` that takes the stream's lock with `flockfile`, or
tries it with `ftrylockfile`, first checks that the stream has a lock (`_lock` non-null), as
`FileRef::print`'s `has_lock` does (`src/libc/stdio_common/fprintf.rs`). If it has none,
the operation returns `io::ErrorKind::Unsupported`, writes nothing and leaves `errno` alone.
glibc's lock routines dereference `_lock` without checking it. glibc's own stdio routines
skip the lock on such a stream, and Rust cannot do the same without breaking `FileRef`'s
`Sync`. Every stream a program opens has a lock, as do the standard streams. The stream
glibc hands the handler of a `register_printf_specifier` conversion has none, and it
honours `FileRef::from_ptr`'s contract.

**Evidence.** wave-3/batch-1/run-1 (`stdio-common/fprintf.c`'s module pass): finding
`contracts/ffi_import/print-locks-a-stream-without-a-lock`, whose reproducer dies with a
SEGV in `flockfile` under ASan on `53dd776`. The unit test
`a_stream_without_a_lock_is_refused` uses the `in_printf_handler` fixture
(`tests/fixtures/stream_unit.rs`), which gives later operations that stream to test against.

**Scope.** Every later operation on `FileRef` that locks the stream, `stderr`'s (wave 4)
included, as long as `FileRef::from_ptr`'s contract admits every open stream.

## A test of a failing stream reads what it holds only after its last write

**Rule.** The stream fixture's `contents()` flushes the stream with `fflush` before it reads
it. On a stream whose writes fail, `/dev/full` or full `fmemopen` storage, that flush fails
too: it sets the stream's error indicator and discards its buffer. A test that reads
`contents()` between two writes therefore changes what the second write meets, whether it
fails and whether its failure shows in the indicator. Such a test reads `contents()` once,
after its last write, as `tests/equiv_pub/iofwrite.rs`' `compare_flushing` does with
`Flush::AtEnd`. A stream whose writes succeed may be read after each write.

**Evidence.** wave-5/batch-0/run-1 (`libio/iofwrite.c`):
`a_failed_line_buffered_flush_fails_as_fprintf_does`, read after each write, failed at its
second write. The flush after `"status: "` had set the indicator, which hid the failure of
the `"failed\n"` that followed. Read after the last write, it passes, and it fails when the
binding ignores the indicator.

**Scope.** Every test that writes a failing stream through the fixture, any later stream
operation's, such as `fputs`' or `fflush`'s, included.
