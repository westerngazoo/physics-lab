# RFC-001 — From demo pile to learning framework

- **Status:** Proposal — awaiting owner review. Nothing here is built.
- **Owner:** Gustavo Delgadillo (westerngazoo)
- **Scope:** physics-lab's architecture, its pedagogic structure, and its
  seams with motoreel/garust/goosethropic.

**Mission, stated so the architecture can be judged against it:**
physics-lab is the interactive classroom of a **GA-first study program**.
garust is its mathematical kernel, motoreel its film studio, and the
conventional-notation lessons (optics, gravity, waves) are on-ramps — the
syllabus's destination is geometric algebra, and the house measures its
angles against **τ**. The first draft of this RFC omitted both; this
revision makes them structural (§3.6–3.7, §4.7).

---

## 1. The ecosystem — four repos, four jobs

The question "why another folder other than motoreel" deserves a real
answer, because the split is load-bearing:

| Repo | What it is | Language | Fidelity regime | Ships as |
|---|---|---|---|---|
| **garust** | The math kernel: geometric algebra generic over `Cl(P,Q,R)`, plus rigid-body dynamics on the motor group | Rust | `cargo test` + proptest laws | crates |
| **motoreel** | The *film studio*: deterministic offline rendering of motion — keyframed or simulated — to numbered frames | Rust | requirement loop, QA sign-off, byte-golden fixtures, measured error bands | frames → ffmpeg videos |
| **physics-lab** | The *classroom*: interactive lessons you scrub, drag, and interrogate live in a browser | JS/HTML, zero-build | closed forms only; assertion checks | static site (physics.goosethropic.systems) |
| **goosethropic** | The studio's front door; links out to both | HTML | — | static site |

**Why physics-lab is not a motoreel folder.** They produce different
artifacts under different physics regimes, and the regimes are the point:

- **motoreel exists for what has no closed form.** A tumbling body, a
  double pendulum — you must integrate, so its honesty discipline is
  *bit-determinism plus measured error bands* (energy drift −0.52% at
  dt = 1/960; constraint separation ≤ 1.03 mm — measured, then binding).
  Output is offline frames precisely so determinism is testable
  byte-for-byte.
- **physics-lab exists for what has a closed form.** Thin-lens imaging,
  the bouncing ball's geometric series, a chain's eigenmodes — here the
  honesty discipline is *exactness*: every on-screen state is a pure
  function of `(params, t)`, so scrubbing cannot drift and checks compare
  against the formula itself.
- One rule spans both: **closed form when it exists; honest integration
  with measured bounds when it doesn't.** The repos are the two halves of
  that sentence.
- Practically: Rust-writing-SVG-files vs browser-JS-under-strict-CSP are
  different runtimes, test benches, and release cadences. Embedding
  motoreel *live* in a page would mean compiling garust to WASM — a real
  option someday (§7), not a reason to merge today.

**The proposed seam** (new): motoreel → physics-lab, one-way, as
*assets*. A lesson on chaos embeds the twinned-pendulum film motoreel
renders; the asset ships with a provenance file (motoreel commit,
requirement id, determinism statement). The classroom shows films from
the studio; it never re-implements the studio.

---

## 2. What exists today — inventory and honest critique

```
public/
  js/lab.js                    world→screen (skew guard), el(), bind()   [70 lines]
  css/tokens.css, fonts/       Goosethropic brand, self-hosted
  index.html                   hub — hand-written cards
  examples/optics/             4 elements over one solver; traits table from data
  examples/bouncing-ball/      event-driven closed form; g slider
  examples/wave-equation/      exact eigenmodes; 4-step derivation stepper
checks/                        Python assertions per example + run.py
```

What's wrong, stated plainly:

1. **The checks guard a hand-port, not the running code.** Python
   re-implements math that lives separately in each page's JS. They can
   drift silently — the exact failure mode this lab claims to prevent.
2. **No model/view separation.** Each page's physics is welded into its
   DOM script. You cannot import "the thin-lens model" anywhere.
3. **Triplicated shell.** Playback loop, speed buttons, readout wiring,
   and stepper logic are written three times with small variations.
4. **No pedagogic structure.** Pages are exhibits, not lessons: no
   prerequisites, no objectives, no guided explorations, no misconception
   traps, no ordering. The hub is a hand-maintained list.
5. **No content model.** "A lesson" is not a thing any tool can read —
   metadata, claims, and controls live as ad-hoc HTML.

What's right and must survive: claim-first pages; exact models;
CONFIG-as-data; the skew guard; zero build, zero runtime deps, strict
CSP; the drive-it-in-a-browser verification habit.

---

## 3. Principles for the framework

1. **Claim-first pedagogy.** A lesson exists to make falsifiable
   statements and let the student check them — by interacting, and by
   running the same assertions we run.
2. **Models are pure.** `stateAt(params, t)` — no accumulated state, no
   integrators in the browser. If it needs an integrator, it is a
   motoreel film embedded as an asset, not a browser simulation.
3. **One source of truth for the math.** The model the browser executes
   is the model the checks import. Same file, same bits.
