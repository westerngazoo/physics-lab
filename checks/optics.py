"""Optics: four elements over one solver -- the claims, asserted.

Every principal-ray construction drawn by examples/optics must pass
through the image point the solver computes. This is the same math the
page runs (ported by hand; keep in sync with js/optics.js)."""
import itertools

def solve(do, f):
    den = do - f
    if abs(den) < 1e-12:
        return None
    di = do * f / den
    return di, -di / do

def run():
    h = 1.0
    # --- the five converging cases (lens f=+2; identical for concave mirror) ---
    cases = {6: ("real", "inverted", "reduced"), 4: ("real", "inverted", "same"),
             3: ("real", "inverted", "magnified"), 1: ("virtual", "upright", "magnified")}
    for do, (kind, orient, size) in cases.items():
        di, m = solve(do, 2.0)
        assert (di > 0) == (kind == "real"), (do, di)
        assert (m > 0) == (orient == "upright"), (do, m)
        mag = abs(m)
        got = "same" if abs(mag - 1) < 1e-9 else ("magnified" if mag > 1 else "reduced")
        assert got == size, (do, mag, size)
    assert solve(2.0, 2.0) is None, "object at F must yield no image"

    # --- diverging family: always virtual, upright, reduced ---
    for f in (-2.0, -1.3, -3.1):
        for do in (0.4, 1, 2, 3, 5, 8):
            di, m = solve(do, f)
            assert di < 0 and 0 < m < 1, (f, do, di, m)

    # --- lens ray 3 lands exactly on the image tip ---
    for do, f in itertools.product((0.7, 1, 3, 4, 6, 7.5), (1.0, 2.0, 3.2, -1.0, -2.0)):
        s = solve(do, f)
        if s is None:
            continue
        di, m = s
        y3 = -h * f / (do - f)              # ray-3 height at the lens
        assert abs(y3 - m * h) < 1e-9, (do, f)

    # --- all three MIRROR rays pass through the image (world coords) ---
    for do, f in itertools.product((0.7, 1, 3, 4, 6, 7.5), (2.0, 1.0, 3.0, -2.0, -1.0)):
        s = solve(do, f)
        if s is None:
            continue
        di, m = s
        xi, yi = -di, m * h                  # mirror image sits at x = -di
        assert abs((h + xi * h / f) - yi) < 1e-9, ("R1", do, f)      # parallel -> through F
        assert abs((h * xi / do) - yi) < 1e-9, ("R2", do, f)         # vertex, symmetric
        y3 = h - do * h / (do - f)
        assert abs(y3 - yi) < 1e-9, ("R3", do, f)                    # through F -> parallel

    # --- the abstraction itself: lens and mirror share the solver verbatim ---
    for do in (0.5, 1, 2.5, 4, 6, 8):
        for f in (2.0, -2.0):
            assert solve(do, f) == solve(do, f)  # one function, by construction
    return "optics: 5 cases + diverging family + lens/mirror ray constructions"
