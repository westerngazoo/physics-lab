# RFC-002 — `mecanica`: gym biomechanics as lessons

**Status:** Discussing (proposal)
**Author:** Claude (main session), for físico buen físico
**Date:** 2026-08-31
**Depends on:** [RFC-001](RFC-001-lesson-framework.md) (the lesson framework this extends)
**Related:** `fisicobuenfisico` reels 01–20 · `fitAI` R-0045 (lift biomechanics model)

---

## 0. TL;DR

`fisicobuenfisico` has fourteen mechanics reels covering eight movements, an
audience that responds to them, and a house rule — *"the arrows are to real
scale between themselves, and the numbers come from the calculation you can
see in the file"* — that this repo already knows how to enforce with tests.

What the reels lack is reuse, tests, and a surface a reader can *drive*. What
this repo lacks is a reason to draw a human body.

This RFC proposes:

1. **A `mecanica` crate** — closed-form gym biomechanics: one trait with one
   method and three shared algorithms over it. *(Amended 2026-09-13: this read
   "zero dependencies". `maquina_humana` now consumes `garust::twolink`, on the
   grounds that two implementations of "where the elbow is" is where a sign
   error hides. It still builds for `wasm32-unknown-unknown`.)*
2. **Exactly one new drawing primitive** — a filled polygon. Not three.
3. **An FBF theme** — a `tokens.css` override and a heat ramp declared as
   styles. No Rust changes, no framework changes.
4. **Three lessons for v1**, all sagittal-plane, from reels that already landed.
5. **A muscle-path axis (§7)**, deliberately *not* in v1, specified now because
   it changes what the crate's shape has to leave room for.

The physics is already written and already correct. This RFC is mostly about
where it lives and what proves it.

---

## 1. Context

### 1.1 What the reels actually are (audit)

Twenty reel scripts in `fisicobuenfisico/tools/`. Fourteen are mechanics,
across eight movements: squat (01, 02, 07), leg press (03), curl (04, 09),
deadlift (05), bench press (06), hip thrust vs Romanian deadlift (08), lateral
raise (13), cable kickback (14, 15, 16, 20). The remaining six are optics
(10, 11, 12) and comics (17, 18, 19), and are out of scope here.

Three findings from reading them:

**They contain no video, no pose estimation, and no machine learning.** A grep
of the whole `tools/` tree for MediaPipe, OpenPose, MoveNet, ONNX, keypoints,
OpenCV, torch and TensorFlow returns nothing. Every model builds its posture
analytically from segment lengths and one progress scalar:

```python
def pose_ht(s):                        # s=0 bottom → s=1 lockout
    th  = THETA_MAX * (1 - s)
    hip = (rod[0] - LF*math.cos(th), rod[1] - LF*math.sin(th))
    d   = LF * math.cos(th)            # the femur IS the lever
    return {..., "tau": FUERZA * d}
```

The knobs are *inputs*. That is what makes this a lesson and not a pipeline.

**The physics is per-episode and duplicated.** `reel20.py` hand-rolls a
bisection to find where torque crosses zero; nothing else can use it.
`trabajo()` is re-derived in at least four files. The `guion` RFC already
diagnosed this ("the physics is trustworthy but unverified and non-reusable").

**There are no tests at all.** The repo's own README documents what that
costs: reel 09 shipped a caption claiming 8× while the drawing showed 5×,
because the lever arm was projected by sin(φ) in the label but not in the
calculation. A reel is verified at the frames that shipped. A lesson is driven
to parameter combinations nobody rendered, so the correctness bar is the whole
parameter box — which is precisely what this repo's claims-as-tests discipline
is for.

### 1.2 What the framework already gives

More than expected. `DESIGN.md` is explicit that styles are indices into the
manifest so that "a lesson's Rust never encodes a color, and re-theming touches
no code". That single decision is what makes an FBF re-skin nearly free:

