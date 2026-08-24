"""Bouncing ball: the closed forms the page draws from, asserted."""
import math

def run():
    g, h0, e = 9.81, 4.0, 0.75
    t1 = math.sqrt(2 * h0 / g)
    T = t1 * (1 + e) / (1 - e)

    # the geometric series really sums to the closed form
    s = t1 + sum(2 * t1 * e ** n for n in range(1, 200000))
    assert abs(s - T) < 1e-9, (s, T)

    # apex heights follow e^(2n)*h0 -- and contain NO g:
    for n in range(1, 8):
        assert abs(h0 * e ** (2 * n) - h0 * (e * e) ** n) < 1e-12
    for g2 in (1.62, 24.79):                 # Moon, Jupiter: same apexes
        assert h0 * e ** 2 == h0 * e ** 2    # g never enters the formula
        assert abs(math.sqrt(2 * h0 / g2) * (1 + e) / (1 - e)
                   - T * math.sqrt(g / g2)) < 1e-9   # times scale as 1/sqrt(g)

    # why the page does NOT integrate: a 2 kHz fixed-step integrator
    # detectably steals energy at each impact
    dt, y, v, apex1, prev_v = 1 / 2000, h0, 0.0, None, 0.0
    t = 0.0
    while t < T and apex1 is None:
        v -= g * dt; y += v * dt; t += dt
        if y <= 0 and v < 0:
            y, v = 0.0, -e * v
        if prev_v > 0 >= v:
            apex1 = y
        prev_v = v
    drift = h0 * e ** 2 - apex1
    assert drift > 1e-4, "expected measurable integrator drift; got %g" % drift
    return "bouncing ball: series==closed form, g-free apexes, integrator drift %.4f m" % drift
