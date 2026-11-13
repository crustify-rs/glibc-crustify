# `unit_pub`'s `fprintf` tests call the raw bindings themselves, outside the stream fixture

- **item**: `fn::fprintf`'s public-interface unit tests, `tests/unit_pub/fprintf.rs`, and
  the suite root, `tests/unit_pub/main.rs`
- **affected commit**: `53dd776` (wave 3, batch 0)
- **objective**: `conventions`. No reproducer is needed: this is a placement deviation
  with no behaviour to prove.

## What is wrong

`testing` defines unit tests as tests that check the Rust "by passing Rust assertions,
without the C". It allows one documented unsafe fixture per type that has no safe
constructor yet. The conventions' Testing section lets two suites take `unsafe_code` back
at their `main.rs`: those whose tests call the raw bindings, `tests/ub_unsafe` and
`tests/equiv_pub`. `unit_pub` is not one of them.

`tests/unit_pub/main.rs` took `unsafe_code` back for the whole suite with a crate-level
`#![expect]`. Its reason names only the stream fixture. Two of the tests also called the C
directly:

- `a_stream_not_open_for_writing_fails_with_ebadf` read the error indicator with
  `ffi::ferror`;
- `print_waits_for_the_lock_on_a_stream_locked_by_its_caller` took and released the lock
  with `ffi::flockfile` and `ffi::funlockfile`.

So the expectation covered unsafe code the fixture does not hold. Nothing kept further
raw calls out of the tests.

## Fix

The tests now reach the C only through fixtures, and the suite denies unsafe code
everywhere else.

- `Stream::has_error` (`tests/fixtures/stream.rs`) reads `ferror`. `equiv_pub`, which had
  its own copy of the same helper, uses it too.
- `Stream::hold_lock` returns a `LockHold`, which takes the lock with `flockfile` and
  releases it with `funlockfile` when dropped, on the thread that took it, since it
  borrows the non-`Sync` fixture. It sits in `tests/fixtures/stream_unit.rs`, a fixture
  only `unit_pub` includes, beside the new `in_printf_handler` (see
  `contracts/ffi_import/print-locks-a-stream-without-a-lock`). Shared with `equiv_pub`, it
  would be dead code there.
- `tests/unit_pub/main.rs` now has `#![deny(unsafe_code)]` at its root. Each fixture
  module takes the lint back with its own `#[expect(unsafe_code, reason = …)]`.

The two tests assert what they asserted before, and pass plain and under ASan and TSan.
`print_waits_for_the_lock_on_a_stream_locked_by_its_caller` drops the hold where it
called `funlockfile` before.
