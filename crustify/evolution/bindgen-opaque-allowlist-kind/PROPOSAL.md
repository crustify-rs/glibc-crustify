# The raw bindings' allowlist can keep a type opaque that its routines take only by pointer

Status: `open`. Filed by glibc-crustify `wave-3/batch-1/run-1` (2026-11-13), under the
default `repair` autonomy (`crustify/CRUSTIFY.md` gives no `evolution` answer). It is an
improvement to the spec and to the `-sys` build script it specifies, both owned by other
roles, so it is proposed.

## Signal

A rule worked around (`evolution` §2). The module pass's regression test needs
`register_printf_specifier` (`<printf.h>`). Allowlisting it, as wave 3's batch did for the
stream routines its tests call, pulls in `struct printf_info` through the handler types,
with its bit-fields. Regenerating `libc-sys/src/bindings.rs` grew it by 1295 lines,
almost all of them bindgen's `__BindgenBitfieldUnit` and `printf_info`'s accessors. The
test passes the struct on only by pointer. The run reverted the change and declared the
routine in its fixture, `tests/fixtures/stream_unit.rs`, with `*const c_void` for the
struct. That keeps a raw declaration outside the raw bindings, which the conventions do
not provide for.

`bindgen` §2 (allowlists) and the build script's `allowlist.txt` take three kinds,
`function`, `type` and `var` (`libc-sys/build.rs`). They cannot ask for bindgen's
`opaque_type`, which would generate `printf_info` as a sized blob of a few lines.

## Destination

- The spec: `bindgen`'s allowlist rules.
- The `-sys` build script it specifies: an `opaque <regex>` kind in `allowlist.txt`,
  mapped to `Builder::opaque_type`.

## Change

`allowlist.txt` gains `opaque <regex>`: a type the allowlisted items reach that the
bindings hold only by pointer, or whose layout no binding reads, is generated opaque.
`bindgen` states it beside the other kinds, with the rule that a type a safe wrapper
projects is never opaque.

## Validation

Not run. With the kind, `function register_printf_specifier` and `opaque printf_info`
should regenerate the bindings with a handful of lines rather than 1295. The fixture's
declaration would then move to the raw bindings.

## Review

Not reviewed.
