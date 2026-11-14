# binding-symbols: a global the C contract lets a program set is mutable, whatever its known users do

Status: `open`: the change is crustify `30fa33f` on `libgit2-crustify/evolution/global-the-program-may-set`, made on
2026-11-14 at the user's direction after the triage of open proposals, awaiting its review.

Goal: Safety, its soundness measure.

## Signal

A finding a how-to rule should have prevented (`evolution` §2, a rule the how-to skills
lack). Planner run 1 (`25b3fe2`) planned glibc's `stderr` as "a 'static shared handle: no
C of either parent writes it, and its storage (_IO_2_1_stderr_) is static". Wave 4's
binder bound it so, as a safe `stderr() -> FileRef<'static>`. Its doc listed what the
borrow assumes of the whole program: nothing assigns `stderr`, closes its stream or unlocks
it. Its justification was that neither glibc's routines nor the parents' C do. The module
pass found the contract broken through safe code. glibc's manual offers `fclose (stdout);
stdout = fopen (…);` as the way to redirect a standard stream, and a kept handle then
outlives the stream `fclose` freed: a heap-use-after-free under ASan
(`crustify/reviews/modules/libc/libio/stdio.c/wave-4/batch-1/contracts/ffi_import/stderr-lends-a-static-stream-the-program-may-close/`).

The Globals section's split is by "every C access", which both the planner and the binder
read as the accesses in the code the campaign can see. For a wrapper, that is never all of
the program. A library cannot see its consumers' C, and a parent campaign's C is not its
consumers' C either: libgit2 is a library too.

## Destination

The spec: `binding-symbols` §2, Globals.

## Change

After the list of access disciplines, add:

> The accesses that decide are those the C declaration's contract allows, not those the
> campaign's C makes: a global the library documents as one a program may set, such as
> libc's standard streams, `environ` or `optarg`, is otherwise mutable however the code
> the campaign sees uses it, since the crate cannot see every program it is linked into.
> Where it holds a pointer, what it points to may be freed by the C that reassigns it, so
> a safe accessor never lends what it names beyond the call: the accessor is `unsafe`,
> its caller vouching for the lifetime it chooses, or a safe operation reads the global
> and uses what it names within one call, as the C expression does, holding the lock C
> takes there.

## Record

The row for `EVOLUTION.md`, copied in when the change lands:

| change | from | landed as | signal answered | recurrence signal | status |
|---|---|---|---|---|---|
| a global's access discipline follows from what its C declaration's contract allows, so one a program may set is otherwise mutable and a safe accessor never lends what it points to beyond a call | `glibc-crustify`, `global-the-program-may-set` | crustify `30fa33f` | "every C access" was read as the campaign's C: a safe handle to a standard stream outlived the program's `fclose`, a use-after-free from safe code | a `ub` finding on a safe accessor of a global a program outside the campaign's C may set or free | watched |

## Validation

The rerun is the module pass's fix (`wave-4/batch-1`). Under the rule, `stderr` is bound
as `unsafe fn stderr<'a>() -> FileRef<'a>`. The reproducer's safe step then fails to
compile (`E0133`), which `tests/ub_safe/compile_fail/stdio_stderr_is_lent_by_no_safe_code.rs`
keeps. The workspace's tests pass plain and under ASan and TSan, and Miri passes per test
binary.

Plan side: `planner/run-2` (2026-11-13) withdrew run 1's shape for `stderr` as a plan
correction
(`crustify/reviews/planner/modules/stderr-planned-as-a-safe-static-handle/report.md`).
The plan now names the entity alone, and leaves a global's discipline to this section of
`binding-symbols`. `decomposition` states no access discipline, so the proposal asks
nothing of it.

## Review

Not reviewed.
