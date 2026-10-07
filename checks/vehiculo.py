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


def reparto_iterado(k_del, k_tras, k_ch, alfa, vueltas=20000):
    """El modelo de dos nodos por relajación de Gauss-Seidel: cada nodo se
    equilibra con el otro fijo, una y otra vez. Sin la forma cerrada."""
    th_d = th_t = 0.0
    for _ in range(vueltas):
        th_d = (alfa + k_ch * th_t) / (k_del + k_ch)
        th_t = ((1 - alfa) + k_ch * th_d) / (k_tras + k_ch)
    return k_del * th_d, k_tras * th_t


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
    # la transferencia de carga del kart del reel 1.3 (valores supuestos)
    J = math.pi / 32 * (0.030 ** 4 - 0.026 ** 4)
    k_ch = 2 * 80e9 * J / 1.04
    k_del, k_tras = 100e3 * 0.975 ** 2 / 2, 100e3 * 1.205 ** 2 / 2
    assert round(k_ch) == 5332 and round(k_del) == 47531 and round(k_tras) == 72601
    d, t = reparto_iterado(k_del, k_tras, k_ch, 0.0)
    assert abs(d + t - 1) < 1e-9 and abs(t - 0.938) < 5e-4, (d, t)
    assert abs(0.58 * 1.205 / (2 * t * 0.28) - 1.33) < 5e-3     # la trasera, masa atrás
    d, t = reparto_iterado(k_del, k_tras, k_ch, 0.42)
    assert abs(t - 0.584) < 5e-4, t
    assert abs(0.42 * 0.975 / (2 * d * 0.28) - 1.76) < 5e-3     # la delantera, masa repartida
    # chasis casi rígido (mil veces las llantas): manda k_del : k_tras, entre
    # donde entre el par. La relajación se arrastra cuando el chasis acopla
    # mucho (cada vuelta corrige ~0.4 %), así que lleva sus 20 000 vueltas.
    for alfa in (0.0, 0.5, 1.0):
        d, t = reparto_iterado(1.0, 3.0, 1e3, alfa)
        assert abs(t - 0.75) < 1e-3, (alfa, t)
    return "vehiculo: círculo de fricción + tope (√2, 55/78 km/h, 23.6 m) + trazada por construcción + dos nodos por relajación (94 %, 1.33 g; 58 %, 1.76 g)"
