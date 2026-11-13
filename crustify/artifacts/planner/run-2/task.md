# planner/run-2 (glibc-crustify, the libc side campaign)

- Repository: `/target/glibc-crustify`, on `crustify/planner/run-2`, forked from
  `crustify/devel`.
- Task: `crustify/artifacts/planner/run-2/task.json`.

## Instructions

- A second advance of this side campaign, as the `dependencies` skill's Side campaigns
  section describes. Waves 2 to 4 wrapped `FILE`, `fprintf` and `stderr` and are integrated
  (`1e60130`), and the parent, `/target/libgit2-crustify`, pins them. Its first consumer
  found a gap the wrapper must close before the parent's `src/cli/opt.c` switches from C:
  `FileRef::print` and `write_fmt` take `&str`, but `cli_opt_status_fprint` prints the
  user's own arguments with `%s`, which are bytes, not UTF-8 (an argument such as
  `--caf\xe9`). Its export, which the C that remains calls with a `FILE *`, now replaces
  such bytes with U+FFFD, where C wrote them as they are. The parent's batch filed it as
  `crustify/evolution/fileref-prints-bytes` on its branch `crustify/waves/wave-186`
  (`git -C /target/libgit2-crustify show
  refs/heads/crustify/waves/wave-186:crustify/evolution/fileref-prints-bytes/PROPOSAL.md`),
  and its port is `src/git2/src/cli/opt.rs` there (`OptRef::write_status` and its export).
- Plan what gives the parent a byte-exact write to a C stream it shares with C: the libc
  entity it is reached through (`fwrite`, `fputs`, or `fprintf`'s `%s` over a C string, which
  is already bound), its module, and one wave appended to the schedule, as
  `schedule-schema`'s Re-planning section allows for a frozen module a consumer needs more
  of. Plan nothing else. Whatever it is must hold the stream's lock as `print` does
  (`flockfile`, refusing a stream without a lock), and serve an `io::Write` implementation
  if the skills lead to one, since the parent's native API writes to any `io::Write`.
- Correct what run 1 decided that the waves since have shown wrong, as plan corrections:
  `stderr` is not a safe `'static` handle (its module pass made it an `unsafe fn`), and
  glibc does not lock every stream operation. Apply or decline the open proposals that name
  the plan (`crustify/evolution/stream-lock-for-shared-writes` is settled).
- The settings lack the parent's `evolution` and `ship` answers; the assistant adds them
  after your run. One planner run is asked for.
- Several agents of the parent run on this host. Cap builds at 16 jobs, keep scratch in
  this repository's `crustify/tmp/`, and end only processes of your own.
- Commit as the settings' `identity` answer gives it, your subject ending with
  `(planner/run-2)`. Commit `task.json` and `task.md` with your run.
