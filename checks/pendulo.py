"""El péndulo simple — the period and the motion, by routes the Rust never takes.

Nothing here shares code with lessons/pendulo, and none of it uses the
arithmetic-geometric mean the page evaluates:
  - the exact period comes from direct quadrature of the elliptic integral,
    T = (4/ω₀)·∫₀^{π/2} dφ / √(1 − k² sin² φ), k = sin(θ₀/2) — composite
    Simpson, which converges spectrally on this smooth periodic integrand;
  - the motion comes from velocity Verlet (cargo's cross-check is RK4), with
    the mass carried explicitly, and its period is measured from zero crossings;
  - the series bracket, the √2 scaling and energy are checked on those.
"""
import math

G = 9.81


def periodo_cuadratura(theta0, l, g, n=400):
    """Exact period by Simpson on the complete elliptic integral of the first kind."""
    k2 = math.sin(theta0 / 2) ** 2
    h = (math.pi / 2) / n
    f = lambda phi: 1.0 / math.sqrt(1.0 - k2 * math.sin(phi) ** 2)
    s = f(0.0) + f(math.pi / 2)
    for i in range(1, n):
        s += f(i * h) * (4 if i % 2 else 2)
    return 4.0 * math.sqrt(l / g) * s * h / 3.0


def verlet(theta0, l, g, m, dt, steps):
    """Velocity Verlet on m·L·θ̈ = −m·g·sin θ. Returns (downward zero
    crossings, worst relative energy error)."""
    acc = lambda th: (-m * g * math.sin(th)) / (m * l)
    th, om, a = theta0, 0.0, acc(theta0)
    e0 = g * l * (1 - math.cos(theta0))
    cruces, peor = [], 0.0
    for i in range(steps):
        th1 = th + om * dt + 0.5 * a * dt * dt
        a1 = acc(th1)
        om = om + 0.5 * (a + a1) * dt
        if th > 0 >= th1:
            cruces.append(i * dt + dt * th / (th - th1))
        th, a = th1, a1
        e = 0.5 * l * l * om * om + g * l * (1 - math.cos(th))
        peor = max(peor, abs(e / e0 - 1))
    return cruces, peor


def periodo_verlet(theta0, l, g, m=1.0, por_periodo=40000):
    t = periodo_cuadratura(theta0, l, g)
    cruces, peor = verlet(theta0, l, g, m, t / por_periodo, int(2.5 * por_periodo))
    assert len(cruces) >= 2, ("pocos cruces", theta0)
    return cruces[1] - cruces[0], peor


def run():
    grid = [math.radians(d / 2) for d in range(2, 161)]  # 1°..80°, step 0.5°
    t0 = lambda l, g: 2 * math.pi * math.sqrt(l / g)

    # P4: strictly increasing, and bracketed by the all-positive series
    antes = 1.0
    for th in grid:
        r = periodo_cuadratura(th, 1.0, G) / t0(1.0, G)
        assert r > antes, ("P4 crece", math.degrees(th))
        antes = r
        tres = 1 + th ** 2 / 16 + 11 * th ** 4 / 3072
        cuarto = 173 * th ** 6 / 737280
        assert tres - 1e-14 < r < tres + 1.2 * cuarto + 1e-14, ("P4 serie", math.degrees(th))

    # P3: small angles → T₀, excess ~ θ₀²/16
    for d in (2.0, 1.0, 0.5):
        th = math.radians(d)
        exceso = periodo_cuadratura(th, 1.0, G) / t0(1.0, G) - 1
        assert abs(exceso / (th * th / 16) - 1) < 1e-3, ("P3", d)

    # P1 (the period the page reads): Verlet's measured period == quadrature
    for d in (1, 30, 60, 80):
        th = math.radians(d)
        medido, peor = periodo_verlet(th, 1.0, G)
        assert abs(medido / periodo_cuadratura(th, 1.0, G) - 1) < 1e-6, ("P1", d)
        assert peor < 1e-7, ("P6 Verlet", d, peor)  # P6: energy, bounded

    # P2: the mass cancels — 50 g and 200 kg measure the same period
    th = math.radians(40)
    a, _ = periodo_verlet(th, 0.8, G, m=0.05)
    b, _ = periodo_verlet(th, 0.8, G, m=200.0)
    assert abs(a / b - 1) < 1e-10, ("P2", a, b)

    # P5: doubling L multiplies the measured period by √2
    for d in (5, 45, 80):
        th = math.radians(d)
        p1, _ = periodo_verlet(th, 0.5, 1.62)
        p2, _ = periodo_verlet(th, 1.0, 1.62)
        assert abs(p2 / p1 - math.sqrt(2)) < 1e-6, ("P5", d)

    # the angle where T differs from T₀ by 1 % (the try-this question)
    lo, hi = math.radians(10), math.radians(40)
    for _ in range(60):
        mid = (lo + hi) / 2
        if periodo_cuadratura(mid, 1.0, G) / t0(1.0, G) < 1.01:
            lo = mid
        else:
            hi = mid
    uno = math.degrees(lo)
    assert 22.5 < uno < 23.0, uno
    return "pendulo: T by quadrature brackets the series 1°–80°, Verlet period ==, mass-free, √2, 1%% at θ₀ = %.2f°" % uno
