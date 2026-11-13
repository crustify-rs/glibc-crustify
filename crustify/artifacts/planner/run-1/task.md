# planner/run-1 (glibc-crustify, the libc side campaign)

- Repository: `/target/glibc-crustify`, on `crustify/planner/run-1`, forked from
  `crustify/devel`.
- Task: `crustify/artifacts/planner/run-1/task.json`.

## Instructions

- This is an advance of a side campaign, as the `dependencies` skill's Side campaigns
  section describes. Its parent, `/target/libgit2-crustify` (a `port` campaign of libgit2),
  is about to run waves whose items reach three libc entities through this wrapper, which
  lacks them. Plan the waves that wrap them, and nothing else:
  - `_IO_FILE` (`FILE`, `bits/types/struct_FILE.h` / `stdio.h`): the parent's
    `cli_opt_status_fprint(FILE *file, ...)` and `spec_name_fprint(FILE *file, ...)`
    (`c/src/cli/opt.c`) are handed a `FILE *` by C that stays C for now, and write to it;
  - `fprintf` (`stdio.h`): what those two functions, and `llhttp__debug`
    (`c/deps/llhttp/api.c`, to `stderr`), write with. It is variadic: plan what a safe
    wrapper offers for the parent's uses, formatted text written to a C stream the remaining
    C shares, rather than the variadic itself, if that is what the skills lead to; and
  - `stderr` (`stdio.h`, a global).
  The parent's records of them are `c_entities` of `libc` in
  `/target/libgit2-crustify/crustify/artifacts/dependencies.json`, and its modules'
  `depends_on.external` in its `schedule.json`. Read the parent's uses in its `c/`.
- This repository predates the current layout and has no plan: no `modules.json`, no
  `schedule.json`, no `dependencies.json`, its raw bindings in a crate of their own
  (`libc-sys/`) rather than under `sys/`, and what it wraps (`errno`, `struct tm`) landed
  without a schedule. Emit the plan the scope needs under the current schemas, record what
  already landed as landed rather than scheduling it again, and leave the tree's layout as
  it is: a migration of the layout is not this run's, and is a proposal if you judge it
  needed before the waves can run. `ffmpeg-crustify` is this wrapper's other parent
  (`struct tm`, and `FILE` for `libavformat/dump.c`, not yet wrapped): what you plan for
  `FILE` serves both.
- libc is the system's: `crustify/artifacts/build.json` has no variant, `c/` is glibc 2.39's
  source for reading, and the raw bindings come from the host's headers. wavefront has no
  database over glibc here; say in your final message what you could not query and how you
  decided without it.
- The settings (`crustify/CRUSTIFY.md`) follow the parent's since this branch's base. One
  planner run is asked for.
- Other agents run on this host (five batches of the parent). Cap builds at 16 jobs, keep
  scratch in this repository's `crustify/tmp/`, and end only processes of your own.
- Commit as the settings' `identity` answer gives it, your subject ending with
  `(planner/run-1)`. Commit `task.json` and `task.md` with your run.
