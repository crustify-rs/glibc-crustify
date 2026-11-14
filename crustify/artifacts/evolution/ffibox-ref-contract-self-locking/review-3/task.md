This run is not a review batch: it reviews one change to ffibox, a tool, as Crustify's
`evolution` skill's Review section describes, and changes, commits and lands nothing. Its
task is `/target/glibc-crustify/crustify/artifacts/evolution/ffibox-ref-contract-self-locking/review-3/task.json`.
You run under ffibox's `libgit2-crustify/devel` (`9b9047f`), this checkout, `/crustify-rs/ffibox`.

This is a third, narrow review. Review-2 rejected `eefc57a` with one required fix: `CCell`'s
rule on `Ref` did not carry the limit to a type Rust never holds by value, so a routine that
takes its pointer otherwise than through `as_ptr` was still allowed and is undefined
behaviour under Miri. `568bcb5` adds the limit to that rule, for any path, and rewrites the commit
message; nothing else changed. Review-2's verdict is in the proposal,
`/target/glibc-crustify/crustify/evolution/ffibox-ref-contract-self-locking/PROPOSAL.md`. The change: `git -C /crustify-rs/ffibox diff 9b9047f 568bcb5`;
from what review-2 read: `git -C /crustify-rs/ffibox diff eefc57a 568bcb5`. The documentation
builds without a warning and the 58 doctests pass.

Check only that the fix is made as review-2 asked, that its reproducer's case is now outside
what `CCell`'s rule allows, that nothing else changed, and that the row matches. Do not
repeat the earlier reviews' checks, and do not raise again what they listed as not required.
Verdict, approved or rejected, in a few lines, in your final message. Keep any scratch under
`/target/libgit2-crustify/crustify/tmp/`, and remove it from outside it before you end.
