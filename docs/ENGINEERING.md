# Engineering guide

Implementation-level documentation for people modifying the framework
itself. [ARCHITECTURE.md](ARCHITECTURE.md) is the shape of the system;
this is its mechanics — memory layout, calling convention, macro
expansion, the runtime's exact order of operations, measured limits,
failure modes, and extension recipes.

Every number in this document was measured on the shipped artifacts,
not estimated. Where something is unverified it says so. For the
type-level view — what abstractions exist across the ecosystem, which
were deliberately refused, and the seams ranked by how strongly they
are enforced — see [DESIGN.md](DESIGN.md).

---

## 1. Repository layout

```
Cargo.toml                     workspace: lessons-common + every lessons/*/crate
lessons-common/src/lib.rs      137 lines: the ABI macro + Prims/Readouts writers
lessons/<slug>/crate/          one crate per lesson (cdylib + rlib)
  Cargo.toml                   deps: lessons-common, optionally garust
  src/lib.rs                   model + draw() + lesson!() + #[cfg(test)] claims
public/
  js/runtime.js                371 lines: the whole browser half
  js/hub.js                    28 lines: renders hub cards from lessons/index.json
  js/lab.js                    legacy, used only by public/examples/*
  css/{tokens,fonts,lesson}.css  brand tokens · @font-face · framework page styles
  lessons/<slug>/              index.html (21-line stub), lesson.json, notes.html,
                               lesson.wasm (committed binary)
  lessons/index.json           generated hub index
  classroom/                   course + authoring guide (static HTML)
  examples/                    legacy JS-era lessons, retired as ports land
  _headers                     CSP and cache policy (Workers Assets honors it)
checks/                        independent Python implementations (L2)
tools/build-wasm.sh            16 lines: build every lesson crate → stage wasm
tools/gen-index.py             30 lines: manifests → lessons/index.json
.github/workflows/checks.yml   CI: L2 + cargo test + clippy + wasm build
```

`crate-type = ["cdylib", "rlib"]` is deliberate: **cdylib** produces the
`.wasm`, **rlib** lets `cargo test` link the same code natively. The
tests and the browser therefore execute identical logic — the "one
source of truth" property is structural, not a convention.

## 2. Build pipeline

```sh
export PATH="$(dirname "$(rustup which rustc)"):$PATH"   # see §9 landmine
cargo build --target wasm32-unknown-unknown --release -p lesson-<slug>
cp target/wasm32-unknown-unknown/release/lesson_<slug>.wasm \
   public/lessons/<slug>/lesson.wasm
```

`tools/build-wasm.sh` does this for every `lessons/*/crate`, deriving
crate names from directory names (`ls -d lessons/*/crate`), so a new
lesson needs no edit there. Workspace release profile:

```toml
opt-level = "z"      # size: these ship over the wire
lto = true
codegen-units = 1
panic = "abort"      # no unwinding in wasm; a panic becomes a trap (§7)
```

No `wasm-bindgen`, no `wasm-opt`, no npm — the toolchain is `cargo` plus
the `wasm32-unknown-unknown` target. Binaries are **committed** so the
site deploys from a bare checkout.

Measured artifact sizes (release, as shipped):

| Lesson | bytes | notes |
|---|---|---|
| projectile | 33 024 | no garust; smallest real lesson |
| three-mechanics | 37 637 | no garust (scalar mechanics) |
| two-mirrors | 39 346 | links garust `Vga2` |
| wave-equation | 45 449 | largest: mode tables + dissector |

Most of the floor is Rust core/fmt machinery, not lesson logic; a
second garust-linking lesson costs far less than the first.

## 3. The wasm module: memory and exports

Export section of a shipped lesson (parsed from the binary):

```
memory(mem), params_ptr(func), prims_ptr(func),
readouts_ptr(func), state_at(func), __data_end(global), __heap_base(global)
```

No imports: `WebAssembly.instantiate(bytes, {})` succeeds with an empty
import object. There is no allocator export and no `start` function.

**Linear memory.** Measured at 18–19 pages (≈1.2 MB) after
instantiation and first draw. Lessons that build `Vec`s inside `draw`
(three-mechanics, wave-equation) sit one page higher than those that
don't. Memory can grow when a lesson allocates; the runtime therefore
**re-creates its `Float64Array` views on every frame** rather than
caching them — a `memory.grow` detaches every existing view, and a
cached view would silently read a stale buffer. This is the single
most important rule when touching `draw()` in the runtime.

**The three buffers** are `static mut` arrays in the module, not heap
allocations, so their addresses are fixed for the module's lifetime:

| Buffer | Size | Direction |
|---|---|---|
| `LESSON_PARAMS` | `PARAM_CAP = 16` f64 | JS writes, Rust reads |
| `LESSON_PRIMS` | `PRIM_CAP = 8192` f64 | Rust writes, JS reads |
| `LESSON_READ` | `READ_SLOTS = 8` f64 | Rust writes, JS reads |

