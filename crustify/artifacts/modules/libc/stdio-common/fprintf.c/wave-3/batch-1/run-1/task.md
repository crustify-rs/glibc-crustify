# wave-3/batch-1/run-1 (review)

- Repository: your batch's worktree, `/target/glibc-crustify/crustify/.worktrees/modules/libc/stdio-common/fprintf.c/wave-3/batch-1`, on `crustify/batches/wave-3/batch-1`.
- Worklist: `crustify/artifacts/modules/libc/stdio-common/fprintf.c/wave-3/batch-1/run-1/task.json` in it.
- Batch: `wave-3/batch-1` of the module `libc/stdio-common/fprintf.c` (`modules.json`'s `foundational: false` record of that name).
- Land on the unchecked-out wave branch `crustify/waves/wave-3`. Name it `refs/heads/crustify/waves/wave-3` in every git command.

## Instructions

- This is `stdio-common/fprintf.c`'s module pass and its only review: its run landed one
  batch, `wave-3/batch-0` (`53dd776`), which bound `fprintf` as `FileRef::print(&self, text:
  &str) -> io::Result<usize>` and `FileRef::write_fmt`, writing Rust-formatted text with
  `fprintf(stream, "%s", text)` between `flockfile` and `funlockfile`, the two lock routines
  called privately inside its import region. Review it thoroughly: the parent's `opt.c` and
  `llhttp/api.c` ports write through it as soon as it is integrated, to streams the C that
  remains shares. Points to judge: the lock is released on every path; the `errno` handling
  for a failure that sets none (`crustify/evolution/errno-failure-without-errno`); the NUL
  refusal; and `write_fmt` as an inherent method without an anchor of its own.
- The batch reported that the safe crate does not package (its `ffibox` dependency is a Git
  dependency with no version), as on the base: not this module's.
- No `objectives` are given, so every objective that applies runs, `fix` included. Your gates
  are the wave's and the module's end, without BSan: the workspace's tests plain and under
  ASan/UBSan and TSan, and Miri per test binary. The dashboard's usage cells are the
  assistant's.
- This is the libc side campaign of `/target/libgit2-crustify` (a `port` campaign of
  libgit2), advanced because the parent's next waves need `FILE`, `fprintf` and `stderr`
  through this wrapper: `cli_opt_status_fprint(FILE *file, ...)` and `spec_name_fprint`
  (`c/src/cli/opt.c` there) write formatted text to a `FILE *` that C which stays C hands
  them, and `llhttp__debug` (`c/deps/llhttp/api.c`) writes to `stderr`. Planner run 1
  (`25b3fe2`, its commit message and `modules.json`) says what the wrapper offers for those
  uses; `ffmpeg-crustify`'s `libavformat/dump.c` will use the same `FILE` wrapper.
- libc is the system's: there is no build variant (`crustify/artifacts/build.json`), `c/` is
  glibc 2.39's source for reading, and the raw bindings are the `libc-sys/` crate, generated
  from the host's headers. The tree predates the current layout (`libc-sys/` rather than
  `sys/`, crate `glibc-bindings`, older anchors in what landed before): work in the tree as
  it is, follow the current conventions in what you write, and leave the migration alone.
- Several agents of the parent campaign run on this host. Cap your builds and tests at 16
  jobs (`CARGO_BUILD_JOBS=16`, `RUST_TEST_THREADS=16`). Keep every scratch file, log and
  scratch build in your worktree's `crustify/tmp/`, and delete scratch builds you no longer
  need before you end: the disk is 92% full. End only processes of your own worktree.
- Your run dir in the worktree holds `task.json` and `task.md`; commit them with your
  landing. The process records of your run are kept by the assistant in the base checkout.
- BSan is no regression gate of yours (the settings' `sanitizers` answer, from 2026-11-13, a
  departure from `testing-gates`' Regressions section the user asked for): never run
  `cargo bsan` over the workspace, a whole test binary or as a batch, wave or module-end
  gate. You may run it on small things: the tests your own work adds or changes, a
  reproducer and the regression test it becomes, each test named and under a `timeout`.
