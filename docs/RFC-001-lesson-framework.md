# RFC-001 — From demo pile to learning framework

- **Status:** Proposal — awaiting owner review. Nothing here is built.
- **Owner:** Gustavo Delgadillo (westerngazoo)
- **Scope:** physics-lab's architecture, its pedagogic structure, and its
  seams with motoreel/garust/goosethropic.

**Review log:** rev 2 added τ and the GA track after owner review. Rev 3
records the owner's second review: *structure approved; JS rejected* —
models move to Rust-on-garust compiled to WebAssembly (§4.0), the JS
mini-kernel is deleted, and GA-first derivation becomes the method, not a
track beside the method. "Learning GA means building GA, even if it takes
longer."

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

### 4.0 Language decision — Rust everywhere JS was (owner review, rev 3)

The owner's constraint is blunt and productive: avoid JS at all costs.
The resolution is not a workaround, it is the better architecture:

- **Models are Rust crates that depend on garust**, compiled to
  `wasm32-unknown-unknown`. *Verified on this machine: garust builds for
  that target unmodified, `physics` feature included* (std exists on
  wasm32; the `no_std+libm+physics` combination has 2 errors upstream —
  not needed, noted for honesty).
- **The lesson computes primitives, one fixed JS file paints them.** Each
  lesson exports a C-ABI (`#[no_mangle] extern "C"`): `build(params)` and
  `state_at(t) -> prim buffer` — a flat f64 array of tagged drawing
  records, the same architectural move as motoreel's `Prim2`: physics on
  one side of a dumb boundary, painting on the other. The painter +
  loader + controls + clock is **one shared `runtime.js`, on the order of
  300 lines, written once and never touched per lesson.** Nobody authors
  JS to make a lesson.
- **wasm-bindgen is rejected** for the same reason JS was: it generates
  thousands of lines of glue and drags in npm. The hand C-ABI keeps every
  byte of the boundary understood — the garust discipline, applied to the
  browser.
- **Costs, owned:** the zero-build principle dies — `cargo build
  --target wasm32-unknown-unknown --release` is now the build (the only
  toolchain is the one the house already lives in); and the CSP gains
  `'wasm-unsafe-eval'` in `script-src` — a contained loosening that
  permits wasm compilation only, not JS eval.
- **Checks collapse into honesty:** the same crate runs under
  `cargo test` natively. "One source of truth" stops being a discipline
  and becomes a tautology. Node never enters the project. The Python
  layer stays as the independent second implementation.

### 4.1 A lesson is a folder with a contract

```
lessons/<slug>/
  lesson.json        metadata + declarations (see schema below)
  crate/             Rust: the model AND its drawing-to-primitives,
    src/lib.rs         on garust; multivector-first derivations;
    src/tests.rs       cargo tests assert every claim IN the same crate
  notes.html         prose fragments: intro, derivation steps, notes
  pkg/lesson.wasm    the built artifact the page loads (committed)
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

The crate's contract (C-ABI, no wasm-bindgen):

```rust
#[no_mangle] pub extern "C" fn build(params: *const f64, n: usize);
#[no_mangle] pub extern "C" fn state_at(t: f64) -> *const f64; // prim buffer
#[no_mangle] pub extern "C" fn prim_len() -> usize;
// Pure over (params, t): same inputs, same buffer, bit for bit.
// The prim records use motoreel's vocabulary as a *convention*
// (point / segment / polyline / edges + style), not a dependency — until
// motoreel publishes, at which point depending on it is one line.
```

### 4.2 The engine — one fixed runtime, mostly Rust-side

```
engine/
  runtime.js     THE one JS file (~300 lines, written once): wasm loader,
                 painter (prim buffer → SVG), controls + readouts
                 generated from lesson.json, the clock, the stepper hooks.
                 Carries the skew guard. Nobody edits it to add a lesson.
lessons-common/  a Rust crate: prim-buffer writer, world box, shared
                 drawing helpers (arrows, axes, labels-as-prims) — the
                 shell that was lab.js, now on the right side of the
                 boundary, in Rust, testable under cargo.
