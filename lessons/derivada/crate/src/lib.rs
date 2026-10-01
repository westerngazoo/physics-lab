//! La derivada como pendiente — de la secante a la tangente.
//!
//! Un punto `a`, un paso `h` y tres funciones. La recta que pasa por
//! P = (a, f(a)) y Q = (a+h, f(a+h)) tiene pendiente (f(a+h) − f(a))/h: eso
//! ES la definición de la derivada antes del límite. Al achicar h la secante
//! gira hasta la tangente, cuya pendiente es f′(a), dada aquí en forma
//! cerrada para poder medir cuánto le falta a la secante.
//!
//! Lo que se lee abajo de la gráfica también es una afirmación: el error
//! |secante − f′(a)| dividido entre h tiende a |f″(a)|/2. La secante se
//! acerca en proporción a h — convergencia de primer orden — y el número que
//! lo delata está en pantalla.

/// Las tres funciones, por índice: f₀ = x²/2, f₁ = sen x, f₂ = x³/3 − x.
pub fn f(k: usize, x: f64) -> f64 {
    match k {
        0 => x * x / 2.0,
        1 => x.sin(),
        _ => x * x * x / 3.0 - x,
    }
}

/// f′ exacta — con qué se compara la secante.
pub fn df(k: usize, x: f64) -> f64 {
    match k {
        0 => x,
        1 => x.cos(),
        _ => x * x - 1.0,
    }
}

/// f″ exacta — la que predice cuánto falla la secante.
pub fn d2f(k: usize, x: f64) -> f64 {
    match k {
        0 => 1.0,
        1 => -x.sin(),
        _ => 2.0 * x,
    }
}

/// La pendiente de la secante por (a, f(a)) y (a+h, f(a+h)).
pub fn secante(k: usize, a: f64, h: f64) -> f64 {
    (f(k, a + h) - f(k, a)) / h
}

/// El índice de función que manda el deslizador (puede llegar no entero).
pub fn funcion(p: f64) -> usize {
    (p.round().clamp(0.0, 2.0)) as usize
}

// ---- the wasm boundary --------------------------------------------------

use lessons_common::{Prims, Readouts};

// estilos (índices en lesson.json)
const EJES: usize = 0;
const CURVA: usize = 1;
const SECANTE: usize = 2;
const TANGENTE: usize = 3;
const TRIANGULO: usize = 4;
const TEXTO: usize = 5;
const PUNTO: usize = 6;
// etiquetas (índices en lesson.json → labels)
const L_P: usize = 3;
const L_Q: usize = 4;
const L_H: usize = 5;
const L_DY: usize = 6;

const X0: f64 = -3.0;
const X1: f64 = 3.0;

/// Parámetros en orden de manifiesto: función, a, h.
fn draw(p: &[f64], out: &mut Prims, read: &mut Readouts) {
    let k = funcion(p[0]);
    let (a, h) = (p[1], p[2]);
    let (fa, fq) = (f(k, a), f(k, a + h));
    let ms = secante(k, a, h);
    let mt = df(k, a);
    let recta = |m: f64, x: f64| fa + m * (x - a);

    out.view(0);
    out.segment(X0, 0.0, X1, 0.0, EJES);
    out.segment(0.0, -2.1, 0.0, 2.1, EJES);
    out.curve(X0, X1, 240, CURVA, |x| (x, f(k, x)));
    out.segment(X0, recta(ms, X0), X1, recta(ms, X1), SECANTE);
    out.segment(X0, recta(mt, X0), X1, recta(mt, X1), TANGENTE);
    // el triángulo "avanza h, sube Δy"
    out.segment(a, fa, a + h, fa, TRIANGULO);
    out.segment(a + h, fa, a + h, fq, TRIANGULO);
    out.point(a, fa, PUNTO);
    out.point(a + h, fq, SECANTE);
    out.label(a - 0.12, fa + 0.16, L_P, TEXTO);
    out.label(a + h + 0.12, fq + 0.16, L_Q, TEXTO);
    out.label(a + h / 2.0, fa - 0.18, L_H, TEXTO);
    out.label(a + h + 0.22, (fa + fq) / 2.0, L_DY, TEXTO);
    out.label(-2.3, 1.8, k, TEXTO); // qué función es (labels 0..2)

    let err = (ms - mt).abs();
    read.set(0, a);
    read.set(1, h);
    read.set(2, ms);
    read.set(3, mt);
    read.set(4, err);
    read.set(5, err / h);
}

