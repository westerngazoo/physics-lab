# physics-lab

Interactive physics, taught claim-first, for
[physics.goosethropic.systems](https://physics.goosethropic.systems)
(live today at
[goosethropic-physics…workers.dev](https://goosethropic-physics.gustavo-delgadillo.workers.dev)).

Every page states falsifiable claims and lets you check them — on the
page, in an independent Python implementation, and with `cargo test` on
the **same Rust the browser runs** (lessons compile to WebAssembly).
The site doubles as a course, and the framework is small enough that
writing a new lesson is one function and one manifest.

## Choose your door

| You are… | Start here | The commands |
|---|---|---|
| **a student** | [The Classroom](https://goosethropic-physics.gustavo-delgadillo.workers.dev/classroom/) — six units of predict-then-check exercises over the lessons | none needed; the readouts are live claims |
| **a student who wants proof** | the ladder below | `python3 checks/run.py` → `cargo test --workspace` → break a claim on purpose |
| **an instructor** | the Classroom's instructor note — claims map to named tests, so "break-it" homework grades as a diff + a failing test's output | fork; CI re-proves every claim on your students' pushes |
| **a lesson author** | [Writing a Lesson](https://goosethropic-physics.gustavo-delgadillo.workers.dev/classroom/authoring.html) — walks the live minimal template line by line | `sh tools/build-wasm.sh` · `python3 tools/gen-index.py` |
| **a maintainer** | [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — the stack, the wasm ABI, the manifest schema, the invariants and the bugs that bought them | see §8 Toolchain there (including the Homebrew/rustup landmine) |

Decision history — how the architecture got this way, through three
owner-review rounds — is [docs/RFC-001](docs/RFC-001-lesson-framework.md).

## The rule for these pages

Physics demos drift toward *looking* right rather than *being* right.
Every page here is built the other way round: derive the relationship,
check it numerically (closed forms only — nothing is stepped in the
browser), then draw it — and put any approximation's error **on
screen**, with a control that breaks it on purpose. Where a shortcut
would have been invisible, it is documented instead of taken. τ is the
circle constant throughout.

## For students: run the tests, then break them

The Rust lessons build against [garust](https://github.com/westerngazoo/garust)
as a *sibling checkout* — clone both, side by side:

```sh
git clone https://github.com/westerngazoo/garust
git clone https://github.com/westerngazoo/physics-lab
cd physics-lab
```

1. **No toolchain:** the deployed pages' readouts are the claims, live.
2. **Python only:** `python3 checks/run.py` — the independent second
   implementation, zero dependencies.
3. **The real thing:** `cargo test --workspace` — the exact code the
   pages run.
4. **Break it.** Open a lesson's claims (say
   `lessons/three-mechanics/crate/src/lib.rs`, claim C4), change
   `e * e` to `e`, and watch physics disagree with you. Then rebuild
   the browser side (`sh tools/build-wasm.sh`), serve `public/`, and
   see *your* physics live. A claim failing under your hand is the
   framework working.

CI runs all of it — Python, `cargo test`, clippy, and the wasm build —
on every push (`.github/workflows/checks.yml`).

## What a new lesson costs

Four small files, no HTML layout, no JavaScript — the framework
(`lessons-common` + the one shared `public/js/runtime.js`) does the
rest, and the hub regenerates itself:

```
lessons/<slug>/crate/src/lib.rs      model + ONE draw() + claims as tests
public/lessons/<slug>/lesson.json    the whole page as data
public/lessons/<slug>/notes.html     the prose
public/lessons/<slug>/index.html     a 21-line stub
```

Full walkthrough (with the real template source embedded):
[the authoring guide](https://goosethropic-physics.gustavo-delgadillo.workers.dev/classroom/authoring.html).

## Local

```sh
cd public && python3 -m http.server 8000
```

## Deploy

```sh
npx wrangler deploy
```

Cloudflare Workers Static Assets; `public/_headers` carries the strict
CSP (`script-src 'self' 'wasm-unsafe-eval'` — everything first-party,
fonts self-hosted, no telemetry). The custom domain attaches once in
the Cloudflare dashboard.
