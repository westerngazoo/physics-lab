"""Wave chain: exact eigenmode evolution, asserted against an
independent Verlet integration, plus the dispersion facts the page states."""
import math

L, c = 10.0, 1.5

def modes(N):
    dx = L / (N + 1)
    return dx, [(2 * c / dx) * math.sin(k * math.pi / (2 * (N + 1))) for k in range(1, N + 1)]

def pluck(N, pos, amp):
    out = []
    for i in range(1, N + 1):
        x = i / (N + 1)
        out.append(amp * (x / pos if x <= pos else (1 - x) / (1 - pos)))
    return out

def dst(y, N):
    return [(2 / (N + 1)) * sum(y[i - 1] * math.sin(k * math.pi * i / (N + 1))
            for i in range(1, N + 1)) for k in range(1, N + 1)]

def evolve(a, N, t, w):
    return [sum(a[k - 1] * math.cos(w[k - 1] * t) * math.sin(k * math.pi * i / (N + 1))
            for k in range(1, N + 1)) for i in range(1, N + 1)]

def run():
    N = 24
    dx, w = modes(N)
    y0 = pluck(N, 0.3, 1.3)
    a = dst(y0, N)

    # DST round-trips exactly
    y_rt = evolve(a, N, 0.0, w)
    assert max(abs(u - v) for u, v in zip(y0, y_rt)) < 1e-12

    # mode evolution == independent Verlet integration
    dt, T_test = 1e-4, 2.0
    y, v = y0[:], [0.0] * N
    def acc(y):
        return [c * c * ((y[i - 1] if i else 0.0) - 2 * y[i] +
                         (y[i + 1] if i < N - 1 else 0.0)) / (dx * dx) for i in range(N)]
    A = acc(y)
    for _ in range(int(T_test / dt)):
        y = [y[i] + v[i] * dt + 0.5 * A[i] * dt * dt for i in range(N)]
        A2 = acc(y)
        v = [v[i] + 0.5 * (A[i] + A2[i]) * dt for i in range(N)]
        A = A2
    err = max(abs(u - q) for u, q in zip(evolve(a, N, T_test, w), y))
    assert err < 1e-6, err

    # energy in mode form is constant (it is, identically -- spot-check)
    def E(t):
        tot = 0.0
        for k in range(1, N + 1):
            q = a[k - 1] * math.cos(w[k - 1] * t)
            qd = -a[k - 1] * w[k - 1] * math.sin(w[k - 1] * t)
            tot += qd * qd + (w[k - 1] * q) ** 2
        return tot
    assert abs(E(0) - E(3.7)) < 1e-9 * E(0)

    # the honest wrinkle: the lattice cutoff pins at 2/pi of c forever --
    # it descends toward 2/pi from above and never gets below it
    ratios = []
    for n in (24, 80, 400):
        dxn, wn = modes(n)
        ratios.append(wn[-1] / (n * math.pi / L) / c)
    lim = 2 / math.pi
    assert ratios[0] > ratios[1] > ratios[2] > lim, ratios
    assert ratios[2] - lim < 2e-3, ratios[2]

    # while the pluck's energy-weighted speed climbs with N
    def fid(n):
        dxn, wn = modes(n)
        an = dst(pluck(n, 0.3, 1.3), n)
        num = den = 0.0
        for k in range(1, n + 1):
            th = k * math.pi / (2 * (n + 1))
            Ek = (an[k - 1] * wn[k - 1]) ** 2
            num += Ek * math.sin(th) / th
            den += Ek
        return num / den
    f5, f24, f80 = fid(5), fid(24), fid(80)
    assert f5 < f24 < f80 < 1.0, (f5, f24, f80)
    return "wave: DST exact, modes==Verlet to %.1e, E const, cutoff->2/pi, fidelity %.3f<%.3f<%.3f" % (err, f5, f24, f80)
