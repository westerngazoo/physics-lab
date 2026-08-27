# Design: abstractions, interfaces, and the objects that exist

The type-level view. [ARCHITECTURE.md](ARCHITECTURE.md) is the shape,
[ENGINEERING.md](ENGINEERING.md) the mechanics; this is the *object
model* — what abstractions exist across the three layers, what each one
is for, and, as often, why an obvious abstraction was refused.

Every type listed here was inventoried from the source, not recalled.

---

## 0. The headline: three layers, three strategies

The ecosystem does not have one design philosophy; it has three, chosen
per layer because the layers have different problems:

| Layer | Strategy | Trait count | Why |
|---|---|---|---|
| **garust** (kernel) | deep parametric polymorphism | 5 traits, heavily bounded | one implementation must serve every Clifford signature and scalar type |
| **motoreel** (studio) | closed enums + one narrow trait | 1 trait | a fixed vocabulary of shapes and primitives; exactly one axis of genuine variation (output sink) |
| **physics-lab** (classroom) | **no traits at all** | 0 | one axis of variation (the lesson), and it is a *function*, not an object |

Reading that table top to bottom is reading abstraction being *spent
where it buys something* and refused where it would only decorate. The
rest of this document is the argument for each row.

---

## 1. garust — parametric polymorphism as the product

The kernel's entire value proposition is "one implementation, every
algebra," so its abstraction is type parameters with real bounds.

### The trait surface

```rust
pub trait Algebra: Copy + Debug {
    const P: usize;   // generators squaring to +1
    const Q: usize;   // ... to −1
    const R: usize;   // ... to 0 (degenerate)
    const N: usize = Self::P + Self::Q + Self::R;   // derived
    const DIM: usize = 1 << Self::N;                // derived: 2^N blades
    type Blades<T>: BladeStore<T>;                  // backing storage
    const CAYLEY: …;                                // compile-time product table
}

pub trait BladeStore<T>: Copy + Index<usize, Output = T> + IndexMut<usize>
pub trait Ring:   /* +, −, ×, zero, one */
pub trait Scalar: Ring + Div<Output = Self> + Display { type Magnitude; }
pub trait Real:   Scalar<Magnitude = Self> + PartialOrd { /* sqrt, sin, … */ }
```

Four design decisions worth naming:

1. **The signature is a type, not a value.** `Multivector<A: Algebra,
   T: Ring>` means `Cl(3,0,1)` and `Cl(4,1,0)` are *different types*.
   Mixing a PGA point with a CGA sphere is a compile error, not a
   runtime check — and blade counts are `const`, so storage is exactly
   `2^N` with no allocation.
2. **The scalar ladder is layered by capability.** `Ring` (arithmetic)
   → `Scalar` (division, display) → `Real` (transcendentals, ordering).
   Each operation demands only what it needs, so exact/rational scalars
   remain usable for everything short of `exp`.
3. **`Magnitude` is an associated type**, letting a scalar declare what
   its norm lives in.
4. **Traits describe capability, never identity.** There is no
   `trait Rotor`, no `trait Transform`. A rotor is a `Motor` whose
   *values* satisfy an invariant — which brings us to the most
   instructive type in the ecosystem:

```rust
pub struct Motor<T: Scalar> { versor: Pga<T> }   // field is PRIVATE
```

The field is private because the type's meaning is an invariant — an
even-grade unit versor — that the constructors (`identity`, `rotor`,
`translator`, `from_versor`) establish and the operations preserve. The
type is an *encapsulated invariant*, not a data bundle. Compare
`Multivector`, whose `coeffs` are public because a general multivector
has no invariant to protect.

---

## 2. motoreel — closed vocabularies and one seam

The studio knows its whole world: three-ish shapes, four-ish
primitives, one equation per optical element. Its design is therefore
**enums, not trait objects** — with exactly one trait, at the one place
genuine third-party variation exists.

### The object model

```rust
// Geometry the author writes (model space)          — a closed set
pub enum Shape { Point(pga::Point), Segment(..), Polyline(..), Edges(..) }

// Geometry the sink consumes (image space)          — its mirror
pub enum Prim2 { Point{at,style}, Segment{a,b,style},
                 Polyline{points,style}, Edges{segments,style} }

pub struct Object { pub shape: Shape, pub style: Style, pub track: Track }
pub struct Scene  { pub objects: Vec<Object>, pub camera: Camera, pub duration: f64 }
pub struct ObjectId(usize);                    // opaque newtype handle

pub struct Camera { pub pose: Motor3, pub projection: Projection }
pub enum Projection { Pinhole { focal: f64 }, Orthographic }

pub struct Track { keys: Vec<(f64, Motor3)>, eases: Vec<Ease> }   // private!
pub enum Ease { Linear, SmoothStep, SmootherStep, Custom(fn(f64) -> f64) }

pub enum TrackError  { Empty, NonIncreasing{index}, NoSuchSpan{index}, InvalidSpin }
pub enum RecordError { InvalidTimestep, Track(TrackError) }
```