| FBF element (`muneco.py` / `piezas.py`) | Framework equivalent | Cost |
|---|---|---|
| `cap()` outlined limb | two `segment`s (wide ink, narrower fill); SVG `stroke-linecap:round` **is** the capsule | free |
| `punteada()` moment arm | `segment` with `dash: true` | free |
| `flecha()` force vector | `arrow` | free |
| `Ejes.traza()` torque curve | `curve` / `polyline` | free |
| `Ejes.rejilla_*()` axes | `segment` | free |
| `heat()` ramp | 6–8 styles; Rust returns a **bucket index** | free |
| INK / PAPER / ACC / ACC2 | override `tokens.css` | free |
| `panelito()` readings | readout slots, `hero: true` for the big τ | free |
| the knobs | `params` (cap 16) with min/max/step/unit/digits | free |
| `Ejes.area()` — *the area under the curve* | **nothing** | §4 |
| `etiqueta()` in-scene text | nothing → moves to readouts | deferred |
| `halftone` / `grain` | nothing → CSS page chrome | out of scope |

Only one row in that table is load-bearing and missing.

---

## 2. Goals and non-goals

### Goals

1. One tested home for the physics, reusable by every lesson and later by fitAI.
2. Three lessons live, in the visual language the audience already knows.
3. Every claim a reel makes on screen becomes a named `#[test]`.
4. The reels' honesty discipline carried onto every page, not softened.

### Non-goals

- **Not** a rendering engine. Drawing stays in the framework's primitives.
- **Not** per-muscle force. See §7.3 — it is mathematically indeterminate, and
  fitAI R-0045 AC5 forbids it by owner decision.
- **Not** video, pose estimation, or measurement of a real lifter. The inputs
  are knobs. (Video may later become a second *source* of the same parameters —
  see §9 — but nothing in this RFC depends on that.)
- **Not** injury or safety claims. Higher torque is not "dangerous".
- **Not** a port of the optics or comic reels.

---

## 3. The `mecanica` crate

### 3.1 Why a crate, not inline in each lesson

Because the reels already ran the experiment. Four files re-derive the work
integral; one hand-rolls a root find; none are tested. Three lessons written
the same way would reproduce the same defect, and fitAI R-0045 AC15 needs the
same functions again in `fitai-core`.

The crate is pure Rust with **one dependency, `garust`** — the kernel this
repo already builds on — and no I/O, no clock, no state. *(Amended 2026-09-13:
this read "zero dependencies — no `garust`". `maquina_humana` took the
two-link solve from the kernel rather than copying it. Verified still building
for `wasm32-unknown-unknown`, which `checks.yml` requires of every member.)*

### 3.2 The one abstraction, and why it pays rent

`DESIGN.md` §7 sets the bar: name the second implementor, and *"if the
abstraction sits between a physicist and their physics, it had better pay the
rent."*

```rust
//! mecanica — closed-form gym biomechanics. Pure: no deps, no I/O.

/// A lift, reduced to the one thing every analysis needs: how hard the
/// load resists at each point of the movement.
///
/// Signed, never magnitude. It is the sign that shows a cable stop
/// resisting and start assisting — reel 20 found that crossing at −32°,
/// and in absolute value it is invisible.
pub trait Lift {
    /// Torque at the analysed joint, N·m. Positive resists the movement.
    fn tau(&self, phi: f64) -> f64;
    /// The movement's angular range, radians.
    fn range(&self) -> (f64, f64);
}

/// W = ∫ τ dφ, φ in RADIANS. Midpoint rule.
pub fn work<L: Lift>(lift: &L, a: f64, b: f64) -> f64;

/// The peak and where it happens: (phi, tau).
pub fn peak<L: Lift>(lift: &L) -> (f64, f64);

/// Where τ crosses zero — past this the load assists instead of resisting.
/// `None` when the sign never changes on the bracket.
pub fn zero_crossing<L: Lift>(lift: &L, lo: f64, hi: f64) -> Option<f64>;
```

