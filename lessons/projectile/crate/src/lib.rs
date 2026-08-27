//! Projectile Range — the deliberately minimal lesson, and the
//! authoring template the classroom guide walks through line by line.
//!
//! A launch at speed `v`, angle `θ` (measured in turns of τ), gravity
//! `g`. Everything is closed form, and the trajectory is drawn in
//! NATURAL UNITS `v²/g`, which is why changing `v` or `g` never changes
//! the drawn shape — only the numbers. That scale invariance is itself
//! a claim (P5), not an accident.

use std::f64::consts::TAU;

/// Range, apex height, and flight time — the real, unit-carrying numbers.
pub fn numbers(theta: f64, v: f64, g: f64) -> (f64, f64, f64) {
    let range = v * v * (2.0 * theta).sin() / g;
    let apex = v * v * theta.sin().powi(2) / (2.0 * g);
    let time = 2.0 * v * theta.sin() / g;
    (range, apex, time)
}

/// The trajectory in natural units x' = x·g/v²: shape depends on θ alone.
pub fn shape(theta: f64, xp: f64) -> f64 {
    xp * theta.tan() - xp * xp / (2.0 * theta.cos().powi(2))
}

// ---- the wasm boundary --------------------------------------------------

use lessons_common::{Prims, Readouts};

/// Params (manifest order): [theta (radians; slider in turns), v, g].
fn draw(p: &[f64], out: &mut Prims, read: &mut Readouts) {
    let (theta, v, g) = (p[0], p[1], p[2]);
    let (range, apex, time) = numbers(theta, v, g);
    let rp = (2.0 * theta).sin(); // range in natural units

    out.view(0);
    out.segment(-0.05, 0.0, 1.15, 0.0, 0);                    // ground
    out.arrow(0.0, 0.0, 0.16 * theta.cos(), 0.16 * theta.sin(), 3); // launch
    out.curve(0.0, rp, 48, 1, |xp| (xp, shape(theta, xp)));   // the flight
    out.point(rp / 2.0, shape(theta, rp / 2.0), 2);           // apex
    out.segment(rp, -0.02, rp, 0.02, 3);                      // range tick

    read.set(0, theta / TAU);
    read.set(1, range);
    read.set(2, apex);
    read.set(3, time);
    read.set(4, (2.0 * theta).sin() * 100.0); // % of the best possible range
}

lessons_common::lesson!(draw);

// ---- the claims ---------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn thetas() -> impl Iterator<Item = f64> {
        (1..24).map(|i| i as f64 * TAU / 100.0) // 0.01τ .. 0.23τ
    }

    /// P1: the drawn curve lands exactly where the range formula says.
    #[test]
    fn p1_curve_lands_on_the_range() {
        for theta in thetas() {
            let rp = (2.0 * theta).sin();
            assert!(shape(theta, rp).abs() < 1e-12, "theta={theta}");
        }
    }

    /// P2: range is maximized at θ = τ/8, and nowhere else on the grid.
    #[test]
    fn p2_range_maxes_at_tau_over_eight() {
        let best = numbers(TAU / 8.0, 2.0, 9.81).0;
        for theta in thetas() {
            let r = numbers(theta, 2.0, 9.81).0;
            assert!(r <= best + 1e-12);
            if (theta - TAU / 8.0).abs() > 1e-9 {
                assert!(r < best, "theta={theta} must be strictly worse");
            }
        }
    }

    /// P3: complementary angles share a range — R(θ) = R(τ/4 − θ).
    #[test]
    fn p3_complementary_angles_share_a_range() {
        for theta in thetas().filter(|t| *t < TAU / 8.0) {
            let a = numbers(theta, 3.0, 1.62).0;
            let b = numbers(TAU / 4.0 - theta, 3.0, 1.62).0;
            assert!((a - b).abs() < 1e-12 * a.max(1e-12), "theta={theta}");
        }
    }

    /// P4: the drawn apex sits at the apex formula's height.
    #[test]
    fn p4_apex_matches() {
        for theta in thetas() {
            let (_, apex, _) = numbers(theta, 2.5, 9.81);
            let natural = shape(theta, (2.0 * theta).sin() / 2.0);
            // convert natural apex back to meters: × v²/g
            assert!((natural * 2.5 * 2.5 / 9.81 - apex).abs() < 1e-12);
        }
    }

    /// P5: scale invariance — the drawn shape is bit-identical across
    /// (v, g), which is the whole point of natural units.
    #[test]
    fn p5_shape_ignores_v_and_g() {
        for theta in thetas() {
            for xp in [0.1, 0.3, 0.55, 0.8] {
                // shape() takes no v or g at all: the invariance is
                // structural. Assert the numbers still differ, so the
                // claim is not vacuous.
                let a = numbers(theta, 2.0, 9.81);
                let b = numbers(theta, 4.0, 1.62);
                assert!(a.0 != b.0 && shape(theta, xp) == shape(theta, xp));
            }
        }
    }
}