**Why enums instead of `Box<dyn Drawable>`.** A trait object would let
anyone add a shape — and that is precisely what must *not* be easy
here. Every shape has to be projectable, cullable, and expressible as
primitives a sink can paint; a closed enum makes adding one a
deliberate act that the compiler then drags through every match arm
(`Shape::Edges` did exactly that, and the exhaustiveness error in a
test helper was the feature working). It also keeps `Prim2` `Clone +
PartialEq`, which is what makes byte-golden and bit-identity tests
possible at all.

**`Shape` and `Prim2` are deliberately parallel, not shared.** They
look alike and are never unified, because the boundary between them is
the point: `Shape` holds PGA points in model space, `Prim2` holds
`Pt2` in image space and imports *nothing* from garust. That single
constraint — `prim.rs` has no geometry dependency — is what lets a sink
be written by someone who has never heard of a motor.

**The one trait, and why it earns its place:**

```rust
pub trait FrameSink {
    fn frame(&mut self, index: usize, prims: &[Prim2]) -> io::Result<()>;
}
impl Scene { pub fn render<S: FrameSink + ?Sized>(&self, fps: f64, sink: &mut S) -> io::Result<()>; }
```

This is the *only* axis where an unknown implementor is expected (SVG
today; PPM, raster, a test double tomorrow). One method, no associated
types, no lifetime games. `?Sized` is deliberate so `&mut dyn FrameSink`
works — static dispatch by default, dynamic when a caller wants it.
Note what is absent: no `finish()`/`flush()` hook, because every frame
is an independent file. Speculative interface surface was refused.

**Type-encoded invariants.** `Track`'s fields are private with the
structural rule `eases.len() == keys.len() − 1` and strictly increasing
finite key times, established at the only fallible constructor
(`Track::keys → Result<_, TrackError>`) and preserved by every method.
`Track::hold` exists precisely so the common single-key case is
*infallible* — pushing a `Result` out of call sites that cannot fail.
Deliberately **no `PartialEq` on `Track`**: `Ease::Custom` carries a
`fn` pointer whose address equality is codegen-dependent, so deriving
it would offer a comparison that is not meaningfully true.

**Errors are enums, typed and layered.** `RecordError::Track(TrackError)`
with `From<TrackError>` gives `?` propagation across the layer boundary
while keeping the two vocabularies distinct — and `source()` chains them
for a reader.

---

## 3. physics-lab — the framework with no traits

Inventory of the framework crate, complete:

```rust
pub struct Prims<'a>    { buf: &'a mut [f64], at: usize }
pub struct Readouts<'a> { buf: &'a mut [f64] }
macro_rules! lesson
pub const PRIM_CAP / READ_SLOTS / PARAM_CAP
```

**Zero traits. Zero `impl … for` blocks.** That is not laziness; it is
the design, and here is the argument.

### The lesson interface is a function signature

```rust
fn draw(p: &[f64], out: &mut Prims, read: &mut Readouts)
lessons_common::lesson!(draw);
```

The obvious OO alternative is a trait:

```rust
trait Lesson {                        // considered and rejected
    fn defaults() -> Params;
    fn draw(&self, p: &[f64], out: &mut Prims, read: &mut Readouts);
}
```

Rejected for four reasons:

1. **A lesson has no state to carry**, so `&self` would always be
   `&()`. A trait whose implementor is a unit struct is a function with
   extra steps.
2. **There is exactly one call site**, generated by the macro. Traits
   buy you polymorphism at call sites; there is no polymorphism to buy
   when the wasm module contains exactly one lesson by construction.
3. **The real interface is not in Rust at all.** It is the wasm export
   set plus the manifest's parameter order — a boundary a Rust trait
   cannot type-check anyway. Pretending otherwise would give a false
   sense of safety at the seam that actually matters (see §5).
4. **Teachability.** A lesson author writes one function and a table of
   claims. Every trait, generic, or lifetime in that file is a concept
   between a physicist and their physics.

### What the macro is, in design terms

`lesson!` is **not** a code-generation convenience; it is the
framework's *implementation of an interface it cannot express in the
type system*. The ABI (three fixed-address buffers, four exports, a
purity contract) is uniform across lessons and must be — so it is
generated once, identically, rather than implemented per lesson. Read
it as: *the trait Rust cannot write, expanded by macro instead.*

The design cost is honest and worth stating: macro-generated interfaces
have no `impl` block to look at, so the contract lives in the macro's
docs and in this document. That trade is acceptable because the
interface has exactly one shape and is never overridden.

### `Prims` — a writer, not a scene graph

```rust
impl<'a> Prims<'a> {
    pub fn view(&mut self, v: usize);
    pub fn point(&mut self, x: f64, y: f64, style: usize);
    pub fn segment(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, style: usize);
    pub fn arrow(&mut self, …);
    pub fn polyline<I: IntoIterator<Item = (f64, f64)>>(&mut self, pts: I, style: usize);
    pub fn curve(&mut self, t0: f64, t1: f64, n: usize, style: usize, f: impl Fn(f64) -> (f64, f64));
}
```

Three design notes:

- **It is append-only over a borrowed slice.** `Prims<'a>` owns
  nothing; the lifetime ties it to the static buffer for exactly one
  `state_at` call, so no lesson can stash one and write between frames.
  The type system enforces the frame boundary.
- **The only two generic parameters in the entire framework** live
  here, on `polyline` and `curve` (`IntoIterator`, `impl Fn`), and both
  exist to let lessons pass iterators and closures without allocating.
  Generics as ergonomics, not as architecture.
- **`curve` is the deliberate anti-abstraction.** Rather than a
  `Trajectory` type or a `Plottable` trait, sampling a function into a
  polyline is one method every lesson shares. The hierarchy that
  *could* exist here (curve → trajectory → level set → graph) would be
  four types describing one loop.

### Styles are indices, not objects

`style: usize` indexes the manifest's `styles` array; there is no
`Style` struct on the Rust side of this framework at all. Presentation
lives in data (`{var, width, dash}` pointing at CSS custom properties)
so a lesson's Rust never encodes a color, and re-theming touches no
code. The cost — an unchecked index — is bounded by the painter, which
throws on an unknown style rather than drawing something wrong.

---

## 4. The seams, ranked by strength

The system's real interfaces, strongest guarantee first:

| Seam | Enforced by | Strength |
|---|---|---|
| garust algebra/scalar bounds | the type system | compile error on misuse |
| `Motor` / `Track` invariants | private fields + fallible constructors | unrepresentable illegal states |
| `Shape`/`Prim2` vocabulary | closed enums + exhaustive matches | compile error when extended |
| `FrameSink` | a one-method trait | compile error |
| lesson `draw` signature | the `lesson!` macro | compile error |
| **prim record layout** | shared convention (Rust writer ↔ JS painter) | runtime throw on unknown tag |
| **manifest ↔ `draw` param order** | convention only | **silent misbehavior** |
| **readout slot numbers** | convention only | **silent mislabeling** |

The line between rows 5 and 6 is where Rust stops and the browser
begins, and everything below it is a design debt, not a design.

---

## 5. The weakest seam, named

`draw` receives `&[f64]` in manifest key order and writes readouts by
integer slot. Both are conventions with no checker: renumber a readout
in Rust without editing the JSON and the page mislabels itself
silently; reorder `params` in the manifest and the physics quietly
receives permuted arguments.

This is the one place the framework's minimalism costs real safety, and
the fix is designed but not built:

```rust
// sketch: names, generated into an enum by the macro
lesson! {
    draw,
    params:   [theta, v, g],           // → struct Params { theta: f64, … }
    readouts: [angle, range, apex],    // → enum Slot { Angle, Range, Apex }
}
```

`draw` would then take `&Params` and `read.set(Slot::Range, …)`, and
`tools/gen-index.py` could cross-check the manifest against the
generated names at build time — turning two silent failure modes into
compile errors. Until then the mitigation is convention plus review,
and the honest statement is that this seam is the framework's soft spot.

---

## 6. Rejected designs, and why

- **A `Lesson` trait** — §3: no state, one call site, and the real
  interface is not expressible in Rust anyway.
- **Trait objects for shapes/primitives** — §2: openness is the wrong
  property here; exhaustive matches are how a vocabulary extension gets
  reviewed, and `PartialEq` on `Prim2` is what makes golden tests work.
- **A scene graph in the lab** — a lesson emits primitives directly;
  there is no retained tree to keep in sync with the model, which is
  what makes "pure function of params" enforceable.
- **`wasm-bindgen`** — it would generate thousands of lines of glue and
  pull in npm to erase ~60 lines of hand-written boundary. Sixty lines
  fully understood beat zero lines nobody reads.
- **A JS mini-kernel for GA** (an earlier plan) — deleted the moment
  lessons became Rust: the classroom's kernel *is* garust, so there is
  nothing to cross-check because there is only one implementation.
- **A `Style` type in `lessons-common`** — presentation belongs in the
  manifest; a Rust struct would put colors in physics code.
- **Generic `Prims<T: Scalar>`** — the wire format is `f64` by
  definition; generality there would parameterize a constant.

---

## 7. Rules for adding an abstraction here

Given the above, the bar for introducing a trait, generic, or new type
in this repo:

1. **Name the second implementor.** If you cannot, you want a function.
   (`FrameSink` passed: SVG, PPM, test doubles.)
2. **Prefer a closed enum to an open trait** unless third parties must
   extend it. Exhaustiveness is a review tool.
3. **Encapsulate invariants, expose data.** Private fields when a type
   means something (`Motor`, `Track`); public when it is a bundle
   (`Multivector::coeffs`, `Object`, `Scene`).
4. **Make illegal states unrepresentable before adding a validator** —
   and where a fallible constructor is unavoidable, add the infallible
   convenience for the common case (`Track::hold`).
5. **Do not derive a trait whose semantics you cannot defend**
   (`PartialEq` on a `fn`-pointer-carrying type).
6. **Generics for ergonomics are fine; generics as architecture need a
   second implementor** (rule 1 again).
7. **If the abstraction sits between a physicist and their physics, it
   had better pay for the rent.**
