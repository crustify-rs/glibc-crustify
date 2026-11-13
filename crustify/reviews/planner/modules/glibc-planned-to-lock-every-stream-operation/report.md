# Planner run 1 relied on glibc locking every stream operation

Filed by `planner/run-2` (2026-11-13), as `reviewing`'s Plan corrections section asks of a
planner run that corrects an earlier run's decision.

## What the earlier plan decided

Planner run 1 (`25b3fe2`) planned `_IO_FILE` (wave 2, `libio/genops.c`) as a shared,
borrowed handle, and `fprintf` (wave 3) as a write through it. Its commit message gave the
ground for the handle's sharing:

> glibc locks every stream operation, which the binder confirms before a shared handle
> writes.

No field of `modules.json` or `schedule.json` records it. Waves 2 and 3 are right as items.
The premise was wrong.

## The evidence against it

- glibc's locking stdio routines lock through `_IO_flockfile`, which is a no-op on a stream
  carrying `_IO_USER_LOCK` (`c/libio/libio.h:282-285`). `__fsetlocking (fp,
  FSETLOCKING_BYCALLER)` sets that flag (`c/libio/__fsetlocking.c:30-32`). The walkers of
  every open stream skip the lock the same way: `fflush (NULL)` (`_IO_flush_all`,
  `c/libio/genops.c:698`), `_IO_flush_all_linebuffered` (`:734`) and `exit`'s
  `_IO_unbuffer_all` (`:785`). The `_unlocked` routines never lock. `_IO_acquire_lock`,
  which `fwrite` and `fputs` take (`c/sysdeps/nptl/stdio-lock.h:98-103`), is that macro
  too.
- A stream can have no lock at all. The stream glibc hands a `register_printf_specifier`
  handler has a NULL `_lock`, which `flockfile` dereferences unchecked.
- Wave 2's module pass filed the first point against `File`'s contract
  (`crustify/reviews/modules/libc/libio/genops.c/wave-2/batch-1/contracts/ffi_import/file-ref-contract-trusts-glibc-locking/report.md`,
  fixed in `6e892ea`). `crustify/evolution/stream-lock-for-shared-writes` (landed in
  wave 3's batch, `53dd776`) made every operation bound on `FileRef` take the lock itself,
  with `flockfile`, which locks whatever the stream's locking mode. Wave 3's module pass
  found the second point
  (`crustify/reviews/modules/libc/stdio-common/fprintf.c/wave-3/batch-1/contracts/ffi_import/print-locks-a-stream-without-a-lock/report.md`,
  a SEGV in `flockfile` under ASan on `53dd776`, fixed in `238650a` by `has_lock`). The
  campaign's lesson *An operation that takes a stream's lock refuses a stream without one*
  (`crustify/LESSONS.md`) records the rule for later operations.

Rerun: `sed -n 275,290p c/libio/libio.h`; `sed -n 25,35p c/libio/__fsetlocking.c`;
`sed -n 95,108p c/sysdeps/nptl/stdio-lock.h`; `git show 6e892ea 238650a --stat`.

## The edit and what it moves

The premise is withdrawn. In its place, the plan states the rule the waves since have
established, for every operation it plans on `FileRef`. Each such operation holds the
stream's lock with `flockfile` for the whole of its access, `funlockfile` releasing it. It
does not rely on the lock the C routine takes inside, which skips a `FSETLOCKING_BYCALLER`
stream. It refuses a stream without a lock with `io::ErrorKind::Unsupported`, writing
nothing and leaving `errno` alone.

No artifact entry changes, and no wave is re-planned: waves 2 to 4 are `done`. Wave 5,
appended by this run (`libio/iofwrite.c`, `fwrite`), is the first planned under the rule:
`fwrite`'s own `_IO_acquire_lock` skips a `FSETLOCKING_BYCALLER` stream
(`c/libio/iofwrite.c:37`), so its binding takes `flockfile` around it, as `FileRef::print`
does.

## Questions for the user

None.
