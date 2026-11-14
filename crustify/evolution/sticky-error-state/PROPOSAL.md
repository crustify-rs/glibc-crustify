# A failure read from sticky error state is its change, read before and after the call

Status: `open`. Filed by glibc-crustify `wave-5/batch-0/run-1` (2026-11-13), under the
`evolve` autonomy the settings give (the parent `libgit2-crustify`'s answer). The change is
made, on Crustify's `libgit2-crustify/evolution/sticky-error-state` (`57fd7a3`), forked from
`libgit2-crustify/devel` at `cc84106`. It awaits its review, which is another run's.

## Signal

A rule worked out by hand that a skill should have stated (`evolution` §2).
`binding-symbols`' Functions section reads a failure from "a status return, a sentinel
return, or error state the library keeps beside the call, read immediately after it,
before another call into the library can replace it". It says nothing of error state
that a failure sets and that only a caller clears, a sticky flag. Read after the call
alone, such state reports every earlier failure as the call's own.

glibc's `fwrite` (wave 5's item) needs it. When the bytes fit the buffer of a line-buffered
stream and end with a newline, and the flush that newline calls for fails, `fwrite` returns
the full count (`c/libio/iofwrite.c:45-46`, from `_IO_new_file_xsputn`'s `EOF`,
`c/libio/fileops.c:1244-1247`). Only the stream's error indicator, `_IO_ERR_SEEN`, which
`ferror` reads and `clearerr` alone clears, records the failure. glibc's `fprintf`, which
the parent's C calls, fails there, so the binding has to report it.

The task message said to read the failure from the indicator. It did not say that the
indicator may already be set by a failure before the call, which nothing clears.
`FileRef::write_all` (`src/libc/libio/iofwrite.rs`) reads it before and after the call,
under one hold of the stream's lock, and a failure is its move from clear to set. While it
is already set, the binding cannot see that one failure, and its doc says so.

## Evidence

The batch's tree, with the binding's rule replaced by "failed when the count is short or
the indicator is set after the call", fails six tests. Each one is a failure an earlier
write left in the indicator, reported as a later write's:

- `unit_priv::a_full_count_with_no_new_error_is_the_success`;
- `unit_pub` `iofwrite::a_stream_already_in_error_cannot_tell_a_failed_line_buffered_flush`;
  and
- `equiv_pub` `iofwrite::a_stream_not_open_for_writing_fails_alike`,
  `a_full_stream_fails_alike`, `a_failing_device_fails_alike` and
  `a_failed_line_buffered_flush_fails_alike_after_an_earlier_failure`, where C's `fwrite`
  succeeds on the stream the binding called failed.

With the rule as landed, all pass, plain, under ASan, and under TSan for the stream suites.

### From the module pass

glibc-crustify `wave-5/batch-1/run-1`, `libio/iofwrite.c`'s module pass, found a failure the
rule cannot see even with the state clear: one that never sets it. A line-buffered stream's
flush can fail in the seek glibc makes before it writes (`c/libio/fileops.c:441-447`), which
sets `errno` (`ESPIPE`) and not the indicator. `fwrite` then returns the full count, and the
binding, following the rule, reported success where `fprintf` fails. The finding is
`crustify/reviews/modules/libc/libio/iofwrite.c/wave-5/batch-1/equiv/a-failed-line-buffered-flush-that-sets-no-indicator-succeeds`.
Its fix documents both unseen cases on the binding. So the rule's last clause holds only if
it names both: a failure that leaves the state already set, and one that leaves it
unchanged, such as a failure on a path that does not set the flag. For this proposal's
review, the drafted sentence could end "…that such a failure goes unseen, while the state is
already set or on a path of the library that does not set it, is documented on the binding",
the paths found by reading where the library sets the flag.

## Destination

The spec: `skills/rust/binding/symbols/SKILL.md`, Functions, the **Errors** bullet, a
skill the binder role evolves. Stated without the target's names, as it holds for any C
library that keeps a sticky error flag beside its calls, such as stdio's error indicator or
a handle's "last error seen" bit.

## Change

After "…before another call into the library can replace it.":

> Error state a failure sets and only a caller clears, such as a sticky error flag, tells
> the call's failure by its change alone: it is read before the call and after it, with
> nothing else able to change it in between, and the failure is its move from clear to
> set; that such a failure goes unseen while the state is already set is documented on the
> binding.

Its record entry, drafted in `EVOLUTION.md` in the same commit: recurrence signal "a review
finding of a binding that reports an earlier failure, left in sticky error state, as its own
call's", `watched`. Its `landed as` cell names the commit, which its landing fills.

## Record

The row for `EVOLUTION.md`, copied in when the change lands. The change was made as crustify
`57fd7a3`, which also edited `EVOLUTION.md`; on 2026-11-14 it was put on the working branch's
tip as `83374c5`, the same diff without that edit, since a change's row is drafted in its
proposal. Review-1 read `83374c5`; the commit is now `2c21af3`.

| change | from | landed as | signal answered | recurrence signal | status |
|---|---|---|---|---|---|
| a failure read from sticky error state is its change | `glibc-crustify`, `sticky-error-state` | crustify `2c21af3` | the Errors rule read error state only after the call, which reports a sticky flag's earlier failure as the call's | a review finding of a binding that reports an earlier failure, left in sticky error state, as its own call's | watched |

## Validation

No run failed for want of the rule: the batch worked it out before landing. What the
change does is in no doubt, a rule stated where the Errors convention is. So it is
validated on the item its evidence came from: wave 5's binding follows it, and the six tests
above fail without it. A rerun of the batch under the change is not needed to show that.

## Review

Not yet reviewed.

`review-1` (session `0187d611-78e7-4af2-8c3e-c40ea42ab544`, Opus 5.5, `$1.39`, 4m46s), on `83374c5`: **rejected**, on two fixes one deletion makes: the rule's scope takes in `errno`, for which reading before and after is the wrong method, and the documentation clause named one unseen failure of the two the evidence shows. It reran the validation: the six named tests fail under a read after the call alone. Made as `2c21af3`, on `c59838e`: the sentence ends at "by its change alone, with nothing else able to change it in between".
Its report is the longest `result` of `crustify/artifacts/evolution/sticky-error-state/review-1/stream.jsonl`.