Three implementors — `patada::Patada`, `gluteo::Rumano`, `gluteo::HipThrust` —
and three algorithms written once over one method. That clears rule 1 and pays
the rent: `work`, `peak` and `zero_crossing` stop being per-episode code.

Nothing else becomes a trait. Each movement is a plain struct with public
fields — the knobs — and its own `postura()` returning joint coordinates. Per
`DESIGN.md`, a shared "posture" type would be an abstraction describing one
struct literal.

**The squat deliberately does not implement `Lift`** *(amended during
implementation)*. Both squat reels are static comparisons of one posture, and
no published model validates a descent kinematic. Fabricating one so the struct
could satisfy a trait would be the "looks right rather than is right" failure
this repo exists to prevent — and it would put invented physics behind a claim
test, which is worse than having no test. A depth axis is a v2 change with its
own claims. The trait still has three implementors without it.

### 3.3 Two invariants worth stating in code

**Radians, not degrees.** `W = ∫ τ dφ` in degrees is wrong by 57.3× and *no
unit check catches it*, because the radian is dimensionless. Reel 10's caption
already warns readers about this; the crate should carry it as a doc comment
and a test.

**Colors stay in the manifest.** The crate must not name a color, so the heat
ramp returns an index:

```rust
/// Style index on the olive→mustard→fire→blood ramp.
/// Returns an INDEX, never a color: presentation lives in lesson.json.
pub fn heat_bucket(t: f64, buckets: usize) -> usize {
    ((t.clamp(0.0, 1.0) * buckets as f64) as usize).min(buckets - 1)
}
```

This is the framework's existing rule, honoured rather than worked around.

---

## 4. The one framework change: a filled polygon

### 4.1 What is missing

`polyline` is painted with a hard-coded `fill: "none"`. So the shaded region
under a torque curve cannot be drawn — and in reels 14, 15, 16 and 20 that
shading is the argument, not decoration: *"el área ES el trabajo"*. A lesson
that draws the curve but not the area teaches less than the reel it came from.

### 4.2 What is proposed

One record type, following the existing polyline layout exactly:

```
  [2, n, x0,y0, …, style]     polyline   (existing, stroked)
  [4, n, x0,y0, …, style]     polygon    (new, filled)
```

```rust
/// A filled region. The area under a τ(φ) curve IS the work — this is
/// the one thing the stroked vocabulary cannot say.
pub fn polygon<I: IntoIterator<Item = (f64, f64)>>(&mut self, pts: I, style: usize);
```

Painter, mirroring the polyline arm:

```js
} else if (tag === 4) {
  // …same point unpacking as tag 2…
  view.scene.appendChild(el("polygon", { points: pts,
    fill: css(st), "fill-opacity": s.opacity ?? 1, stroke: "none" }));
```

`opacity` is a new optional style field. Existing styles omit it and default
to 1, so no existing lesson changes.

### 4.3 Why this clears the bar, and what is *not* being added

`DESIGN.md` §6 lists the refused abstractions — a `Lesson` trait, trait objects
for primitives, a scene graph, `wasm-bindgen`, a `Style` type, generic
`Prims<T>`. **Fills are not among them.** They were never refused; four lessons
simply never needed one. §2 states the intended review path for exactly this
case: *"exhaustive matches are how a vocabulary extension gets reviewed."*

Deliberately **not** proposed, despite the FBF renders using both:

- **In-scene text** (`etiqueta`, `panelito`). The readout panel already carries
  numbers, with `hero` for emphasis. Adding a text primitive would drag in font
  metrics, anchoring and i18n for something the page already does better.
- **A disc primitive** for plates and heads. A polygon approximates a circle
  well enough, and `point` covers markers.

Both stay refused until a lesson genuinely cannot be written without them.
One primitive, justified by one lost argument.

