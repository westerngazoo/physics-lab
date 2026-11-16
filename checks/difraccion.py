"""Difracción en un borde recto: las afirmaciones, por otro camino.

El crate `difraccion` calcula C y S con una serie y una fracción continua.
Aquí se calculan con Simpson, que no comparte nada con ellas: si las
afirmaciones salen igual por los dos caminos, ninguno se equivocó solo."""
import math


def fresnel(x, fase_por_paso=0.005):
    """C(x), S(x) por Simpson compuesto; el paso sigue a la fase π t²/2."""
    if x == 0:
        return 0.0, 0.0
    n = max(256, math.ceil(math.pi * x * x / fase_por_paso))
    n += n % 2
    h = x / n
    c = s = 0.0
    for k in range(n + 1):
        t = k * h
        peso = 1 if k in (0, n) else (4 if k % 2 else 2)
        fase = math.pi * t * t / 2
        c += peso * math.cos(fase)
        s += peso * math.sin(fase)
    return c * h / 3, s * h / 3


def borde(w):
    c, s = fresnel(w)
    return 0.5 * ((0.5 + c) ** 2 + (0.5 + s) ** 2)


def extremos(w0, w1, paso):
    """Extremos locales de I(w) por muestreo y una parábola por tres puntos."""
    ws = [w0 + k * paso for k in range(int(round((w1 - w0) / paso)) + 1)]
    i = [borde(w) for w in ws]
    out = []
    for k in range(1, len(i) - 1):
        if (i[k] - i[k - 1]) * (i[k + 1] - i[k]) < 0:
            a, b, c = i[k - 1], i[k], i[k + 1]
            d = 0.5 * (a - c) / (a - 2 * b + c)       # vértice de la parábola
            out.append((ws[k] + d * paso, b - 0.25 * (a - c) * d))
    return out


def run():
    # la tabla: C(1), S(1) (Abramowitz y Stegun 7.7)
    c, s = fresnel(1.0)
    assert abs(c - 0.7798934) < 1e-7 and abs(s - 0.4382591) < 1e-7, (c, s)

    # en el borde, un cuarto; muy adentro de la sombra, la cola 1/(2π²w²)
    assert borde(0.0) == 0.25
    w = -8.0
    cola = 1 / (2 * math.pi ** 2 * w * w)
    assert abs(borde(w) - cola) / cola < 0.01

    # la sombra sube sin oscilar hacia el borde
    prev = borde(-4.0)
    for k in range(1, 81):
        cur = borde(-4.0 + k * 0.05)
        assert cur > prev, ("la sombra oscila", -4.0 + k * 0.05)
        prev = cur

    # las franjas: 1.3704 en 1.2172, 0.7783 en 1.8725, 1.1993 en 2.3445
    ext = extremos(0.5, 4.0, 0.01)
    esperado = [(1.2172, 1.3704), (1.8725, 0.7783), (2.3445, 1.1993)]
    for (w, i), (we, ie) in zip(ext, esperado):
        assert abs(w - we) < 5e-4 and abs(i - ie) < 2e-4, ((w, i), (we, ie))
    # alternan, se juntan, y se acercan a w² = 2(k + 3/4)
    for k, (w, i) in enumerate(ext):
        assert (i > 1) == (k % 2 == 0), (k, i)
        assert abs(w * w - 2 * (k + 0.75)) < 0.03 * (k + 1), (k, w * w)
    gaps = [b[0] - a[0] for a, b in zip(ext, ext[1:])]
    assert all(g1 > g2 for g1, g2 in zip(gaps, gaps[1:])), gaps

    # un láser rojo (633 nm) a 1 m: la primera franja a 0.685 mm del borde
    x = ext[0][0] / math.sqrt(2 / (633e-9 * 1.0))
    assert abs(x * 1e3 - 0.685) < 0.001, x
    return "difraccion: tabla de Fresnel + borde (¼, cola, sombra sin franjas) + %d franjas + láser" % len(ext)