## 4. The calling convention

Per frame, exactly:

1. JS creates `new Float64Array(memory.buffer, params_ptr(), keys.length)`
   and writes parameter values **in manifest key order**, each already
   multiplied by its `scale` (so a τ-slider hands radians to Rust).
2. JS calls `state_at(n_params)`, which returns the number of f64s
   written to the prim buffer.
3. JS creates a view over `prims_ptr()` of exactly that length and
   paints it, then a view over `readouts_ptr()` of `READ_SLOTS`.

**The purity contract:** `state_at` must be a pure function of the
parameter values. No accumulated state, no clock reads, no randomness.
Consequences the framework depends on: scrubbing time backwards is
exact, a repaint never drifts, and `cargo test` can assert on the same
buffers the browser sees. `draw` receiving `&[f64]` (not `&mut`) is the
type-level half of this; the rest is discipline enforced by review.

## 5. `lessons_common::lesson!` — what it generates

The macro (137-line crate, no dependencies) expands to the three
statics and four exports. The body of `state_at`:

```rust
let all: &[f64; PARAM_CAP] = &*core::ptr::addr_of!(LESSON_PARAMS);
let p = &all[..n_params];
let mut prims  = Prims::new(&mut *core::ptr::addr_of_mut!(LESSON_PRIMS));
let mut read   = Readouts::new(&mut *core::ptr::addr_of_mut!(LESSON_READ));
$draw(p, &mut prims, &mut read);
prims.len()
```

Notes for anyone editing it:

- `addr_of!` / `addr_of_mut!` and the explicit `&[f64; PARAM_CAP]`
  binding are required: indexing a raw-pointer deref directly trips
  `clippy::implicit_autoref` under `-D warnings` (this cost a build).
- The `unsafe` is sound because wasm here is single-threaded, the
  statics have exactly one writer, and no reference outlives the call.
  If the framework ever gains threads (`SharedArrayBuffer`), this is
  the first thing that must change.
- `Prims::new` resets the write cursor to 0 every call, which is why a
  trapped frame does not poison the next one (§7).

**`Prims`** is a cursor over the buffer with one method per record type
(`view`, `point`, `segment`, `arrow`, `polyline`, `curve`). `polyline`
back-patches its point count after writing the points — the only
non-linear write in the framework. `curve(t0, t1, n, style, f)` samples
`f` at `n+1` points; it is the workhorse and keeps trajectory code out
of lessons.

**`Readouts`** is a thin `set(slot, value)` over the 8 slots. Slots are
addressed by index from the manifest, so **renumbering readouts in
`draw` without updating `lesson.json` silently mislabels the page** —
there is no compile-time link between them. This is the framework's
sharpest remaining foot-gun; see §10 for the proposed fix.

## 6. The runtime, phase by phase

`public/js/runtime.js`, in execution order:

1. **Refuse `file://`** — `fetch` of the manifest/wasm is blocked
   there; the page prints the serve command instead of dying silently.
2. **Load** manifest → notes → wasm, inside a try that reports failures
   as a visible alert strip (CSP hints included, since a missing
   `wasm-unsafe-eval` presents as an instantiate failure).
3. **Scaffold** the page from the manifest, in fixed order: header
   (eyebrow/title/lede) → claims box → stages grid → stepper → transport
   bar → legend → notes → try-this → footer. The lesson's `index.html`
   contributes only `<div class="ls" id="lesson">` and the script tag.
4. **Mappings**, one per view: `sx = (x−x0)·mx`, `sy = h−(y−y0)·my`,
   with `mx = viewBox.w/(x1−x0)`, `my = viewBox.h/(y1−y0)`. If
   `|mx−my| > 1e-9·max(mx,my)` and the view has not declared
   `uniform:false`, the runtime **throws at load** (§9 invariant 1).
5. **Controls**: one range input per non-hidden param; `input` events
   stop the sweep and redraw.
6. **Readouts**: one `<dd>` per manifest entry; formatters are
   `fix3` (3 decimals), `turns3` (3 decimals + " τ"), `sci`
   (exponential, exact 0 special-cased).
7. **Stepper** (if `steps` present): prev/next buttons mutate the
   hidden `stepParam`, show the matching panel, and redraw — the step
   number reaches `draw` as an ordinary parameter, so per-step
   highlighting is lesson-side Rust, not JS.
8. **Clock**: a single `requestAnimationFrame` loop advances the swept
   param with `dt` clamped to 60 ms (tab-switch protection) and wraps
   modulo its range.
9. **Paint**: walk the prim buffer, dispatch on tag, append SVG into
   the current view's `<g>` (cleared each frame). Arrowheads are
   computed in *screen* space so they stay a constant pixel size
   regardless of world scale.

