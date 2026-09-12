//! Hip thrust versus Romanian deadlift — two lifts, opposite curves.
//!
//! From reel 08. Both extend the hip against the same load; what differs
//! is which segment is the lever. In the RDL it is the **torso**, so the
//! moment dies at standing. In the hip thrust it is the **femur**, so the
//! moment peaks at lockout. The curves cross, which is the whole point:
//! they are two halves of one resistance profile, not competitors.
//!
//! Both reduce to one term, which is why they are worth having as code
//! rather than a diagram: `τ = m·g·L·sin θ` and `τ = m·g·L·cos θ`.

use crate::{Lift, G};

/// The Romanian deadlift. Lever: the torso.
#[derive(Clone, Debug, PartialEq)]
pub struct Rumano {
    /// Bar load, kilograms.
    pub load_kg: f64,
    /// Hip → shoulder, metres.
    pub torso_m: f64,
    /// Hip flexion at the bottom, radians.
    pub bottom_rad: f64,
}

/// The hip thrust. Lever: the femur.
#[derive(Clone, Debug, PartialEq)]
pub struct HipThrust {
    /// Bar load, kilograms.
    pub load_kg: f64,
    /// Hip → knee, metres.
    pub femur_m: f64,
    /// Hip flexion at the bottom, radians.
    pub bottom_rad: f64,
}

impl Lift for Rumano {
    /// `phi` is hip flexion: `bottom_rad` at the stretch, `0` standing.
    fn tau(&self, phi: f64) -> f64 {
        self.load_kg * G * self.torso_m * phi.sin()
    }
    fn range(&self) -> (f64, f64) {
        (0.0, self.bottom_rad)
    }
}

impl Lift for HipThrust {
    /// `phi` is hip flexion: `bottom_rad` at the stretch, `0` at lockout.
    fn tau(&self, phi: f64) -> f64 {
        self.load_kg * G * self.femur_m * phi.cos()
    }
    fn range(&self) -> (f64, f64) {
        (0.0, self.bottom_rad)
    }
}

/// Torque at normalized progress: `0` stretched (bottom), `1` contracted.
///
/// The two lifts sweep different angular ranges, so a shared graph needs
/// a shared axis. This is that axis, and it is presentation, not physics
/// — hence a free function rather than a trait method.
pub fn tau_at_progress<L: Lift + ?Sized>(lift: &L, s: f64) -> f64 {
    let (top, bottom) = lift.range();
    lift.tau(bottom + (top - bottom) * s)
}

/// Where two lifts' curves cross on the normalized axis, if they do.
///
/// Bisection on the difference, so it finds one crossing; the pair this
/// module exists for has exactly one (see the claims).
pub fn crossing_progress<A, B>(a: &A, b: &B) -> Option<f64>
where
    A: Lift + ?Sized,
    B: Lift + ?Sized,
{
    let diff = |s: f64| tau_at_progress(a, s) - tau_at_progress(b, s);
    let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
    if diff(lo).is_sign_positive() == diff(hi).is_sign_positive() {
        return None;
    }
    let rising = diff(lo) < 0.0;
    for _ in 0..crate::BISECTION_ITERS {
        let mid = (lo + hi) / 2.0;
        if (diff(mid) < 0.0) == rising {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some((lo + hi) / 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{peak, work_over_range};

    /// Reel 08's published setup: 100 kg on both.
    fn rdl() -> Rumano {
        Rumano {
            load_kg: 100.0,
            torso_m: 0.53,
            bottom_rad: 72.0_f64.to_radians(),
        }
    }
    fn ht() -> HipThrust {
        HipThrust {
            load_kg: 100.0,
            femur_m: 0.46,
            bottom_rad: 40.0_f64.to_radians(),
        }
    }

    /// C1: the RDL is maximal at the bottom and vanishes at standing.
    #[test]
    fn c1_rdl_peaks_at_the_bottom_and_dies_standing() {
        let r = rdl();
        assert_eq!(peak(&r).1.round(), 494.0);
        assert!(r.tau(0.0).abs() < 1e-12, "standing must be zero");
    }

    /// C2: the hip thrust is maximal at lockout.
    #[test]
    fn c2_hip_thrust_peaks_at_lockout() {
        let h = ht();
        let (phi, tau) = peak(&h);
        assert!(phi.abs() < 1e-9, "peak must be at lockout, got {phi}");
        assert_eq!(tau.round(), 451.0);
        assert_eq!(h.tau(h.bottom_rad).round(), 346.0, "the bottom figure");
    }

    /// C3: the curves cross exactly once on the shared axis.
    ///
    /// "Se cruzan aquí" is the reel's whole argument. Uniqueness is
    /// checked by counting sign changes, not by trusting the root find.
    #[test]
    fn c3_the_curves_cross_exactly_once() {
        let (r, h) = (rdl(), ht());
        let diff = |s: f64| tau_at_progress(&r, s) - tau_at_progress(&h, s);
        let changes = (0..1000)
            .filter(|i| {
                let (a, b) = (f64::from(*i) / 1000.0, f64::from(*i + 1) / 1000.0);
                diff(a).is_sign_positive() != diff(b).is_sign_positive()
            })
            .count();
        assert_eq!(changes, 1, "expected exactly one crossing");

        let s = crossing_progress(&r, &h).expect("they cross");
        assert!(s > 0.0 && s < 1.0, "crossing at s={s}");
    }

    /// C4: at equal load the RDL's peak beats the hip thrust's — 494 vs
    /// 451 N·m — even though the hip thrust wins at lockout.
    #[test]
    fn c4_rdl_peak_exceeds_hip_thrust_peak() {
        assert!(peak(&rdl()).1 > peak(&ht()).1);
    }

    /// C5: numerical work agrees with the closed-form antiderivative.
    ///
    /// `∫ sin = 1 − cos` and `∫ cos = sin`. This tests the integrator
    /// against exact analysis rather than against itself.
    ///
    /// The bound is **relative**: the midpoint rule's error is O(h²), so
    /// on a 359 J integral it lands near 3 × 10⁻⁵ absolute — which is
    /// 8 × 10⁻⁸ of the answer, and an absolute bound would only be
    /// measuring how large the integral happens to be.
    #[test]
    fn c5_work_matches_the_closed_form() {
        let r = rdl();
        let exact_r = r.load_kg * G * r.torso_m * (1.0 - r.bottom_rad.cos());
        assert!((work_over_range(&r) / exact_r - 1.0).abs() < 1e-7, "RDL");

        let h = ht();
        let exact_h = h.load_kg * G * h.femur_m * h.bottom_rad.sin();
        assert!(
            (work_over_range(&h) / exact_h - 1.0).abs() < 1e-7,
            "hip thrust"
        );
    }

    /// C6: the two lifts load opposite ends of the range — the reel's
    /// "son las dos mitades de la misma curva", as a monotonicity claim.
    #[test]
    fn c6_they_load_opposite_ends() {
        let (r, h) = (rdl(), ht());
        for i in 0..100 {
            let (a, b) = (f64::from(i) / 100.0, f64::from(i + 1) / 100.0);
            assert!(tau_at_progress(&r, a) > tau_at_progress(&r, b), "RDL falls");
            assert!(tau_at_progress(&h, a) < tau_at_progress(&h, b), "HT rises");
        }
    }
}
