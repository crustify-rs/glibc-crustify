# The coverage scripts miss a test item whose attribute spans lines, and run outside their copy

Status: `open`. Filed by glibc-crustify `wave-5/batch-1/run-1` (2026-11-13), under the
`evolve` autonomy the settings give. The destination is a campaign artifact another campaign
owns, `libgit2-crustify`'s `crustify/scripts/coverage/` (`mark.py`, `baseline.sh`), which
this campaign measures its Rust coverage with (`crustify/LESSONS.md`, the gates lesson). So
this is a proposal. No change is made.

## Signal

A tool that reports what the source contradicts (`evolution` §2).

## Evidence

1. `mark.py` puts `#[coverage(off)]` on a `#[cfg(test)]` or `#[test]` item only when every
   line between that attribute and the item starts with `#[` or `//`. rustfmt splits a long
   attribute over lines, as `#[expect(\n    unsafe_code,\n    reason = "…"\n)]`, and then
   the item is not marked and its test code counts as code to cover. In this run,
   `src/libc/libio/genops.rs`' `unit_priv` module took such an attribute. The tree then
   measured `genops.rs` at 41 lines rather than 23, and the total at 162 rather than 144, all
   "covered". The run shortened the attribute to one line, and the figures went back to 23
   and 144. Nothing failed, so a later run would not notice.
2. `baseline.sh` runs under `set -uo pipefail` with an `ERR` trap that only prints. When its
   copy holds no package, as here when a copy step left it empty, the package lookup fails,
   `mark.py` fails, and the script goes on: `cd "$(dirname "$root")/.."` resolves against
   the caller's directory, and `cargo test` runs, instrumented, in whatever workspace
   encloses it. Here that was the run's own worktree, which got stray `.profraw` files and a
   measurement of the unmarked tree, reported as if it were the copy's.

## Destination

`libgit2-crustify`'s `crustify/scripts/coverage/`, or the coverage tooling `testing-builds`
names once one ships with Crustify.

## Change

- `mark.py`: skip whole attributes, bracket-balanced, between the test attribute and the
  item, not lines that start with `#[`; and fail when a `#[cfg(test)]` or `#[test]` is
  followed by no item it can mark.
- `baseline.sh`: `set -e` (or `exit` in the `ERR` trap), so that a failed lookup or marking
  ends the run.

## Validation

None: not made. The evidence above reruns by giving any `#[cfg(test)] mod` a multi-line
`#[expect]` in a scratch copy and comparing `llvm-cov report`'s line count for its file.

## Review

None yet.
