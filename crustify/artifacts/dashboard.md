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
- **Rust LoC** — `517` non-test, `1642` tests; **C LoC** — `<n>`

### Coverage stats

| | count | share | C LoC coverage | Rust LoC coverage |
|---|---:|---:|---:|---:|
| UB safe tests | `6` | — | — | `20.14`% |
| UB unsafe tests | `2` | — | — | `2.78`% |
| Equiv tests | `20` | — | — | `61.11`% |
| Unit tests | `47` | — | — | `97.22`% |
| **Total tests** | **`75`** | — | — | **`100.00`%** |
| total types | `2` | — | — | — |
| ported types | `0` | `0`% | — | — |
| wrapped types | `2` | `100`% | — | — |
| total symbols | `4` | — | — | — |
| ported symbols | `0` | `0`% | — | — |
| wrapped symbols | `3` | `75`% | — | — |

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
| 2 | `1` | `0` | none | `3` | `$<n>` | `<n>M` | `<n>m<n>s` |
| **Σ** | **`6`** | **`0`** | | **`3`** | **`$<n>`** | **`<n>M`** | **`<n>m<n>s`** |

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
| `libc/libio/stdio.c` | `4` | `0` | wrap | first | `1` | `172` | `+172/-0` | — / `100.00`% | `$<n>` | `<n>M` | `<n>m<n>s` | `$<n>` | `$<n>` |
| `libc/libio/iofwrite.c` | `5` | `0` | wrap | first | `1` | `954` | `+954/-50` | — / `100.00`% | `$<n>` | `<n>M` | `<n>m<n>s` | `$<n>` | `$<n>` |
| `<link-unit>/<module>` | `<n>` | `<n>` | port | first | `<n>` | `<n>` | `+<n>/-<n>` | `<n>`% / `<n>`% | `$<n>` | `<n>M` | `<n>m<n>s` | `$<n>` | `$<n>` |
| `libc/stdio-common/fprintf.c` | `3` | `1` | wrap | review | `1` | `199` | `+199/-31` | — / `100.00`% | `$<n>` | `<n>M` | `<n>m<n>s` | `$<n>` | `$<n>` |
| `libc/libio/stdio.c` | `4` | `1` | wrap | review | `1` | `196` | `+196/-112` | — / `100.00`% | `$<n>` | `<n>M` | `<n>m<n>s` | `$<n>` | `$<n>` |
| `libc/libio/iofwrite.c` | `5` | `1` | wrap | review | `1` | `246` | `+246/-11` | — / `100.00`% | `$<n>` | `<n>M` | `<n>m<n>s` | `$<n>` | `$<n>` |
| **Σ first** |  |  |  |  | **`3`** | **`1947`** | **`+1947/-51`** | **— / `100.00`%** | **`$<n>`** | **`<n>M`** | — | **`$<n>`** | **`$<n>`** |
| **Σ review** |  |  |  |  | **`3`** | **`641`** | **`+641/-154`** | **— / `100.00`%** | **`$<n>`** | **`<n>M`** | — | **`$<n>`** | **`$<n>`** |

### Batches: tests

`openrouter/anthropic/claude-opus-5.5` via `claude`.

| module | wave | batch | pass | +UB safe tests (+C/+Rust pp) | +UB unsafe tests (+C/+Rust pp) | +equiv tests (+C/+Rust pp) | +unit tests (+C/+Rust pp) |
|---|---|---|---|---|---|---|---|
| `libc/stdio-common/fprintf.c` | `3` | `0` | first | `+0` (—/`-24.28` pp) | `+0` (—/`+0.00` pp) | `+6` (—/`+39.53` pp) | `+16` (—/`+0.00` pp) |
| `libc/libio/stdio.c` | `4` | `0` | first | `+2` (—/`+3.11` pp) | `+0` (—/`+0.00` pp) | `+1` (—/`+7.41` pp) | `+2` (—/`+0.00` pp) |
| `libc/libio/iofwrite.c` | `5` | `0` | first | `+0` (—/`-8.86` pp) | `+0` (—/`-1.22` pp) | `+10` (—/`+17.11` pp) | `+20` (—/`+1.22` pp) |
| `libc/stdio-common/fprintf.c` | `3` | `1` | review | `+0` (—/`-3.51` pp) | `+0` (—/`+0.00` pp) | `+0` (—/`+2.14` pp) | `+1` (—/`+0.00` pp) |
| `libc/libio/stdio.c` | `4` | `1` | review | `-1` (—/`-7.41` pp) | `+2` (—/`+7.41` pp) | `+1` (—/`+0.00` pp) | `-2` (—/`-7.41` pp) |
| `libc/libio/iofwrite.c` | `5` | `1` | review | `+0` (—/`+0.00` pp) | `+0` (—/`+0.00` pp) | `+2` (—/`+0.00` pp) | `+3` (—/`+0.00` pp) |
| **Σ first** |  |  |  | **`+2` (—/`-30.03` pp)** | **`+0` (—/`-1.22` pp)** | **`+17` (—/`+64.05` pp)** | **`+38` (—/`+1.22` pp)** |
| **Σ review** |  |  |  | **`-1` (—/`-10.92` pp)** | **`+2` (—/`+7.41` pp)** | **`+3` (—/`+2.14` pp)** | **`+2` (—/`-7.41` pp)** |

