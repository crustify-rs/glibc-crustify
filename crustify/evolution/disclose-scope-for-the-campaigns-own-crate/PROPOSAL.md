# A disclosure goes to a project outside the campaign, never to the campaign's own crate

Status: `open`. Filed by glibc-crustify `wave-3/batch-1/run-1` (2026-11-13), under the
default `repair` autonomy (`crustify/CRUSTIFY.md` gives no `evolution` answer). The change
is to the reviewing skill, which the reviewer evolves. It needs a review by another run, so
it is proposed.

## Signal

A rule worked out by hand that a skill should have stated (`evolution` §2). `reviewing`
§3.12 asks for a `disclosure.md` "beside each report that the user can send to the
affected project's maintainers unchanged". Where a review runs `disclose`, every objective
being on when a task names none, it does not say who the affected project is when the
defect lies in the crate the campaign itself generates. That crate has no maintainers but
the user, and the review fixes the defect itself under `fix`. Two module passes worked
this out by hand, alike:

- glibc-crustify `wave-2/batch-1/run-1` (`libio/genops.c`) filed three reports in
  `glibc-bindings` with no disclosure, and said nothing of it;
- this run (`stdio-common/fprintf.c`) filed two. One,
  `contracts/ffi_import/print-locks-a-stream-without-a-lock`, rests on glibc behaviour:
  `flockfile` dereferences the NULL lock of the stream glibc hands a printf handler. The
  run had to judge whether that is a glibc defect to disclose. It is not: glibc's routines
  skip the lock on that stream by design, and before 2.37 the helper stream of an
  unbuffered stream had none either. So it wrote no disclosure.

## Destination

The spec: `reviewing` §3.12, Disclosure.

## Change

After §3.12's first sentence: "The affected project is the one whose code holds the
defect. A finding in the crate the campaign generates has no disclosure, since the
campaign fixes it, unless its evidence shows a defect in the C library too. Its report
then says so, and its disclosure goes to that library's maintainers."

## Validation

A documentation change, which no failure prompted. Both runs above would have written
what they wrote, and stated why.

## Review

Not reviewed.
