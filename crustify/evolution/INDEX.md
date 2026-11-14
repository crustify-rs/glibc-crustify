# Index of proposals

One row per proposal of this campaign, as the `evolution` skill's Proposals section describes:
its slug, its signal in a line, its status, the runs its proposal cites as having met the
signal, and the row of the record, Crustify's `EVOLUTION.md`, it matches, if any. Written by
the assistant alone, as each batch's proposals land.

| slug | signal | status | met by | record |
|---|---|---|---|---|
| `bindgen-opaque-allowlist-kind` | The raw bindings' allowlist can keep a type opaque that its routines take only by pointer | `open` | — | — |
| `coverage-marking-misses-multiline-attributes` | The coverage scripts miss a test item whose attribute spans lines, and run outside their copy | `open` | — | — |
| `disclose-scope-for-the-campaigns-own-crate` | A disclosure goes to a project outside the campaign, never to the campaign's own crate | `open` | — | — |
| `doc-alias-equal-to-name` | conventions: no `doc(alias)` where the Rust item's name is the anchor's path | `open` | — | — |
| `errno-failure-without-errno` | A failure that sets no `errno` is not reported as the stale one | `open` | — | — |
| `ffibox-ref-contract-self-locking` | ffibox: the shared seam's contract for a C object that locks itself | `open` | — | — |
| `global-the-program-may-set` | binding-symbols: a global the C contract lets a program set is mutable, whatever its known users do | `open` | — | — |
| `plan-from-landings-before-a-schedule` | A side campaign's first plan: what landed before a schedule, and how far its decomposition reaches | `open` | — | — |
| `reproducer-carries-the-workspace-patches` | reviewing: a reproducer repeats the affected workspace's `[patch]` tables | `superseded` | — | — |
| `sticky-error-state` | A failure read from sticky error state is its change, read before and after the call | `landed` | — | — |
| `stream-lock-for-shared-writes` | Wave 3's write through a shared stream handle takes the stream's lock itself | `landed` | — | — |
| `typedef-named-as-its-type` | A typedef whose Rust name is its type's | `open` | — | — |