### Batches: safety audit

Deterministic `ffimap`; no model.

| module | wave | batch | objective | pass | pub raw ptr (safe) | pub raw ptr | pub void ptr | pub unsafe fn | C borrow | slice borrow | unsafe loc % | unsafe blocks | Send/Sync | C by value | Drop/Clone | deref outside impl |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `libc/stdio-common/fprintf.c` | `3` | `0` | wrap | first | `±0` | `±0` | `±0` | `±0` | `±0` | `±0` | `-6.20` pp | `+3` | `±0` | `±0` | `±0` | `±0` |
| `libc/libio/stdio.c` | `4` | `0` | wrap | first | `±0` | `±0` | `±0` | `±0` | `±0` | `±0` | `-0.43` pp | `+2` | `±0` | `±0` | `±0` | `±0` |
| `libc/libio/iofwrite.c` | `5` | `0` | wrap | first | `±0` | `±0` | `±0` | `±0` | `±0` | `±0` | `-4.10` pp | `+3` | `±0` | `±0` | `+2` | `±0` |
| `<link-unit>/<module>` | `<n>` | `<n>` | port | first | `±<n>` | `±<n>` | `±<n>` | `±<n>` | `±<n>` | `±<n>` | `±<n>` pp | `±<n>` | `±<n>` | `±<n>` | `±<n>` | `±<n>` |
| `libc/stdio-common/fprintf.c` | `3` | `1` | wrap | review | `±0` | `±0` | `±0` | `±0` | `±0` | `±0` | `-0.70` pp | `+1` | `±0` | `±0` | `±0` | `±0` |
| `libc/libio/stdio.c` | `4` | `1` | wrap | review | `±0` | `±0` | `±0` | `+1` | `±0` | `±0` | `±0.00` pp | `±0` | `±0` | `±0` | `±0` | `±0` |
| `libc/libio/iofwrite.c` | `5` | `1` | wrap | review | `±0` | `±0` | `±0` | `±0` | `±0` | `±0` | `±0.00` pp | `±0` | `±0` | `±0` | `±0` | `±0` |
| **Σ first** |  |  |  |  | **`±0`** | **`±0`** | **`±0`** | **`±0`** | **`±0`** | **`±0`** | **`-10.73` pp** | **`+8`** | **`±0`** | **`±0`** | **`+2`** | **`±0`** |
| **Σ review** |  |  |  |  | **`±0`** | **`±0`** | **`±0`** | **`+1`** | **`±0`** | **`±0`** | **`-0.70` pp** | **`+1`** | **`±0`** | **`±0`** | **`±0`** | **`±0`** |

### Batches — review: PoC + reports

`<provider>/<model>` via `<backend>`.

| module | wave | batch | +lifecycle | +unsafe | +UB safe | +UB unsafe | +equiv | +internal | +conventions | +misc | total |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `libc/stdio-common/fprintf.c` | `3` | `1` | — | `+0` | `+0` | `+0` | `+0` | `+1` | `+1` | `+0` | `2` |
| `libc/libio/stdio.c` | `4` | `1` | — | `+0` | `+0` | `+0` | `+0` | `+1` | `+0` | `+0` | `1` |
| `libc/libio/iofwrite.c` | `5` | `1` | — | `+0` | `+0` | `+0` | `+1` | `+0` | `+0` | `+0` | `1` |
| **Σ** |  |  | — | **`+0`** | **`+0`** | **`+0`** | **`+1`** | **`+2`** | **`+1`** | **`+0`** | **`4`** |

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


### Wave 4: `stderr`, the first symbol

