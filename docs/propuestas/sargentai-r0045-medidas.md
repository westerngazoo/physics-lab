# Proposal for sargentAI: measured segment lengths into R-0045

- **From:** physics-lab (`mecanica`), 2026-10-18
- **To:** the Goose Physics repository (sargentAI), for the owner's decision there
- **Touches:** R-0045 (AC2, AC6), SPEC-0045 (§2.1.1, §2.3, §2.6.1, §2.6.2,
  §2.6.3, §2.8), and one new requirement for capture and storage
- **Status:** proposal. Nothing here is implemented in sargentAI. Every change
  needs the owner's acceptance under that repository's requirement loop.

## 1. What exists now on the physics-lab side

A home measurement protocol (`docs/protocolo-medidas-sentadilla.md`), a
data format `medidas-sentadilla/1` (TOML, one table per field, each one
`medido` with value, date and method, or `estimado` with no value), a
validator, and a CLI:

```
mecanica sentadilla --medidas file.toml [--carga kg] [--json]
```

The owner measures **standing heights from the floor** to four palpable
landmarks, so each segment is a difference of two heights. That is exactly
the construction of the table SPEC-0045 §2.3 already cites (Winter 2009,
Fig. 4.1):

| landmark (file field) | Winter fraction of H |
|---|---|
| acromion (`altura_hombro`) | 0.818 |
| greater trochanter (`altura_trocanter`) | 0.530 |
| lateral knee joint line (`altura_rodilla`) | 0.285 |
| lateral malleolus (`altura_maleolo`) | 0.039 |
| foot length, heel → longest toe (`largo_pie`) | 0.152 |

plus stature and body mass (always measured), the height of the bar on the
back when standing (`altura_barra`), and the shank angle from vertical at the
bottom, read off a side photo (`angulo_tibia`).

`--json` emits an `app_input` block meant to be the app's input. It carries
**only measured lengths**; an estimated field is `null`, so the app applies
its own table to it rather than receiving physics-lab's estimate as if it
were a measurement:

```json
"app_input": {
  "height_cm": 172,
  "weight_kg": 78,
  "load_kg": 100,
  "measured_lengths": {
    "thigh_m": 0.425,
    "shank_m": 0.425,
    "trunk_m": 0.495,
    "foot_m": null,
    "ankle_height_m": 0.065
  },
  "bar_on_trunk": 1.0505050505050506,
  "shank_angle_deg": 26
}
```

(the values are from the invented example file, not a person). The
`measurements` array next to it carries each field's provenance: `source`
(`measured` | `estimated`), `on` (date), `how` (method), and the value used.

The `model` block (posture, bar-only hip and knee moments) is physics-lab's
**teaching** model, the one behind the published reels. It is **not** an
input to the app and must not be shown as the app's result: SPEC-0045 §2.2
keeps the product physics in `fitai_core::biomech`, and that decision stands.

## 2. Mapping to what SPEC-0045 already names

| `app_input` | SPEC-0045 | status |
|---|---|---|
| `height_cm` | `HeightCm` (SPEC-0003) | exists |
| `weight_kg` | `WeightKg` (SPEC-0003) | exists |
| `load_kg` (`null` = none) | `LoadKg` (SPEC-0004); `None` → `LoadBasis::BodyweightOnly` | exists |
| `measured_lengths.thigh_m` | `MeasuredLengths` → thigh, hip → knee (0.245 H) | **fields undefined (P1)** |
| `measured_lengths.shank_m` | `MeasuredLengths` → shank, knee → ankle (0.246 H) | **P1** |
| `measured_lengths.trunk_m` | `MeasuredLengths` → trunk, shoulder → hip (0.288 H) | **P1, landmark (P3)** |
| `measured_lengths.foot_m` | foot length (0.152 H), feeds ankle x = −0.26 × foot (§2.6.1) | **P1, P4** |
| `measured_lengths.ankle_height_m` | ankle height (0.039 H, §2.6.1) | **P1, P4** |
| `bar_on_trunk` | bar position knob, baseline 0.95 (§2.6.2) | **range (P3)** |
| `shank_angle_deg` | baseline shank angle | **not an input today (P2)** |

## 3. Gaps and the exact changes proposed

### P1. `MeasuredLengths` has no fields and no source

