# crustify — `glibc / libc`

## Settings

As answered in `crustify/CRUSTIFY.md`; a change during the campaign is in its history
and in Notes.

- **source** — `https://sourceware.org/git/glibc.git` @ `ef321e23c2` (`glibc-2.39`); side campaign of `crustify-rs/ffmpeg-bindings` and `crustify-rs/libgit2-bindings`
- **campaign** — `wrap`
- **scope** — what `crustify-rs/ffmpeg-bindings` consumes from libc: `struct tm`, `FILE`; what `crustify-rs/libgit2-bindings` consumes: 871 functions, 95 structs, 2 globals, counted per module
- **translate-agent** — `openrouter/anthropic/claude-opus-5.5`, `api`
- **review-agent** — `openrouter/anthropic/claude-opus-5.5`, `api`; objectives `all`
- **plan-agent** — `openrouter/anthropic/claude-opus-5.5`, `api`; `2` runs
- **workload** — max-types `2`, max-syms `25`, max-loc `500`, parallelism `assistant's choice` (from 2026-11-03; `16` before)
- **identity** — author `Marius Momeu <marius.momeu@berkeley.edu>`, co-author `crustify <crustify@berkeley.edu>` (from 2026-11-03; `crustify` as the author before)
- **review-workload** — `as many as the reviewed wave requires`
- **sanitizers** — `asan-ubsan`, `tsan`, `bsan`, over the Rust; the system glibc is not instrumented
- **target-config** — none: libc is the system's and is not built, `build.json` version `1`

## Legend

- `module` / `wave` / `batch` — the batch's place in `schedule.json`: its wave's module and
  the wave and batch indices
- `objective` — `wrap` or `port`; `pass` — `first` for a translation batch, `review` for a
  review batch; `Σ first` / `Σ review` total each pass apart
- `types` / `symbols` / `fields` — the batch's scheduled units (callbacks count as
  symbols) and its types' assigned field accessors; a review row counts what it reviewed
- `$` / `tokens` / `wall` — the agent's computed cost (API-equivalent under
  subscription), its tokens in millions as `crustify usage` reports them, and its elapsed
  time, C rebuild included; `$/<unit>` divides `$` by the row's units
- `+LoC` — the landing's Rust insertions/deletions; `loc` — its `.rs` insertions
- `cov (C/Rust)` — line coverage after the landing, all test workloads in one run: C over
  the link unit's TUs, Rust over non-test sources; a `Σ` cell is the pass's last landing
- `+UB safe tests` … `+unit tests` — the tests the landing added, with its C/Rust coverage
  change in pp
- `+lifecycle` … `+misc`, `total` — the reports a review batch filed, by kind
- safety audit columns — the Safety audit's metrics, as a landing changed them
- `cost` / `review cost` / `total` — a module's translation, review and combined cost;
  `module wall` — first launch to final gate; `Σ recorded agents` leaves the assistant
  out, `Σ campaign` adds it, without per-unit cells
- `UB safe tests` … `Unit tests` — `#[test]` functions per suite, each with the coverage of
  running it alone; `Total tests` — all four, whose coverage is one joint run's union,
  never the sum
- `total`, `ported` and `wrapped` types and symbols — the scheduled public-API units, and
  those with a `Replaces:` or a `Wraps:` anchor at the tip; `share` is of the total

Every table below is a heading, a model line and the table. All prose belongs
in Notes.

## Overview

