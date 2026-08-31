//! The squat, solved at the bottom position.
//!
//! This module deliberately does **not** implement [`crate::Lift`]. The
//! two reels it comes from — MECÁNICA 01 and reel 07 — are static
//! comparisons of one posture, and no published model validates a
//! descent kinematic. Inventing one to satisfy a trait would be exactly
//! the "looks right rather than is right" failure this repo exists to
//! avoid. A depth axis is a v2 change with its own claims.
//!
//! Geometry, from `fisicobuenfisico/tools/muneco.py::resolver_sentadilla`:
//! `+x` points toward the toes, the thigh is horizontal at the bottom,
//! and the torso angle is *solved* so the bar lands where it must.

use crate::G;

/// Ankle joint centre, metres. From the source model; the foot is not
/// part of this geometry (only the ankle is), which is why the bar
/// reference is the midfoot line rather than a foot landmark.
const ANKLE: (f64, f64) = (-0.04, 0.09);

/// A squat at the bottom position: thigh to parallel.
#[derive(Clone, Debug, PartialEq)]
pub struct Sentadilla {
    /// Hip → knee, metres.
    pub femur_m: f64,
    /// Ankle → knee, metres.
    pub tibia_m: f64,
    /// Hip → shoulder, metres.
    pub torso_m: f64,
    /// Bar load, kilograms.
    pub load_kg: f64,
    /// Bar displacement forward of the midfoot, metres. `0.0` is over it.
    pub bar_offset_m: f64,
    /// Shank inclination from vertical, radians.
    pub shank_rad: f64,
}

/// A solved bottom position, in metres, with the two moment arms.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Postura {
    pub ankle: (f64, f64),
    pub knee: (f64, f64),
    pub hip: (f64, f64),
    pub bar: (f64, f64),
    /// Torso inclination from vertical, radians. Positive leans forward.
    pub torso_rad: f64,
    /// Horizontal hip → bar distance, metres.
    pub d_hip: f64,
    /// Horizontal knee → bar distance, metres.
    pub d_knee: f64,
}

impl Sentadilla {
    /// Solve the bottom position.
    ///
    /// The torso angle is not a knob: it is whatever angle puts the bar
    /// over its required line given where the hip ended up. That is the
    /// whole mechanism behind "a longer femur forces more lean".
    #[must_use]
    pub fn postura(&self) -> Postura {
        let knee = (
            ANKLE.0 + self.tibia_m * self.shank_rad.sin(),
            ANKLE.1 + self.tibia_m * self.shank_rad.cos(),
        );
        let hip = (knee.0 - self.femur_m, knee.1); // thigh to parallel
        let torso_rad = ((self.bar_offset_m - hip.0) / self.torso_m)
            .clamp(-0.99, 0.99)
            .asin();
        let bar = (
            hip.0 + self.torso_m * torso_rad.sin(),
            hip.1 + self.torso_m * torso_rad.cos(),
        );
        Postura {
            ankle: ANKLE,
            knee,
            hip,
            bar,
            torso_rad,
            d_hip: (bar.0 - hip.0).abs(),
            d_knee: (bar.0 - knee.0).abs(),
        }
    }

    /// Hip moment, N·m.
    #[must_use]
    pub fn tau_hip(&self) -> f64 {
        self.load_kg * G * self.postura().d_hip
    }

    /// Knee moment, N·m.
    #[must_use]
    pub fn tau_knee(&self) -> f64 {
        self.load_kg * G * self.postura().d_knee
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// MECÁNICA 01's published setup: 100 kg, bar over the midfoot.
    fn published(femur_m: f64) -> Sentadilla {
        Sentadilla {
            femur_m,
            tibia_m: 0.43,
            torso_m: 0.53,
            load_kg: 100.0,
            bar_offset_m: 0.0,
            shank_rad: 22.0_f64.to_radians(),
        }
    }

    /// C1: a longer femur strictly increases the hip moment arm.
    #[test]
    fn c1_longer_femur_lengthens_the_hip_lever() {
        let mut last = f64::NEG_INFINITY;
        for i in 0..=14 {
            let d = published(0.38 + 0.01 * f64::from(i)).postura().d_hip;
            assert!(d > last, "d_hip must be strictly increasing, got {d}");
            last = d;
        }
    }

    /// C2: the knee moment arm does not depend on femur length at all.
    ///
    /// The reel's surprising claim — "119 N·m in both" — and the reason
    /// a long femur is a hip problem, not a knee one.
    #[test]
    fn c2_knee_lever_is_invariant_to_femur_length() {
        let reference = published(0.40).postura().d_knee;
        for i in 0..=14 {
            let d = published(0.38 + 0.01 * f64::from(i)).postura().d_knee;
            assert!((d - reference).abs() < 1e-12, "knee arm moved to {d}");
        }
    }

    /// C3: walking the bar forward raises the hip moment monotonically.
    #[test]
    fn c3_bar_forward_raises_hip_torque() {
        let mut last = f64::NEG_INFINITY;
        for i in 0..=12 {
            let mut s = published(0.45);
            s.bar_offset_m = 0.01 * f64::from(i);
            let t = s.tau_hip();
            assert!(t > last, "tau_hip must rise with offset, got {t}");
            last = t;
        }
    }

    /// C4: with no offset the solved bar sits exactly on the midfoot line.
    #[test]
    fn c4_zero_offset_puts_the_bar_over_the_midfoot() {
        for i in 0..=14 {
            let bar_x = published(0.38 + 0.01 * f64::from(i)).postura().bar.0;
            assert!(bar_x.abs() < 1e-12, "bar drifted to x={bar_x}");
        }
    }

    /// C5: the model reproduces MECÁNICA 01's published numbers.
    ///
    /// 28 cm / 38 cm of hip lever, 274 and 372 N·m (+36%), and 119 N·m
    /// at the knee in both. Rounded as the graphic rounds them.
    #[test]
    fn c5_reproduces_the_published_mecanica_01_numbers() {
        let short = published(0.40);
        let long = published(0.50);

        assert_eq!((short.postura().d_hip * 100.0).round(), 28.0);
        assert_eq!((long.postura().d_hip * 100.0).round(), 38.0);
        assert_eq!(short.tau_hip().round(), 274.0);
        assert_eq!(long.tau_hip().round(), 372.0);

        assert_eq!((short.postura().d_knee * 100.0).round(), 12.0);
        assert_eq!(short.tau_knee().round(), 119.0);
        assert_eq!(long.tau_knee().round(), 119.0);

        let rise = long.tau_hip() / short.tau_hip() - 1.0;
        assert_eq!((rise * 100.0).round(), 36.0, "the +36% headline");
    }

    /// C6: a longer femur forces more forward lean — the mechanism the
    /// reel describes in words, asserted as a number.
    #[test]
    fn c6_longer_femur_forces_more_forward_lean() {
        assert!(published(0.50).postura().torso_rad > published(0.40).postura().torso_rad);
    }
}
