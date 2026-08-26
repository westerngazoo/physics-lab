//! One Fall, Three Mechanics — the bouncing ball through Newton,
//! Lagrange, and Hamilton.
//!
//! One motion, three characterizations, each independently assertable:
//!
//! - **Newton:** the acceleration is `-g`, everywhere in flight (C1).
//! - **Lagrange:** the flown path makes the action `S = ∫(T - V) dt`
//!   stationary — perturb it by `ε·η(t)` with `η` vanishing at the
//!   endpoints and the action rises as exactly `ε²·K`, minimum at the
//!   true path (C2, with `K` in closed form and cross-checked by an
//!   independent numerical quadrature).
//! - **Hamilton:** `H = p²/2 + g·y` (unit mass) is constant along every
//!   flight, and each bounce multiplies it by exactly `e²` — which is
//!   both the phase-space picture and the honest boundary of it:
//!   the bounce is where this system stops being conservative (C3, C4).
//!
//! Everything is closed form (flights are exact parabolas with solved
//! bounce times — the same event-driven construction the original
//! gravity lesson used), so all three claims are checked to
//! floating-point scale, not "approximately". Angles/periods measured
//! against τ where they appear. Unit mass throughout, stated on-page.

use std::f64::consts::TAU;

// ---- the motion: exact flight table ------------------------------------

/// One parabolic flight: starts at `t0` on the floor (or at the drop
/// apex for the first), with upward speed `v_up`.
#[derive(Clone, Copy)]
pub struct Flight {
    pub t0: f64,
    pub dur: f64,
    pub v_up: f64,
    pub y0: f64,
    pub drop: bool,
}

pub struct Motion {
    pub flights: Vec<Flight>,
    pub t_rest: f64,
}

/// Solve the bounce times in closed form; no integration anywhere.
pub fn motion(h0: f64, e: f64, g: f64, count: usize) -> Motion {
    let v0 = (2.0 * g * h0).sqrt();
    let mut flights = Vec::with_capacity(count + 1);
    let mut t = (2.0 * h0 / g).sqrt();
    flights.push(Flight { t0: 0.0, dur: t, v_up: 0.0, y0: h0, drop: true });
    for n in 1..=count {
        let v = v0 * e.powi(n as i32);
        let dur = 2.0 * v / g;
        flights.push(Flight { t0: t, dur, v_up: v, y0: 0.0, drop: false });
        t += dur;
    }
    Motion { flights, t_rest: (2.0 * h0 / g).sqrt() * (1.0 + e) / (1.0 - e) }
}

/// Height and velocity at `t` — pure functions of time, nothing stepped.
pub fn state(m: &Motion, g: f64, t: f64) -> (f64, f64) {
    for f in &m.flights {
        if t >= f.t0 && t < f.t0 + f.dur {
            let dt = t - f.t0;
            return if f.drop {
                (f.y0 - 0.5 * g * dt * dt, -g * dt)
            } else {
                (f.v_up * dt - 0.5 * g * dt * dt, f.v_up - g * dt)
            };
        }
    }
    (0.0, 0.0)
}

/// The Hamiltonian, unit mass: kinetic plus potential.
pub fn hamiltonian(y: f64, p: f64, g: f64) -> f64 {
    0.5 * p * p + g * y
}

// ---- Lagrange: the action on the first full flight ----------------------
//
// Perturb the flown arc by ε·A·sin(τ t'/(2 T)) — zero at both endpoints,
// so the variational boundary terms vanish. Because L is quadratic, the
// action is EXACTLY S(ε) = S(0) + ε²·K with K = A²τ²/(16 T): the linear
// term is the Euler-Lagrange equation, and it is zero precisely because
// the flown path obeys Newton. That equivalence is the lesson.

/// The first full bounce arc (floor to floor) — the variational stage.
pub fn variational_flight(m: &Motion) -> Flight {
    m.flights[1]
}

/// Perturbation amplitude: scaled to the arc's apex so the wrong paths
/// are visible at every parameter setting.
pub fn pert_amp(f: &Flight, g: f64) -> f64 {
    0.35 * f.v_up * f.v_up / (2.0 * g)
}

/// Action of the ε-perturbed path over the variational flight,
/// by direct quadrature of L = ½ẏ² − g·y (composite Simpson, n even).
pub fn action_numeric(f: &Flight, g: f64, amp: f64, eps: f64, n: usize) -> f64 {
    let h = f.dur / n as f64;
    let integrand = |k: usize| {
        let tt = k as f64 * h;
        let y = f.v_up * tt - 0.5 * g * tt * tt + eps * amp * (TAU * tt / (2.0 * f.dur)).sin();
        let yd = f.v_up - g * tt + eps * amp * (TAU / (2.0 * f.dur)) * (TAU * tt / (2.0 * f.dur)).cos();
        0.5 * yd * yd - g * y
    };
    let mut s = integrand(0) + integrand(n);
    for k in 1..n {
        s += integrand(k) * if k % 2 == 1 { 4.0 } else { 2.0 };
    }
    s * h / 3.0
}

