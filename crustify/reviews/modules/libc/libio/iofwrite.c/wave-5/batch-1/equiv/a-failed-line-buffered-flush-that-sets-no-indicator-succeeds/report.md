# `write_all` reports a failed line-buffered flush as written when the failure sets no error indicator

- **item**: `fn::fwrite`, bound as `FileRef::write_all(&self, &[u8]) -> io::Result<()>`,
  `src/libc/libio/iofwrite.rs`: its failure rule and its `# Failure` doc
- **affected commit**: `d796900` (wave 5, batch 0)
- **objective**: `equiv`. The binding departs from `fwrite` on purpose, to fail where
  `fprintf(stream, "%s", text)` fails, the call the parent's C makes and its export stands in
  for. The doc says one case escapes that, a stream whose error indicator is already set.
  There is a second, with the indicator clear, that the doc does not name. No test reached
  it.

## What is wrong

The doc's `# Failure` section, as landed:

> `fwrite` reports a failure by returning fewer bytes than it was given, with one exception:
> when the bytes fit the buffer of a line-buffered stream and end with a newline, and the
> flush that newline calls for fails, it returns the full count, though the stream's error
> indicator records the failure. […] A stream whose error indicator is already set […]
> cannot tell that exception from a success, which is what it reports.

The rule assumes that every failure of that flush sets the error indicator. One does not.
Before it writes the buffer, `new_do_write` seeks back over the bytes read into the buffer
and not yet consumed (`c/libio/fileops.c:441-447`). When that seek fails, it returns `0` at
once, without setting `_IO_ERR_SEEN` and without resetting the buffer. `_IO_new_file_xsputn`
then returns `EOF` for a line that fit (`c/libio/fileops.c:1244-1247`), and `_IO_fwrite` turns
that into the full count (`c/libio/iofwrite.c:45-46`). `errno` holds the seek's `ESPIPE`, the
indicator stays clear, and the bytes stay in the buffer. The binding sees a full count and
an indicator clear before and after, and returns `Ok(())`. `fprintf` checks `_IO_sputn`'s
return itself (`c/stdio-common/printf_buffer_to_file.c:59-61`) and returns `-1`.

The seek fails on a file that cannot seek, a pipe or a socket, opened for reading and
writing, after C read from the stream and wrote to it without a positioning call between,
the first write leaving the read bytes unconsumed. ISO C (7.21.5.3p7) leaves output directly
after input undefined without a positioning call. glibc defines it: it seeks back, as above.
The stream is the C's, shared with Rust, so the C may hand it on in that state. Nothing
unsound follows: the binding reports success where `fprintf` fails, and `errno` is restored.

The parent's uses (`stderr`, a terminal's `stdout`) never read the stream they write. The
departure is real, but rare, and its sequence is one ISO C leaves undefined.

## Reproducer

`reproducer/`, a standalone package over the affected crate. It repeats the workspace's
`[patch]` of ffibox, which a path dependency does not carry. It builds two streams from one
description, each one end of a `UnixStream::pair`, opened `"r+"` with `fdopen` and made
line-buffered, from which C reads one byte of three. It then writes the same two lines to
each, one through `write_all` and the other through `fprintf(stream, "%s", text)`, and exits
non-zero when the two disagree. `fdopen` and `fgetc` are declared in the package, as C would
call them, since the raw bindings of `d796900` lack them.

From a clean checkout of `d796900`, with this directory copied in:

```console
$ cd crustify/reviews/modules/libc/libio/iofwrite.c/wave-5/batch-1/equiv/a-failed-line-buffered-flush-that-sets-no-indicator-succeeds/reproducer
$ cargo run -q
step 0: write_all Err(Os { code: 29, kind: NotSeekable, message: "Illegal seek" }), error indicator false; fprintf -1, errno 29, error indicator false
step 1: write_all Ok(()), error indicator false; fprintf -1, errno 29, error indicator false
step 1: write_all and fprintf(stream, "%s", text) disagree: the binding reports "cd\n" written where fprintf fails
$ echo $?
1
```

The first line goes to the buffer byte by byte, since the stream is not yet in write mode,
and fails short, as both sides report. The second fits the buffer and is the case above. The
host's glibc is `Ubuntu GLIBC 2.39-0ubuntu8.9`. The same sequence in C, with `fwrite` in
place of `write_all`, gives `fwrite` `2` and then `3`, the error indicator clear after
each.

## Fix

The code cannot tell this failure from a success. No indicator is set. `errno` is set, but
a successful `fwrite` may leave `errno` set too, which the binding's own unit test
`a_full_count_with_no_new_error_is_the_success` keeps as no failure. No routine the bindings
reach returns `_IO_sputn`'s `EOF` for bytes that may hold a NUL. So the fix makes the
departure deliberate and documented, as `reviewing`'s `equiv` objective allows. The doc now
names two failures that cannot be told from a success: one on a stream whose indicator is
already set, and one that sets no indicator, the failed seek above. In both, the call reports
success, as `fwrite` does, where `fprintf` fails. The doc says the bytes stay in the buffer
and `errno` is left as it was.

Regression tests, over a fixture stream built as the reproducer builds it
(`Stream::line_buffered_socket_after_a_read`, `tests/fixtures/stream.rs`):

- `equiv_pub` `iofwrite::a_flush_that_fails_in_its_seek_is_reported_as_fwrite_reports_it`:
  the binding equals `fwrite` on that stream at each step, and the departure from
  `fprintf` the doc states is asserted: `fprintf` fails both lines with `ESPIPE` where the
  binding fails the first and reports the second written, with no indicator set on either
  side;
- `unit_pub` `iofwrite::a_flush_that_fails_in_its_seek_is_reported_as_written`: the first line
  fails with `ESPIPE`, the second succeeds, `errno` is left as it was, no indicator is set,
  the lock is released, and nothing reaches the socket.

The raw bindings gain `fdopen`, `fgetc` and `ESPIPE` for them, test-only. The reproducer still
exits `1` on the fixed tree, by design: it shows the departure the doc now states.

No disclosure is written: the defect is in the campaign's own crate, and glibc's `fwrite`
does what its source comment intends, reporting bytes that stay in the buffer as written
(`c/libio/iofwrite.c:41-44`).
