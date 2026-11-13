# Wave 3's write through a shared stream handle takes the stream's lock itself

Status: `landed`, as the alternative its Destination names, in glibc-crustify
`wave-3/batch-0/run-1`'s landing (2026-11-13): `flockfile` and `funlockfile` are bound
privately inside `fprintf`'s module, as the lock its one call takes, and the plan does not
change. Filed by glibc-crustify `wave-2/batch-0/run-1` (2026-11-13), under the default
`repair` autonomy (`crustify/CRUSTIFY.md` gives no `evolution` answer): a correction to a
campaign artifact the planner owns, proposed.

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

Wave 3's binder confirmed it from `fprintf`'s analysis, and the assistant's task message
settled the destination: the two routines are raw bindings in `fprintf.rs`'s `ffi_import`,
allowlisted, with no safe API. `FileRef::print` holds the lock from before `fprintf` to
after `errno` is read. Two unit tests show that the lock is what serializes the writes
(`tests/unit_pub/fprintf.rs`). `writes_from_threads_never_interleave` shares one handle
among 8 threads on a `FSETLOCKING_BYCALLER` stream. It passes, and with the binding's
`flockfile` and `funlockfile` removed it fails three runs in three with interleaved lines
(an `open_memstream` stream aborts in glibc instead).
`print_waits_for_the_lock_on_a_stream_locked_by_its_caller` passes, and with them removed
it sees the waiting thread's text written first. No consumer of the parent campaigns needs
the lock routines as API of their own: `cli_opt_status_fprint`, `spec_name_fprint` and
`llhttp__debug` each make one `fprintf` per message.

## Review

Not reviewed.
