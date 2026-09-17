//! `mecanica` — closed-form gym biomechanics.
//!
//! A lift is reduced to one function: how hard the load resists at each
//! point of the movement. Everything else here is an algorithm over that
//! one function, written once instead of per exercise.
//!
//! Pure by construction: no dependencies, no I/O, no clock, no state.
//! [`RFC-002`](../../docs/RFC-002-mecanica.md) is the design.
//!
//! # The model, and what it is not
//!
//! Two-dimensional, sagittal plane, quasi-static, external load only. It
//! does not measure stabilisers, and it does not divide load between the
//! muscles crossing a joint — that division is mathematically
//! indeterminate (RFC-002 §7.3), not merely unimplemented.

#![forbid(unsafe_code)]

pub mod gluteo;
pub mod maquina_humana;
pub mod patada;
pub mod sentadilla;

/// Standard gravity, m/s².
pub const G: f64 = 9.81;

/// Progress along a lift's own range: `0` at the start of the declared
/// range, `1` at its end.
///
/// Two lifts sweep different angular ranges, so comparing them needs a
/// shared axis. This is that axis. It is presentation, not physics —
/// hence a free function rather than a trait method — but it belongs
/// here and not inside one exercise's module: nothing about it is about
/// glutes, and a comparison layer that has to reach into `gluteo` to
/// compare a curl is a layering mistake.
pub fn tau_at_progress<L: Lift + ?Sized>(lift: &L, s: f64) -> f64 {
    let (a, b) = lift.range();
    lift.tau(a + (b - a) * s)
}

