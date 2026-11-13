# `File`'s contract for a borrowed stream trusts glibc's own stdio routines to hold its lock, which they skip on a `FSETLOCKING_BYCALLER` stream

- **item**: `st::_IO_FILE` / `tdf::FILE`, bound as `File` with `FileRef` and `FileMut`,
  `src/libc/libio/genops.rs`: the contract the type's documentation gives
  `FileRef::from_ptr` for a stream, on which the `Send` and `Sync` assertions of `File` and
  `FileRef` rest
- **affected commit**: `915f5a4` (wave 2, batch 0)
- **objective**: `contracts`, of the `ffi_import` region. The obligation is incorrect. It
  names glibc's own stdio routines as writers that meet it, and on some streams they do not.
  No safe path reaches undefined behaviour today, since nothing bound on `FileRef` reads or
  writes the stream yet. So there is no Rust reproducer. The evidence is glibc's source, and
  the C demonstration below.

## What is wrong

`File`'s documentation states the contract a caller of `FileRef::from_ptr` keeps for a
stream:

> the pointer is NULL or an open stream that stays open for the handle's lifetime, `fclose`
> included, and whatever else writes it while the handle lives does so under its lock, as
> glibc's own stdio routines do.

`FileRef`'s `Sync` relies on that last clause. Every operation bound on `FileRef` holds the
stream's lock, taken with `flockfile`, so two Rust operations exclude each other. A Rust
operation excludes a C writer only if that writer takes the same lock. The clause tells the
caller that glibc's own routines do. They do so only while the stream's locking is internal:

- the locking stdio routines lock through the `_IO_flockfile` macro, which is a no-op when
  the stream carries `_IO_USER_LOCK` (`c/libio/libio.h:282-285`), and
  `__fsetlocking(fp, FSETLOCKING_BYCALLER)` sets that flag (`c/libio/__fsetlocking.c:30-32`);
- the walkers of every open stream lock each one with the same macro: `fflush(NULL)`
  (`_IO_flush_all`, `c/libio/genops.c:698`), `_IO_flush_all_linebuffered` (`:734`) and
  `exit`'s `_IO_unbuffer_all` (`:785`). So on such a stream they write without the lock too,
  from whichever thread calls them;
- the `_unlocked` routines never take it, on any stream.

The type's own *Shared* paragraph says the first and third of these. The contract's last
clause contradicts it. A caller who reads that clause and checks only that the C writing
the stream is glibc's (`fprintf`, `fputs`, `fflush`) honours the contract as written. But
it can hand over a stream that C has set to `FSETLOCKING_BYCALLER` and writes on another
thread without `flockfile`. Once wave 3 binds a write on `FileRef`, that write and C's would
race on the stream's buffer.

## Evidence

The C side of the scenario, on the host's glibc (`Ubuntu GLIBC 2.39-0ubuntu8.9`). One thread
holds the stream's lock, as a Rust operation on `FileRef` does. Another thread writes the
stream with glibc's `fputs`, as C that stays C would. The program records whether the write
finished while the lock was still held:

```c
#include <stdio.h>
#include <stdio_ext.h>
#include <pthread.h>
#include <unistd.h>
#include <stdatomic.h>
static FILE *fp; static atomic_int wrote;
static void *writer(void *a) { (void)a; fputs("x", fp); atomic_store(&wrote, 1); return 0; }
static int run(int mode) {
  fp = fopen("/dev/null", "w"); atomic_store(&wrote, 0);
  __fsetlocking(fp, mode);
  flockfile(fp);                       /* what a Rust operation on FileRef holds */
  pthread_t t; pthread_create(&t, 0, writer, 0);
  usleep(300000);
  int w = atomic_load(&wrote);          /* did glibc's fputs write while the lock was held? */
  funlockfile(fp); pthread_join(t, 0); fclose(fp);
  return w;
}
int main(void) {
  int internal = run(FSETLOCKING_INTERNAL), bycaller = run(FSETLOCKING_BYCALLER);
  printf("fputs wrote under another thread's flockfile: INTERNAL=%d BYCALLER=%d\n", internal, bycaller);
  return bycaller ? 1 : 0;
}
```

```console
$ gcc -O1 -pthread bycaller.c -o bycaller && ./bycaller; echo exit=$?
fputs wrote under another thread's flockfile: INTERNAL=0 BYCALLER=1
exit=1
```

With internal locking, `fputs` waits for the lock. On a `FSETLOCKING_BYCALLER` stream it
writes while another thread holds it. TSan cannot report the race from Rust, since the
system's glibc is not instrumented and the writes on both sides are glibc's.

## Judgment of the thread-safety claim

The claim is otherwise sound, and it is the shape ffibox gives a C object that locks
itself. `docs/ownership.md`'s shared-state table puts "routines that lock internally" on
`FooRef`. ffibox's `CGuarded`, the other shape, refuses a recursive lock, and the stream's
lock is recursive (`flockfile` on the owning thread only counts up). Basing `Sync` on
`flockfile` rather than on the stream's locking mode is right: that mode is data the native
code changes, and `flockfile` takes the lock whatever the mode (`c/stdio-common/flockfile.c`).
`FileRef` reads nothing of the stream, and ffibox's seams form no reference to it, so
nothing reachable today touches the stream. What was wrong was the condition the contract
puts on the stream's other writers.

## Fix

The fix is to the documentation only. `File`'s documentation now gives the contract as:

> the pointer is NULL or an open stream that stays open for the handle's lifetime, `fclose`
> included, and every other access to it while the handle lives, C's or Rust's, on any
> thread, holds its lock. glibc's stdio routines take that lock themselves only while the
> stream's locking is internal, its default. On a stream set to `FSETLOCKING_BYCALLER`,
> every routine skips it, `fflush(NULL)` and `exit`'s flush of every open stream included,
> and the `_unlocked` routines skip it on every stream: the C that uses them holds the lock
> itself, with `flockfile`, or the stream is not borrowed while it runs.

The justification of `Sync for File` now names that contract as what makes every other
access hold the lock. The demonstration above shows glibc's behaviour, not the binding's.
It still exits 1 after the fix, which removes from the contract the claim it disproves. A
revisit judges this report by whether the contract states glibc's routines as meeting the
obligation unconditionally.

For waves 3 and 4: an operation bound on `FileRef` holds `flockfile` for the whole of its
access, as the batch's `stream-lock-for-shared-writes` proposal plans for `fprintf`. A
`'static` handle to `stderr` takes this contract over the whole program, and the program's C
never sets `stderr` to `FSETLOCKING_BYCALLER` without locking it.
