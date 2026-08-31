//! The cable glute kickback — four knobs, one model.
//!
//! From reels 14, 15, 16 and 20. The hip is the pivot; the cable pulls
//! the strap toward the pulley; the moment is the plain 2D cross product
//! of the hip→strap vector with that force.
//!
//! **Signed**, following reel 20's correction. Reel 14 returned
//! magnitude, which hides the fact that past a certain angle the cable
//! stops resisting hip extension and starts driving it — a stretch of
//! range that feels like work and is not.
//!
//! ## The range is a knob, not a derived quantity
//!
//! Torso angle in the reels decides how much range is available: bent to
//! 90° the hip travels 90°, standing it has ~40° of hyperextension left.
//! No published model derives range *from* torso angle, so this exposes
//! the range directly rather than inventing a law connecting them. The
//! two published presets are [`Patada::AGACHADA`] and [`Patada::DE_PIE`].

use crate::{Lift, G};

/// The cable kickback.
#[derive(Clone, Debug, PartialEq)]
pub struct Patada {
    /// Stack load, kilograms.
    pub load_kg: f64,
    /// Hip height above the floor, metres.
    pub hip_height_m: f64,
    /// Hip → strap along the leg, metres. Ankle ≈ 0.85, above-knee ≈ 0.46.
    pub strap_m: f64,
    /// Pulley position `(x, y)` in metres: `x` ahead of the hip, `y`
    /// above the floor.
    pub pulley: (f64, f64),
    /// Where the movement starts, radians. Negative begins ahead of
    /// vertical, with the leg still forward (reel 20).
    pub start_rad: f64,
    /// Where the movement ends, radians of hip extension past vertical.
    pub end_rad: f64,
}

impl Patada {
    /// Bent to 90°: the hip travels a full 90° (reels 14–16).
    pub const AGACHADA: (f64, f64) = (0.0, std::f64::consts::FRAC_PI_2);
    /// Standing: only the ~40° of hyperextension is left (reel 14).
    pub const DE_PIE: (f64, f64) = (0.0, 0.698_131_700_797_731_8);

    /// Where the strap sits at hip-extension angle `psi`.
    ///
    /// `psi = 0` hangs the leg vertically below the hip; positive swings
    /// the foot backwards and up.
    #[must_use]
    pub fn strap_at(&self, psi: f64) -> (f64, f64) {
        (
            -self.strap_m * psi.sin(),
            self.hip_height_m - self.strap_m * psi.cos(),
        )
    }

    /// Cable tension, newtons.
    #[must_use]
    pub fn tension_n(&self) -> f64 {
        self.load_kg * G
    }
}

impl Lift for Patada {
    /// Hip moment at extension angle `psi`, N·m.
    fn tau(&self, psi: f64) -> f64 {
        let (sx, sy) = self.strap_at(psi);
        let (rx, ry) = (sx, sy - self.hip_height_m); // hip → strap
        let (dx, dy) = (self.pulley.0 - sx, self.pulley.1 - sy);
        let n = dx.hypot(dy);
        let t = self.tension_n();
        rx * (t * dy / n) - ry * (t * dx / n)
    }

