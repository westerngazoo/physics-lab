//! Two Mirrors Make a Rotation — the first GA-native lesson.
//!
//! The claim: reflecting a vector across mirror `a`, then across mirror
//! `b`, is one rotation — by TWICE the angle between the mirrors, no
//! matter which vector you started with. In GA this is not a theorem to
//! memorize but an identity you can read off:
//!
//! ```text
//! b (a v a) b  =  (b a) v (a b)  =  R v R~,   with R = b a
//! ```
//!
//! The rotor IS the pair of reflections, reassociated. This crate
//! computes both sides with garust's `Vga2` — the same kernel motoreel
//! films with — and the browser draws whatever it computes. `cargo test`
//! asserts the lesson's claims on the same code the page runs.
//!
//! Angles are measured against TAU: a full turn is one τ (house
//! convention, garust's included).

use garust::Vga2;
use std::f64::consts::TAU;

/// A unit vector at angle `theta` (τ-measured radians) from e1.
fn unit(theta: f64) -> Vga2 {
    Vga2::basis(1) * theta.cos() + Vga2::basis(2) * theta.sin()
}

/// Reflect `v` across the line through the origin along unit vector `a`:
/// the sandwich `a v ã`, and a vector is its own reverse.
fn reflect(a: &Vga2, v: &Vga2) -> Vga2 {
    a.sandwich(v)
}

/// The lesson's state, all of it derived per frame from two angles.
pub struct MirrorState {
    /// Input vector v (unit, at angle `phi`).
    pub v: Vga2,
    /// After the first reflection (across mirror A).
    pub v1: Vga2,
    /// After the second reflection (across mirror B) — the two-step path.
    pub v2_steps: Vga2,
    /// The same endpoint computed as ONE rotor sandwich `R v R̃`.
    pub v2_rotor: Vga2,
    /// The rotor `R = b a`: scalar and e12 coefficients.
    pub rotor_scalar: f64,
    pub rotor_e12: f64,
    /// Rotation actually performed, in turns (fraction of τ), in [0, 1).
    pub rotation_turns: f64,
}

/// Everything the lesson claims, computed from mirror angles `alpha`,
/// `beta` and input angle `phi` (all τ-measured radians).
pub fn eval(alpha: f64, beta: f64, phi: f64) -> MirrorState {
    let a = unit(alpha);
    let b = unit(beta);
    let v = unit(phi);

    let v1 = reflect(&a, &v);
    let v2_steps = reflect(&b, &v1);

    let r = b * a; // the rotor: two reflections, reassociated
    let v2_rotor = r.sandwich(&v);

    let (x, y) = (v2_steps.coeffs[1], v2_steps.coeffs[2]);
    let rotation = (y.atan2(x) - phi).rem_euclid(TAU);

    MirrorState {
        v,
        v1,
        v2_steps,
        v2_rotor,
        rotor_scalar: r.coeffs[0],
        rotor_e12: r.coeffs[3],
        rotation_turns: rotation / TAU,
    }
}

// ---- the wasm boundary: one function, the framework does the rest ------

use lessons_common::{Prims, Readouts};

/// Params (manifest order): [alpha, beta, phi], radians.
fn draw(p: &[f64], out: &mut Prims, read: &mut Readouts) {
    let (alpha, beta, phi) = (p[0], p[1], p[2]);
    let s = eval(alpha, beta, phi);
    let m = 1.35; // mirror half-length in world units
    let tip = |w: &Vga2| (w.coeffs[1], w.coeffs[2]);
    let (vx, vy) = tip(&s.v);
    let (v1x, v1y) = tip(&s.v1);
    let (v2x, v2y) = tip(&s.v2_steps);
    let (rx, ry) = (s.v2_rotor.coeffs[1], s.v2_rotor.coeffs[2]);

    // mirrors A and B: full lines through the origin
    out.segment(-m * alpha.cos(), -m * alpha.sin(), m * alpha.cos(), m * alpha.sin(), 0);
    out.segment(-m * beta.cos(), -m * beta.sin(), m * beta.cos(), m * beta.sin(), 1);
    // v, v after mirror A (ghost), v after both mirrors
    out.arrow(0.0, 0.0, vx, vy, 2);
    out.arrow(0.0, 0.0, v1x, v1y, 3);
    out.arrow(0.0, 0.0, v2x, v2y, 4);
    // the rotor's answer as a cross-tick: coincides iff the claim holds
    let t = 0.07;
    out.segment(rx - t, ry - t, rx + t, ry + t, 5);
    out.segment(rx - t, ry + t, rx + t, ry - t, 5);

    read.set(0, ((beta - alpha) / TAU).rem_euclid(1.0));
    read.set(1, s.rotation_turns);
    read.set(2, s.rotor_scalar);
    read.set(3, s.rotor_e12);
    read.set(4, ((v2x - rx).powi(2) + (v2y - ry).powi(2)).sqrt());
}

