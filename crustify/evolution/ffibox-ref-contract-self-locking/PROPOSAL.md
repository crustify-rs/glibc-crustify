# ffibox: the shared seam's contract for a C object that locks itself

Status: `open`. Filed by glibc-crustify `wave-2/batch-0/run-1` (2026-11-13), under the
default `repair` autonomy: a documentation defect in ffibox that does not block the batch,
proposed.

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

## Validation

A documentation change. No reproducer: no current consumer's soundness changes.

## Review

Not reviewed.