    fn range(&self) -> (f64, f64) {
        (self.start_rad, self.end_rad)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{peak, work_over_range, zero_crossing};
    use std::f64::consts::PI;

    /// The published setup: 15 kg, ankle strap, pulley at floor level.
    fn published() -> Patada {
        Patada {
            load_kg: 15.0,
            hip_height_m: 1.00,
            strap_m: 0.85,
            pulley: (0.60, 0.05),
            start_rad: Patada::AGACHADA.0,
            end_rad: Patada::AGACHADA.1,
        }
    }

    /// C1: bent over yields more work than standing — the reel's "casi
    /// el doble", from range alone.
    #[test]
    fn c1_bent_over_does_more_work_than_standing() {
        let bent = published();
        let mut standing = published();
        (standing.start_rad, standing.end_rad) = Patada::DE_PIE;
        assert!(work_over_range(&bent) > work_over_range(&standing) * 1.5);
    }

    /// C2: the peak torque is *identical* between the two, because the
    /// peak lies inside the shorter range. This is the reel's headline —
    /// same peak, different range — and it only holds if the peak really
    /// does fall below 40°.
    #[test]
    fn c2_peak_is_identical_between_torso_angles() {
        let bent = published();
        let mut standing = published();
        (standing.start_rad, standing.end_rad) = Patada::DE_PIE;

        let (phi_bent, tau_bent) = peak(&bent);
        let (_, tau_standing) = peak(&standing);
        assert!(
            phi_bent < Patada::DE_PIE.1,
            "the claim needs the peak inside the short range; it is at \
             {:.1}°",
            phi_bent.to_degrees()
        );
        assert!(
            (tau_bent - tau_standing).abs() < 0.5,
            "{tau_bent} vs {tau_standing}"
        );
    }

    /// C3: the ankle strap is 1.85× the lever of the above-knee strap.
    #[test]
    fn c3_ankle_strap_is_1_85x_the_knee_strap() {
        let ankle = published();
        let mut knee = published();
        knee.strap_m = 0.46;
        assert_eq!(((ankle.strap_m / knee.strap_m) * 100.0).round(), 185.0);
        assert!(work_over_range(&ankle) > work_over_range(&knee));
    }

    /// C4: raising the pulley lowers torque at **every** angle in range.
    ///
    /// Reel 16 asserts "la naranja va arriba SIEMPRE" — a universally
    /// quantified claim, so it is tested as one rather than at the peak.
    #[test]
    fn c4_raising_the_pulley_lowers_torque_everywhere() {
        let floor = published();
        let mut raised = published();
        raised.pulley = (0.60, 0.45);
        for i in 0..=180 {
            let psi = PI / 2.0 * f64::from(i) / 180.0;
            assert!(
                raised.tau(psi) < floor.tau(psi),
                "raised pulley must load less at {:.1}°",
                psi.to_degrees()
            );
        }
    }

    /// C5: starting ahead of vertical, the cable assists before it
    /// resists, and the crossing is near −32° (reel 20).
    #[test]
    fn c5_the_cable_assists_before_the_crossing() {
        let mut early = published();
        early.start_rad = -45.0_f64.to_radians();
        let crossing = zero_crossing(&early, early.start_rad, 0.0)
            .expect("torque must change sign before vertical");
        assert_eq!(crossing.to_degrees().round(), -32.0);
        assert!(early.tau(crossing - 0.05) < 0.0, "assists before it");
        assert!(early.tau(crossing + 0.05) > 0.0, "resists after it");
    }

    /// C6: more range from an earlier start — reel 20's +29%.
    #[test]
    fn c6_starting_early_adds_work() {
        let base = published();
        let mut early = published();
        early.start_rad = -30.0_f64.to_radians();
        let rise = work_over_range(&early) / work_over_range(&base) - 1.0;
        assert_eq!((rise * 100.0).round(), 29.0);
    }

    /// C7: the published work figure, 165.5 J.
    ///
    /// Tolerant to 1 J because the reels hard-code the cable tension as
    /// 147 N while this computes 15 × 9.81 = 147.15.
    #[test]
    fn c7_reproduces_the_published_work() {
        assert!(
            (work_over_range(&published()) - 165.5).abs() < 1.0,
            "got {:.1} J",
            work_over_range(&published())
        );
    }

    /// C8: the peak is just *past* the vertical, at ≈8.6° and ≈125 N·m —
    /// **not** at the start, and not the 123.2 N·m the reels publish.
    ///
    /// Reel 20 says *"arrancas en el pico y de ahí baja"*. It does not:
    /// torque rises for the first 8.6° and falls afterwards. 123.2 N·m is
    /// the torque at ψ = 0 (this model gives 123.4 with tension computed
    /// as 15 × 9.81 rather than the reels' hard-coded 147 N), so the
    /// published figure is the *starting* torque labelled as the peak.
    ///
    /// The headline claim is unharmed — 8.6° still falls inside the
    /// standing range, so C2's "identical peak" holds — but the number
    /// and the sentence are both off, and the house rule is that a label
    /// disagreeing with the calculation is a bug, not licence.
    #[test]
    fn c8_the_peak_sits_just_past_the_vertical() {
        let p = published();
        let (phi, tau) = peak(&p);
        assert_eq!((phi.to_degrees() * 10.0).round() / 10.0, 8.6);
        assert_eq!((tau * 10.0).round() / 10.0, 125.1);
        assert!(tau > p.tau(0.0), "the peak must beat the starting torque");
    }
}
