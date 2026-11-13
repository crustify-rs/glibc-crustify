# Wave 3's write through a shared stream handle takes the stream's lock itself

Status: `open`. Filed by glibc-crustify `wave-2/batch-0/run-1` (2026-11-13), under the
default `repair` autonomy (`crustify/CRUSTIFY.md` gives no `evolution` answer): a correction
to a campaign artifact the planner owns, proposed.

## Signal

A plan that would ask a later batch for a binding its analysis cannot make sound
(`evolution` §2, "plan corrections"). Planner run 1 (`25b3fe2`) plans `fprintf` (wave 3) as
`fprintf(stream, "%s", message)` through `_IO_FILE`'s shared handle, on the ground that
"glibc locks every stream operation, which the binder confirms before a shared handle
writes". Wave 2's analysis does not confirm it for every stream:

- `vfprintf` (`c/stdio-common/vfprintf-internal.c:1540-1563`) locks with the
  `_IO_flockfile` macro, which `c/libio/libio.h:282` makes a no-op when the stream carries
  `_IO_USER_LOCK`. `__fsetlocking(fp, FSETLOCKING_BYCALLER)` sets that flag
  (`c/libio/__fsetlocking.c:31-32`). Native code, not Rust, sets it.
- `flockfile` (`c/stdio-common/flockfile.c`) takes the stream's lock (`_IO_lock_lock`) with
  no flag check, and the lock is recursive (`sysdeps/nptl/stdio-lock.h`: owner and count).
  So `fprintf` called inside `flockfile`/`funlockfile` is serialized whatever the stream's
  locking mode.

`File`'s `Sync` (`src/libc/libio/genops.rs`, wave 2) therefore rests on this rule: every
operation bound on `FileRef` holds the stream's lock, taken with `flockfile`, for the
whole of its access. A `Sync` that relied on the stream's locking mode would rest on data
the native code can change, which `binding`'s Safe boundary section forbids.

## Destination

A campaign artifact: `modules.json` and `schedule.json`, the planner's. `flockfile` and
`funlockfile` (`stdio-common/flockfile.c`, `stdio-common/funlockfile.c`, declared in
`libio/stdio.h`) join the plan ahead of wave 3, or in its batch, since wave 3's `fprintf`
binding needs them. They could also be bound privately inside `fprintf`'s module, as the
lock its one call takes, with no public API of their own.

## Change

Planner run 2 adds the two functions as items wave 3 reaches. The fixed-arity form stays
as planned: `flockfile(stream)`, `fprintf(stream, "%s", message)`, `funlockfile(stream)`,
with the count returned, or `errno`'s `io::Error` on a negative return.

## Validation

None yet. Wave 3's binder confirms it from `fprintf`'s analysis.

## Review

Not reviewed.
