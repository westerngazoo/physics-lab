//! Deriving the Wave Equation — the chain of beads, ported to the
//! framework, now with the snapshot dissector (RFC-001 §4.3.1).
//!
//! The model is the derivation: N beads under tension obey
//! `y_i'' = c²·(y_{i+1} − 2y_i + y_{i−1})/Δx²`, and Δx → 0 turns Newton
//! into `y_tt = c² y_xx`. Evolution is EXACT eigenmode superposition —
//! a pure function of `t`, nothing stepped, verified here against an
//! independent Verlet integration (W2).
//!
//! The dissector freezes any instant and magnifies one bead: its two
//! tension pulls drawn along the actual string segments, each split
//! into the TRUE transverse component `T·sin θ` and the component the
//! derivation uses, `T·tan θ = T·slope`. At honest amplitudes they
//! coincide to a fraction of a percent; drag the exaggeration slider
//! and watch the small-angle approximation fail — a modeling choice
//! with a budget, not a magic step (W6).
//!
//! Angles and mode numbers measured against τ throughout: the mode
//! frequencies are `ω_k = (2c/Δx)·sin(kτ/(4(N+1)))`, and the lattice
//! cutoff speed is `4/τ ≈ 63.7 %` of `c` — the wrinkle the fidelity
//! readout is honest about (W4).

use std::f64::consts::TAU;

const L: f64 = 10.0;
const BASE_AMP: f64 = 1.3;
const FOCUS_FRAC: f64 = 0.45;

/// The chain's exact description at one instant, from closed form.
pub struct Chain {
    pub n: usize,
    pub dx: f64,
    /// Bead displacements y_1..y_n (walls are y_0 = y_{n+1} = 0).
    pub y: Vec<f64>,
    /// Energy-weighted mean phase-speed ratio of this pluck's spectrum.
    pub fidelity: f64,
}

/// Mode frequencies of the N-bead chain.
pub fn omegas(n: usize, c: f64) -> Vec<f64> {
    let dx = L / (n + 1) as f64;
    (1..=n)
        .map(|k| (2.0 * c / dx) * (k as f64 * TAU / (4.0 * (n + 1) as f64)).sin())
        .collect()
}

/// Triangle pluck at `pos` (fraction of L), amplitude `amp`.
pub fn pluck(n: usize, pos: f64, amp: f64) -> Vec<f64> {
    (1..=n)
        .map(|i| {
            let x = i as f64 / (n + 1) as f64;
            amp * if x <= pos { x / pos } else { (1.0 - x) / (1.0 - pos) }
        })
        .collect()
}

/// Discrete sine transform: mode coefficients of a shape.
pub fn dst(y: &[f64], n: usize) -> Vec<f64> {
    (1..=n)
        .map(|k| {
            let s: f64 = (1..=n)
                .map(|i| y[i - 1] * (k as f64 * TAU * i as f64 / (2.0 * (n + 1) as f64)).sin())
                .sum();
            2.0 / (n + 1) as f64 * s
        })
        .collect()
}

/// Exact bead displacements at time `t` (zero initial velocity pluck).
pub fn evolve(a: &[f64], w: &[f64], n: usize, t: f64) -> Vec<f64> {
    (1..=n)
        .map(|i| {
            (1..=n)
                .map(|k| {
                    a[k - 1]
                        * (w[k - 1] * t).cos()
                        * (k as f64 * TAU * i as f64 / (2.0 * (n + 1) as f64)).sin()
                })
                .sum()
        })
        .collect()
}

/// Everything the views need at `(params, t)` — one pure call.
pub fn chain(n: usize, c: f64, pos: f64, amp: f64, t: f64) -> Chain {
    let dx = L / (n + 1) as f64;
    let w = omegas(n, c);
    let a = dst(&pluck(n, pos, amp), n);
    let y = evolve(&a, &w, n, t);
    let (mut num, mut den) = (0.0, 0.0);
    for k in 1..=n {
        let th = k as f64 * TAU / (4.0 * (n + 1) as f64);
        let e = (a[k - 1] * w[k - 1]).powi(2);
        num += e * th.sin() / th;
        den += e;
    }
    Chain { n, dx, y, fidelity: if den > 0.0 { num / den } else { 1.0 } }
}

/// The dissected bead: its two segment slopes and the sin-vs-tan story.
pub struct Dissection {
    pub j: usize,
    /// Slopes of the left and right string segments at the focus bead.
    pub slope_l: f64,
    pub slope_r: f64,
    /// Worst relative error of using tan θ (the slope) for sin θ.
    pub tan_err: f64,
    /// The second difference — the net transverse pull, in slope units.
    pub d2: f64,
}