SPEC-0045 §2.3 declares `scale(height, mass_kg, measured: &MeasuredLengths)`
and §2.1.1 says: *"No source for one exists in v1."* The struct's fields are
never listed, and nothing says how provenance travels.

**Proposed addition to SPEC-0045 §2.3**, after the `scale` signature:

```rust
/// A length the user measured, with where it came from. Never constructed
/// from the table: a table value is `None` in `MeasuredLengths`.
pub struct Measured<T> {
    pub value: T,
    pub on: NaiveDate,
    /// Free text from the user: instrument and posture. User content —
    /// rendered back to its author only, never to anyone else.
    pub method: String,
}

/// Every length the table can supply, each overridable by a measurement.
/// Landmarks as in the physics-lab protocol: standing heights from the floor,
/// segment = difference of two heights (Winter Fig. 4.1's own construction).
pub struct MeasuredLengths {
    pub thigh: Option<Measured<Metres>>,        // greater trochanter → lateral knee joint line
    pub shank: Option<Measured<Metres>>,        // lateral knee joint line → lateral malleolus
    pub trunk: Option<Measured<Metres>>,        // acromion → greater trochanter (see P3)
    pub foot: Option<Measured<Metres>>,         // heel → longest toe
    pub ankle_height: Option<Measured<Metres>>, // floor → lateral malleolus
}
```

**Proposed change to SPEC-0045 §2.8**, the `Assumption` enum:

```rust
-    TableDerivedLengths,                     // iff any length came from the table (v1: always)
+    TableDerivedLengths { lengths: Vec<Length> }, // the lengths that came from the table; absent iff none did
```

with `Length` a closed enum `{ Thigh, Shank, Trunk, Foot, AnkleHeight }`, so
a result says *which* lengths are population means, not only that some are.
Test B13 then reads: *"carries `TableDerivedLengths` listing exactly the
unmeasured lengths, and no such assumption when all five are measured."*

**Proposed amendment to R-0045 AC2** (it currently says lengths come from the
table, which a measured override contradicts word for word):

> **AC2. Anthropometric scaling from published tables.** Segment masses and
> centre-of-mass positions come from a standard anthropometric table
> (Dempster/Winter or equivalent), scaled by the user's height and weight.
> **Segment lengths come from the same table unless the user measured them;
> a measured length carries its date and method, and the result names every
> length that was table-derived.** The table and its source are cited in the
> spec — no invented constants.

The source of measurements is a new capability, so it is proposed as its own
requirement (P5) rather than folded into R-0045.

### P2. The baseline posture is under-determined

SPEC-0045 §2.6.1 lists the baseline shank angle as *"the unknown (§2.6.3) —
not the reel's fixed 22°"*. §2.6.3 says that without a torso-angle knob *"the
trunk angle θ is"* the unknown. With the thigh horizontal and the ankle
placed, the baseline has **two** unknowns (shank angle and θ) and **one**
balance equation. Any implementation must pick a second constraint, and the
text does not say which: exactly the "three implementations would all match
the spec" situation the architect review closed for the knobs.

**Proposed replacement for the shank-angle row of the §2.6.1 table:**

| quantity | value | provenance |
|---|---|---|
| shank angle from vertical | the user's **measured** shank angle at the bottom (side photo, `shank_angle_deg`) when present; otherwise **22°** | measured, or **convention** (the reel's value, no published source) carried as `Assumption::ConventionalShankAngle` |

and in §2.6.3, *"Which unknown"*: *"In the baseline the shank angle is
given (measured or conventional, §2.6.1) and θ is solved. With a torso-angle
knob θ is given and the shank angle is solved."* Add
`ConventionalShankAngle` to `Assumption`, present iff the baseline shank
angle was not measured, and a test: *"baseline with a measured shank angle
carries no `ConventionalShankAngle`; without one it does, and the solved
posture uses 22°."*

The protocol shows the shank angle is the field that moves the bar-only
moments most per unit of uncertainty (±3° measured, ±10° conventional), so
this is the input most worth capturing.

### P3. Bar position range and the trunk landmark

SPEC-0045 §2.6.2 bounds the bar position at 0.75–1.00 of hip → shoulder,
high-bar 0.95. Measured on the body, a **high bar sits on the upper
trapezius, above the acromion**: the example file's bar is 2.5 cm above it,
`bar_on_trunk = 1.05`, and the knob would reject a real lifter's real bar
position as `OutOfRange`.

