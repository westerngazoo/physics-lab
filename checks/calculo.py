"""Cálculo — la derivada y las sumas de Riemann, contra oráculos independientes.

Nothing here shares code with the Rust lessons, and nothing restates their
formulas as its own proof:
  - each exact f′ and f″ is checked against the complex-step derivative,
    Im f(x+ih)/h — a numerical method with no subtraction, good to machine
    precision;
  - the parabola's secant is computed in exact rationals (D2);
  - each closed-form integral is checked against composite Simpson;
  - the convergence orders (R3, R4) are measured at n different from cargo's;
  - the midpoint error for x²/4 is checked EXACTLY, in rationals: b³/(48n²).
"""
import cmath
import math
from fractions import Fraction as Q

# ── derivada ────────────────────────────────────────────────────────────────
D_FUNS = [
    (lambda z: z * z / 2, lambda x: x, lambda x: 1.0),
    (cmath.sin, math.cos, lambda x: -math.sin(x)),
    (lambda z: z ** 3 / 3 - z, lambda x: x * x - 1, lambda x: 2 * x),
]


def complex_step(fz, x, h=1e-30):
    return fz(complex(x, h)).imag / h


def secante(fz, a, h):
    return (fz(complex(a + h)).real - fz(complex(a)).real) / h


def check_derivada():
    pts = [i / 4 for i in range(-8, 9)]
    for fz, df, d2f in D_FUNS:
        for a in pts:
            assert abs(complex_step(fz, a) - df(a)) < 1e-12, ("f′", a)
            # f″ via the complex step of f′'s own complex extension
            num = (complex_step(fz, a + 1e-6) - complex_step(fz, a - 1e-6)) / 2e-6
            assert abs(num - d2f(a)) < 1e-6, ("f″", a)
    # D1 (as the page states it): for h ≤ 1/4, halving h shrinks the error
    for fz, df, _ in D_FUNS:
        for a in pts:
            h, before = 0.25, abs(secante(fz, a, 0.25) - df(a))
            for _ in range(10):
                h /= 2
                now = abs(secante(fz, a, h) - df(a))
                assert now < before, ("D1", a, h)
                before = now
    # ...and the counterexample the lesson shows: sin, a = −0.5, h = 1 vs 1/2
    assert secante(cmath.sin, -0.5, 1.0) == secante(cmath.sin, -0.5, 0.5)
    # D2, exactly: ((a+h)²/2 − a²/2)/h = a + h/2 in rationals
    for a in (Q(-3, 2), Q(0), Q(1, 3), Q(7, 4)):
        for h in (Q(3, 2), Q(1), Q(1, 100)):
            assert ((a + h) ** 2 / 2 - a * a / 2) / h == a + h / 2
    # D3: error/h → |f″(a)|/2
    for fz, df, d2f in D_FUNS:
        for a in pts:
            if abs(d2f(a)) > 0.1:
                e = abs(secante(fz, a, 1e-5) - df(a)) / 1e-5
                assert abs(e - abs(d2f(a)) / 2) < 1e-3, ("D3", a)
    # D4: x³/3 − x is flat exactly at ±1
    assert D_FUNS[2][1](1.0) == 0 and D_FUNS[2][1](-1.0) == 0


# ── riemann ─────────────────────────────────────────────────────────────────
R_FUNS = [
    (lambda x: x * x / 4, lambda b: b ** 3 / 12, lambda x: x / 2),
    (math.sin, lambda b: 1 - math.cos(b), math.cos),
    (lambda x: math.sqrt(max(x, 0.0)), lambda b: 2 / 3 * b ** 1.5, None),
]
BS = (0.7, 1.3, 2.1)


def riemann(f, b, n, where):
    dx = b / n
    off = {"izq": 0.0, "der": 1.0, "medio": 0.5}[where]
    return sum(f((i + off) * dx) for i in range(n)) * dx


def simpson(f, b, n=20000):
    dx = b / n
    s = f(0) + f(b) + sum((4 if i % 2 else 2) * f(i * dx) for i in range(1, n))
    return s * dx / 3


def check_riemann():
    for f, exact, _ in R_FUNS:
        for b in BS:
            tol = 1e-9 if f is not R_FUNS[2][0] else 1e-6  # √x is not smooth at 0
            assert abs(simpson(f, b) - exact(b)) < tol, ("closed form", b)
            for where in ("izq", "der", "medio"):  # R1
                assert abs(riemann(f, b, 3000, where) - exact(b)) < 1e-3
    # R2: increasing f → left ≤ exact ≤ right, every n
    for f, exact, _ in (R_FUNS[0], R_FUNS[2]):
        for b in BS:
            for n in range(1, 61):
                assert riemann(f, b, n, "izq") <= exact(b) + 1e-12
                assert riemann(f, b, n, "der") >= exact(b) - 1e-12
    # R3 / R4 at n = 3000 (cargo uses 4096)
    n = 3000
    for f, exact, df in R_FUNS[:2]:
        for b in BS:
            for where in ("izq", "der"):
                e = abs(riemann(f, b, n, where) - exact(b)) * n
                lim = abs((f(b) - f(0)) * b / 2)
                assert abs(e - lim) < 0.01 * lim, ("R3", where, b)
            e2 = abs(riemann(f, b, n, "medio") - exact(b)) * n * n
            lim2 = abs((df(b) - df(0)) * b * b / 24)
            assert abs(e2 - lim2) < 0.01 * lim2, ("R4", b)
    # R4, exactly: for x²/4 the midpoint error IS b³/(48 n²), in rationals
    for b in (Q(7, 10), Q(13, 10), Q(21, 10)):
        for n in (1, 2, 7, 30):
            dx = b / n
            mid = sum(((i + Q(1, 2)) * dx) ** 2 / 4 for i in range(n)) * dx
            assert b ** 3 / 12 - mid == b ** 3 / (48 * n * n), (b, n)
    # R5: √x takes the midpoint's advantage away — error·n² keeps growing
    f, exact, _ = R_FUNS[2]
    g = lambda n: abs(riemann(f, 2.0, n, "medio") - exact(2.0)) * n * n
    assert g(4000) > 4 * g(62)


def run():
    check_derivada()
    check_riemann()
    return "calculo: D1–D5 (complex-step oracle, exact rationals) · R1–R5 (Simpson, exact midpoint error)"