```

Pages shrink to a static shell that names the slug; `runtime.js` does the
rest. The clock lives in the runtime, so principle 2 (models are pure over
`t`) is enforced mechanically — no lesson ever owns time.

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

### 4.3.1 The snapshot dissector (owner, rev 3 review)

A notebook-style device for derivation steps, specified here so the wave
port builds it in Rust rather than anyone bolting it onto the old JS:

- **Freeze:** any derivation step can pin a *time snapshot* — the chain
  stops at `t*`, chosen by scrubbing, like a cell output in a notebook.
- **Zoom:** an inset magnifies one bead and its two string segments at
  that frozen instant.
- **Decompose:** the inset draws **the two tension vectors** along the
  actual segments and splits each into components. This is where the
  derivation's one approximation becomes *visible and interrogable*:
  the true transverse pull is `T·sin θ`, the derivation uses the slope
  `T·tan θ = T·Δy/Δx`, and the inset shows both — geometrically and as
  numbers — with a live `|tan−sin|/sin` error readout.
- **Break it on purpose:** an exaggerate-amplitude control drives the
  angles large, so the student watches the small-angle approximation
  fail and understands it as a *modeling choice with a budget*, not a
  magic step. (The main stage stays honest; the inset is where the
  student is allowed to bend it.)

All computed lesson-side in Rust (the components are just projections);
the runtime only gains a generic inset viewport — a second world box
painting the same prim vocabulary.

### 4.4 The hub becomes a generated learning path

`tools/index.mjs` (run by hand or CI, output committed) walks
`lessons/*/lesson.json` into `lessons/index.json`. The hub page fetches
it (same-origin — CSP already allows) and renders: topic clusters, a
prerequisite ordering ("start here"), and each card's claims. Adding a
lesson = adding a folder; the hub cannot forget it or misdescribe it.

### 4.5 Verification — three layers, honestly divided

| Layer | What | Runs where |
|---|---|---|
| L1 | `cargo test --workspace` — asserts every claim against **the same Rust that compiles to the wasm the browser runs** | native, CI |
| L2 | The existing Python checks, kept: an independent second implementation of the flagship claims (N-version cross-check) | `python3 checks/run.py`, CI |
| L3 | Page behavior: the browser drive (clock advances, controls bound) + the runtime's load-time skew guard | manual today; scriptable later |

A GitHub Action runs L1 + L2 on every push, plus the wasm build so a
lesson that stops compiling for the browser fails loudly. No Node
anywhere in the project.

### 4.6 The motoreel seam, concretely

`lessons/chaos/assets/twin-pendulum/{frames or .mp4, provenance.json}`
where provenance records: motoreel commit, requirement (R-0005), dt,
parameters, and the determinism statement ("bit-identical re-render,
same machine"). The lesson's claims cite the film's *measured* bands.
This is how integrator-physics enters the classroom without smuggling an
integrator into the browser.

### 4.7 GA is the method, not a track beside it (rev 3)

The rev-2 design — a JS mini-kernel cross-checked against garust — is
**deleted**. With lessons written in Rust, the classroom's kernel IS
garust; there is nothing to cross-check because there is only one
implementation, and it is the real one. The owner's directive sharpens
the pedagogy too: *learning GA means building with GA* — lessons derive
and integrate multivectors directly, because that is how the owner finds
the physics clearest, and taking longer is the point, not a cost.

Concretely, GA-first means:

- **Optics:** the image point stops being `di = do·f/(do−f)` plugged into
  a formula and becomes **the meet of two refracted rays** — PGA
  `line ∧ line`, computed live by garust, exactly the incidence trick
  motoreel's RFC was founded on (`MeetPoint`), now teaching itself. The
  thin-lens equation is *derived on the page* from the construction, not
  assumed.
- **Mechanics:** poses are motors, rotations are rotors applied by
  sandwich, angular quantities are bivectors — the same objects the
  student meets again in motoreel's films.
- **Where GA is not the honest tool** (the wave chain's scalar mode
  amplitudes), the lesson says so — knowing where an algebra earns its
  keep is part of learning it.

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
| S1 | **The pilot, end to end:** *Two Mirrors Make a Rotation* as a Rust crate on garust `Vga2` → wasm via the C-ABI, `runtime.js` written once, `cargo test` asserting its claims, τ throughout | toolchain proven by the first GA-native lesson; the one JS file exists and is finished |
| S2 | **Port the wave-equation lesson first** (promoted from S5 at owner request): Rust crate, the snapshot dissector (§4.3.1), the sin→tan inset, τ throughout | the derivation's one approximation becomes interactive |
| S2b | Port the bouncing ball and optics to Rust crates; optics goes GA-first (image = meet of rays, thin-lens equation derived); retire their page JS and the hand-ported Python where superseded | all lessons on one architecture; τ retrofit complete |
| S3 | `lesson.json` full schema + stepper in the runtime; dissections for ball and optics | the pedagogic template exists |
| S4 | hub generated from manifests with prereq ordering | the learning path |
| S5 | `tryThis`/`misconceptions` rendering; the wave chain ported; *One Turn Is τ*, *The Wedge*, *Rotors Not Angles*; first motoreel film lesson (chaos, once R-0005 ships) | the framework, proven by seven lessons |

Estimate: S1 is a focused session (most of it is `runtime.js` and the
prim ABI, paid once); S2–S3 another; S4–S5 another. No stage breaks the
deployed site — old pages stand until their replacement lands.

---

## 6. Decisions so far, and what remains open

**Settled by owner review (recorded):** the structure (lesson contract,
pedagogic template, generated hub) stands; JS is rejected — Rust on
garust via WASM, one fixed runtime file; GA-first derivation is the
method; τ is the circle constant; Node never enters (mooted rev 2's Q2);
rev 2's Q5 (park WASM) is reversed — WASM is the core; `ga2.mjs` is
deleted (mooted Q6).

**Settled by owner review, round three:** built `.wasm` binaries are
committed to git (Q4 → yes); prim vocabulary follows motoreel's as a
convention, chosen for pedagogy — the classroom and the studio draw with
the same concepts (Q5); and the **program order** is fixed top-down:
*physics learning first, then engineering implementation on physics
systems (motoreel/garust), then engineering for its own sake (the
Clifford language).* The lab is stage 1; S1 starts immediately.

**Landed 2026-08-26 (S2-shell/S3-data/S4-hub, owner demand: "something
composable"):** `lessons-common` (the `lesson!` macro and `Prims` writer
— a lesson is one `draw` function on a uniform ABI); the runtime builds
the entire page from `lesson.json` (claims, views, bar, legend, notes,
try-this) so a lesson's HTML is a 21-line stub; the hub is generated
from the manifests by `tools/gen-index.py`. Both existing lessons ported
as the proof; claims render on-page with their test names, tying each
page to CI.

**Landed 2026-08-26 (S2: the wave port + §4.3.1):** the wave-equation
lesson is a Rust crate on the framework — exact eigenmode evolution
(verified against independent Verlet, W2), τ throughout (the cutoff is
4/τ of c, W4) — and the **snapshot dissector is real**: freeze any
instant, magnify the focus bead, see its two tension pulls split into
T·sin θ (true) versus T·tan θ (used), with the error as a live readout
that the tests pin to the closed form and that grows monotonically
under the exaggeration control (W6). The runtime gained data-driven
derivation steps (`steps` + a hidden step param driving per-step
highlights in `draw`). The legacy JS wave page is retired.

**Still open:**

1. **Naming:** `examples/` → `lessons/`? (Recommend yes; S1 uses
   `lessons/` for the new path, `examples/` stands until S2 ports.)
2. **Process weight:** motoreel's full R-/SPEC- loop, or RFC + checks +
   CI? (Recommend the lighter one; revisit if the runtime ABI starts
   changing under lessons.)
3. **Prereq path UI now or at ~6 lessons?** (Schema now, UI at 6.)
4. **Built `.wasm` in git, or CI-built on deploy?** (Recommend commit
   them: tens of KB each, reproducible from a pinned toolchain, and the
   site stays deployable from a bare checkout with no toolchain.)
5. **Prim vocabulary:** motoreel's `Prim2` as a *convention* now, a
   *dependency* when motoreel publishes? (Recommend convention now.)
6. **A physics language, someday.** The owner: if a lesson-authoring DSL
   is ever needed, "use Lisp or a better, more modern option." Recorded
   as parked with a named trigger — two concrete authoring pains Rust
   cannot express cleanly. **Amended by owner decision (2026-08-24,
   final): both.** cliffordc continues its May pivot (effects-language;
   GA framing stays retired), and the GA notation layer is its own small
   language **on top of garust** — proposed as garust RFC-014, whose
   first milestone is exactly this lab's trigger case: an interactive
   multivector console lesson. Until RFC-014 ships something: Rust *is*
   the physics language here.

## 7. Explicitly out of scope

Frameworks, bundlers, React; telemetry; accounts/progress tracking;
server-side anything; WASM (parked per Q5); generic "physics engine
data" — the models stay bespoke code forever, that is the product.
