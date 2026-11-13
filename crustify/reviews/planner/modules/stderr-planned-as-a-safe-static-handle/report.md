# Planner run 1 planned `stderr` as a safe `'static` shared handle

Filed by `planner/run-2` (2026-11-13), as `reviewing`'s Plan corrections section asks of a
planner run that corrects an earlier run's decision.

## What the earlier plan decided

Planner run 1 (`25b3fe2`) planned wave 4 (`libc/libio/stdio.c`, item `stderr`, `kind`
`global`, home `src/libc/libio/stdio.rs`). Its commit message gave the binding's shape:

> - stderr is bound as a 'static shared handle: no C of either parent writes it, and its
>   storage (_IO_2_1_stderr_) is static.

No field of `modules.json` or `schedule.json` records that shape. Wave 4's item is right as
it stands. The decision was the run's reasoning, which wave 4's binder followed
(`2a792a8`: a safe `stderr() -> FileRef<'static>`).

## The evidence against it

- The access discipline of a global follows from every C access the C declaration's
  contract allows, not from the accesses the parents' C makes. glibc's manual says the
  standard streams are "normal variables which you can set just like any others", and gives
  `fclose (stdout); stdout = fopen (…);` as the way to redirect one
  (`c/manual/stdio.texi:115-122`). `fclose` frees every stream but the three static ones
  (`c/libio/iofclose.c:74`, `_IO_deallocate_file`, `c/libio/libioP.h:948-958`). Even
  `_IO_2_1_stderr_` is closed by it, and C11 7.21.3p4 leaves a closed stream's pointer
  indeterminate. The wrapper is a library, so it never sees all of the program's C, and
  libgit2 is a library too.
- Wave 4's module pass reproduced it: a safe `'static` handle kept past a redirect is a
  heap-use-after-free under ASan
  (`crustify/reviews/modules/libc/libio/stdio.c/wave-4/batch-1/contracts/ffi_import/stderr-lends-a-static-stream-the-program-may-close/report.md`,
  reproducer beside it, on `2a792a8`).
- Its fix (`a5448ee`) made the accessor `pub unsafe fn stderr<'a>() -> FileRef<'a>`, the
  caller vouching for the lifetime it chooses (`src/libc/libio/stdio.rs`). A compile-fail
  test keeps it that way (`tests/ub_safe/compile_fail/stdio_stderr_is_lent_by_no_safe_code.rs`).
  The rule it rests on is proposed as `crustify/evolution/global-the-program-may-set`, on
  `binding-symbols`' Globals section, which is not the planner's.

Rerun: `git show 25b3fe2 --format=%B -s`; `git show a5448ee -- src/libc/libio/stdio.rs`;
`sed -n 110,125p c/manual/stdio.texi`; `sed -n 940,960p c/libio/libioP.h`.

## The edit and what it moves

Withdrawn: the plan no longer states a shape for `stderr`. A global's access discipline is
the binder's, from the analysis under `binding-symbols`' Globals section, and the plan's
records name the entity alone. No artifact entry changes. Wave 4 is `done`, and its record
keeps its fields. No wave is re-planned or appended for it.

Downstream: the parent's consumers of `stderr`, libgit2-crustify's wave 194
(`libgit2/deps/llhttp/api.c`, `llhttp__debug`), call `unsafe { stderr() }` for one call, as
C's `fprintf (stderr, …)` reads the variable at each call. They vouch for no longer
lifetime. The pin `1e60130` already carries the `unsafe fn`.

## Questions for the user

None.
