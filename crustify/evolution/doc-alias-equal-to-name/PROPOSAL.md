# conventions: no `doc(alias)` where the Rust item's name is the anchor's path

Status: `superseded` by libgit2-crustify's `doc-alias-equal-to-the-name`, on 2026-11-14: the same
signal, settled there as this proposal asked: the alias is dropped, and `ffimap api` exempts it.

## Signal

A rule the compiler refuses (`evolution` §2, "a rule an agent works around").
`conventions`' Anchors section asks every stand-in for two attributes, the doc-comment
anchor and `#[doc(alias = "<path>")]`. Its Naming section keeps a C name where it is already
idiomatic Rust, as a global or function named in `snake_case` is. Wave 4 binds the global
`stderr` as `pub fn stderr() -> FileRef<'static>` (`src/libc/libio/stdio.rs`), and rustc
rejects the alias as an error, with workspace warnings denied or not:

```text
error: `#[doc(alias = "stderr"]` is the same as the item's name
  --> src/libc/libio/stdio.rs:46:19
```

The alias serves a consumer who searches the documentation for the C name, and the item's
own name already serves that search. The same holds for any C function or global whose
`snake_case` name the Rust keeps, such as wave 3's `fprintf` if its fixed-arity form keeps
the name.

## Destination

The spec: `conventions` §3, Anchors. And a tool: `ffimap api`'s `anchors.without_alias`
(`docs/api.md` §2, at `9aa2ef1`), which lists the item as an anchor without its alias:

```text
"without_alias": [{"anchor": "glb::stderr", "item": "glibc_bindings::libc::libio::stdio::stderr"}]
```

## Change

After "The alias is the bare `<path>`, without the backticks, …", add: "Where the item's
own name is that path, as a C name the Rust keeps, it carries no alias: rustdoc rejects an
alias equal to the item's name, and the name already finds it. Its anchor is unchanged."
`ffimap api` keys items by their anchors, so its measure does not change; its
`without_alias` leaves out an item whose own name is the anchor's last path step, the
same exemption, with a regression test over a fixture crate that holds one.

## Validation

Wave 4's `stderr` with the alias fails to compile with the error above, as
`#[doc(alias = "f")] pub fn f() {}` does under plain `rustc`; without it, the crate builds, lints clean and documents, and the anchor
``/// Wraps `glb::stderr` `` still names the item.

## Review

Not reviewed.