pub fn dissect(ch: &Chain) -> Dissection {
    let j = (FOCUS_FRAC * (ch.n + 1) as f64).round() as usize;
    let j = j.clamp(2, ch.n - 1);
    let (yl, yj, yr) = (ch.y[j - 2], ch.y[j - 1], ch.y[j]);
    let slope_l = (yj - yl) / ch.dx;
    let slope_r = (yr - yj) / ch.dx;
    let err = |s: f64| {
        let sin = s / (1.0 + s * s).sqrt();
        if sin.abs() < 1e-15 { 0.0 } else { ((s - sin) / sin).abs() }
    };
    Dissection {
        j,
        slope_l,
        slope_r,
        tan_err: err(slope_l).max(err(slope_r)),
        d2: slope_r - slope_l,
    }
}

// ---- the wasm boundary --------------------------------------------------

use lessons_common::{Prims, Readouts};

/// Params (manifest order): [N, c, pos, xa (amplitude exaggeration), t, step].
fn draw(p: &[f64], out: &mut Prims, read: &mut Readouts) {
    let n = (p[0] as usize).clamp(5, 80);
    let (c, pos, xa, t, step) = (p[1], p[2], p[3], p[4], p[5] as i32);
    let amp = BASE_AMP * xa;
    let ch = chain(n, c, pos, amp, t);
    let ds = dissect(&ch);

    // ---- view 0: the chain ---------------------------------------------
    out.view(0);
    out.segment(0.0, 0.0, L, 0.0, 0); // rest line
    out.segment(0.0, -2.9, 0.0, 2.9, 0);
    out.segment(L, -2.9, L, 2.9, 0);
    // the string through walls and beads
    out.polyline(
        std::iter::once((0.0, 0.0))
            .chain((1..=n).map(|i| (i as f64 * ch.dx, ch.y[i - 1])))
            .chain(std::iter::once((L, 0.0))),
        1,
    );
    for i in 1..=n {
        let style = if i == ds.j && step >= 1 { 3 } else { 2 };
        out.point(i as f64 * ch.dx, ch.y[i - 1], style);
    }
    // step >= 2: emphasize the focus bead's two segments on the chain too
    if step >= 2 {
        let (xj, yj) = (ds.j as f64 * ch.dx, ch.y[ds.j - 1]);
        out.segment(xj - ch.dx, ch.y[ds.j - 2], xj, yj, 4);
        out.segment(xj, yj, xj + ch.dx, ch.y[ds.j], 4);
    }

    // ---- view 1: the snapshot dissector --------------------------------
    // Bead-local frame, zoomed so one spacing spans 0.85 inset units.
    out.view(1);
    let z = 0.85 / ch.dx;
    let (yl, yj, yr) = (ch.y[ds.j - 2], ch.y[ds.j - 1], ch.y[ds.j]);
    let (lx, ly) = (-0.85, (yl - yj) * z);
    let (rx, ry) = (0.85, (yr - yj) * z);
    // the two string segments through the bead
    out.segment(lx, ly, 0.0, 0.0, 1);
    out.segment(0.0, 0.0, rx, ry, 1);
    out.point(0.0, 0.0, 3);
    out.point(lx, ly, 2);
    out.point(rx, ry, 2);
    if step >= 2 {
        // unit tension pulls along each segment (drawn at fixed length)
        let draw_side = |out: &mut Prims, tx: f64, ty: f64, slope: f64| {
            let len = (tx * tx + ty * ty).sqrt();
            let (ux, uy) = (tx / len, ty / len);
            let s = 0.62;
            out.arrow(0.0, 0.0, s * ux, s * uy, 4); // T along the string
            let sin = slope / (1.0 + slope * slope).sqrt();
            // components drawn from the bead, side by side:
            let off = if tx > 0.0 { 0.10 } else { -0.10 };
            out.arrow(off, 0.0, off, s * sin, 5);          // T·sin θ (true)
            out.arrow(off * 1.6, 0.0, off * 1.6, s * slope, 6); // T·tan θ (used)
        };
        draw_side(out, lx, ly, ds.slope_l);
        draw_side(out, rx, ry, ds.slope_r);
    }
    if step >= 3 {
        // the net pull: the second difference, Newton's right-hand side
        let d2v = (ds.d2 * 0.30).clamp(-1.05, 1.05);
        out.arrow(0.0, 0.0, 0.0, d2v, 3);
    }

    read.set(0, t);
    read.set(1, ch.dx);
    read.set(2, ch.fidelity * 100.0);
    read.set(3, ds.tan_err * 100.0);
    read.set(4, ds.slope_r.atan2(1.0) / TAU); // right-segment angle, turns
    read.set(5, n as f64);
}

lessons_common::lesson!(draw);