/// Where two lifts' curves cross on the shared progress axis, if at all.
///
/// Bisection on the difference, so it finds **one** crossing; a pair with
/// several would need a different question asked of it. Returns `None`
/// when the difference does not change sign, which is the common case and
/// not an error: most pairs simply do not cross.
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
    for _ in 0..BISECTION_ITERS {
        let mid = (lo + hi) / 2.0;
        if (diff(mid) < 0.0) == rising {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some((lo + hi) / 2.0)
}

/// Midpoint samples for the work integral.
const WORK_SAMPLES: usize = 900;

/// Samples for the peak scan.
const PEAK_SAMPLES: usize = 720;

/// Bisection halvings for [`zero_crossing`]. Sixty halvings of any human
/// joint range lands far below f64 noise.
const BISECTION_ITERS: usize = 60;

/// A lift, reduced to the one thing every analysis needs.
///
/// `phi` is the movement's own angle in **radians** — which joint and
/// which direction is the implementor's business, documented there.
pub trait Lift {
    /// Torque at the analysed joint, N·m.
    ///
    /// **Signed, never magnitude.** The sign is what shows a cable stop
    /// resisting and start assisting: positive resists the movement,
    /// negative drives it. In absolute value that crossing is invisible,
    /// and a rider would read assistance as load.
    fn tau(&self, phi: f64) -> f64;

    /// The movement's angular range, radians, as `(from, to)` **in the
    /// direction the repetition travels**.
    ///
    /// The order is not decoration: [`tau_at_progress`] builds the
    /// shared comparison axis from it. A lift that declares its range
    /// backwards reads as running in reverse against every other one,
    /// and nothing about the numbers looks wrong — the curve is simply
    /// mirrored. `gluteo` declared `(0, bottom)` for exactly as long as
    /// its progress helper lived inside that module and compensated.
    fn range(&self) -> (f64, f64);
}

/// `W = ∫ τ dφ` over `[a, b]` by the midpoint rule.
///
/// # φ is in RADIANS
///
/// Integrating in degrees returns a number `180/π ≈ 57.3` times too
/// large, and **no unit check catches it**: the radian is dimensionless,
/// so the result still carries joules. This is the one silent error in
/// the whole crate, which is why it has its own test.
pub fn work<L: Lift + ?Sized>(lift: &L, a: f64, b: f64) -> f64 {
    let step = (b - a) / WORK_SAMPLES as f64;
    (0..WORK_SAMPLES)
        .map(|i| lift.tau(a + (i as f64 + 0.5) * step))
        .sum::<f64>()
        * step
}

/// [`work`] over the lift's whole declared range, **ordered**.
///
/// `range()` is `(from, to)` in the direction the repetition travels, and
/// some lifts travel toward a smaller angle — a hip extension ends at
/// zero flexion. Integrating `from → to` there would return the work with
/// a minus sign, which is a true statement about the external torque and
/// a useless one about the repetition. The interval is what is being
/// asked for, so it is integrated in order.
pub fn work_over_range<L: Lift + ?Sized>(lift: &L) -> f64 {
    let (a, b) = lift.range();
    work(lift, a.min(b), a.max(b))
}

/// The largest resisting torque and where it occurs, as `(phi, tau)`.
///
/// Largest *signed* torque, not largest magnitude: a strongly assisting
/// position is not a peak of resistance.
pub fn peak<L: Lift + ?Sized>(lift: &L) -> (f64, f64) {
    let (a, b) = lift.range();
    let mut best = (a, lift.tau(a));
    for i in 1..=PEAK_SAMPLES {
        let phi = a + (b - a) * i as f64 / PEAK_SAMPLES as f64;
        let tau = lift.tau(phi);
        if tau > best.1 {
            best = (phi, tau);
        }
    }
    best
}

/// The angle at which `tau` crosses zero — past it the load assists
/// instead of resisting.
///
/// `None` when the bracket ends carry the same sign, which is the common
/// case: most lifts resist throughout. Bisection, so a bracket holding
/// two crossings yields one of them.
pub fn zero_crossing<L: Lift + ?Sized>(lift: &L, lo: f64, hi: f64) -> Option<f64> {
    let (mut lo, mut hi) = (lo, hi);
    let (t_lo, t_hi) = (lift.tau(lo), lift.tau(hi));
    if t_lo == 0.0 {
        return Some(lo);
    }
    if t_hi == 0.0 {
        return Some(hi);
    }
    if t_lo.is_sign_positive() == t_hi.is_sign_positive() {
        return None;
    }
    let rising = t_lo < 0.0;
    for _ in 0..BISECTION_ITERS {
        let mid = (lo + hi) / 2.0;
        if (lift.tau(mid) < 0.0) == rising {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some((lo + hi) / 2.0)
}

/// Style index on the tension ramp, for a normalized `t`.
///
/// Returns an **index, never a colour**: presentation lives in the
/// lesson manifest, so this crate names no palette. `buckets == 0`
/// yields 0 rather than underflowing.
#[must_use]
pub fn heat_bucket(t: f64, buckets: usize) -> usize {
    if buckets == 0 {
        return 0;
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let idx = (t.clamp(0.0, 1.0) * buckets as f64) as usize;
    idx.min(buckets - 1)
}

#[cfg(test)]
mod tests {

    /// The progress axis starts where the repetition starts.
    ///
    /// This is the claim that would have caught a real defect: the helper
    /// used to live inside `gluteo`, where it compensated for that
    /// module declaring its range backwards. Promoted to the root and
    /// applied to a lift that declares its range honestly, the
    /// compensation silently mirrored the curve. Nothing about the
    /// numbers looks wrong when that happens — which is why it is a
    /// claim and not a comment.
    #[test]
    fn progress_starts_where_the_repetition_starts() {
        /// A lift whose range runs the other way, to cover both signs.
        struct Sube;
        impl Lift for Sube {
            fn tau(&self, phi: f64) -> f64 {
                phi
            }
            fn range(&self) -> (f64, f64) {
                (0.0, 1.0)
            }
        }
        let baja = gluteo::HipThrust {
            load_kg: 100.0,
            femur_m: 0.46,
            bottom_rad: 40.0_f64.to_radians(),
        };
        for lift in [&Sube as &dyn Lift, &baja as &dyn Lift] {
            let (from, to) = lift.range();
            assert!(
                (tau_at_progress(lift, 0.0) - lift.tau(from)).abs() < 1e-12,
                "s=0 must be the start of the range"
            );
            assert!(
                (tau_at_progress(lift, 1.0) - lift.tau(to)).abs() < 1e-12,
                "s=1 must be the end of the range"
            );
        }
    }

    /// Work over a range does not change sign because the angle decreases.
    #[test]
    fn work_over_range_is_orientation_free() {
        let ht = gluteo::HipThrust {
            load_kg: 100.0,
            femur_m: 0.46,
            bottom_rad: 40.0_f64.to_radians(),
        };
        assert!(work_over_range(&ht) > 0.0, "a repetition costs joules");
    }
    use super::*;
    use std::f64::consts::PI;

    /// A constant-torque lift over one radian: work must equal the torque.
    struct Flat(f64);
    impl Lift for Flat {
        fn tau(&self, _phi: f64) -> f64 {
            self.0
        }
        fn range(&self) -> (f64, f64) {
            (0.0, 1.0)
        }
    }

    /// τ(φ) = cos φ over [0, τ/4]: ∫ = sin(τ/4) = 1 exactly.
    struct Cosine;
    impl Lift for Cosine {
        fn tau(&self, phi: f64) -> f64 {
            phi.cos()
        }
        fn range(&self) -> (f64, f64) {
            (0.0, PI / 2.0)
        }
    }

    #[test]
    fn work_of_a_constant_is_torque_times_angle() {
        assert!((work(&Flat(50.0), 0.0, 2.0) - 100.0).abs() < 1e-9);
    }

    #[test]
    fn work_integrates_a_known_antiderivative() {
        // ∫₀^{π/2} cos = 1. The midpoint rule's error is O(h²).
        assert!((work_over_range(&Cosine) - 1.0).abs() < 1e-6);
    }

    /// The one silent unit error in the crate: integrating dφ in degrees
    /// inflates the answer by exactly 180/π and still reports joules.
    #[test]
    fn degrees_inflate_the_work_integral_by_180_over_pi() {
        let rad = work(&Flat(120.0), 0.0, PI / 2.0);
        let deg = work(&Flat(120.0), 0.0, 90.0);
        assert!((deg / rad - 180.0 / PI).abs() < 1e-9);
    }

    #[test]
    fn peak_finds_the_largest_resisting_torque() {
        let (phi, tau) = peak(&Cosine);
        assert!(phi.abs() < 1e-9, "cos peaks at 0, got {phi}");
        assert!((tau - 1.0).abs() < 1e-9);
    }

    #[test]
    fn peak_prefers_resistance_over_magnitude() {
        // τ = −10 at φ<0.5, +1 above: the peak is the +1, not the −10.
        struct Step;
        impl Lift for Step {
            fn tau(&self, phi: f64) -> f64 {
                if phi < 0.5 {
                    -10.0
                } else {
                    1.0
                }
            }
            fn range(&self) -> (f64, f64) {
                (0.0, 1.0)
            }
        }
        assert!((peak(&Step).1 - 1.0).abs() < 1e-9);
    }

    #[test]
    fn zero_crossing_is_none_without_a_sign_change() {
        assert!(zero_crossing(&Flat(30.0), 0.0, 1.0).is_none());
        assert!(zero_crossing(&Cosine, 0.0, PI / 4.0).is_none());
    }

    #[test]
    fn zero_crossing_finds_a_known_root() {
        // cos crosses zero at π/2, approached from either direction.
        let root = zero_crossing(&Cosine, 0.0, PI).expect("cos changes sign on [0, π]");
        assert!((root - PI / 2.0).abs() < 1e-12, "got {root}");
    }

    #[test]
    fn heat_bucket_clamps_and_stays_in_range() {
        assert_eq!(heat_bucket(-5.0, 8), 0);
        assert_eq!(heat_bucket(0.0, 8), 0);
        assert_eq!(heat_bucket(1.0, 8), 7);
        assert_eq!(heat_bucket(99.0, 8), 7);
        for i in 0..=100 {
            assert!(heat_bucket(f64::from(i) / 100.0, 8) < 8);
        }
    }

    #[test]
    fn heat_bucket_survives_zero_buckets() {
        assert_eq!(heat_bucket(0.5, 0), 0);
    }
}