`libio/stdio.c`'s wave 4 binds the global `stderr` as `stderr() -> FileRef<'static>`, which
reads C's `stderr` on every call, as C's expression does, and borrows the stream it names
for the rest of the program. Its doc states what the `'static` borrow assumes: no write to
`stderr` while another thread may read it, no `fclose` or failing `freopen` of the stream,
and the stream's internal locking left alone. No C of glibc's library or of either parent
breaks these. The raw bindings gain `var stderr`, and the row's `+LoC` counts their 3 lines
with the safe module and the tests. The anchor ``/// Wraps `glb::stderr` `` carries no
`#[doc(alias)]`, because rustc rejects an alias equal to the item's name; proposal
`doc-alias-equal-to-name` covers this. Its tests are 2 `ub_safe`, 2 `unit_pub` and 1
`equiv_pub`, the first `equiv_pub` suite. All of them read the C global, so they carry Miri's
ignore mark. The Rust figures were measured as before, over `src/`, now 54 lines, all
covered. A suite's figure is the share of those 54 lines that suite covers alone. The
safety audit's `unsafe blocks` (+2) are the global's read and the stream's borrow, both in
`ffi_import`. No other metric moved. The API measure, `ffimap api diff`, finds
`glb::stderr` `new` and the other modules `kept`.

### Wave 4's module pass: `stderr` becomes an unsafe accessor

`libio/stdio.c`'s only review, `wave-4/batch-1`, found the safe `stderr() ->
FileRef<'static>` unsound. glibc's `stderr` is a variable its manual lets a program set,
and its example closes a standard stream and points the variable at one `fopen` opened.
`fclose` frees such a stream, so a kept `'static` handle outlives it. The reproducer shows
a heap-use-after-free under ASan from the safe call. The finding is a `contracts` one of the
`ffi_import` region, filed under `+internal` as wave 2's are. The fix makes `stderr` `unsafe
fn stderr<'a>() -> FileRef<'a>`, its `# Safety` section moving onto the caller what the
safe function assumed of the program. `ffimap api diff` classifies `glb::stderr`
`changed`, kept for soundness. `st::_IO_FILE` and `tdf::FILE` are `kept`. The safety audit's
`pub unsafe fn` (+1) is that accessor. No other metric moved. The tests moved with the API:
the two `ub_safe` tests went to a new `ub_unsafe` suite, and the two `unit_pub` tests and
their suite were removed, since `unit_pub` forbids `unsafe_code` and `equiv_pub` asserts the
same against C. `equiv_pub` gained a test that C's reassignment of `stderr` is followed, for
which the raw bindings gain `fopen` and `fclose`, test-only. A compile-fail case keeps the
safe call rejected. Hence the tests row's negative cells: each suite's count and Rust
figure are as measured over `src/` (54 lines) before and after. `stdio.rs`'s 4 lines moved
from the `ub_safe` and `unit_pub` runs to `ub_unsafe`'s, and the joint figure stays
`100.00`%. The row's `+LoC` counts the Rust tree's `.rs` changes, the regenerated
bindings' 9 lines included. The reproducer's 78 lines, in the reviews dir, are left out.

### Integration of `stdio-common/fprintf.c` after `libio/stdio.c`

The merge keeps every row both modules added. The `Σ` rows of the Symbols tables and the
Overview's `Rust LoC` are ours plus theirs less the merge base's; the Coverage stats' counts
are the merged suites', with `fprintf.c`'s percentages, the later landing's, and `stdio.c`'s
for `ub_unsafe`, until a gate measures the merged tree.

### Wave 5: `fwrite`, the byte-exact write

