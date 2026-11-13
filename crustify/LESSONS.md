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
  `RUSTFLAGS="-Zsanitizer=address -Cforce-frame-pointers=yes"`, and `RUSTDOCFLAGS` the
  same for the doctests, run with `--doc` on their own, which otherwise fail to link
  (`undefined symbol: __asan_init`); TSan: the same with `-Zsanitizer=thread` and
  `-Zbuild-std`. They instrument the Rust alone, since the
  system's glibc is not instrumented, so TSan sees no race inside glibc;
- coverage: Rust only, measured as the parent's `crustify/scripts/coverage/baseline.sh`
  does, over a scratch copy without `c/`. Its `report.py` rejects the scope, since
  `llvm-cov` prints this tree's file names relative to `src/`, so the `TOTAL` line is read
  from `llvm-cov report` itself. The compile-fail suite fails in the marked copy, which
  only a nightly builds, and it covers nothing.
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
