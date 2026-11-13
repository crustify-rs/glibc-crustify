# `genops.rs`' unit tests hold unsafe code without taking `unsafe_code` back

- **item**: `#[cfg(test)] mod unit_priv` inside `mod ffi_import`, `src/libc/libio/genops.rs`
- **affected commit**: `915f5a4` (wave 2, batch 0)
- **objective**: `conventions`

## What is wrong

`toolchain`'s Unsafe code section asks a region with unsafe code of its own to take
`unsafe_code` back where it is declared, with `#[expect(unsafe_code, reason = …)]`, "and so
does each test suite and unit-test module that holds unsafe code". `unit_priv` holds four
`unsafe` blocks (`FileRef::from_ptr` and `FileMut::from_ptr` on stand-in storage). It
carries no expectation of its own, and relies on `ffi_import`'s, which the region's
`unsafe impl`s already fulfil. If the tests lost their unsafe code, nothing would say that
the module no longer needs it back. If the region lost its own, its expectation would stay
fulfilled only in test builds.

## Fix

`unit_priv` now carries
`#[expect(unsafe_code, reason = "borrows stand-in streams through the seams")]`. The
workspace's `clippy --all-targets -D warnings` passes, so both expectations are fulfilled.
