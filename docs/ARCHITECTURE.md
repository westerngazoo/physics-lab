# Architecture

The current-state reference for physics-lab. This document describes
**what is**, normatively; the *decisions* that got here — including
three owner-review rounds — live in [RFC-001](RFC-001-lesson-framework.md)
and are not repeated. When they disagree, this file is stale: fix it.

---

## 1. The ecosystem

physics-lab is one of four repos with one owner and one rule spanning
them — *closed form when it exists; honest integration with measured
bounds when it doesn't*:

| Repo | Role | Physics regime |
|---|---|---|
| **garust** | the GA kernel: `Cl(P,Q,R)` generic multivectors, motors, rigid-body dynamics | — |
| **motoreel** | the film studio: deterministic *offline* rendering of motion, keyframed or simulated | no closed form → integrate; bit-determinism + measured error bands |
| **physics-lab** | the classroom: *live* interactive lessons | closed forms only; exactness |
| **goosethropic** | the front door site; links here | — |

The seam: motoreel films may enter lessons as assets with provenance
(commit, requirement id, determinism statement). The classroom never
re-implements the studio, and never runs an integrator (that rule is a
pedagogy contract — scrub-safety and honesty — not an engine limit).

## 2. The stack

```
public/lessons/<slug>/index.html      21-line stub (never edited twice)
        │ loads
public/js/runtime.js                  THE one JS file, written once
        │  builds the entire page from lesson.json + notes.html,
        │  generates controls/readouts/stepper, owns the clock,
        │  writes params into wasm memory, paints prims back
        │            [ uniform C-ABI ]
public/lessons/<slug>/lesson.wasm     the physics (committed binary)
        │  = lessons/<slug>/crate: a model + ONE draw() + claims-as-tests
lessons-common                        the framework's Rust half:
        │  lesson! macro (generates the whole ABI), Prims/Readouts writers
garust (sibling checkout)             where a lesson's math is geometric
```

Everything above the ABI is data or the shared runtime; everything
below it is Rust that `cargo test` runs natively — the same bits the
browser executes. "One source of truth" is a tautology here, not a
discipline.

## 3. The uniform wasm ABI

Generated identically for every lesson by `lessons_common::lesson!(draw)`:

| Export | Meaning |
|---|---|
| `params_ptr() -> *mut f64` | the runtime writes parameter values here, **in manifest order**, pre-multiplied by each param's `scale` |
| `state_at(n_params) -> usize` | recompute everything; returns prim-buffer length in f64s. **Pure over the params**: same values, same buffers, bit for bit |
| `prims_ptr() -> *const f64` | the primitive buffer |
| `readouts_ptr() -> *const f64` | 8 readout slots |

Capacities (in `lessons-common`): `PRIM_CAP = 8192` f64s,
`READ_SLOTS = 8`, `PARAM_CAP = 16`. No allocator exports, no
wasm-bindgen, no imports: a lesson wasm instantiates with `{}`.

### The primitive records

Flat f64 records, motoreel's drawing vocabulary as a convention:

| Record | Layout |
|---|---|
| point | `[0, x, y, style]` |
| segment | `[1, x1, y1, x2, y2, style]` |
| polyline | `[2, n, x0, y0, …, x(n-1), y(n-1), style]` |
| arrow | `[3, x1, y1, x2, y2, style]` (runtime draws the head) |
| view switch | `[9, viewIndex]` — routes subsequent records |

Coordinates are world units of the *current view*. `style` indexes the
manifest's `styles` array. The `Prims` writer (`view/point/segment/
arrow/polyline/curve`) is the only sanctioned producer; `curve(t0, t1,
n, style, f)` is the workhorse for trajectories, graphs and level sets.

## 4. `lesson.json` — the manifest schema

Everything a page is, as data. Fields marked ○ are optional.

| Field | Meaning |
|---|---|
| `slug`, `title`, `topic` | identity; `topic` shows on the hub card |
| ○ `eyebrow`, `lede` | header strip and intro paragraph (HTML allowed in `lede`) |
| `views[]` | one per stage: `world {x0,x1,y0,y1}`, `viewBox {w,h}`, ○ `wide` (span the grid), ○ `title`/`law` (stage header), ○ `uniform: false` — permitted **only** where axes carry different quantities (phase portrait, graph); geometry views must scale both axes alike or the runtime throws at load |
| `params{}` | ordered; each: `label, min, max, step, value`, ○ `unit`, ○ `digits`, ○ `scale` (multiplier applied before the ABI — e.g. τ so sliders read in turns), ○ `widget: "hidden"` (no slider; e.g. the stepper's param) |
| ○ `sweep` | `{param, rate, label}` — the Play button animates that param, wrapping over its range |
| `readouts[]` | `{slot, label, fmt, ○hero}`; `fmt ∈ fix3 | turns3 | sci` |
| `styles[]` | `{var, ○width, ○dash}` — `var` is a tokens.css custom property |
| ○ `legend[]` | `{style, label}` swatch rows |
| ○ `claims[]` | `{id, text, ○test}` — rendered up top; `test` names the cargo test enforcing it, tying the page to CI |
| ○ `steps[]` + `stepParam` | derivation stepper: `{title, html}` panels; the named (hidden) param carries the current step into `draw` for per-step highlights |
| ○ `tryThis[]` | predict-then-check prompts |
| ○ `notes` | filename of the prose fragment to fetch (usually `notes.html`) |
| ○ `footer`, `card`, `cardClaim` | page footer text; hub-card blurb and claim line |

## 5. The runtime (`public/js/runtime.js`)

The one JS file, ~430 lines, written once. Responsibilities, in page
order: fail loudly (visible alert strip; `file://` refused with the
serve command), load manifest + notes + wasm, build the scaffold
(header → claims → stages → stepper → transport bar → legend → notes →
try-this → footer), generate controls and readouts, enforce the
per-view scale guard, run the sweep clock, and per frame: params →
wasm memory → `state_at` → paint the prim buffer.