`libio/iofwrite.c`'s wave 5 binds `fwrite` as `FileRef::write_all(&self, &[u8]) ->
io::Result<()>`. It calls `fwrite(bytes, 1, len, stream)` between `flockfile` and
`funlockfile`, so the bytes land in the stream's buffer as they are, NUL and invalid UTF-8
included. It is the write the parent's `FileWriter` (`src/git2/src/cli/opt.rs`) makes, one
call per `io::Write::write`, without the U+FFFD it substitutes today. The planner's "a short
count on failure" became an error. A short count is how `fwrite` reports a failure, and after
a failed flush the stream has discarded what its buffer held, so a caller that retries the
rest, as `write_all` would, writes on as if nothing had failed. `Ok` is therefore always the
whole slice, and `binding-symbols` leaves it out of the return. The count is lost on failure,
as `print`'s is.

`fwrite` returns its full count when the bytes fit a line-buffered stream's buffer, end
with a newline, and the flush that newline calls for fails. `fprintf` fails there. The
binding reads the stream's error indicator with `ferror` before and after the call, under
the same hold of the lock, and reports that write as failed when the indicator goes from
clear to set. An indicator already set by an earlier, uncleared failure hides that one
case. The binding's doc says so, and a unit test keeps the behaviour. The parent's export
differs from C's `fprintf` there alone. A wide-oriented stream gets `0` from `fwrite`, with
neither `errno` nor the indicator set, so the binding reports `Other`, as `print` does. An
empty slice succeeds without touching the stream, as `fwrite` of no bytes returns at once.
`fprintf(stream, "%s", "")` orients the stream or fails, so the equivalence tests against
`fprintf` write no empty text.

The lock moved to `File`'s home. `FileRef::lock` (`genops.rs`, crate-private) checks
`has_lock` and takes `flockfile`. The `StreamLock` hold it returns releases the lock with
`funlockfile` when dropped, unwinding included, and is neither `Send` nor `Sync`. `print`
now takes it too: `fprintf.rs` loses its own `has_lock`, `flockfile` and `funlockfile`, and
its tests pass unchanged. With the lock routines made no-ops, both modules' threaded and
lock-waiting tests fail.

The row's `+LoC` and `loc` count the `.rs` diff: the regenerated raw bindings' 20 lines
(`fwrite`, and `setvbuf`, `clearerr` and `_IOLBF`, test-only), the module, the lock's move and
the tests. Tests: 10 `equiv_pub`, against the raw `fwrite` through the mapping the binding
documents and against `fprintf(stream, "%s", text)`, on growing, file, line-buffered,
by-caller-locked, read-only, full, failing and wide streams. Also 17 `unit_pub` and 3
`unit_priv`. They run plain and under ASan (the workspace), the two stream suites under TSan,
two named tests under BSan, and every new test alone under Miri. The 27 that call C stop at
their first foreign call there and carry the mark. The fixture gains `line_buffered`,
`clear_error` and `orientation`. Rust coverage is measured over `src/` as before, on the
merged tree, which no gate had measured since `fprintf.c`'s integration: `100.00`% (144 of
144 lines, 161 of 161 regions; `iofwrite.rs` 33 of 33), from `100.00`% (100 of 100). The
suites' moves are their figures on the base and the tree, each against its own denominator.
`Coverage stats` now holds this measurement for every suite. Unit is `unit_priv` and
`unit_pub` together. Doctests and the compile-fail case, which fails in the marked copy,
count in no row. The one `fwrite` path no test reaches is `_IO_vtable_offset`'s, which is
nonzero only for streams of binaries linked against libio's old ABI. The safety audit's
`unsafe blocks` (`+3`) are the two `ferror`s and the `fwrite`. `has_lock`, `flockfile` and
`funlockfile` moved, uncounted. `Drop/Clone` (`+2`) is `StreamLock`'s `Drop` and the
`Clone` the private `Call` derives. `ffimap api diff` finds `fn::fwrite` `new` and the other
four C items and both unattributed modules `kept`. `Rust LoC` moved by one counter's delta
over the base and the tree, `+82` non-test and `+561` test lines.

### Wave 5's module pass: a failed flush that sets no indicator

`libio/iofwrite.c`'s only review, `wave-5/batch-1`, judged the planner's "a short count on
failure" against what landed. It keeps the error. `fwrite` returns a short count only after
a failure, or on a wide stream. A failed flush discards what the buffer held, and the
parent's C stops at the first failed `fprintf`. A count would let `io::Write::write_all`
retry the rest, and lose the middle without a trace when the retry succeeds, as after
`EAGAIN`. The lock hold is released on every path of `print` and `write_all`, unwinding
included, and a new `unit_priv` test in `genops.rs` takes `StreamLock` directly: held,
recursive, and dropped on unwind. That test fails when the hold's `Drop` skips `funlockfile`.
The `ferror` before and after the call follows `binding-symbols`' Errors rule as it stands at
`cc84106`. The before-read only keeps an earlier failure from being reported as the call's.

One finding, `equiv`: a line-buffered flush can fail in the seek glibc makes before it writes
(`ESPIPE`, on a pipe or socket that C read and then wrote). That failure sets no error
indicator, so `write_all` reports the line written where `fprintf` fails. The code cannot
see it, so the doc now names it beside the already-set indicator, and two tests keep the
behaviour. The reproducer still exits `1`, by design. The parent's uses never read the
stream they write. The raw bindings gain `fdopen`, `fgetc`, `ESPIPE` and `_IONBF`, test-only.
The fixture gains a line-buffered socket stream read before it is written, and
`unbuffered`, glibc's mode for `stderr`. No test had written an unbuffered stream, so
`equiv_pub` and `unit_pub` each gain one (`test-adequacy`).

The row's `+LoC` counts the Rust tree's `.rs` changes, the bindings' 11 lines included. The
reproducer's lines are left out. Rust coverage is unchanged at `100.00`% (144 of 144 lines)
in every suite, measured on the base and the tree. A first measurement counted `genops.rs`'
test module as code, because its `#[expect]` spanned lines, which the marking script misses
(`crustify/evolution/coverage-marking-misses-multiline-attributes`). The attribute was
shortened to one line. No safety audit metric moved: the new unsafe code is in tests.
`ffimap api diff` keeps `fn::fwrite` and both unattributed modules. `Rust LoC` moved by the
counter's delta, `+11` non-test and `+158` test lines.
