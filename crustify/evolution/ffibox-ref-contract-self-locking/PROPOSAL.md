# ffibox: the shared seam's contract for a C object that locks itself

Status: `open`: the change is ffibox `2ce020f` on `libgit2-crustify/evolution/ffibox-ref-contract-self-locking`, made on
2026-11-14 at the user's direction after the triage of open proposals, awaiting its review.

Goal: Safety, its soundness measure; ffibox's G9 and G12.

## Signal

Two ffibox documents disagree (`evolution` §2, "skills that contradict each other", here a
tool's docs). `CCell::ref_from_ptr` (`src/traits.rs`, at `9c22a73`), which
`CRefSeam::from_ptr` forwards to, requires that "nothing may write it while the handle
lives except through a `Mut` derived from the same borrow". `docs/ownership.md`'s
shared-and-mutable table puts "routines that lock internally" on `FooRef`. The macro's own
`CCell` safety comment, "`$rf` has no write operation", says the same as the trait.

A C object that serializes its own state, such as glibc's `FILE`, whose every stdio
routine writes its buffer under the stream's recursive lock, cannot meet the trait's
wording. C writes such an object while a `FooRef` lives, on another thread too, and that
is exactly what the table allows. Wave 2 of glibc-crustify binds `FILE` opaque, as
`File`/`FileRef`, and states the contract for a stream on the type
(`src/libc/libio/genops.rs`): the stream stays open for `'a`, and whatever else writes it
does so under its lock, while the handle reads nothing without the lock. That is the
table's meaning, but it is not the trait's text.

## Destination

ffibox: `CCell::ref_from_ptr`'s `# Safety` section, and the `CCell` safety comment
`define_ctype!` emits.

## Change

`ref_from_ptr`'s contract reads: "`ptr` must address a live, initialised `Self::C` (or be
null) that outlives `'a`, and nothing may write it while the handle lives except through a
`Mut` derived from the same borrow, or under a synchronization every operation reachable
through `Self::Ref` takes too (an object that locks itself, whose shared handle reads
nothing without its lock)." The macro's comment says that `$rf` has no write operation of
ffibox's.

Added by the module pass, wave-2/batch-1/run-1 (2026-11-13): the shared seam's pointer
needs the same exception. `CRefSeam::as_ptr` (`src/traits.rs:1038`, at `9c22a73`) is
documented as a "Read-only pointer to the C object, for FFI calls taking `*const C`". The
routines that the table puts on `FooRef` take a `foo *` and write the object under its
lock: `fprintf(FILE *, …)` is one. `FileRef` is the only handle wave 3's `fprintf` binding
can take, and it passes the stream as `stream.as_ptr().cast_mut()`. That pointer carries
the write provenance of the one C handed to `from_ptr`, but the binding still writes through
a pointer ffibox calls read-only. `as_ptr`'s documentation adds: "or for C routines that
write the object under a lock they take themselves, which `Self::Ref`'s contract allows."
The module pass judged `File`'s thread-safety claim against these documents
(`crustify/reviews/modules/libc/libio/genops.c/wave-2/batch-1/contracts/ffi_import/file-ref-contract-trusts-glibc-locking/report.md`),
and found the self-locking shape that `docs/ownership.md` gives sound for `FILE`, whose
recursive lock rules out `CGuarded`. Both text changes widen what a producer may claim of a
shared handle. A review should mark them as relaxing ffibox's stated contract to match its
own table, rather than as a new permission.

## Record

The row for `EVOLUTION.md`, copied in when the change lands:

| change | from | landed as | signal answered | recurrence signal | status |
|---|---|---|---|---|---|
| the shared handle's contract allows writes under a synchronization every operation reachable through the handle takes, and `as_ptr` serves C routines that write under their own lock | `glibc-crustify`, `ffibox-ref-contract-self-locking` | ffibox `2ce020f` | three runs put self-locking routines on a shared handle, as `docs/ownership.md` allows, against a trait contract that forbade every write | a `contracts` finding on a shared handle of a self-locking object whose binding the trait's text forbids, or a `ub` finding of the `assumed-locking` pattern on one | watched |

## Validation

A documentation change. No reproducer: no current consumer's soundness changes.
Wave 3 (`wave-3/batch-0/run-1`) now relies on it as the module pass foresaw:
`FileRef::print` passes `stream.as_ptr().cast_mut()` to `flockfile`, `fprintf` and
`funlockfile` (`src/libc/stdio_common/fprintf.rs`), each of which writes the stream under
its lock.

## Review

Not reviewed.