Why JS exists at all: the browser exposes DOM/events/fetch/rAF to
JavaScript only — this is the syscall shim, not an application layer.
Queued shrink (RFC-001): scaffold and painter move to Rust as emitted
markup strings, leaving a ~80-line pipe. `public/js/hub.js` renders the
hub cards from `lessons/index.json`; `public/js/lab.js` + the
`examples/*/js` files are legacy and die with the remaining ports.

## 6. The hub and the classroom

`tools/gen-index.py` walks `public/lessons/*/lesson.json` (plus a
static legacy list) into `public/lessons/index.json`; the hub renders
itself from it — adding a lesson never edits the hub.
`public/classroom/index.html` is the course (units, objectives,
predict-then-check exercises, break-a-claim homework);
`public/classroom/authoring.html` is the authoring guide, which embeds
the **actual shipped source** of `lessons/projectile/` (the minimal
template) so it cannot drift from reality.

## 7. Verification

| Layer | What | Where |
|---|---|---|
| L1 | `cargo test --workspace` — every claim, on the same crates the browser runs; plus `clippy -D warnings` and the wasm build | native + CI |
| L2 | `checks/run.py` — independent Python implementations of the flagship claims (N-version) | native + CI |
| L3 | drive the deployed/served pages in a real browser: readouts against theory, controls, error strips. Screenshots don't count as behavior | manual |

CI (`.github/workflows/checks.yml`) runs L1 + L2 on every push with
garust checked out as the sibling the path-deps expect. Green CI means
every quantitative claim on the site held, in public.

## 8. Toolchain and operations

- **Sibling checkout**: lesson crates use `garust = { path =
  "../../../../garust" }` — clone garust beside physics-lab.
- **The Homebrew landmine**: on the dev machine, Homebrew Rust shadows
  rustup in PATH and ships no cross-targets; *mixing* them breaks
  clippy (E0514). Always prepend the toolchain:
  `export PATH="$(dirname "$(rustup which rustc)"):$PATH"` —
  `tools/build-wasm.sh` does this itself.
- **Build**: `sh tools/build-wasm.sh` compiles every `lessons/*/crate`
  to `wasm32-unknown-unknown --release` and stages binaries into
  `public/`. Built `.wasm` files are **committed** (owner decision:
  tens of KB, deployable from a bare checkout).
- **Serve locally**: `cd public && python3 -m http.server 8000`. To
  exercise the real headers, use a CSP-applying server (CI-adjacent
  scripts exist in session tooling; `_headers` is the source of truth).
- **Deploy**: `npx wrangler deploy` (Workers Static Assets, no worker
  script; `_headers` carries the CSP). Custom domain
  `physics.goosethropic.systems` attaches once in the Cloudflare
  dashboard (owner-only).
- **CSP**: `default-src 'self'`; `script-src 'self' 'wasm-unsafe-eval'`
  (the one loosening the architecture costs — permits wasm
  compilation, not JS eval); fonts self-hosted; no third-party
  requests anywhere.

## 9. Invariants, and the bugs that bought them

Each rule below is enforced somewhere (a throw, a test, a generated
structure) because prose-only rules rot:

1. **Uniform axes in geometry views** (runtime throws) — a silently
   skewed world box once bent every ray angle 6%.
2. **Failures render on the page** (alert strip; `file://` refused) —
   a lesson once shipped that painted perfectly and was dead: the
   script died after first paint, before any listener attached.
3. **Verify by driving, not reading** (L3) — the dead page above passed
   a screenshot check; a missing `</script>` passed code review.
4. **Exit codes over green banners** — a piped `tail` once swallowed a
   red clippy; a commit then claimed it green. Clippy is in CI now so
   the claim never rests on say-so.
5. **Models are pure over params** (the ABI admits nothing else) — no
   accumulated state means scrubbing cannot drift, ever.
6. **τ is the circle constant**; π appears only for a genuine half-turn.
7. **Labels carry their anchors** ("1 elastic · 0 beanbag") — a slider
   the student cannot define is a slider they cannot learn from.
8. **Approximations put their error on screen** (the wave dissector) —
   with a control that breaks them on purpose.
9. **Docs embed real source or none** (authoring guide) — listings
   drift; shipped files cannot.

## 10. Adding things

- **A lesson**: follow `public/classroom/authoring.html` (crate + manifest
  + notes + stub, then `build-wasm.sh` + `gen-index.py`). The hub and CI
  pick it up unaided.
- **A runtime capability**: only if it's data-drivable from the
  manifest and useful to ≥2 lessons; it's written once and never
  per-lesson. Everything else belongs lesson-side in Rust.
- **Retiring a legacy example**: delete `public/examples/<name>`, drop
  its entry from `tools/gen-index.py`'s legacy list, regenerate.