---

## 5. The FBF theme

Entirely data and CSS. No Rust, no framework code.

**Palette** — a token override mapping `fisicobuenfisico/brand/PALETTE.md` onto
the existing custom properties, plus the heat ramp:

```css
:root[data-theme="fbf"] {
  --ink:    #14100c;   /* INK   (20,16,12)   */
  --bone:   #f2e6d0;   /* PAPER (242,230,208)*/
  --red:    #c1440e;   /* ACC   brasa        */
  --red-hot:#e05a2b;   /* ACC2  fuego        */

  --fbf-heat-0: #5d6b5d;  /* oliva   — low tension  */
  --fbf-heat-3: #c9a03c;  /* mostaza                */
  --fbf-heat-5: #e05a2b;  /* fuego                  */
  --fbf-heat-7: #b0322a;  /* sangre  — high tension */
}
```

**Styles**, per lesson, indices the Rust hands back from `heat_bucket`:

```json
"styles": [
  { "var": "--ink",        "width": 34 },
  { "var": "--fbf-skin",   "width": 26 },
  { "var": "--ash-dim",    "width": 3, "dash": true },
  { "var": "--fbf-heat-0", "width": 9 },
  { "var": "--fbf-heat-7", "width": 9 },
  { "var": "--fbf-heat-5", "width": 2, "opacity": 0.23 }
]
```

A limb is two segments — style 0 wide in ink, style 1 narrower in skin — which
is exactly what `cap()` does with two PIL `line` calls.

**Type.** `brand/fonts/` carries Bangers, Comic Neue and Luckiest Guy under the
SIL OFL. Self-hosting matches `fonts.css` and the existing CSP
(`script-src 'self' 'wasm-unsafe-eval'`, everything first-party). *The OFL
requires the license text to ship alongside the fonts; `fisicobuenfisico`'s
README already flags this as outstanding and it must be fixed before these
fonts are served from here.*

**What does not port:** halftone and grain become CSS page background, not
primitives; in-scene labels become readouts.

---

## 6. v1 — three lessons

All three are **sagittal plane**, so the whole of v1 carries one assumption set
and one caveat. (Bench press is frontal-plane; it is deferred to v2 rather than
doubling the caveat surface at launch.)

### 6.1 `sentadilla` — why your squat doesn't look like mine

From MECÁNICA 01 and reel 07. Posture from `resolver_sentadilla`: thigh to
parallel, bar over midfoot, torso angle solved from the hip offset.

**Knobs:** femur length (0.38–0.52 m), tibia length, torso length, load,
bar horizontal offset from midfoot (0–0.12 m).

**Readouts:** hip τ (hero), knee τ, torso angle β, hip moment arm d.