4. **Bespoke models, shared kernels, shared shell.** Physics is never
   flattened into generic engine data; everything *around* it (controls,
   readouts, playback, steppers, the hub) is declarative and generated.
   Genuine mathematical kernels — the world mapping, and the small GA
   algebra of §4.7 — are shared code, exactly like garust is to motoreel.
5. **Zero build, zero runtime dependencies, CSP unchanged.** ES modules
   give us imports without a bundler; Node runs the same modules for
   checks. Node becomes a *dev* dependency only.
6. **τ is the circle constant.** A full turn is one τ, matching garust
   and motoreel's standing convention ("angles are radians measured
   against TAU"). This governs prose, formulas, and code: mode angles are
   `kτ/(2(N+1))`, the wave chain's cutoff speed is `4/τ ≈ 64%` of c, and
   π survives only where a genuine half-turn is meant. The current
   lessons are retrofitted in S1.
7. **GA is the destination.** The lab exists inside a GA-first study
   program. The syllabus carries a first-class GA track (§4.7) whose
   reference implementation is garust itself — the classroom's algebra is
   held to the kernel's bits, not to a second opinion.

---

## 4. Proposed architecture

### 4.1 A lesson is a folder with a contract

```
lessons/<slug>/
  lesson.json        metadata + declarations (see schema below)
  model.mjs          PURE physics. No DOM, no imports from engine/.
  view.mjs           drawing only: (state, scene, world) → SVG
  notes.html         prose fragments: intro, derivation steps, notes
  check.mjs          imports model.mjs, asserts every claim. Runs in Node.
  assets/            optional: motoreel films + provenance.json
```

`lesson.json` (the pedagogic data model):

```json
{
  "slug": "wave-equation",
  "title": "Deriving the Wave Equation",
  "topic": "waves",
  "level": 2,
  "prereqs": ["bouncing-ball"],
  "objectives": ["see F=ma become y_tt = c^2 y_xx", "..."],
  "claims": [
    { "id": "C1",
      "statement": "Evolution matches an independent integrator to 1e-6",
      "checkedBy": "check.mjs#modes_vs_verlet" }
  ],
  "params": { "N": {"min":5,"max":80,"step":1,"value":24,"label":"Beads N"} },
  "readouts": [ {"id":"fid","label":"Pluck speed vs c","fmt":"pct1"} ],
  "tryThis": [
    "Set N=5 and watch the trailing wiggles — that is dispersion.",
    "Pluck dead centre: which modes vanish, and why?"
  ],
  "misconceptions": [
    { "belief": "more beads make the shortest waves reach c",
      "truth": "the lattice cutoff pins at 2/π of c forever; raising N fixes the waves a smooth pluck contains" }
  ]
}
```

`model.mjs` contract:

```js
export const defaults = {...};              // mirrors lesson.json params
export function build(params) {...}         // precompute (modes, tables)
export function stateAt(built, t) {...}     // pure; the ONLY time input
export function derived(built) {...}        // readout quantities
```

### 4.2 The engine — lab.js grows into six small modules

```
engine/
  world.mjs      the uniform mapping + skew guard        (exists)
  svg.mjs        element helpers                          (exists)
  controls.mjs   control panel GENERATED from lesson.json params
  readouts.mjs   readout strip generated from lesson.json
  playback.mjs   the one clock: owns t, rates, replay; calls stateAt
  stepper.mjs    derivation steps: shows notes fragments, drives view highlights
  lesson.mjs     loader: fetch lesson.json + notes, wire everything
```

Pages shrink to a shell: `index.html` declares containers and loads
`lesson.mjs` with the slug. Controls/readouts stop being hand-written
three ways. The playback module enforces principle 2 mechanically — no
lesson ever owns a clock again.

### 4.3 The pedagogic template — every lesson has the same spine

**Claim → Play → Dissect → Verify → Extend**

1. **Claim** — the falsifiable statements, from `lesson.json`, up top.
2. **Play** — the interactive stage: drag, scrub, sliders.
3. **Dissect** — the stepper (wave already has this; optics gets one
   walking the three principal rays; ball gets the series construction).
4. **Verify** — the claims again, each linked to its assertion, plus the
   one-liner to run them (`node checks/run.mjs`). The student can hold
   the page to account.
5. **Extend** — `tryThis` explorations and `misconceptions`, rendered
   from data.

### 4.4 The hub becomes a generated learning path

`tools/index.mjs` (run by hand or CI, output committed) walks
`lessons/*/lesson.json` into `lessons/index.json`. The hub page fetches
it (same-origin — CSP already allows) and renders: topic clusters, a
prerequisite ordering ("start here"), and each card's claims. Adding a
lesson = adding a folder; the hub cannot forget it or misdescribe it.

### 4.5 Verification — three layers, honestly divided

| Layer | What | Runs where |
|---|---|---|
| L1 | `check.mjs` per lesson — asserts claims against **the same model.mjs the browser runs** | Node ≥18, `node checks/run.mjs`, CI |
| L2 | The existing Python checks, **kept and reframed**: an independent second implementation of the flagship claims (N-version cross-check, the strongest kind) | `python3 checks/run.py`, CI |
| L3 | Page behavior: the browser drive (clock advances, controls bound, culls hold) + `world.mjs`'s load-time skew guard | manual today; scriptable later |

