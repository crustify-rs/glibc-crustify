# wave-5/batch-1/run-1 (review)

- Repository: your batch's worktree, `/target/glibc-crustify/crustify/.worktrees/modules/libc/libio/iofwrite.c/wave-5/batch-1`, on `crustify/batches/wave-5/batch-1`.
- Worklist: `crustify/artifacts/modules/libc/libio/iofwrite.c/wave-5/batch-1/run-1/task.json` in it.
- Batch: `wave-5/batch-1` of the module `libc/libio/iofwrite.c` (`modules.json`'s `foundational: false` record of that name).
- Land on the unchecked-out wave branch `crustify/waves/wave-5`. Name it `refs/heads/crustify/waves/wave-5` in every git command.

## Instructions

- This is `libio/iofwrite.c`'s module pass and its only review: its run landed one batch,
  `wave-5/batch-0` (`d796900`), which bound `fwrite` as `FileRef::write_all(&self, &[u8]) ->
  io::Result<()>`, a byte-exact write under the stream's lock, and moved the lock into
  `genops.rs` as a crate-private hold released on drop, which `print` now uses too. Review it
  thoroughly: the parent's `opt.c` export writes through it as soon as it is integrated, to
  streams the C that remains shares. Points to judge: a short write is an error, not a
  count, against what the planner expected; the line-buffered flush failure read from the
  stream's error indicator before and after the call, and the one case it cannot see (an
  indicator already set), which the doc states; the shared lock hold, on every path of both
  callers; and `crustify/evolution/sticky-error-state`, a `binding-symbols` change on an
  evolution branch: judge the code against the skills as they are on your revision.
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
