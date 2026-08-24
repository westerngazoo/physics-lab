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

// ---- the wasm boundary --------------------------------------------------
// A flat f64 primitive buffer, motoreel's drawing vocabulary as a
// convention: [tag, ...payload] records. Tags: 1 segment(x1,y1,x2,y2,style),
// 3 arrow(x1,y1,x2,y2,style). Styles index the palette in lesson.json.
// One fixed runtime.js paints it; nobody writes lesson JS.

const CAP: usize = 128;
static mut PRIMS: [f64; CAP] = [0.0; CAP];
static mut PRIM_LEN: usize = 0;
static mut READ: [f64; 6] = [0.0; 6];

fn put(buf: &mut [f64], at: &mut usize, rec: &[f64]) {
    buf[*at..*at + rec.len()].copy_from_slice(rec);
    *at += rec.len();
}

/// Fill the primitive buffer for mirror angles and input angle.
/// Returns the number of f64s written. Pure over its arguments: same
/// inputs, same buffer, bit for bit.
///
/// # Safety
/// Single-threaded wasm; the statics are written only here and read via
/// the pointer accessors below between calls.
#[no_mangle]
pub extern "C" fn state_at(alpha: f64, beta: f64, phi: f64) -> usize {
    let s = eval(alpha, beta, phi);
    let m = 1.35; // mirror half-length in world units
    let (ax, ay) = (alpha.cos(), alpha.sin());
    let (bx, by) = (beta.cos(), beta.sin());
    let tip = |w: &Vga2| (w.coeffs[1], w.coeffs[2]);
    let (vx, vy) = tip(&s.v);
    let (v1x, v1y) = tip(&s.v1);
    let (v2x, v2y) = tip(&s.v2_steps);
    let (rx, ry) = (s.v2_rotor.coeffs[1], s.v2_rotor.coeffs[2]);

    let mut at = 0;
    // SAFETY: see fn docs.
    unsafe {
        let buf = &mut *core::ptr::addr_of_mut!(PRIMS);
        // mirrors A and B: full lines through the origin (style 0, 1)
        put(buf, &mut at, &[1.0, -m * ax, -m * ay, m * ax, m * ay, 0.0]);
        put(buf, &mut at, &[1.0, -m * bx, -m * by, m * bx, m * by, 1.0]);
        // v, v1 (ghost), v2 via two reflections (styles 2, 3, 4)
        put(buf, &mut at, &[3.0, 0.0, 0.0, vx, vy, 2.0]);
        put(buf, &mut at, &[3.0, 0.0, 0.0, v1x, v1y, 3.0]);
        put(buf, &mut at, &[3.0, 0.0, 0.0, v2x, v2y, 4.0]);
        // the rotor's answer, drawn as a short cross-tick at its tip
        // (style 5): if the claim holds it sits exactly on v2's tip.
        let t = 0.07;
        put(buf, &mut at, &[1.0, rx - t, ry - t, rx + t, ry + t, 5.0]);
        put(buf, &mut at, &[1.0, rx - t, ry + t, rx + t, ry - t, 5.0]);
        PRIM_LEN = at;

        let rd = &mut *core::ptr::addr_of_mut!(READ);
        rd[0] = ((beta - alpha) / TAU).rem_euclid(1.0); // mirror angle, turns
        rd[1] = s.rotation_turns;                       // rotation, turns
        rd[2] = s.rotor_scalar;
        rd[3] = s.rotor_e12;
        // coincidence of the two constructions, in ulps-ish world units
        rd[4] = ((v2x - rx).powi(2) + (v2y - ry).powi(2)).sqrt();
        rd[5] = 0.0;
    }
    at
}

/// Pointer to the primitive buffer (f64s; `state_at`'s return is the length).
#[no_mangle]
pub extern "C" fn prims_ptr() -> *const f64 {
    core::ptr::addr_of!(PRIMS) as *const f64
}

/// Pointer to the 6-slot readout buffer.
#[no_mangle]
pub extern "C" fn readouts_ptr() -> *const f64 {
    core::ptr::addr_of!(READ) as *const f64
}

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

    /// C5: the drawn coincidence readout is honest — the buffer's own
    /// separation figure is at floating-point scale for a τ-fraction grid.
    #[test]
    fn c5_buffer_coincidence_is_fp_small() {
        for alpha in angles().take(8) {
            for beta in angles().take(8) {
                state_at(alpha, beta, TAU / 6.0);
                let sep = unsafe { (*core::ptr::addr_of!(READ))[4] };
                assert!(sep < 1e-12, "alpha={alpha} beta={beta} sep={sep}");
            }
        }
    }
}
