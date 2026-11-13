# A typedef whose Rust name is its type's

Status: `open`. Filed by glibc-crustify `wave-2/batch-0/run-1` (2026-11-13), under the
default `repair` autonomy: a gap in the conventions, proposed.

## Signal

A rule an agent had to work around (`evolution` §2). The conventions' Types section makes
a typedef listed in an item's `typedefs` "a `pub type` alias of the type's Rust form", and
its Anchors section annotates "the alias that stands in for it". Wave 2's item is
`_IO_FILE` with the typedef `FILE`. With `_IO_`, libio's prefix, dropped (Naming), both
names are `File` in Rust casing. An alias of `File` named `File` cannot exist. No lint
rejects `pub type FILE = File` (checked with the workspace's clippy), but it would keep C's
spelling on the safe surface, against the Naming section's casing. The run put both anchors on the one type, `define_ctype!`'s `File`:
``/// Wraps `st::_IO_FILE` `` and ``/// Wraps `tdf::FILE` ``, with both aliases
(`src/libc/libio/genops.rs`).

The case recurs wherever C tags a struct and typedefs it under the same name, such as
`typedef struct foo foo`, or under a prefixed or underscored tag, such as
`typedef struct _GList GList` or `typedef struct git_repository git_repository`.

## Destination

The spec: `conventions` §1.2 (Types) and §3.1 (What is annotated).

## Change

§1.2, after the alias sentence: "Where the typedef's Rust name is its type's own, there is
no alias: the type's Rust form stands in for both, and carries both annotations." §3.1's
typedef entry: "a typedef: the alias that stands in for it, or the type it names, where its
Rust name is that type's."

## Validation

The run's own binding is the case. ffimap's and any anchor scan's attribution of
`tdf::FILE` to `File` would show whether tools read two anchors on one item.

## Review

Not reviewed.
