"""Las medidas de la sentadilla: los archivos y la cuenta, sin el crate.

Dos cosas que el lado de Rust no puede atestiguar de sí mismo:

1. Que la plantilla y el ejemplo son TOML de verdad: aquí los lee
   `tomllib`, un lector completo que no es el subconjunto escrito a mano.
2. Que el ejemplo da los números que la línea de comandos publica. La
   cuenta se rehace con geometría: el ángulo del tronco se busca por
   bisección (la barra sobre el medio pie), no con el arcoseno del modelo.
"""
import math
import os
import tomllib

G = 9.81
TOBILLO_X = -0.04          # el tobillo fijo del modelo, m
AQUI = os.path.dirname(os.path.abspath(__file__))
CAMPOS = {
    "estatura": "valor_cm", "masa": "valor_kg", "altura_hombro": "valor_cm",
    "altura_trocanter": "valor_cm", "altura_rodilla": "valor_cm",
    "altura_maleolo": "valor_cm", "largo_pie": "valor_cm",
    "altura_barra": "valor_cm", "angulo_tibia": "valor_grados",
}


def leer(nombre):
    with open(os.path.join(AQUI, "..", "medidas", nombre), "rb") as f:
        return tomllib.load(f)


def forma(doc):
    """Solo las claves de arriba y las nueve tablas, cada una con las suyas."""
    tablas = {k for k, v in doc.items() if isinstance(v, dict)}
    raiz = set(doc) - tablas
    assert raiz == {"formato", "persona", "fecha"}, raiz
    assert doc["formato"] == "medidas-sentadilla/1"
    assert tablas == set(CAMPOS), tablas ^ set(CAMPOS)
    for t, clave in CAMPOS.items():
        sobra = set(doc[t]) - {"fuente", "fecha", "como", clave}
        assert not sobra, (t, sobra)
        assert doc[t]["fuente"] in ("medido", "estimado"), t


def brazos(femur, tibia, torso, tibia_grados):
    rodilla_x = TOBILLO_X + tibia * math.sin(math.radians(tibia_grados))
    cadera_x = rodilla_x - femur
    lo, hi = 0.0, math.pi / 2          # barra_x(θ) = cadera_x + torso·sen θ sube con θ
    for _ in range(200):
        th = 0.5 * (lo + hi)
        lo, hi = (th, hi) if cadera_x + torso * math.sin(th) < 0 else (lo, th)
    return -cadera_x, rodilla_x, math.degrees(lo)


def run():
    forma(leer("plantilla-sentadilla.toml"))
    ej = leer("ejemplo-sentadilla.toml")
    forma(ej)
    v = {t: ej[t].get(c) for t, c in CAMPOS.items()}
    femur = (v["altura_trocanter"] - v["altura_rodilla"]) / 100
    tibia = (v["altura_rodilla"] - v["altura_maleolo"]) / 100
    torso = (v["altura_barra"] - v["altura_trocanter"]) / 100
    d_cadera, d_rodilla, tronco = brazos(femur, tibia, torso, v["angulo_tibia"])
    # Lo que publica `mecanica sentadilla --medidas medidas/ejemplo-sentadilla.toml --carga 100`.
    assert round(100 * G * d_cadera, 2) == 273.40, 100 * G * d_cadera
    assert round(100 * G * d_rodilla, 2) == 143.53, 100 * G * d_rodilla
    assert round(tronco, 2) == 32.41, tronco
    # MECÁNICA 01 por la misma cuenta: 274 y 119 N·m.
    d_cadera, d_rodilla, _ = brazos(0.40, 0.43, 0.53, 22.0)
    assert round(100 * G * d_cadera) == 274 and round(100 * G * d_rodilla) == 119
    return "medidas: plantilla y ejemplo son TOML; ejemplo 273.40 / 143.53 N·m, tronco 32.41°"
