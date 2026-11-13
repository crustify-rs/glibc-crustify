# reviewing: a reproducer repeats the affected workspace's `[patch]` tables

Status: `open`. Filed by glibc-crustify `wave-4/batch-1/run-1` (2026-11-13), under the
default `repair` autonomy. `reviewing` is the reviewer's to evolve. This run proposes the
change rather than making it: no Crustify working branch of this campaign exists, and the
checkout its runs read is `libgit2-crustify/devel`, the parent campaign's own.

## Signal

A rule worked out by hand that the skill should have stated (`evolution` §2, a rule an
agent works around). `reviewing` §5 makes each reproducer "a standalone Cargo package: its
`Cargo.toml` declares an empty `[workspace]` table and depends on the affected crate
through a relative `path`". The workspace pins ffibox by Git and patches it to the local
checkout, as `toolchain` and the workspace's `Cargo.toml` do. Cargo applies `[patch]` only
from the root of the workspace being built. So a reproducer written to the letter of §5
builds ffibox at its Git pin, from Cargo's cache or from GitHub, not from the checkout the
affected workspace builds against. Here that is `40621a2` rather than `9c22a73`: checked
offline, the package builds with the pinned revision. A reproducer is evidence about the
affected commit, so it then proves something about another build, and a finding in, or
fixed by, an ffibox change between the two would not reproduce. The reproducer of
`stderr-lends-a-static-stream-the-program-may-close` repeats the workspace's
`[patch."https://github.com/crustify-rs/ffibox"]` table for that reason.

## Destination

The spec: `reviewing` §5, Reproducers.

## Change

After "…depends on the affected crate through a relative `path`", add: ", and repeats the
affected workspace's `[patch]` tables, which apply only from the root of the workspace
being built, so that it builds against the dependencies the affected commit builds with".

## Validation

A package over the fixed tree with no `[patch]` table, checked `--offline`, builds
`ffibox v0.1.0 (…?rev=40621a29…)`. The reproducer, which repeats the table, builds the local
checkout (`9c22a73`), as the workspace does, and reproduces the finding.

## Review

Not reviewed.