/// The closed-form quadratic coefficient: S(ε) − S(0) = ε²·K.
pub fn action_k(f: &Flight, amp: f64) -> f64 {
    amp * amp * TAU * TAU / (16.0 * f.dur)
}

// ---- the wasm boundary: one function, the framework does the rest ------

use lessons_common::{Prims, Readouts};

/// Params (manifest order): [h0, e, g, eps, tfrac].
fn draw(p: &[f64], out: &mut Prims, read: &mut Readouts) {
    let (h0, e, g, eps, tfrac) = (p[0], p[1], p[2], p[3], p[4]);
    let m = motion(h0, e, g, 14);
    let t = tfrac.clamp(0.0, 1.0) * m.t_rest;
    let (y, v) = state(&m, g, t);
    let vf = variational_flight(&m);
    let amp = pert_amp(&vf, g);

    // ---- view 0: Newton -- trajectory in (scaled time, height) ---------
    out.view(0);
    let vx = 10.4 / m.t_rest;
    out.segment(-0.5, 0.0, 10.9, 0.0, 0);
    for f in &m.flights {
        out.curve(f.t0, f.t0 + f.dur, 22, 1, |tt| (tt * vx, state(&m, g, tt).0));
    }
    if eps.abs() > 1e-9 {
        out.curve(0.0, vf.dur, 30, 2, |dt| {
            let yy = vf.v_up * dt - 0.5 * g * dt * dt
                + eps * amp * (TAU * dt / (2.0 * vf.dur)).sin();
            ((vf.t0 + dt) * vx, yy)
        });
    }
    out.point(t * vx, y + 0.07, 3);
    out.arrow(t * vx, y + 0.07, t * vx, y + 0.07 - 0.16 * g.clamp(1.0, 12.0) / 3.0, 4);

    // ---- view 1: Lagrange -- the action parabola over eps ---------------
    out.view(1);
    out.segment(-1.15, 0.0, 1.15, 0.0, 0);
    out.segment(0.0, -0.1, 0.0, 1.3, 0);
    out.curve(-1.1, 1.1, 40, 1, |x| (x, x * x)); // (S(eps)-S(0))/K = eps^2
    out.point(eps, eps * eps, 3);
    out.point(0.0, 0.0, 4);

    // ---- view 2: Hamilton -- phase portrait (y, p) ----------------------
    out.view(2);
    out.segment(0.0, -16.0, 0.0, 16.0, 0);
    out.segment(0.0, 0.0, 5.6, 0.0, 0);
    let here = flight_index(&m, t);
    for (i, f) in m.flights.iter().take(5).enumerate() {
        let h_level = if f.drop { g * f.y0 } else { 0.5 * f.v_up * f.v_up };
        let y_max = h_level / g;
        let style = if i == here { 1 } else { 5 };
        out.curve(0.0, y_max, 32, style, |yy| (yy, (2.0 * (h_level - g * yy)).max(0.0).sqrt()));
        out.curve(0.0, y_max, 32, style, |yy| (yy, -(2.0 * (h_level - g * yy)).max(0.0).sqrt()));
    }
    out.point(y, v, 3);

    read.set(0, t);
    read.set(1, eps * eps);
    read.set(2, hamiltonian(y, v, g));
    read.set(3, hamiltonian(y, v, g) / (g * h0) * 100.0);
    read.set(4, bounces_by(&m, t) as f64);
    read.set(5, m.t_rest);
}

lessons_common::lesson!(draw);

fn flight_index(m: &Motion, t: f64) -> usize {
    for (i, f) in m.flights.iter().enumerate() {
        if t >= f.t0 && t < f.t0 + f.dur {
            return i;
        }
    }
    usize::MAX
}

fn bounces_by(m: &Motion, t: f64) -> usize {
    m.flights.iter().filter(|f| !f.drop && t >= f.t0).count()
}

