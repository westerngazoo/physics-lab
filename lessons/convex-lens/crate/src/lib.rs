//! Convex thin lens — one solver, three principal rays.
//!
//! Port of `checks/optics.py` and `public/examples/optics/js/optics.js`.
//! The object sits at `d_o` to the left of the lens; the image position and
//! magnification come from the thin-lens equation. Every drawn ray must pass
//! through the image point the solver computes — that is the lesson's claim.

/// Thin-lens solve: `di = d_o·f/(d_o−f)`, `m = −di/d_o`.  Returns `None` at F.
#[must_use]
pub fn solve(d_o: f64, f: f64) -> Option<(f64, f64)> {
    let den = d_o - f;
    if den.abs() < 1e-12 {
        return None;
    }
    let di = d_o * f / den;
    Some((di, -di / d_o))
}

/// Where ray 3 meets the lens (verified in `checks/optics.py`).
fn ray3_lens_y(h: f64, d_o: f64, f: f64) -> f64 {
    -h * f / (d_o - f)
}

/// Max distance from `(xi, yi)` to the three refracted rays (world units).
fn ray_error(d_o: f64, f: f64, h: f64, di: f64, m: f64) -> f64 {
    let xi = di;
    let yi = m * h;
    let tip = (-d_o, h);
    let y3 = ray3_lens_y(h, d_o, f);

    // R1 refracted: (0,h) → through (+f,0) → image
    let e1 = dist_to_line((0.0, h), (f, 0.0), (xi, yi));
    // R2: through origin
    let e2 = dist_to_line(tip, (0.0, 0.0), (xi, yi));
    // R3 refracted: horizontal from (0, y3)
    let e3 = if (yi - y3).abs() < 1e-9 {
        0.0
    } else {
        dist_to_line((0.0, y3), (1.0, y3), (xi, yi))
    };
    e1.max(e2).max(e3)
}

fn dist_to_line(a: (f64, f64), b: (f64, f64), p: (f64, f64)) -> f64 {
    let (ax, ay) = a;
    let (bx, by) = b;
    let (px, py) = p;
    let dx = bx - ax;
    let dy = by - ay;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 1e-15 {
        return ((px - ax).powi(2) + (py - ay).powi(2)).sqrt();
    }
    let t = ((px - ax) * dx + (py - ay) * dy) / (len * len);
    let (qx, qy) = (ax + t * dx, ay + t * dy);
    ((px - qx).powi(2) + (py - qy).powi(2)).sqrt()
}

fn draw(p: &[f64], out: &mut lessons_common::Prims, read: &mut lessons_common::Readouts) {
    let dobj = p[0];
    let f = p[1];
    let h = p[2];
    let x_right = 9.0;

    // axis
    out.segment(-8.5, 0.0, x_right, 0.0, 0);
    // lens (biconvex hint)
    let lh = 2.2;
    let curve = 0.55;
    out.polyline(
        [
            (0.0, lh),
            (curve, 0.0),
            (0.0, -lh),
            (-curve, 0.0),
            (0.0, lh),
        ],
        1,
    );
    // F and F′
    out.point(-f, 0.0, 2);
    out.point(f, 0.0, 2);
    out.point(-2.0 * f, 0.0, 2);
    out.point(2.0 * f, 0.0, 2);

    out.arrow(-dobj, 0.0, -dobj, h, 3);

    match solve(dobj, f) {
        None => {
            read.set(0, f64::NAN);
            read.set(1, f64::NAN);
            read.set(2, f64::NAN);
            read.set(3, -1.0);
            // parallel emergent rays when object at F
            out.segment(-dobj, h, 0.0, h, 4);
            out.segment(0.0, h, x_right, h, 4);
            let y3 = ray3_lens_y(h, dobj, f);
            out.segment(-dobj, h, 0.0, y3, 5);
            out.segment(0.0, y3, x_right, y3, 5);
            out.segment(-dobj, h, x_right, -h * x_right / dobj, 6);
        }
        Some((di, m)) => {
            let xi = di;
            let yi = m * h;
            let real = di > 0.0;
            let img_style = if real { 7 } else { 8 };
            out.arrow(xi, 0.0, xi, yi, img_style);

            let y3 = ray3_lens_y(h, dobj, f);

            // R1: parallel in, refracted through F′
            out.segment(-dobj, h, 0.0, h, 4);
            out.segment(0.0, h, xi, yi, 4);
            if !real {
                out.segment(0.0, h, xi, yi, 9); // dashed back-extension
            } else if xi < x_right {
                out.segment(xi, yi, x_right, h + (x_right - f) * (-h / f), 4);
            }

            // R2: through centre
            out.segment(-dobj, h, xi, yi, 5);
            if !real {
                out.segment(-dobj, h, xi, yi, 9);
            }

            // R3: through F, parallel out
            out.segment(-dobj, h, 0.0, y3, 6);
            if real {
                out.segment(0.0, y3, x_right, y3, 6);
            } else {
                out.segment(0.0, y3, x_right, y3, 6);
                out.segment(0.0, y3, xi, yi, 9);
            }

            let err = ray_error(dobj, f, h, di, m);
            read.set(0, di);
            read.set(1, m);
            read.set(2, err);
            read.set(3, if real { 1.0 } else { 0.0 });
        }
    }
}

