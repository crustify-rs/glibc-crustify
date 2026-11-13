# Planner run 1 planned `fprintf`'s failure as `errno`'s error alone

Filed by `planner/run-2` (2026-11-13), as `reviewing`'s Plan corrections section asks of a
planner run that corrects an earlier run's decision.

## What the earlier plan decided

Planner run 1 (`25b3fe2`) planned wave 3 (`stdio-common/fprintf.c`, `fprintf`) with this
outcome in its commit message:

> the formatted message, passed to fprintf(stream, "%s", message), the count or errno's
> io::Error returned.

No field of `modules.json` or `schedule.json` records it.

## The evidence against it

`fprintf` fails on two ordinary paths without setting `errno`. One is a wide-oriented
stream: `vfprintf`'s orientation check returns `EOF` (`c/stdio-common/vfprintf-internal.c:134`,
`ORIENT`). The other is a cookie stream whose write function fails without setting `errno`,
as `fmemopen`'s does once its storage is full (`c/libio/fmemopen.c`, `fmemopen_write`).
Wave 3's batch (`53dd776`) found both. Its equivalence tests `a_full_stream_fails_alike`
and `a_wide_stream_fails_alike` (`tests/equiv_pub/fprintf.rs`) compare the two with C. It
bound them as `io::ErrorKind::Other` with no OS error, `errno` restored
(`src/libc/stdio_common/fprintf.rs`, `outcome`). A binding that reads `errno` alone
reports such a failure as a stale error or as `Os { code: 0 }`. The general rule is proposed
as `crustify/evolution/errno-failure-without-errno`, on `binding-symbols`.

Rerun: `sed -n 125,140p c/stdio-common/vfprintf-internal.c`; `git show 53dd776 --stat`.

## The edit and what it moves

The outcome is withdrawn from the plan: an item's error convention is the analysis's. No
artifact entry changes, and no wave is re-planned. It bears on wave 5 (`libio/iofwrite.c`,
`fwrite`), appended by this run. `fwrite` returns `0` with `errno` untouched on a
wide-oriented stream (`c/libio/iofwrite.c:38-39`, `_IO_fwide (fp, -1) == -1` false). It
also returns its full count when the flush of a line-buffered write fails
(`c/libio/iofwrite.c:41-46`, `_IO_new_file_xsputn`'s `EOF`, `c/libio/fileops.c:1244-1247`),
which `fprintf`'s `%s` reports as a failure. The batch's analysis settles how the binding
reports each.

## Questions for the user

None.