lessons_common::lesson!(draw);

// ---- the claims ---------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use lessons_common::{PRIM_CAP, READ_SLOTS};

    fn puntos() -> impl Iterator<Item = f64> {
        (-8..=8).map(|i| i as f64 * 0.25) // −2 .. 2
    }

    /// Lo que la página mostraría para (k, a, h): las 8 lecturas.
    fn lecturas(k: usize, a: f64, h: f64) -> [f64; READ_SLOTS] {
        let mut buf = [0.0; PRIM_CAP];
        let mut rd = [0.0; READ_SLOTS];
        draw(
            &[k as f64, a, h],
            &mut Prims::new(&mut buf),
            &mut Readouts::new(&mut rd),
        );
        rd
    }

    /// D1: para h pequeño (h ≤ 1/4), el error baja cada vez que h se reduce a
    /// la mitad, con las tres funciones.
    ///
    /// "Pequeño" no es adorno: lejos de 0 no tiene por qué. Con sen x en
    /// a = −0.5, h = 1 y h = 0.5 dan la MISMA secante (el seno es impar), y
    /// este test, escrito primero sin la condición, lo encontró.
    #[test]
    fn d1_la_secante_se_acerca_a_la_tangente() {
        let (a, h1, h2) = (-0.5, 1.0, 0.5);
        assert_eq!(secante(1, a, h1), secante(1, a, h2), "the counterexample stands");
        for k in 0..3 {
            for a in puntos() {
                let mut h = 0.25;
                let mut antes = (secante(k, a, h) - df(k, a)).abs();
                for _ in 0..10 {
                    h /= 2.0;
                    let ahora = (secante(k, a, h) - df(k, a)).abs();
                    assert!(ahora < antes, "k={k} a={a} h={h}: {ahora} ≮ {antes}");
                    antes = ahora;
                }
            }
        }
    }

    /// D2: para x²/2 la secante tiene pendiente a + h/2, exactamente — el
    /// álgebra de la definición, sin límite todavía.
    #[test]
    fn d2_la_secante_de_la_parabola_es_a_mas_h_medios() {
        for a in puntos() {
            for h in [1.5, 1.0, 0.3, 0.01] {
                assert!((secante(0, a, h) - (a + h / 2.0)).abs() < 1e-12, "a={a} h={h}");
            }
        }
    }

    /// D3: el error se va como h — error/h tiende a |f″(a)|/2. La lectura
    /// "error / h" converge a un número que la página no dibuja: la curvatura.
    #[test]
    fn d3_el_error_sobre_h_tiende_a_la_mitad_de_f_segunda() {
        for k in 0..3 {
            for a in puntos().filter(|a| d2f(k, *a).abs() > 0.1) {
                let rd = lecturas(k, a, 1e-4);
                let limite = d2f(k, a).abs() / 2.0;
                assert!((rd[5] - limite).abs() < 2e-3, "k={k} a={a}: {} vs {limite}", rd[5]);
            }
        }
    }

    /// D4: x³/3 − x tiene tangente horizontal justo en x = ±1.
    #[test]
    fn d4_la_cubica_se_aplana_en_mas_menos_uno() {
        assert_eq!(df(2, 1.0), 0.0);
        assert_eq!(df(2, -1.0), 0.0);
        for a in puntos().filter(|a| (a.abs() - 1.0).abs() > 1e-9) {
            assert!(df(2, a) != 0.0, "a={a}");
        }
    }

    /// D5: la derivada del seno es el coseno — y eso es lo que marca la lectura.
    #[test]
    fn d5_la_derivada_del_seno_es_el_coseno() {
        for a in puntos() {
            let rd = lecturas(1, a, 0.5);
            assert_eq!(rd[3], a.cos(), "a={a}");
        }
    }

    /// El deslizador de función llega a veces sin redondear (el barrido lo
    /// mueve en continuo): siempre elige una de las tres, nunca otra cosa.
    #[test]
    fn el_indice_de_funcion_siempre_es_valido() {
        for p in [-1.0, 0.0, 0.4, 0.6, 1.49, 2.0, 7.0, f64::NAN] {
            assert!(funcion(p) <= 2, "p={p}");
        }
    }
}
