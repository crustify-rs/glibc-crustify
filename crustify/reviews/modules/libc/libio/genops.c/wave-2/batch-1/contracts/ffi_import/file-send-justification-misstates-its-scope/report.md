# `Send for File`'s justification says it covers only the exclusive handle, which asserts its own

- **item**: `st::_IO_FILE` / `tdf::FILE`, bound as `File`: `unsafe impl Send for File`,
  `src/libc/libio/genops.rs`
- **affected commit**: `915f5a4` (wave 2, batch 0)
- **objective**: `contracts`, of the `ffi_import` region. The `// SAFETY:` justification
  misstates what the assertion covers. The assertion itself is sound, so there is no
  reproducer.

## What is wrong

```rust
// SAFETY: a stream is tied to no thread: glibc opens, writes and closes it from any
// thread, its lock recording an owner only while a call holds it, and no `File` value
// or owner exists in Rust (the field is private and this region builds none), so the
// assertion covers only the exclusive handle, which reaches the stream as a
// `FileRef` does, through its locked operations.
unsafe impl Send for File {}
```

`FileMut`'s thread safety does not come from `File: Send`. `define_ctype!`'s `FileMut`
holds a `FileRef` and a `PhantomData<&mut File>`, and the region asserts
`Send for FileMut<'_>` and `Sync for FileMut<'_>` itself, two lines further down. What
`File: Send` does govern is every ffibox owner and view over a `File`. `docs/ownership.md`
("Thread safety is opt-in for wrapped C types") makes `CBox`, `CVal`, `CVec` and the views
over a `Foo` inherit `Send` from it, and `CGuardedRef` take its `Sync` from it, as
`&Mutex<_>` does. None of them exists over `File` today. But the justification tells a later
region that adds one, such as an owner of `fopen`'s or `tmpfile`'s stream, that this
assertion does not reach it. That region then inherits an assertion whose proof never
considered it.

The assertion holds for those owners too: glibc lets any thread close a stream, and an
owner that sends a stream sends only a pointer to it. Only the justification is wrong.

## Fix

The fix is to the justification only. It now says that no `File` value exists in Rust, so
a `File` crosses threads only behind a pointer. That pointer is an ffibox owner or view
over it, which inherits this assertion and which a later region may add. Its stream is then
closed or reached on whichever thread holds it. The handles assert their own `Send` and
`Sync`.