// ---- the claims ---------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    /// W1: the sine transform round-trips exactly.
    #[test]
    fn w1_dst_round_trips() {
        for n in [5, 24, 80] {
            let y0 = pluck(n, 0.3, BASE_AMP);
            let w = omegas(n, 1.5);
            let back = evolve(&dst(&y0, n), &w, n, 0.0);
            let err = y0.iter().zip(&back).map(|(a, b)| (a - b).abs()).fold(0.0, f64::max);
            assert!(err < 1e-12, "n={n} err={err}");
        }
    }

    /// W2: exact mode evolution matches an independent Verlet integration.
    #[test]
    fn w2_modes_match_verlet() {
        let (n, c) = (24, 1.5);
        let dx = L / (n + 1) as f64;
        let y0 = pluck(n, 0.3, BASE_AMP);
        let w = omegas(n, c);
        let a = dst(&y0, n);
        let (dt, t_end) = (1e-4, 2.0);
        let mut y = y0.clone();
        let mut v = vec![0.0; n];
        let acc = |y: &[f64]| -> Vec<f64> {
            (0..n)
                .map(|i| {
                    let l = if i > 0 { y[i - 1] } else { 0.0 };
                    let r = if i < n - 1 { y[i + 1] } else { 0.0 };
                    c * c * (l - 2.0 * y[i] + r) / (dx * dx)
                })
                .collect()
        };
        let mut acc0 = acc(&y);
        for _ in 0..(t_end / dt) as usize {
            for i in 0..n {
                y[i] += v[i] * dt + 0.5 * acc0[i] * dt * dt;
            }
            let acc1 = acc(&y);
            for i in 0..n {
                v[i] += 0.5 * (acc0[i] + acc1[i]) * dt;
            }
            acc0 = acc1;
        }
        let exact = evolve(&a, &w, n, t_end);
        let err = exact.iter().zip(&y).map(|(a, b)| (a - b).abs()).fold(0.0, f64::max);
        assert!(err < 1e-6, "err={err}");
    }

    /// W3: energy in mode form is constant.
    #[test]
    fn w3_energy_constant() {
        let (n, c) = (24, 1.5);
        let w = omegas(n, c);
        let a = dst(&pluck(n, 0.3, BASE_AMP), n);
        let energy = |t: f64| -> f64 {
            (0..n)
                .map(|k| {
                    let q = a[k] * (w[k] * t).cos();
                    let qd = -a[k] * w[k] * (w[k] * t).sin();
                    qd * qd + (w[k] * q).powi(2)
                })
                .sum()
        };
        let e0 = energy(0.0);
        for t in [0.7, 1.9, 3.7, 8.3] {
            assert!((energy(t) - e0).abs() < 1e-9 * e0);
        }
    }

    /// W4: the lattice cutoff descends to 4/τ of c from above, never below.
    #[test]
    fn w4_cutoff_pins_at_four_over_tau() {
        let lim = 4.0 / TAU;
        let ratio = |n: usize| {
            let w = omegas(n, 1.5);
            w[n - 1] / (n as f64 * (TAU / 2.0) / L) / 1.5
        };
        let (r1, r2, r3) = (ratio(24), ratio(80), ratio(400));
        assert!(r1 > r2 && r2 > r3 && r3 > lim, "{r1} {r2} {r3}");
        assert!(r3 - lim < 2e-3);
    }

    /// W5: the pluck's energy-weighted fidelity climbs with N.
    #[test]
    fn w5_fidelity_monotone() {
        let f = |n| chain(n, 1.5, 0.3, BASE_AMP, 0.0).fidelity;
        let (f5, f24, f80) = (f(5), f(24), f(80));
        assert!(f5 < f24 && f24 < f80 && f80 < 1.0, "{f5} {f24} {f80}");
    }

    /// W6 (the dissector): the emitted sin-vs-tan error matches the
    /// closed form for the actual slopes, vanishes as amplitude does,
    /// and grows monotonically with exaggeration.
    #[test]
    fn w6_dissector_error_is_honest() {
        let mut prev = -1.0;
        for xa in [0.2, 0.5, 1.0, 1.6, 2.4] {
            let ch = chain(24, 1.5, 0.3, BASE_AMP * xa, 0.31);
            let ds = dissect(&ch);
            let expect = |s: f64| {
                let sin = s / (1.0 + s * s).sqrt();
                ((s - sin) / sin).abs()
            };
            let want = expect(ds.slope_l).max(expect(ds.slope_r));
            assert!((ds.tan_err - want).abs() < 1e-12);
            assert!(ds.tan_err > prev, "xa={xa}: error must grow with amplitude");
            prev = ds.tan_err;
        }
        // tiny amplitude: the approximation is honest to ~1e-6 relative
        let ds = dissect(&chain(24, 1.5, 0.3, 1e-3, 0.31));
        assert!(ds.tan_err < 1e-6);
    }
}