- **branch** — `crustify/devel`, tip `<sha>`; `main` shipped at `<sha>`; crustify `<sha>`, ffibox `<sha>`, wavefront `<sha>`
- **C test baseline** — not run: glibc is not built (the system's libc); disabled: its whole suite
- **Rust LoC** — `409` non-test, `809` tests; **C LoC** — `<n>`

### Coverage stats

| | count | share | C LoC coverage | Rust LoC coverage |
|---|---:|---:|---:|---:|
| UB safe tests | `6` | — | — | `30.21`% |
| UB unsafe tests | `0` | — | — | — |
| Equiv tests | `6` | — | — | `41.67`% |
| Unit tests | `24` | — | — | `100.00`% |
| **Total tests** | **`36`** | — | — | **`100.00`%** |
| total types | `2` | — | — | — |
| ported types | `0` | `0`% | — | — |
| wrapped types | `2` | `100`% | — | — |
| total symbols | `3` | — | — | — |
| ported symbols | `0` | `0`% | — | — |
| wrapped symbols | `2` | `67`% | — | — |

### Module stats

Implementation `<provider>/<model>` via `<backend>`; review
`<provider>/<model>` via `<backend>`.

| module | nr types | nr symbols | module wall | cost | tokens | $/type | $/sym | review cost | review tokens | review $/type | review $/sym | total | total tokens | total $/type | total $/sym |
|---|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `<link-unit>/<module> (foundational)` | `<n>` | `<n>` | `<n>h<n>m<n>s` | `$<n>` | `<n>M` | `$<n>` | `$<n>` | `$<n>` | `<n>M` | `$<n>` | `$<n>` | `$<n>` | `<n>M` | `$<n>` | `$<n>` |
| `<link-unit>/<module>` | `<n>` | `<n>` | `<n>h<n>m<n>s` | `$<n>` | `<n>M` | `$<n>` | `$<n>` | `$<n>` | `<n>M` | `$<n>` | `$<n>` | `$<n>` | `<n>M` | `$<n>` | `$<n>` |
| side campaigns | `<n>` | `<n>` | `<n>h<n>m<n>s` | `$<n>` | `<n>M` | — | — | `$<n>` | `<n>M` | — | — | `$<n>` | `<n>M` | — | — |
| planners | — | — | `<n>h<n>m<n>s` | — | — | — | — | — | — | — | — | `$<n>` | `<n>M` | — | — |
| assistant | — | — | — | — | — | — | — | — | — | — | — | `$<n>` | `<n>M` | — | — |
| **Σ recorded agents** | **`<n>`** | **`<n>`** | **`<n>h<n>m`** | **`$<n>`** | **`<n>M`** | **`$<n>`** | **`$<n>`** | **`$<n>`** | **`<n>M`** | **`$<n>`** | **`$<n>`** | **`$<n>`** | **`<n>M`** | **`$<n>`** | **`$<n>`** |
| **Σ campaign** | **`<n>`** | **`<n>`** | **`<n>h<n>m`** | **`$<n>`** | **`<n>M`** | — | — | **`$<n>`** | **`<n>M`** | — | — | **`$<n>`** | **`<n>M`** | — | — |

## Safety audit

Deterministic `ffimap <workdir> --json`; no model. Every counter is in the `ffimap.json`
tracked in the campaign's artifact dir, in each wave's `waves/wave-<index>/` and in each
batch run's dir.

### Metrics

| metric | counters | what it counts |
|---|---|---|
| pub raw ptr (safe) | `raw_ptr_pub_safe_interface` | raw pointers on the safe public interface: public fields and safe public fns |
| pub raw ptr | `raw_ptr_pub` | raw pointers anywhere on the public interface, `unsafe fn`s included |
| pub void ptr | `void_ptr_ty_pub_fields_pub` + `void_ptr_fn_pub_safe` + `void_ptr_fn_pub_unsafe` | type-erased pointers left to the API's users |
| pub unsafe fn | `unsafe_fns_pub` | the obligations the crate exports to its callers |
| C borrow | `c_type_borrow_non_sanctioned` | Rust references into C memory outside the sanctioned handle positions; target 0 |
| slice borrow | `raw_ptr_as_slice_borrow` | slices and strings borrowed over raw pointers, which C must neither free nor change |
| unsafe loc % | `unsafe_loc_ratio` | unsafe block lines over code lines; context, not a score |
| unsafe blocks | `unsafe_blocks` | unsafe blocks, the denominator of the ratio's reading |
| Send/Sync | `unsafe_send_impl` + `unsafe_sync_impl` | thread safety asserted rather than derived from the fields |
| C by value | Σ `c_type_by_val_*` + Σ `c_type_std_owner_*` | C types held outside an ffibox owner, which a move can leave dangling |
| Drop/Clone | `drop_impl` + `clone_impl` | lifecycles written by hand rather than through an owner |
| deref outside impl | `raw_ptr_derefs_outside_impl` | raw pointer dereferences outside any `impl` or trait body |

## Planning

`openrouter/anthropic/claude-opus-5.5` via `claude`. One row per planner run, in order; its reports are
the corrections it made to what earlier runs committed.

| run | modules changed | items moved | wavefront artifacts changed | reports | $ | tokens | wall |
|---|---:|---:|---|---:|---|---|---|
| 1 | `5` | `0` | none | `0` | `$<n>` | `<n>M` | `<n>m<n>s` |
| 2 | `<n>` | `<n>` | `<files>` | `<n>` | `$<n>` | `<n>M` | `<n>m<n>s` |
| **Σ** | **`5`** | **`0`** | | **`0`** | **`$<n>`** | **`<n>M`** | **`<n>m<n>s`** |

## Side campaigns

One row per out-of-tree library the campaign wraps in a side campaign, as
`specs/dependencies.md` describes. Each wrapper's own dashboard, in its
`<source>-bindings` repository, is its record; its row here copies that dashboard's
figures as of the commit the campaign pins, and the `side campaigns` row of `Module stats`
sums them.

| library | repository | pin | shipped at | linked from | waves | types | symbols | wall | cost | tokens | review cost | review tokens | total | total tokens |
|---|---|---|---|---|---:|---:|---:|---|---:|---:|---:|---:|---:|---:|
| `<library>` | `crustify-rs/<source>-bindings` | `<sha>` | `<sha>` | `side-campaign` | `<n>` | `<n>` | `<n>` | `<n>h<n>m<n>s` | `$<n>` | `<n>M` | `$<n>` | `<n>M` | `$<n>` | `<n>M` |
| `libc` | `crustify-rs/glibc-bindings` | `<sha>` | `<sha>` | `system` | `<n>` | `<n>` | `<n>` | `<n>h<n>m<n>s` | `$<n>` | `<n>M` | `$<n>` | `<n>M` | `$<n>` | `<n>M` |
| **Σ** | | | | | **`<n>`** | **`<n>`** | **`<n>`** | **`<n>h<n>m`** | **`$<n>`** | **`<n>M`** | **`$<n>`** | **`<n>M`** | **`$<n>`** | **`<n>M`** |

## Types

### Batches — wrap

`openrouter/anthropic/claude-opus-5.5` via `claude`.

| module | wave | batch | pass | types | fields | +LoC | cov (C/Rust) | $ | tokens | wall | $/type | $/field |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `libc/libio/genops.c` | `2` | `0` | first | `1` | `0` | `+335/-0` | — / `100.00`% | `$<n>` | `<n>M` | `<n>m<n>s` | `$<n>` | — |
| `libc/libio/genops.c` | `2` | `1` | review | `1` | `0` | `+25/-7` | — / `100.00`% | `$<n>` | `<n>M` | `<n>m<n>s` | `$<n>` | — |
| **Σ first** |  |  |  | **`1`** | **`0`** | **`+335/-0`** | **— / `100.00`%** | **`$<n>`** | **`<n>M`** | — | **`$<n>`** | — |
| **Σ review** |  |  |  | **`1`** | **`0`** | **`+25/-7`** | **— / `100.00`%** | **`$<n>`** | **`<n>M`** | — | **`$<n>`** | — |

### Batches — wrap: tests

`openrouter/anthropic/claude-opus-5.5` via `claude`.

| module | wave | batch | pass | +UB safe tests (+C/+Rust pp) | +UB unsafe tests (+C/+Rust pp) | +equiv tests (+C/+Rust pp) | +unit tests (+C/+Rust pp) |
|---|---|---|---|---|---|---|---|
| `libc/libio/genops.c` | `2` | `0` | first | `+3` (—/`+0.00` pp) | `+0` (—/`+0.00` pp) | `+0` (—/`+0.00` pp) | `+3` (—/`+0.00` pp) |
| `libc/libio/genops.c` | `2` | `1` | review | `+1` (—/`+0.00` pp) | `+0` (—/`+0.00` pp) | `+0` (—/`+0.00` pp) | `+0` (—/`+0.00` pp) |
| **Σ first** |  |  |  | **`+3` (—/`+0.00` pp)** | **`+0` (—/`+0.00` pp)** | **`+0` (—/`+0.00` pp)** | **`+3` (—/`+0.00` pp)** |
| **Σ review** |  |  |  | **`+1` (—/`+0.00` pp)** | **`+0` (—/`+0.00` pp)** | **`+0` (—/`+0.00` pp)** | **`+0` (—/`+0.00` pp)** |

### Batches — port

`<provider>/<model>` via `<backend>`.

| module | wave | batch | pass | types | fields | +LoC | cov (C/Rust) | $ | tokens | wall | $/type | $/field |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `<link-unit>/<module>` | `<n>` | `<n>` | first | `<n>` | `<n>` | `+<n>/-<n>` | `<n>`% / `<n>`% | `$<n>` | `<n>M` | `<n>m<n>s` | `$<n>` | `$<n>` |
| `<link-unit>/<module>` | `<n>` | `<n>` | review | `<n>` | `<n>` | `+<n>/-<n>` | `<n>`% / `<n>`% | `$<n>` | `<n>M` | `<n>m<n>s` | `$<n>` | `$<n>` |
| **Σ first** |  |  |  | **`<n>`** | **`<n>`** | **`+<n>/-<n>`** | **`<n>`% / `<n>`%** | **`$<n>`** | **`<n>M`** | — | **`$<n>`** | **`$<n>`** |
| **Σ review** |  |  |  | **`<n>`** | **`<n>`** | **`+<n>/-<n>`** | **`<n>`% / `<n>`%** | **`$<n>`** | **`<n>M`** | — | **`$<n>`** | **`$<n>`** |

### Batches — port: tests

`<provider>/<model>` via `<backend>`.

| module | wave | batch | pass | +UB safe tests (+C/+Rust pp) | +UB unsafe tests (+C/+Rust pp) | +equiv tests (+C/+Rust pp) | +unit tests (+C/+Rust pp) |
|---|---|---|---|---|---|---|---|
| `<link-unit>/<module>` | `<n>` | `<n>` | first | `+<n>` (`+<n>`/`+<n>` pp) | `+<n>` (`+<n>`/`+<n>` pp) | `+<n>` (`+<n>`/`+<n>` pp) | `+<n>` (`+<n>`/`+<n>` pp) |
| `<link-unit>/<module>` | `<n>` | `<n>` | review | `+<n>` (`+<n>`/`+<n>` pp) | `+<n>` (`+<n>`/`+<n>` pp) | `+<n>` (`+<n>`/`+<n>` pp) | `+<n>` (`+<n>`/`+<n>` pp) |
| **Σ first** |  |  |  | **`+<n>` (`+<n>`/`+<n>` pp)** | **`+<n>` (`+<n>`/`+<n>` pp)** | **`+<n>` (`+<n>`/`+<n>` pp)** | **`+<n>` (`+<n>`/`+<n>` pp)** |
| **Σ review** |  |  |  | **`+<n>` (`+<n>`/`+<n>` pp)** | **`+<n>` (`+<n>`/`+<n>` pp)** | **`+<n>` (`+<n>`/`+<n>` pp)** | **`+<n>` (`+<n>`/`+<n>` pp)** |

### Batches: safety audit

Deterministic `ffimap`; no model.

| module | wave | batch | objective | pass | pub raw ptr (safe) | pub raw ptr | pub void ptr | pub unsafe fn | C borrow | slice borrow | unsafe loc % | unsafe blocks | Send/Sync | C by value | Drop/Clone | deref outside impl |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `libc/libio/genops.c` | `2` | `0` | wrap | first | `±0` | `±0` | `±0` | `±0` | `±0` | `±0` | `-3.18` pp | `+2` | `+6` | `+1` | `+1` | `±0` |
| `<link-unit>/<module>` | `<n>` | `<n>` | port | first | `±<n>` | `±<n>` | `±<n>` | `±<n>` | `±<n>` | `±<n>` | `±<n>` pp | `±<n>` | `±<n>` | `±<n>` | `±<n>` | `±<n>` |
| `libc/libio/genops.c` | `2` | `1` | wrap | review | `±0` | `±0` | `±0` | `±0` | `±0` | `±0` | `±0.00` pp | `±0` | `±0` | `±0` | `±0` | `±0` |
| **Σ first** |  |  |  |  | **`±0`** | **`±0`** | **`±0`** | **`±0`** | **`±0`** | **`±0`** | **`-3.18` pp** | **`+2`** | **`+6`** | **`+1`** | **`+1`** | **`±0`** |
| **Σ review** |  |  |  |  | **`±0`** | **`±0`** | **`±0`** | **`±0`** | **`±0`** | **`±0`** | **`±0.00` pp** | **`±0`** | **`±0`** | **`±0`** | **`±0`** | **`±0`** |

### Batches — review: PoC + reports

`<provider>/<model>` via `<backend>`.

| module | wave | batch | +lifecycle | +unsafe | +UB safe | +UB unsafe | +equiv | +internal | +conventions | +misc | total |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `libc/libio/genops.c` | `2` | `1` | `+0` | `+0` | `+0` | `+0` | `+0` | `+2` | `+1` | `+0` | `3` |
| **Σ** |  |  | **`+0`** | **`+0`** | **`+0`** | **`+0`** | **`+0`** | **`+2`** | **`+1`** | **`+0`** | **`3`** |

## Symbols

### Batches

`openrouter/anthropic/claude-opus-5.5` via `claude`.

| module | wave | batch | objective | pass | symbols | loc | +LoC | cov (C/Rust) | $ | tokens | wall | $/symbol | $/loc |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `libc/stdio-common/fprintf.c` | `3` | `0` | wrap | first | `1` | `821` | `+821/-1` | — / `100.00`% | `$<n>` | `<n>M` | `<n>m<n>s` | `$<n>` | `$<n>` |
| `<link-unit>/<module>` | `<n>` | `<n>` | port | first | `<n>` | `<n>` | `+<n>/-<n>` | `<n>`% / `<n>`% | `$<n>` | `<n>M` | `<n>m<n>s` | `$<n>` | `$<n>` |
| `libc/stdio-common/fprintf.c` | `3` | `1` | wrap | review | `1` | `199` | `+199/-31` | — / `100.00`% | `$<n>` | `<n>M` | `<n>m<n>s` | `$<n>` | `$<n>` |
| **Σ first** |  |  |  |  | **`1`** | **`821`** | **`+821/-1`** | **— / `100.00`%** | **`$<n>`** | **`<n>M`** | — | **`$<n>`** | **`$<n>`** |
| **Σ review** |  |  |  |  | **`1`** | **`199`** | **`+199/-31`** | **— / `100.00`%** | **`$<n>`** | **`<n>M`** | — | **`$<n>`** | **`$<n>`** |

### Batches: tests

`openrouter/anthropic/claude-opus-5.5` via `claude`.

| module | wave | batch | pass | +UB safe tests (+C/+Rust pp) | +UB unsafe tests (+C/+Rust pp) | +equiv tests (+C/+Rust pp) | +unit tests (+C/+Rust pp) |
|---|---|---|---|---|---|---|---|
| `libc/stdio-common/fprintf.c` | `3` | `0` | first | `+0` (—/`-24.28` pp) | `+0` (—/`+0.00` pp) | `+6` (—/`+39.53` pp) | `+16` (—/`+0.00` pp) |
| `libc/stdio-common/fprintf.c` | `3` | `1` | review | `+0` (—/`-3.51` pp) | `+0` (—/`+0.00` pp) | `+0` (—/`+2.14` pp) | `+1` (—/`+0.00` pp) |
| **Σ first** |  |  |  | **`+0` (—/`-24.28` pp)** | **`+0` (—/`+0.00` pp)** | **`+6` (—/`+39.53` pp)** | **`+16` (—/`+0.00` pp)** |
| **Σ review** |  |  |  | **`+0` (—/`-3.51` pp)** | **`+0` (—/`+0.00` pp)** | **`+0` (—/`+2.14` pp)** | **`+1` (—/`+0.00` pp)** |

### Batches: safety audit

Deterministic `ffimap`; no model.

| module | wave | batch | objective | pass | pub raw ptr (safe) | pub raw ptr | pub void ptr | pub unsafe fn | C borrow | slice borrow | unsafe loc % | unsafe blocks | Send/Sync | C by value | Drop/Clone | deref outside impl |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `libc/stdio-common/fprintf.c` | `3` | `0` | wrap | first | `±0` | `±0` | `±0` | `±0` | `±0` | `±0` | `-6.20` pp | `+3` | `±0` | `±0` | `±0` | `±0` |
| `<link-unit>/<module>` | `<n>` | `<n>` | port | first | `±<n>` | `±<n>` | `±<n>` | `±<n>` | `±<n>` | `±<n>` | `±<n>` pp | `±<n>` | `±<n>` | `±<n>` | `±<n>` | `±<n>` |
| `libc/stdio-common/fprintf.c` | `3` | `1` | wrap | review | `±0` | `±0` | `±0` | `±0` | `±0` | `±0` | `-0.70` pp | `+1` | `±0` | `±0` | `±0` | `±0` |
| **Σ first** |  |  |  |  | **`±0`** | **`±0`** | **`±0`** | **`±0`** | **`±0`** | **`±0`** | **`-6.20` pp** | **`+3`** | **`±0`** | **`±0`** | **`±0`** | **`±0`** |
| **Σ review** |  |  |  |  | **`±0`** | **`±0`** | **`±0`** | **`±0`** | **`±0`** | **`±0`** | **`-0.70` pp** | **`+1`** | **`±0`** | **`±0`** | **`±0`** | **`±0`** |

### Batches — review: PoC + reports

`<provider>/<model>` via `<backend>`.

| module | wave | batch | +lifecycle | +unsafe | +UB safe | +UB unsafe | +equiv | +internal | +conventions | +misc | total |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `libc/stdio-common/fprintf.c` | `3` | `1` | — | `+0` | `+0` | `+0` | `+0` | `+1` | `+1` | `+0` | `2` |
| **Σ** |  |  | — | **`+0`** | **`+0`** | **`+0`** | **`+0`** | **`+1`** | **`+1`** | **`+0`** | **`2`** |

## Notes

The only prose outside the setup and legend above: pitfalls, findings, and the
context each table cannot carry. One `###` subsection per finding, titled by
what it is about. Describe the EXPERIMENT and its results — a fix made to
crustify or ffibox along the way belongs in that repo's history, not here.

> Gate misses and anything the oracle and `translate` disagreed on; a wave that
> was superseded and why; what each wave's diff actually contained beyond its
> row counts; where a metric moved and what moved it; what the judge found and
> whether it held. Everything else stays in the tables.

State where the LoC figures come from, and note that all of them exclude
comments and blank lines.

`C LoC` is `wavefront query dag --name <every scheduled entity> --loc`:
the oracle's translated-LoC view, a function seed valued at its body LoC and a
type seed at its field and op count. It reports the seeds only, with no closure
expansion, so it is the C the campaign translated rather than the surface it
was drawn from. Give the defining files' and the whole target's raw totals
beside it for scale.

`Rust LoC` is counted from source over the authored `.rs` files of the workspace, `src/`,
`tests/` and the `-sys` crates, excluding anything generated into `target/`,
and split by
`#[cfg(test)]` module.

Say why that non-test figure differs from the `code_lines` ffimap
reports. The audit measures the union of HIR definition spans, so it counts
only what sits inside an item — no `use`, `mod` or free-standing attribute
lines — and by construction cannot see `cfg`-disabled code, which is why it
yields no test figure. Its number is the right denominator for the unsafe
ratios and the wrong one for how much Rust was written; the two must not be
added.

Say plainly that the Rust-to-C ratio is not like-for-like in any measure,
because the Rust carries tests, `// SAFETY:` justifications, `ffi_export`
gateways and scaffolding with no C counterpart.

The four unit counts are de-duplicated over ENTITIES, not scheduled units: an
entity appears once, under the last objective it ran, so a type wrapped and
later ported counts as ported and never in both. Name the entities that took
both paths. Callbacks count with symbols.

Some of it is structural and belongs here every time: whether a review wave
reused the translation batches or the oracle re-batched it under distinct caps;
which units a review schedule dropped and why; which modules the Overview
lists but no table details, and the cost that leaves unaccounted; and any column
a campaign could not fill, said once rather than left as a field of em-dashes.

### `struct tm` moved in from `crustify-rs/ffmpeg-bindings`

The wrapper's first type, `Tm` (`time/bits/types/struct_tm.h`), landed outside any wave:
`crustify-rs/ffmpeg-bindings`' libavutil core wave 2 (bprint) had written it by hand in
`bprint.rs`, as libc's type with no batch, and the user had it moved here unchanged on
2026-10-23, with its unit tests and its compile-fail test, plus two accessors
(`TmRef::as_tm_ptr`, `TmMut::as_mut_tm_ptr`) through which the parent's raw bindings
reach it. No agent ran, so it has no batch row and no cost; the side campaign's first
planner run plans it as wrapped.

### A second parent: `crustify-rs/libgit2-bindings`

On 2026-10-24 `crustify-rs/libgit2-bindings` took this wrapper for its libc, which it
links from the system as `crustify-rs/ffmpeg-bindings` does: the same glibc 2.39. Its
modules' dependency records count 871 functions, 95 structs and 2 globals of libc,
counted per module over the 168 modules that use it; they join this campaign's scope,
which still grows only as each parent's waves need more. The `scope` setting names
both parents' consumption.

### ffibox's seam traits

ffibox `7950244` (`main`, merged from `feat/public-handle-seam`) moves the handles' raw
seam (`from_ptr`, `as_ptr`, `as_mut_ptr` and their `void *` twins) from inherent
`pub(crate)` methods onto the traits `CRefSeam`, `CMutSeam` and `CLockedSeam`, so that a
parent holding this wrapper's handles can reach the pointer. `Tm`'s module imports the
traits it calls; its public API is unchanged.

### Settings changes of 2026-11-03

As its parents, `crustify-rs/ffmpeg-bindings` and `crustify-rs/libgit2-bindings`, do, the
settings leave the parallelism to the assistant, which decides it from the host's resources
and the workloads sharing it, and name the campaign's commit identity: Marius Momeu as the
author, with Crustify as the co-author.

### Wave 2: `FILE`, and the cells a system libc leaves empty

`libio/genops.c`'s wave 2 binds `_IO_FILE` and its typedef `FILE` as one opaque type,
`File`, with its borrowed handles and no owner. Its row's `+LoC` counts the regenerated
raw bindings (153 lines) with the safe module and the tests. Its three `+UB safe tests` are
compile-fail cases of the one `compile_fail` test, so the suite's `#[test]` count in
`Coverage stats` does not move. glibc is not built, so every C coverage cell is `—`. The
Rust figure is line coverage over `src/`, measured on the pinned nightly with
`-C instrument-coverage`. `File`'s code is the expansion of ffibox's `define_ctype!`,
which `llvm-cov` counts at the macro's source, outside `src/`, so the batch moves no Rust
figure. `C LoC` stays unmeasured: there is no wavefront database over glibc. `Rust LoC`
counts nonblank, noncomment lines of `src/`, `tests/` and `libc-sys/` (its generated
`bindings.rs` left out), split at `#[cfg(test)]` modules and `tests/`. The safety audit's
`C by value` (+1) is `File`'s layout field, and its `Drop/Clone` (+1) the `Clone` that
`FileRef` derives.

### Wave 2's module pass: `File`'s thread-safety claim

`libio/genops.c`'s only review, `wave-2/batch-1`, judged `File`'s `Send` and `Sync` sound.
They are ffibox's shape for a C object that locks itself: every operation on `FileRef` holds
`flockfile`, and `FileRef` touches nothing of the stream today. The review corrected two
justifications. One was the contract for a borrowed stream, which claimed glibc's own stdio
routines always hold the stream's lock: they skip it on a `FSETLOCKING_BYCALLER` stream.
The other was `Send for File`'s, which misstated its scope. It also fixed one convention,
an `unsafe_code` expectation on the unit tests, and added a compile-fail case: no `File`
from a stream's layout. Its row's `+LoC` counts those `.rs` changes. Its reports sit in
this table's columns as the dashboard has them: the two `contracts` findings of the
`ffi_import` region under `+internal`, since the dashboard predates the template's
`+contracts import` column. The dashboard also lacks the template's `Module passes` and
`Batches: API stability` tables, which a batch does not add: the module pass fills its row
in the Types tables, and its API measure (`ffimap api diff`) keeps both items. The Rust
coverage was measured again, over `src/`, as the earlier batch measured it: `100.00`% (50
of 50 lines), unchanged, since the landing changes no executable line. No safety-audit
counter moved.

### Wave 3: `fprintf`, written under the stream's lock

`stdio-common/fprintf.c`'s wave 3 binds `fprintf` as `FileRef::print(&self, &str) ->
io::Result<usize>`, the fixed-arity form: Rust formats, and the text goes to
`fprintf(stream, "%s", text)` between `flockfile` and `funlockfile`. So it lands in the
stream's own buffer and holds the lock whatever the stream's locking mode. The two lock
routines are raw bindings inside the region, with no API of their own. A `write_fmt` method
beside it lets `write!` format into one locked call, which returns the count as C's does. It
carries no anchor of its own, and `ffimap api` counts it as an extension of `st::_IO_FILE`
and `tdf::FILE`, whose handles it is a method of. `fn::fprintf` is `new`, and the two
unattributed modules are kept. The dashboard has no `Batches: API stability` table to hold
that measure.

The binding clears `errno` before the call and restores it unless the call failed and set
it. glibc returns `-1` without setting `errno` on a wide-oriented stream and when a cookie
stream's write fails silently, as `fmemopen`'s does once its storage is full, so a failure
that leaves `errno` at 0 is an `io::ErrorKind::Other` error with no OS code. The
equivalence suite checks this against C on the same streams.

The row's `+LoC` and `loc` count the regenerated raw bindings (63 lines: `fprintf`,
`flockfile`, `funlockfile`, and the stream routines and constants the tests open streams
with) with the module and the tests. The tests' stream fixture, `tests/fixtures/stream.rs`,
is shared by `unit_pub` and `equiv_pub`. `+UB safe tests` is `+0`: no safe producer of a
`FileRef` exists until wave 4's `stderr`, so the batch's tests open their streams through
the raw bindings. The suite's `-24.28` pp comes from the denominator alone: it covers the same
lines over 86 lines rather than 50. Rust line coverage stays at `100.00`% (86 of 86 lines,
101 of 101 regions; `fprintf.rs` 36 of 36), measured as wave 2's was. Branch coverage is not
instrumented, so no branch figure exists. `EOVERFLOW`, for a text over `INT_MAX` bytes, is the
one C outcome no test reaches, as it needs more than 4 GiB. The safety audit's `unsafe loc %`
(`-6.20` pp) and `unsafe blocks` (`+3`) are the region's three calls. No public `unsafe fn` and
no raw pointer reaches the safe interface.

### Wave 3's module pass: a stream with no lock

`stdio-common/fprintf.c`'s only review, `wave-3/batch-1`, judged `FileRef::print` sound on
the points its task named. The lock is released on every path: nothing between `flockfile`
and `funlockfile` can unwind, short of a thread cancellation inside `fprintf`'s write, a
forced unwind Rust does not permit through a frame that owns a `CString`. `errno` is right
on every path the binding reaches, as the `errno-failure-without-errno` proposal now
records. The NUL refusal is the one `binding-symbols` asks of a string. `write_fmt`
correctly carries no anchor, since `conventions` repeats one only for ownership variants
and accessors.

The review found one stream the binding could not take. glibc hands the handler of a
`register_printf_specifier` conversion a stream with a NULL lock, and `flockfile`
dereferences it. The binding now refuses such a stream with `io::ErrorKind::Unsupported`.
That finding is a `contracts` one of the `ffi_import` region, under `+internal`, as wave
2's were. The other finding is a `conventions` one: two `unit_pub` tests called the C
outside the fixture.

The row's `+LoC` and `loc` count the `.rs` changes of `src/` and `tests/`. The
reproducer's 71 lines, under `crustify/reviews/`, are not part of the crate and are left
out. The one test added is a unit test, through a new `unit_pub`-only fixture,
`tests/fixtures/stream_unit.rs`. Rust line coverage stays at `100.00`% (96 of 96 lines, 109
of 109 regions; `fprintf.rs` 46 of 46), measured as wave 2's was. llvm-cov counts std's
`thread_local` code, which the fixture instantiates, under the toolchain's sources, and the
measure leaves it out. The `UB safe` (`-3.51` pp) and `equiv` (`+2.14` pp) moves come
from the new lines alone: the equivalence suite reaches `has_lock` on every stream. The
`Rust LoC` line moved by the delta of one counter run over the base and the landed tree,
`+10` non-test and `+90` test lines. That counter does not reproduce the earlier absolute
figures exactly. The safety audit's `+1` unsafe block is `has_lock`'s read of `_lock`.
`ffimap api diff` keeps `fn::fprintf` (`kept`) and both unattributed modules.

