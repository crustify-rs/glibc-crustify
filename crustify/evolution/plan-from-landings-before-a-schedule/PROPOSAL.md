# A side campaign's first plan: what landed before a schedule, and how far its decomposition reaches

Status: `open`. Filed by glibc-crustify `planner/run-1` (2026-11-13), under the default
`repair` autonomy (`crustify/CRUSTIFY.md` gives no `evolution` answer): an improvement,
proposed rather than made.

## Signal

Rules a run had to work out by hand that the skills should state (`evolution` §2, "a rule
an agent works around"). glibc-crustify's first planner run met a repository with no plan, whose
`struct tm` (`b7d01b0`, `ea38101`, `8220840`) and `__errno_location` (`787a802`) wrappers
had landed on `crustify/devel` with no `schedule.json`, no wave and no batch. It was asked
to record them as landed rather than schedule them again, and to plan three more entities
(`_IO_FILE`, `fprintf`, `stderr`), without a wavefront database over the system's glibc.

1. **Landings with no schedule.** `schedule-schema` gives a started wave's record only to
   waves whose batches ran (§2, §4, §5), and `scheduling` drops an item a batch has landed
   (§3). Nothing says how a plan records an item that landed before any schedule existed,
   so a planner either schedules it again or leaves it out of the order entirely. The run
   recorded each as a `done` wave of its module, with one `done` `wrap` batch whose
   `commits` are the commits that touched its home, in landing order, and numbered them
   before the `todo` waves (`crustify/artifacts/schedule.json`, waves 0 and 1). Those are
   progress fields, the assistant's, which the run wrote because its task message asked.
2. **How far a side campaign's decomposition reaches.** `decomposition` §2 covers "the
   selected target and its complete imported producer closure: every translation unit the
   build compiles in scope". For a side campaign over a system library, whose scope is a
   few entities and which is wrapped, never built or ported, a body-deep closure would pull
   in most of the library (`stdio-common/fprintf.c` calls `__vfprintf_internal`, which
   reaches locales, wide characters and the allocator). The wavefront skill schedules a wrap
   campaign in API mode, but counts the modules' dependency records "from a schedule of the
   whole closure" (Decomposition › Dependencies) without saying in which mode. The run
   decomposed only the TUs that own a scoped entity and what their public signatures reach
   (`libio/genops.c`, the owner of `_IO_FILE`), and counted the dependency records over
   that signature graph.

## Destination

The spec: `schedule-schema` §5 (Re-planning) and `scheduling` §3 for (1); `decomposition`
§2 and the wavefront skill's Decomposition › Dependencies for (2).

## Change

1. `schedule-schema`, after §5's first paragraph: "An item that landed before the campaign
   had a schedule is recorded as landed: a `done` wave of its module, holding it, with one
   `done` batch of its objective whose `commits` are the commits that landed it, numbered
   among the started waves; it is not scheduled again."
2. `decomposition` §2, after "every translation unit the build compiles in scope is a
   module's": "In a wrap campaign, whose waves bind the API, the closure is that of the
   public signatures, as the wrap batches' closure is: the modules that own the scope's
   entities and those their signatures and published fields reach." The wavefront skill's
   Decomposition › Dependencies then says that a wrap campaign counts the records in API
   mode.

## Validation

None yet: the rules are what the run did by hand. A second planner run of this campaign,
or the first plan of another side campaign with landings before its schedule (zlib-,
openssl- and pcre2-crustify have no plan either), would show whether the wording leads it
to the same records.

`planner/run-2` (2026-11-13) is the second planner run. It kept waves 0 and 1 as run 1
recorded them, which the change's rule (1) gives, and moved none of their fields. It
decomposed its one new module, `libio/iofwrite.c`, as rule (2) words it. `fwrite`'s
public signature reaches `_IO_FILE` alone, so the module's one record is on
`libio/genops.c`, 1 edge. Its body's callees (`_IO_sputn` through the stream's vtable,
`_IO_fwide` of `libio/iofwide.c`, the lock macros) became no modules. Without the rule,
`decomposition` §2's producer closure would have added them. So the wording leads a
second run to the records the first wrote by hand. Neither rule is made here: the
proposal is an improvement, and the settings give no `evolution` answer, so the default
`repair` lands none. Rule (1) is on `schedule-schema`, which is not the planner's.

## Review

Not reviewed.

## Record

Recurrence signal: a planner run that schedules again an item that landed before its
campaign's schedule, or that emits modules a wrap campaign's public signatures do not reach.