The trunk's top landmark is also stated two ways: the §2.3 table's trunk row
says *shoulder → hip* (0.288 H, which is Winter's acromion height 0.818 minus
trochanter 0.530), while the HAT row says *greater trochanter →
glenohumeral*. The glenohumeral centre is not palpable; the acromion is.

**Proposed:** define the trunk length as **acromion → greater trochanter**
(the 0.288 H that the table actually gives), and change the bar-position row
of §2.6.2 to

| knob | sagittal effect | range | range provenance |
|---|---|---|---|
| bar position | attachment along the trunk line, as a fraction of acromion → trochanter; measured when the user gave `bar_on_trunk`, else high-bar 0.95 / low-bar 0.80 | **0.75–1.10** | stated convention: 1.10 admits a high bar on the upper trapezius above the acromion; candidate citation as before |

### P4. Ankle placement from measured foot and ankle height

SPEC-0045 §2.6.1 places the ankle at x = −0.26 × foot length and y = 0.039 H.
**Proposed:** use `foot` and `ankle_height` from `MeasuredLengths` when
present, table values otherwise, and list them under `TableDerivedLengths`
when they are table values (P1). No new convention is needed.

### P5. A new requirement: segment measurements (capture and storage)

The app stores body measurements as weight and body-fat percentage only
(`body_measurements`, migration 00008). Lengths need a home, an import path
and ownership rules, which are not R-0045's concern.

**Proposed requirement** (number assigned in sargentAI):

> **Segment measurements.** A user can record their own segment lengths, each
> with date and method, by importing a `medidas-sentadilla/1` document (or the
> `app_input` JSON that `mecanica sentadilla --json` emits) or by entering
> the same fields by hand. Each field is either measured (value, date,
> method) or absent; the app never stores an estimate as a measurement.
> Values outside the stated plausibility bands are rejected with the field
> named; values outside the typical bands are accepted with a warning. Per
> user, author-only, 404 never 403. The pseudonym in an imported file is
> discarded: the record belongs to the authenticated user. The free-text
> method is the user's own content and is never shown to anyone else. No
> photo is uploaded: only the numbers.

with the plausibility rules taken from physics-lab's validator as a starting
point (each one a labelled convention, not a cited range): landmark heights
within ±0.060 H (acromion, trochanter), ±0.050 H (knee) and ±0.025 H
(malleolus, foot) of Winter's fraction, warning beyond ±0.030 / ±0.025 /
±0.012 H; landmarks in head-to-floor order; thigh/shank between 0.70 and
1.40 (warning outside 0.85–1.20); shank angle 0–60° (warning outside
10–45°); bar within −20…+15 cm of the acromion (warning outside −12…+10 cm).

### P6. The reel's numbers and the app's numbers will differ, by design

For the same measurements, the app's R-0045 result and physics-lab's `model`
block differ for four stated reasons: the app adds segment weights (§2.4.3:
274 → ≈366 N·m for MECÁNICA 01), places the ankle from the foot (P4; the
teaching model fixes it at x = −0.04 m), attaches the bar at 0.95 or the
measured fraction (the teaching model puts it at the end of the trunk line),
and balances the system centre of mass rather than the bar over mid-foot.

**Proposed test for slice A** (extends B3): with physics-lab's conventions
(ankle at (−0.04, 0.09) m, bar over mid-foot as in B5b's heavy-bar limit)
and the inputs of `medidas/ejemplo-sentadilla.toml` (its measured lengths,
`bar_on_trunk` 1.0505 and shank angle 26°), the app's **bar component** of
the hip and knee moments reproduces the CLI's `model.joints[*].moment_nm`
for that file: 273.40 N·m and 143.53 N·m with 100 kg. One fixture, two repositories, the same number; any later difference
is then a stated assumption, never an accident.

## 4. What does not need to change

- R-0045 AC4–AC16 and the `Mechanics` output shape: the app computes its own
  result from the inputs above.
- The squat model's single bottom posture, quasi-static, sagittal: the
  protocol states the same limits.
- Stance width and foot rotation: the protocol does not measure them,
  because the teaching model does not use them. When the stance knob ships
  (slice B), hip breadth (0.191 H) and stance width can be added to the
  format as two more `medido | estimado` fields without changing anything
  else.