lessons_common::lesson!(draw);

#[cfg(test)]
mod tests {
    use super::*;

    /// Same five converging cases as `checks/optics.py`.
    #[test]
    fn c1_five_converging_cases() {
        let cases = [
            (6.0, "real", false, "reduced"),
            (4.0, "real", false, "same"),
            (3.0, "real", false, "magnified"),
            (1.0, "virtual", true, "magnified"),
        ];
        for (d_o, kind, upright, size) in cases {
            let (di, m) = solve(d_o, 2.0).expect("image");
            assert_eq!((di > 0.0), kind == "real", "d_o={d_o}");
            assert_eq!((m > 0.0), upright, "d_o={d_o} m={m}");
            let mag = m.abs();
            let got = if (mag - 1.0).abs() < 1e-9 {
                "same"
            } else if mag > 1.0 {
                "magnified"
            } else {
                "reduced"
            };
            assert_eq!(got, size, "d_o={d_o} mag={mag}");
        }
        assert!(solve(2.0, 2.0).is_none());
    }

    /// Ray 3 height at the lens equals the image tip (optics.py loop).
    #[test]
    fn c2_ray3_lands_on_image_tip() {
        for d_o in [0.7, 1.0, 3.0, 4.0, 6.0, 7.5] {
            let (_di, m) = solve(d_o, 2.0).unwrap();
            let y3 = ray3_lens_y(1.0, d_o, 2.0);
            assert!((y3 - m * 1.0).abs() < 1e-9, "d_o={d_o}");
        }
    }

    /// All three principal rays pass through the image point.
    #[test]
    fn c3_rays_through_image() {
        for d_o in [0.7, 1.0, 3.0, 4.0, 6.0, 7.5] {
            let (di, m) = solve(d_o, 2.0).unwrap();
            let err = ray_error(d_o, 2.0, 1.0, di, m);
            assert!(err < 1e-9, "d_o={d_o} err={err}");
        }
    }

    /// Virtual upright when object is inside the focal length.
    #[test]
    fn c4_virtual_inside_f() {
        let (di, m) = solve(1.0, 2.0).unwrap();
        assert!(di < 0.0 && m > 0.0);
    }

    /// Readout error slot is honest through the wasm ABI.
    #[test]
    fn c5_buffer_ray_error_small() {
        unsafe {
            let p = params_ptr();
            *p.add(0) = 4.0;
            *p.add(1) = 2.0;
            *p.add(2) = 1.0;
        }
        state_at(3);
        let err = unsafe { *readouts_ptr().add(2) };
        assert!(err < 1e-9);
        let di = unsafe { *readouts_ptr().add(0) };
        assert!((di - 4.0).abs() < 1e-9);
    }
}
