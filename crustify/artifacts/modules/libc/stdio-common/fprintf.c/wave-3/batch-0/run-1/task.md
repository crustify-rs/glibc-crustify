# wave-3/batch-0/run-1

- Repository: your batch's worktree, `/target/glibc-crustify/crustify/.worktrees/modules/libc/stdio-common/fprintf.c/wave-3/batch-0`, on `crustify/batches/wave-3/batch-0`.
- Worklist: `crustify/artifacts/modules/libc/stdio-common/fprintf.c/wave-3/batch-0/run-1/task.json` in it.
- Batch: `wave-3/batch-0` of the module `libc/stdio-common/fprintf.c` (`modules.json`'s `foundational: false` record of that name).
- Land on the unchecked-out wave branch `crustify/waves/wave-3`. Name it `refs/heads/crustify/waves/wave-3` in every git command.

## Instructions

- Your worklist is wave 3: `fprintf`, `wrap`, in `src/libc/stdio_common/fprintf.rs`
  (scaffolded). What the wrapper offers is a fixed-arity safe write: the caller formats the
  text in Rust and you write it to the C stream with `fprintf(stream, "%s", text)`, so that it
  lands in the buffer the remaining C writes to. `libio/genops.c` is integrated (`5dfe87e`):
  `File`, `FileRef<'a>` and `FileMut<'a>` in `src/libc/libio/genops.rs`.
- Hold the stream's lock around the call: `flockfile`, then `fprintf`, then `funlockfile`,
  released on every path. glibc's `fprintf` skips its own lock on a stream C set to
  `FSETLOCKING_BYCALLER`, and `FileRef`'s `Send`/`Sync` rests on every operation bound on it
  holding that lock (the module pass's corrected contract in `genops.rs`, and
  `crustify/evolution/stream-lock-for-shared-writes`). The two lock routines are not
  scheduled: they are part of your binding's implementation, called through their raw
  bindings inside its import region and added to the allowlist, not safe API of their own.
  This settles that proposal for this wave; say in your final message if you find a consumer
  that needs them as API.
- `wave-4/batch-0` binds `stderr` beside you, in another module's file and on its own wave
  branch; its tests may write to `stderr` through your wrapper only once both are
  integrated. Test yours on a stream your tests open and close through raw bindings.
- The parent's uses: `cli_opt_status_fprint`, `spec_name_fprint` (`c/src/cli/opt.c`) and
  `llhttp__debug` (`c/deps/llhttp/api.c`) in `/target/libgit2-crustify`. An `io::Write` or
  `fmt::Write` adapter over the handle, if the skills lead to one, serves `write!` there.
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
