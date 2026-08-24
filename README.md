# physics-lab

Interactive physics notes for [physics.goosethropic.systems](https://physics.goosethropic.systems).

A zero-build static site served by Cloudflare Workers Static Assets. No
framework, no bundler, no third-party requests at runtime — the strict CSP in
`public/_headers` keeps everything first-party, which is why every page's
JavaScript lives in its own external file and the fonts are self-hosted.

## The rule for these pages

Physics demos drift toward *looking* right rather than *being* right, and on a
screen the two are easy to confuse. So each page is built the other way round:
the relationship is derived first, checked numerically against a closed form,
and only then drawn. Where a shortcut would have been invisible, it is
documented instead of taken.

Concretely, so far that has meant:

- **Lens thickness scales as `1/f`**, because the lensmaker's equation
  `1/f = (n−1)(1/R₁ − 1/R₂)` says it must. A fixed outline would have let you
  drag the focal length while the glass stayed put.
- **Both axes of a diagram share one scale.** An 18 × 6.8 world mapped into a
  1000 × 400 viewBox skews every ray angle by ~6% — invisible to the eye,
  fatal to a geometry diagram.
- **Bounce times are solved, not stepped.** A fixed-step integrator notices the
  floor a fraction of a step late and quietly loses ~0.1% of the energy per
  impact. The apexes here match `h_n = e^(2n)·h₀` exactly.

## Contents

| Page | Claim it makes |
|---|---|
| `examples/optics/` | Four devices, one equation: a convex lens and a concave mirror share the same five cases — the deep category is the sign of f, not lens vs mirror. |
| `examples/bouncing-ball/` | Infinitely many bounces, finite total time: `T = √(2h₀/g)·(1+e)/(1−e)` — and the apex heights contain no g at all. |
| `examples/wave-equation/` | Newton on a chain of beads *is* the wave equation, up to a limit you can take with a slider. Evolution by exact eigenmodes, verified against an independent integrator to 2e-9. |

Shared shell: `public/js/lab.js` — the world→screen mapping (throws on a
skewed aspect), SVG construction, control binding. Models stay per-page
code on purpose; element/family tables and configs are data.

## Local

```sh
cd public && python3 -m http.server 8000
```

## Deploy

```sh
npx wrangler deploy
```

The custom domain is attached once, either in the Cloudflare dashboard
(Workers → `goosethropic-physics` → Settings → Domains & Routes) or by
uncommenting the `routes` block in `wrangler.toml`.