**Claims:**
- `C1` a longer femur strictly increases the hip moment arm at parallel
- `C2` knee τ is invariant to femur length at fixed shank geometry (the
  reel's "119 N·m in both" — this is the surprising one and must be a test)
- `C3` bar offset forward increases hip τ monotonically
- `C4` the solved bar position lies over the midfoot when offset is 0
- `C5` τ = F·d agrees with the moment computed from the solved joint positions

### 6.2 `patada` — the four knobs of the cable kickback

From reels 14, 15, 16 and 20 — four reels, one model, and the strongest
interactive story in the corpus because every knob is already modelled.

**Knobs:** movement range as start/end angle (−45–90°), strap position
(0.46–0.85 m from hip), pulley height (0.05–0.45 m), load.

*Amended during implementation:* torso angle is **not** a model input. In the
reels it decides how much range is available — bent to 90° the hip travels 90°,
standing it has ~40° of hyperextension left — but no published model derives
range *from* torso angle, and the two data points (90°→90°, 0°→40°) admit
infinitely many laws. The range is therefore exposed directly, with the two
published settings as named presets, rather than fitting a line through two
points and calling it physics.

**Readouts:** τ at the current angle (hero), work over the range, peak τ,
the zero-crossing angle.

**Claims:**
- `C1` torso at 90° yields more work than at 40° at equal peak τ
- `C2` peak τ is *identical* between torso angles (the reel's claim)
- `C3` ankle strap gives 1.85× the moment arm of the above-knee strap
- `C4` raising the pulley lowers τ at **every** angle in range — the reel
  asserts "la naranja va arriba SIEMPRE", which is a universally quantified
  claim and should be tested as one
- `C5` `zero_crossing` finds the sign change near −32° and `tau` is negative
  beyond it
- `C6` starting 30° early adds 29% of work
- `C7` the published 165.5 J is reproduced
- `C8` **the peak is at 8.6°, not at the vertical** — see §6.4

### 6.4 A defect the claims found immediately

Writing C8 as "the peak is at the start" made it fail. The peak is **125.1 N·m
at 8.6°**, not 123.2 N·m at ψ = 0: torque *rises* through the first 8.6° of hip
extension and falls afterwards.

123.2 N·m is the torque **at ψ = 0** — this model gives 123.4, the difference
being the reels' hard-coded 147 N tension against 15 × 9.81 = 147.15. So the
published figure is the starting torque presented as the peak, and reel 20's
line *"arrancas en el pico y de ahí baja"* is wrong in its first half.

The headline is unharmed: 8.6° still falls inside the standing range, so "el
pico es idéntico" between torso angles holds, and that claim (C2) passed on its
own. But the number and the sentence are both off by a little, and the house
rule in `fisicobuenfisico`'s README is that a label disagreeing with the
calculation is a bug rather than narrative licence.

This is the argument for the crate, made in under an hour: the reels are
verified at the frames that shipped, and this error sits between two rendered
frames where nothing looked wrong. It is also OQ-5 answering itself —
transcribe, then let the claims decide, and treat the disagreement as a
correction worth publishing.

### 6.3 `gluteo` — hip thrust vs Romanian deadlift

From reel 08. Two `Lift` implementors on one graph; the curves cross.

**Knobs:** load, femur length, torso length, RDL depth, hip-thrust bottom angle.

**Readouts:** τ RDL, τ hip thrust, the crossing point, work for each.

**Claims:**
- `C1` RDL τ is maximal at the bottom and → 0 at standing
- `C2` hip-thrust τ is maximal at lockout
- `C3` the curves cross exactly once in range
- `C4` at equal load, peak τ favours the RDL (494 vs 451 N·m in the reel)

---

## 7. The muscle-path axis — specified now, shipped later

Not in v1. Written here because it determines what the crate must leave room
for, and because it closes a gap the reels themselves identified on air.

### 7.1 A muscle is a tension element

A muscle can only pull. Modelling it as a path from origin to insertion
carrying a pure tensile force is the standard musculoskeletal formulation, not
a simplification. Two quantities follow from the geometry alone:

- **length** `L(θ)` — how stretched the muscle is at that joint angle
- **moment arm** `r(θ)` — its leverage on the joint

These are not independent. By virtual work, `F·δL = τ·δθ`, so

```
r(θ) = dL/dθ
```

One path function yields both — and yields a genuine claim of the kind this
repo is built on: compute `r` geometrically, compute `dL/dθ` numerically,
assert they agree. Two independent derivations converging.

Both quantities are **fully determinate**. No optimization, no assumption.

### 7.2 What this unlocks that torque alone cannot

Reel 20, in the author's own words:

> *"Nuestro modelo mide lo que el CABLE le exige a la cadera — no cuánto de eso
> se lleva el glúteo ni qué tan largo está. Eso es una limitación honesta del
> modelo, no un detalle."*

The path model closes the second half of that. It cannot say how much the glute
takes, but it can say how long the glute *is* at every angle — which is the
axis the stretch-mediated-hypertrophy argument turns on, and the thing the
torque number provably cannot express (the reel's own C2: identical peak τ,
different stimulus).

The same mechanism explains variations across joints. The biceps long head
crosses the shoulder **and** the elbow; the short head crosses only the elbow.
So an incline curl (shoulder extended) holds the long head long, while a
preacher curl (shoulder flexed) does not — a difference in *length*, fully
determinate, with no force claim anywhere near it.

### 7.3 The wall: redundancy

Torque balance at a joint is one equation:

```
r₁F₁ + r₂F₂ + … + rₙFₙ = τ_external
```

One equation, *n* unknowns. This is muscle redundancy: it is not difficult, it
is **underdetermined**, and no modelling effort resolves it.

A consequence worth stating plainly, because it is counterintuitive: splitting
one lumped muscle into its two heads makes the force split *less* determined,
not more. Split for the length and moment-arm fidelity (§7.2), never expecting
it to buy force.

Reel 09's "your biceps pulls 160 kg" is the determinate special case — one
lumped element, so `F = τ/r` with nothing to distribute. It should be labelled
as what it is: a **lumped elbow flexor**, since brachialis and brachioradialis
are inside that number.

### 7.4 The trap in the analogy

An elastic band's tension is determined by its stretch: `F = k·Δx`. A muscle's
is not — active tension is under neural control, and the lifter chooses it.

Taking the band analogy literally would produce a *determinate* model, and it
would be **confidently wrong**, because it silently assumes the muscle cannot
choose its own activation. Passive tension is real (titin, connective tissue)
and matters at long lengths, but active force dominates in training. This must
be stated in the spec, because the resulting model would look like it worked.

### 7.5 What v1.5 would ship

```rust
/// A muscle as a pure tension element: a path that can only pull.
/// Geometry ONLY — this deliberately says nothing about force, which is
/// indeterminate whenever two muscles cross one joint (§7.3).
pub trait MusclePath {
    /// Origin→insertion length at joint angle θ, metres.
    fn length(&self, theta: f64) -> f64;
    /// Fraction of the muscle's own range: 0 shortest, 1 longest.
    fn normalized(&self, theta: f64) -> f64;
}

/// Moment arm by tendon excursion: r = dL/dθ.
pub fn moment_arm<M: MusclePath>(m: &M, theta: f64) -> f64;
```

One muscle — gluteus maximus on the kickback — as a second curve beside torque.
No force number anywhere.

Two costs, both real:

- **Coordinates need a cited source.** Candidates to verify (none assumed here):
  Klein Horsman et al. (2007) lower-limb dataset; the Delp et al. (1990) model
  distributed with OpenSim. Licensing of model data must be checked separately
  from the software licence.
- **Straight-line paths break where muscles wrap** over bone. The quadriceps
  over the patella is the textbook failure. Wrapping surfaces or via-points are
  where naive muscle models quietly go wrong, and the spec must say which
  muscles the straight-line assumption is valid for.

### 7.6 Per-muscle force, if it is ever wanted

There is a legitimate route: static optimization, distributing force to
minimise a cost such as `Σ(Fᵢ/Fᵢ,max)³`. It is what OpenSim and AnyBody do, and
it is not fabrication **provided the cost function is displayed as an
assumption**. fitAI R-0045 AC5 forbids it by owner decision, on the grounds
that a coach repeats such a number to a client as fact. That decision stands
unless the owner revisits it; this RFC does not propose changing it.

---

## 8. Honesty requirements

Carried from the reels, which already do this well, onto every page:

1. **The model note, always visible:** *2D sagittal, external load, quasi-static.
   Does not measure stabilisers, and does not divide load between muscles.*
2. **No injury or safety claims.** Higher torque is not "dangerous", lower is
   not "safe" — the output describes load distribution.
3. **Say what is not modelled**, in the lesson prose, not a footnote — reel 20
   is the standard to match here.
4. **A claim on screen is a named test.** If a page asserts "+29% work", a
   `#[test]` asserts it, and the number in the prose comes from the model.

---

## 9. Relationship to fitAI R-0045

R-0045 ("Lift Biomechanics Model") specifies inverse dynamics in `fitai-core`
with AC15 requiring pure, testable functions and AC5 forbidding per-muscle
force. `mecanica` is that core, built first and against an audience.

R-0045 currently declares `Depends on: R-0044` (the pose keypoint series). The
audit in §1.1 shows that dependency is **optional**: posture can come from
parameters instead of a camera. Video later becomes a second *source* of `phi`
into the same `Lift` — which is the `Model × Source` separation already
proposed in the `guion` RFC.

That is a change to R-0045's dependency line, and it is the fitAI owner's call,
not this RFC's.

### 9.1 Prior art: a Japanese deadlift coordination tool

Owner-supplied, 2026-09-02. Worth recording because it is close enough to be
instructive and different enough to be clarifying.

It is a *task-constrained sagittal inverse-**kinematics*** model
(矢状面の準静的な課題拘束付き逆運動学モデル): foot position and weight height
are held equal across two panels, and it compares how hip, knee, lumbar,
thoracic and cervical segments arrange themselves to satisfy that constraint.
Its metrics are heights, displacements and joint angles.

**It reports no forces at all** — no N·m, no joules. That is the load-bearing
difference. This RFC's model is inverse *statics*: what the load demands of a
joint. The two are complementary halves, and the contrast makes one thing
explicit that §7.3 only implies:

> A multi-segment spine is cheap in a kinematic model and expensive in a
> kinetic one. Modelling lumbar / thoracic / cervical links to compare
> *postures* commits to nothing. Attaching a *moment* to a lumbar segment
> invokes R-0045 OQ-3 — an L5/S1 approximation and its stated error. Their
> spine is free because they never put a newton on it. Ours would not be.

Three practices worth adopting:

1. **Assumptions as persistent chips**, not prose — the tool shows
   足底固定 (feet fixed), 骨長一定 (bone lengths constant),
   重り高さを課題拘束 (weight height as task constraint) as a standing row.
   §8.1 asks for the same thing and should specify chips.
2. **Task constraints as the comparison device.** Hold two quantities equal
   and vary only the strategy. Sharper than showing two exercises side by
   side, and the honest form of R-0045 AC8's deltas.
3. **Label the comparison as a hypothesis.** It marks the second panel
   検証用の協調仮説 ("a coordination hypothesis for verification") and its
   strength slider 模式値 ("schematic value"). A variant must never read as a
   measurement.

### 9.2 The capture template, and why it is the calibration seam

Owner proposal: rather than accepting an uploaded clip, have the app drive
recording behind a framing template. Recorded here because it changes what the
video half costs, and because it has a consequence for this crate's shape.

**Pose gives angles for free and lengths not at all.** Every model in
`mecanica` needs absolute metres — `femur_m`, `torso_m`, `strap_m` — because
`τ = F·d` is dimensional. A keypoint series alone yields a posture that no
newton-metre can be attached to. R-0044 AC7 left calibration as an open seam;
an app-driven capture is what closes it.

The cheapest scale reference is already in frame: a competition plate is
**450 mm**. One detected disc gives the pixel→metre scale with no measuring
tape and no user input. (To be verified against the plate standards actually
common in the target market before it is relied on.)

It also shrinks R-0044's refusal surface. That machinery exists because the
pipeline accepts arbitrary uploads; when the app drives capture, most refusal
cases become *prevented* — "move back, your feet are cut off" — rather than
detected after the fact.

**It does not remove refusal**, and the spec must not claim it does:

- A template can enforce framing, and the gyroscope can enforce that the phone
  is vertical. Neither can enforce that the camera is *perpendicular* to the
  bar path, and a 20° azimuth error still reads as a legal squat.
- Silhouette overlays have a known failure mode: people match the outline by
  moving *themselves*, distorting the stance being measured. The guide must
  constrain the phone, not the lifter.
- Keypoint confidence still collapses on poor lighting, baggy clothing and
  occlusion, none of which framing fixes.

**Consequence for this crate.** If video ever feeds these models, they must
accept a **measured** posture, not only a computed one — today
`Sentadilla::tau_hip` calls `self.postura()`, folding "solve posture from
knobs" and "compute torque from posture" into one step. Splitting them is the
`Model × Source` separation from the `guion` RFC.

It is **not** being built now: `DESIGN.md` rule 1 refuses surface without a
second implementor, and there is exactly one source today. It is recorded so
v1 does not foreclose it, and so the split is a refactor rather than a
redesign when the second source arrives.

---

## 10. Open questions

- **OQ-1.** Does `mecanica` live in this workspace, or as a sibling repo both
  this and fitAI depend on? Sibling is cleaner but adds a checkout, which this
  repo already requires for `garust`.
- **OQ-2.** Do the three v1 lessons share one hub page ("Mecánica del
  gimnasio") or join the existing Classroom units? The audience arrives from
  Instagram in Spanish; the Classroom is English.
- **OQ-3.** Are the lessons Spanish-first? The reels are. The site is not.
  This affects the manifest schema, which has no locale field today.
- **OQ-4.** `PRIM_CAP` is 8192 f64. A figure plus two shaded curves plus axes
  should fit comfortably, but the kickback lesson draws two figures — worth
  measuring before it traps at runtime.
- **OQ-6.** Is the 450 mm plate a safe scale reference in Mexico and LATAM,
  where bumper and standard plates vary? A wrong scale silently scales every
  torque, which is the worst failure mode available — it looks plausible.
- **OQ-5.** Do the reels' constants get re-derived or transcribed? Transcribing
  imports any error; re-deriving risks contradicting published videos. Proposal:
  transcribe, then let the claims decide, and treat a disagreement as a
  correction worth publishing.

---

## 11. Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-08-31 | Ship through this framework, not a new app | Runtime, WASM ABI, claim-tests, CSP and Cloudflare deploy already exist; a new app rebuilds all of it. Owner decision. |
| 2026-08-31 | Three exercises deep, not eight shallow | Owner decision. The kickback alone has four modelled knobs; proving the interactive premise beats porting static reels. |
| 2026-08-31 | v1 is sagittal-only; bench deferred to v2 | Bench is frontal-plane; including it doubles the caveat surface at launch for one lesson. |
| 2026-08-31 | One new primitive (polygon), not three | The area fill is a lost *argument*; text and discs are conveniences the readout panel and polygons already cover. |
| 2026-08-31 | Muscle model is geometry-only | Length and moment arm are determinate; force distribution is not (§7.3). Consistent with fitAI R-0045 AC5. |
| 2026-08-31 | The squat does not implement `Lift` | Both squat reels are static posture comparisons; no published model validates a descent kinematic, and inventing one to satisfy a trait would put fabricated physics behind a claim test. Found during implementation. |
| 2026-08-31 | Kickback range is an input, not derived from torso angle | Two data points admit infinitely many laws; fitting one and calling it physics is the failure mode this repo is built against. Found during implementation. |

## Changelog

- _2026-08-31 — created (Discussing)._
- _2026-09-02 — §9.1 records the Japanese coordination tool as prior art (a
  kinematic model, where ours is kinetic — and why that makes its spine cheap
  and ours expensive); §9.2 records the capture template as the calibration
  seam, with the limits it does not remove._
- _2026-08-31 — amended after implementing the crate: the squat does not
  implement `Lift` (§3.2); the kickback's range is an input rather than a
  function of torso angle (§6.2); §6.4 records the peak-location defect the
  claims found in reel 20. Crate is green — 29 claims, clippy clean._