lessons_common::lesson!(draw);

// ---- the claims, asserted on the code the page runs ---------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn angles() -> impl Iterator<Item = f64> {
        // τ-fraction sweep, deliberately including 0, τ/2 and awkward spots
        (0..24).map(|i| i as f64 * TAU / 24.0)
    }

    /// C1: two reflections and one rotor sandwich land on the same vector.
    #[test]
    fn c1_two_reflections_equal_one_rotor() {
        for alpha in angles() {
            for beta in angles() {
                for phi in [0.0, TAU / 7.0, TAU / 3.0, 0.9 * TAU] {
                    let s = eval(alpha, beta, phi);
                    for k in 0..4 {
                        assert!(
                            (s.v2_steps.coeffs[k] - s.v2_rotor.coeffs[k]).abs() < 1e-12,
                            "alpha={alpha} beta={beta} phi={phi} k={k}"
                        );
                    }
                }
            }
        }
    }

    /// C2: the rotation is twice the mirror angle (mod τ), whatever v was.
    #[test]
    fn c2_rotation_is_twice_the_mirror_angle() {
        for alpha in angles() {
            for beta in angles() {
                for phi in [TAU / 11.0, TAU / 5.0, 0.77 * TAU] {
                    let s = eval(alpha, beta, phi);
                    let expect = (2.0 * (beta - alpha) / TAU).rem_euclid(1.0);
                    let diff = (s.rotation_turns - expect).abs();
                    let diff = diff.min(1.0 - diff); // wrap distance on the circle
                    assert!(diff < 1e-12, "alpha={alpha} beta={beta} phi={phi}");
                }
            }
        }
    }

    /// C3: the rotor's components are cos and −sin of the mirror angle —
    /// pinned EMPIRICALLY against garust's own conventions, not assumed.
    #[test]
    fn c3_rotor_components_follow_the_mirror_angle() {
        for alpha in angles() {
            for beta in angles() {
                let s = eval(alpha, beta, TAU / 8.0);
                let d = beta - alpha;
                assert!((s.rotor_scalar - d.cos()).abs() < 1e-12);
                assert!((s.rotor_e12 - (-d.sin())).abs() < 1e-12);
            }
        }
    }

    /// C4: a reflection alone reverses orientation (det −1): reflecting
    /// twice across the SAME mirror is the identity.
    #[test]
    fn c4_same_mirror_twice_is_identity() {
        for alpha in angles() {
            for phi in [0.3, 1.9, 4.4] {
                let a = unit(alpha);
                let v = unit(phi);
                let back = reflect(&a, &reflect(&a, &v));
                for k in 0..4 {
                    assert!((back.coeffs[k] - v.coeffs[k]).abs() < 1e-12);
                }
            }
        }
    }

    /// C5: the drawn coincidence readout is honest — through the same
    /// uniform ABI the browser uses: write params, call state_at, read.
    #[test]
    fn c5_buffer_coincidence_is_fp_small() {
        for alpha in angles().take(8) {
            for beta in angles().take(8) {
                unsafe {
                    let p = params_ptr();
                    *p.add(0) = alpha;
                    *p.add(1) = beta;
                    *p.add(2) = TAU / 6.0;
                }
                state_at(3);
                let sep = unsafe { *readouts_ptr().add(4) };
                assert!(sep < 1e-12, "alpha={alpha} beta={beta} sep={sep}");
            }
        }
    }
}