A GitHub Action runs L1 + L2 on every push. The README's claims section
points at the workflow badge instead of at prose.

### 4.6 The motoreel seam, concretely

`lessons/chaos/assets/twin-pendulum/{frames or .mp4, provenance.json}`
where provenance records: motoreel commit, requirement (R-0005), dt,
parameters, and the determinism statement ("bit-identical re-render,
same machine"). The lesson's claims cite the film's *measured* bands.
This is how integrator-physics enters the classroom without smuggling an
integrator into the browser.

### 4.7 The GA track — the point of the whole thing

**A tiny exact kernel, `engine/ga2.mjs`.** Deliberately small: `Cl(2,0)`
— four-component multivectors `(1, e1, e2, e12)`, the geometric product
as a 4×4 table, reverse, rotor `exp` — and, in a second stage, planar PGA
`Cl(2,0,1)` for points, lines, meet/join, and 2D motors. On the order of
150 lines, exact arithmetic, pure functions. This is a *kernel* in the
§3.4 sense: shared like `world.mjs`, never "physics flattened to data".

**Held to garust's bits.** For GA lessons, layer L2 is not Python — it is
**garust itself**: `checks/ga-cross/` holds a small cargo test that runs
the same operations through the real kernel and emits JSON; `check.mjs`
compares the classroom's numbers against it. The lab teaches the algebra
the engine actually computes with.

**First GA lessons (syllabus order):**

1. *One Turn Is τ* — arcs, angle addition, and the house convention
   itself as a lesson: why the circle constant is the full turn.
2. *Two Mirrors Make a Rotation* — reflections compose into a rotor; the
   angle doubles. Ties directly to the mirror bench that already exists,
   and it is the deepest elementary fact in GA: rotations are what
   reflections do in pairs.
3. *The Wedge* — oriented area, why `b∧a = −a∧b`, and where `e12` lives.
4. *Rotors, Not Angles* — the sandwich `R v R̃`, composition without
   angle addition headaches; checked against garust `Vga2`.
5. *Motors* (film-assisted) — screws in the plane and in space, with the
   motoreel M3 glyph films as embedded assets: the classroom shows what
   the studio measured.

**Bridges from the on-ramp lessons.** The optics bench gains a GA note —
a mirror *is* a reflection sandwich, and the two-mirror rotor lesson
reuses its bench; conventional lessons point forward into the track
rather than existing beside it.

---

## 5. Migration plan — five stages, each shippable

| Stage | Work | Outcome |
|---|---|---|
| S1 | Extract each page's math into `model.mjs` (ES module); write `check.mjs` importing it; keep pages working via `<script type="module">`. **Retrofit τ across current lessons' code and prose** | one-source-of-truth math; Node checks in CI; house convention holds |
| S2 | `controls.mjs` + `readouts.mjs` + `playback.mjs` from `lesson.json`; delete the triplicated wiring | pages shrink to view + notes |
| S3 | `lesson.json` full schema + `stepper.mjs`; port the wave stepper, add optics/ball dissections | the pedagogic template exists |
| S4 | `tools/index.mjs` + generated hub with prereq ordering | the learning path |
| S5 | `tryThis`/`misconceptions` rendering; **`engine/ga2.mjs` (Cl(2,0)) + the first two GA lessons + the garust cross-check**; first motoreel asset lesson (chaos, once R-0005 ships) | the full framework, proven by six lessons incl. the GA track |

Estimate: S1–S2 are a focused session; S3–S5 another. No stage breaks
the deployed site.

---

## 6. Open questions for the owner

1. **Naming:** `examples/` → `lessons/`? (Recommend yes — the word is
   the mission.)
2. **Node as a dev dependency** (runtime stays zero-dep) for same-code
   checks? (Recommend yes; Python stays as the independent layer.)
3. **Process weight:** adopt motoreel's full R-/SPEC- requirement loop
   here, or stay RFC + checks + CI? (Recommend the lighter one: this is
   a content site; the loop's cost belongs where irreversibility lives.)
4. **Prereq graph now or at ~6 lessons?** (Recommend the schema now —
   it is one JSON field — and the fancy path UI at 6.)
5. **garust → WASM for live GA lessons:** park explicitly as
   out-of-scope until motoreel M4? (Recommend park — `ga2.mjs` covers 2D
   lessons exactly; WASM becomes worth it when 3D PGA lessons want the
   real kernel live.)
6. **`ga2.mjs` scope:** start with `Cl(2,0)` only, adding planar PGA when
   the motors lesson needs it? (Recommend yes — smallest kernel that
   makes lessons 1–4 exact.)
7. **τ retrofit timing:** fold into S1 as proposed, or do it immediately
   as its own commit? (Recommend S1 — one pass over the math while it is
   being extracted anyway.)

## 7. Explicitly out of scope

Frameworks, bundlers, React; telemetry; accounts/progress tracking;
server-side anything; WASM (parked per Q5); generic "physics engine
data" — the models stay bespoke code forever, that is the product.
