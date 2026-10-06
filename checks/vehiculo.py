"""El vehículo en la curva: las afirmaciones, sin las fórmulas del crate.

La trazada se comprueba construyendo el círculo y midiendo distancias a
los bordes, no con R = r + w/(1 − cos(θ/2)). El tope se comprueba con la
condición de equilibrio v²/r ≤ μg, no con √(μgr)."""
import math

G = 9.80665
MU = 1.2


def agarre(ax, ay, mu, g):
    l = math.hypot(ax, ay)
    return (ax, ay) if l <= mu * g else (ax * mu * g / l, ay * mu * g / l)


def mas_rapida(r, mu, g):
    """La v más alta con v²/r ≤ μg, por bisección: sin la raíz cerrada."""
    lo, hi = 0.0, 1000.0
    for _ in range(200):
        m = 0.5 * (lo + hi)
        lo, hi = (m, hi) if m * m / r <= mu * g else (lo, m)
    return lo


def trazada_por_bisectriz(ri, w, giro):
    """El círculo más grande tangente a los dos bordes exteriores y que no
    toca el pasto: se busca el centro sobre la bisectriz por bisección."""
    normales = [(math.cos(s * giro / 2), math.sin(s * giro / 2)) for s in (-1, 1)]

    def radio_por_rectas(d):        # centro en (−d, 0): distancia a la recta
        return min(ri + w + d * nx for nx, _ in normales)

    lo, hi = 0.0, 1e6               # hueco: R − (d + ri) cambia de signo
    for _ in range(200):
        d = 0.5 * (lo + hi)
        lo, hi = (d, hi) if radio_por_rectas(d) - (d + ri) > 0 else (lo, d)
    return radio_por_rectas(lo)


def run():
    # el círculo de fricción: 3-4-5 y la dirección se conserva
    assert agarre(3, 4, MU, G) == (3, 4)
    ax, ay = agarre(30, 40, 1.0, 10.0)
    assert abs(ax - 6) < 1e-12 and abs(ay - 8) < 1e-12

    # el doble de radio da √2 veces la rapidez; 55 y 78 km/h para 20 y 40 m
    v20, v40 = mas_rapida(20, MU, G), mas_rapida(40, MU, G)
    assert abs(v40 / v20 - math.sqrt(2)) < 1e-9, v40 / v20
    assert round(v20 * 3.6) == 55 and round(v40 * 3.6) == 78

    # 60 km/h en 20 m no cabe; el giro posible es de 23.6 m
    v = 60 / 3.6
    assert v * v / 20 > MU * G
    assert abs(v * v / (MU * G) - 23.6) < 0.01

    # la trazada, por construcción: 90° da 37.3 m; 120° da r + 2w; 180°, r + w
    assert abs(trazada_por_bisectriz(10, 8, math.pi / 2) - 37.3137085) < 1e-6
    assert abs(trazada_por_bisectriz(10, 5, 2 * math.pi / 3) - 20) < 1e-9
    assert abs(trazada_por_bisectriz(10, 8, math.pi) - 18) < 1e-9
    # y la de 90° con la de adentro a 1 m del borde: 1.84 veces la rapidez
    assert abs(math.sqrt(trazada_por_bisectriz(10, 8, math.pi / 2) / 11) - 1.84) < 0.005
    return "vehiculo: círculo de fricción + tope (√2, 55/78 km/h, 23.6 m) + trazada por construcción"