`draw()` guards itself with its own try/catch: it runs from event
listeners and rAF, i.e. **outside** the startup try block, so a wasm
trap there would otherwise reach only the console. On failure it
disarms the sweep, prints the strip with the parameter values at
failure, and latches so a broken rAF loop cannot spam.

## 7. Failure modes (measured)

Probed with a purpose-built overflow wasm through the real ABI:

| Condition | Behavior | Recovery |
|---|---|---|
| prims ≤ `PRIM_CAP` | normal | — |
| prims > `PRIM_CAP` | Rust slice bounds panic → `panic=abort` → wasm `unreachable` → JS `RuntimeError: unreachable` | **the instance survives**: the next `state_at` rewrites from index 0 and returns normally (verified) |
| panic anywhere in `draw` | same trap path | same |
| `file://` | refused before any fetch, with instructions | serve over HTTP |
| missing/invalid manifest or wasm | alert strip naming the error | fix and reload |
| skewed view aspect | throws during scaffold → alert strip | fix the world box or declare `uniform:false` |
| CSP without `wasm-unsafe-eval` | instantiate throws; strip says so | fix `_headers` |

Measured worst-case prim usage, sweeping each lesson's full parameter
grid (5 values per axis, all combinations) through the real ABI:

| Lesson | worst f64s | % of `PRIM_CAP` | combos tested |
|---|---|---|---|
| two-mirrors | 42 | 0.5 % | 125 |
| projectile | 125 | 1.5 % | 125 |
| wave-equation | 587 | 7.2 % | 15 625 |
| three-mechanics | 1 633 | 19.9 % | 3 125 |

Headroom is ~5× on the worst lesson. A lesson whose prim count scales
with a parameter (wave-equation: beads; three-mechanics: level sets)
should be swept like this before shipping — the sweep above is a
~20-line browser snippet, not a tool the repo owns yet (§10).

## 8. Testing

| Layer | Command | What it proves |
|---|---|---|
| L1 | `cargo test --workspace` | every claim, on the rlib build of the exact crates that become wasm |
| L1a | `cargo clippy --workspace --all-targets -- -D warnings` | no lint regressions (in CI since a commit once claimed green without checking the exit code) |
| L1b | `cargo build --workspace --target wasm32-unknown-unknown --release` | the browser build still compiles |
| L2 | `python3 checks/run.py` | independent Python implementations of flagship claims — N-version, catches a wrong-but-consistent Rust model |
| L3 | drive the page in a browser | that the *page* works: listeners bound, clock advancing, readouts against theory |

L3 is manual and deliberately not screenshot-based: a screenshot passed
a page whose script had died after first paint. The reliable probe is
reading DOM state and readouts after synthetic `input` events.

CI (`.github/workflows/checks.yml`) checks out **garust as a sibling**
(matching the path dependency), then runs L2 → L1 → L1a → L1b. Green CI
means every quantitative claim on the site held on a clean machine.

## 9. Landmines

1. **Homebrew Rust shadows rustup** on the dev machine and ships no
   cross-targets. `RUSTC=$(rustup which rustc)` alone is *worse than
   nothing*: `cargo` then mixes toolchains and clippy fails with E0514.
   Always prepend the whole bin directory to `PATH`.
2. **Never cache `Float64Array` views** across frames — `memory.grow`
   detaches them (§3).
3. **Readout slot numbers are unchecked** across the Rust/JSON seam (§5).
4. **`polyline` back-patches its count**; if you add a record type that
   writes a length prefix, follow that pattern exactly or the painter
   desynchronizes and throws "unknown prim tag" mid-buffer.
5. **`draw` must not read a clock** — time arrives as a parameter. A
   `Date.now()` equivalent inside Rust would break scrub-safety and
   every golden-style test.

## 10. Extension recipes

**Add a prim record.** Pick the next free tag; write the `Prims`
method; add a painter branch; document the layout in ARCHITECTURE §3.
Keep records self-describing in length — the painter advances by
computing record size from the tag alone.

**Add a readout formatter.** Extend the `FMT` map in the runtime and
the `fmt` enum in ARCHITECTURE §4.

**Add a runtime capability.** The bar: it must be *data-drivable from
the manifest* and useful to at least two lessons — otherwise it belongs
lesson-side in Rust. The stepper is the model to follow: the runtime
renders panels and owns navigation, while the semantic effect travels
into `draw` as an ordinary parameter.

**Known work, deliberately not done yet:**

- *Slot-name safety* (§5, §9.3): let manifests address readouts by name
  and have `lesson!` generate an enum, so a renumber is a compile error.
- *A capacity sweep tool*: the §7 sweep should live in `tools/` and run
  in CI against each manifest's parameter grid.
- *Runtime shrink*: move scaffold and painting into Rust as emitted
  markup strings, leaving JS as a ~80-line pipe (RFC-001).
- *Legacy retirement*: `public/examples/*` and `public/js/lab.js` die
  with the optics and bouncing-ball ports.