// ---- the claims ---------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    const CASES: [(f64, f64, f64); 5] = [
        (4.0, 0.75, 9.81),
        (4.0, 0.75, 1.62),   // the Moon: same claims, slower clocks
        (2.5, 0.60, 24.79),  // Jupiter
        (5.0, 0.90, 9.81),
        (1.0, 0.30, 3.71),   // Mars
    ];

    /// C1 (Newton): the second difference of every flight is exactly -g.
    #[test]
    fn c1_acceleration_is_minus_g_in_flight() {
        for (h0, e, g) in CASES {
            let m = motion(h0, e, g, 8);
            for f in m.flights.iter().take(6) {
                let h = f.dur / 64.0;
                for i in 1..8 {
                    let tt = f.t0 + f.dur * i as f64 / 8.0;
                    let acc = (state(&m, g, tt + h).0 - 2.0 * state(&m, g, tt).0
                        + state(&m, g, tt - h).0) / (h * h);
                    assert!((acc + g).abs() < 1e-6 * g, "acc={acc} g={g}");
                }
            }
        }
    }

    /// C2 (Lagrange): the action is stationary at the flown path —
    /// S(ε) − S(0) = ε²·K with K in closed form, verified against an
    /// independent Simpson quadrature; minimum at ε = 0; symmetric.
    #[test]
    fn c2_action_is_stationary_at_the_flown_path() {
        for (h0, e, g) in CASES {
            let m = motion(h0, e, g, 4);
            let f = variational_flight(&m);
            let amp = pert_amp(&f, g);
            let k = action_k(&f, amp);
            let s0 = action_numeric(&f, g, amp, 0.0, 4096);
            for eps in [-1.0, -0.5, -0.1, 0.1, 0.5, 1.0] {
                let s = action_numeric(&f, g, amp, eps, 4096);
                let excess = s - s0;
                assert!((excess - eps * eps * k).abs() < 1e-9 * k.max(1.0),
                    "eps={eps} excess={excess} pred={}", eps * eps * k);
                assert!(excess > 0.0, "the true path must be the minimum");
            }
            let sp = action_numeric(&f, g, amp, 0.4, 4096);
            let sm = action_numeric(&f, g, amp, -0.4, 4096);
            assert!((sp - sm).abs() < 1e-9 * k.max(1.0), "S must be even in eps");
        }
    }

    /// C3 (Hamilton): H is constant along every flight, to fp scale.
    #[test]
    fn c3_hamiltonian_constant_in_flight() {
        for (h0, e, g) in CASES {
            let m = motion(h0, e, g, 8);
            for f in m.flights.iter().take(6) {
                let h_ref = {
                    let tt = f.t0 + 0.25 * f.dur;
                    let (y, v) = state(&m, g, tt);
                    hamiltonian(y, v, g)
                };
                for i in 1..12 {
                    let tt = f.t0 + f.dur * i as f64 / 12.0;
                    let (y, v) = state(&m, g, tt);
                    assert!((hamiltonian(y, v, g) - h_ref).abs() < 1e-9 * h_ref.max(1e-9));
                }
            }
        }
    }

    /// C4 (Hamilton's boundary): each bounce multiplies H by exactly e²
    /// — conservation ends where the impact begins.
    #[test]
    fn c4_each_bounce_costs_e_squared() {
        for (h0, e, g) in CASES {
            let m = motion(h0, e, g, 8);
            let flight_h = |f: &Flight| 0.5 * f.v_up * f.v_up;
            for w in m.flights[1..].windows(2) {
                let ratio = flight_h(&w[1]) / flight_h(&w[0]);
                assert!((ratio - e * e).abs() < 1e-12, "ratio={ratio} e2={}", e * e);
            }
        }
    }

    /// C5 (the three agree): the trajectory whose acceleration is −g
    /// (C1) is the same one that minimizes the action (C2) and rides the
    /// H level sets (C3) — one motion, three characterizations, all
    /// asserted on the same closed form. Buffer-level spot check: the
    /// phase point emitted by state_at sits on its flight's level set.
    #[test]
    fn c5_phase_point_rides_its_level_set() {
        for (h0, e, g) in CASES {
            for tf in [0.05, 0.3, 0.55, 0.8] {
                unsafe {
                    let p = params_ptr();
                    for (i, v) in [h0, e, g, 0.0, tf].iter().enumerate() {
                        *p.add(i) = *v;
                    }
                }
                state_at(5);
                let rd = unsafe { core::slice::from_raw_parts(readouts_ptr(), 6) };
                let m = motion(h0, e, g, 14);
                let t = tf * m.t_rest;
                let idx = flight_index(&m, t);
                if idx == usize::MAX { continue; }
                let f = &m.flights[idx];
                let level = if f.drop { g * f.y0 } else { 0.5 * f.v_up * f.v_up };
                assert!((rd[2] - level).abs() < 1e-9 * level.max(1e-9),
                    "H readout {} vs level {}", rd[2], level);
            }
        }
    }
}
