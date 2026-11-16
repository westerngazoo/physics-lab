//! Sumas de Riemann — el área como el límite de rectángulos.
//!
//! n rectángulos sobre [0, b], con la altura tomada a la izquierda, a la
//! derecha o en el punto medio de cada uno. La suma se compara con la
//! integral exacta, en forma cerrada.
//!
//! La lección no es sólo "con más rectángulos se acerca". Las lecturas
//! "error × n" y "error × n²" muestran CÓMO se acerca: izquierda y derecha
//! fallan en proporción a 1/n, el punto medio en proporción a 1/n² — un orden
//! entero más rápido, gratis, sólo por elegir dónde medir la altura. Y √x, cuya
//! pendiente no es finita en 0, le quita esa ventaja al punto medio.

/// Las tres funciones: f₀ = x²/4, f₁ = sen x, f₂ = √x.
pub fn f(k: usize, x: f64) -> f64 {
    match k {
        0 => x * x / 4.0,
        1 => x.sin(),
        _ => x.max(0.0).sqrt(),
    }
}

/// ∫₀ᵇ f, exacta.
pub fn integral(k: usize, b: f64) -> f64 {
    match k {
        0 => b * b * b / 12.0,
        1 => 1.0 - b.cos(),
        _ => 2.0 / 3.0 * b.max(0.0).powf(1.5),
    }
}

/// f′ exacta, donde existe (√x no la tiene finita en 0).
pub fn df(k: usize, x: f64) -> f64 {
    match k {
        0 => x / 2.0,
        1 => x.cos(),
        _ => 0.5 / x.sqrt(),
    }
}

/// Dónde se mide la altura de cada rectángulo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Regla {
    Izquierda,
    Derecha,
    Medio,
}

impl Regla {
    /// La que manda la elección (0, 1, 2), también si llega sin redondear.
    pub fn de(p: f64) -> Regla {
        match p.round().clamp(0.0, 2.0) as usize {
            0 => Regla::Izquierda,
            1 => Regla::Derecha,
            _ => Regla::Medio,
        }
    }
    /// El punto de muestra del rectángulo [x, x + dx].
    pub fn muestra(self, x: f64, dx: f64) -> f64 {
        match self {
            Regla::Izquierda => x,
            Regla::Derecha => x + dx,
            Regla::Medio => x + dx / 2.0,
        }
    }
}

/// La suma de Riemann de f sobre [0, b] con n rectángulos.
pub fn suma(k: usize, b: f64, n: usize, regla: Regla) -> f64 {
    let n = n.max(1);
    let dx = b / n as f64;
    (0..n)
        .map(|i| f(k, regla.muestra(i as f64 * dx, dx)))
        .sum::<f64>()
        * dx
}

/// n desde el deslizador: entero, al menos 1 (el barrido lo mueve en continuo).
pub fn rectangulos(p: f64) -> usize {
    if p.is_finite() {
        p.round().clamp(1.0, 60.0) as usize
    } else {
        1
    }
}

// ---- the wasm boundary --------------------------------------------------

use lessons_common::{Prims, Readouts};

const EJES: usize = 0;
const CURVA: usize = 1;
const RECT: usize = 2;
const MUESTRA: usize = 3;
const LIMITE: usize = 4;
const TEXTO: usize = 5;

/// Parámetros en orden de manifiesto: función, regla, n, b.
fn draw(p: &[f64], out: &mut Prims, read: &mut Readouts) {
    let k = (p[0].round().clamp(0.0, 2.0)) as usize;
    let regla = Regla::de(p[1]);
    let n = rectangulos(p[2]);
    let b = p[3];
    let dx = b / n as f64;

    out.view(0);
    out.segment(-0.3, 0.0, 3.3, 0.0, EJES);
    out.segment(0.0, -0.3, 0.0, 2.3, EJES);
    for i in 0..n {
        let x = i as f64 * dx;
        let s = regla.muestra(x, dx);
        let y = f(k, s);
        out.polyline([(x, 0.0), (x, y), (x + dx, y), (x + dx, 0.0)], RECT);
        out.point(s, y, MUESTRA);
    }
    out.curve(0.0, 3.2, 200, CURVA, |x| (x, f(k, x)));
    out.segment(b, -0.12, b, 2.3, LIMITE);
    out.label(2.25, 2.05, k, TEXTO); // qué función es (labels 0..2)

    let s = suma(k, b, n, regla);
    let exacta = integral(k, b);
    let err = (s - exacta).abs();
    let nf = n as f64;
    read.set(0, s);
    read.set(1, exacta);
    read.set(2, err);
    read.set(3, err * nf);
    read.set(4, err * nf * nf);
}

lessons_common::lesson!(draw);

// ---- the claims ---------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    const REGLAS: [Regla; 3] = [Regla::Izquierda, Regla::Derecha, Regla::Medio];
    const BS: [f64; 3] = [0.7, 1.3, 2.1]; // lejos de π, donde sen(b) = sen(0)

    /// R1: con más rectángulos la suma se acerca a la integral — para las tres
    /// funciones y las tres reglas.
    #[test]
    fn r1_mas_rectangulos_se_acercan_a_la_integral() {
        for k in 0..3 {
            for regla in REGLAS {
                for b in BS {
                    let err = |n| (suma(k, b, n, regla) - integral(k, b)).abs();
                    assert!(err(4096) < 1e-3, "k={k} {regla:?} b={b}");
                    assert!(err(512) < err(8), "k={k} {regla:?} b={b}");
                }
            }
        }
    }

    /// R2: si f crece, la izquierda se queda corta y la derecha se pasa — para
    /// cualquier n, no sólo en el límite.
    #[test]
    fn r2_si_f_crece_la_izquierda_queda_corta_y_la_derecha_se_pasa() {
        for k in [0, 2] {
            for b in BS {
                for n in 1..=60 {
                    let exacta = integral(k, b);
                    assert!(suma(k, b, n, Regla::Izquierda) <= exacta + 1e-12, "k={k} b={b} n={n}");
                    assert!(suma(k, b, n, Regla::Derecha) >= exacta - 1e-12, "k={k} b={b} n={n}");
                }
            }
        }
    }

    /// R3: izquierda y derecha fallan como 1/n: error·n → |f(b) − f(0)|·b/2.
    #[test]
    fn r3_izquierda_y_derecha_fallan_como_uno_sobre_n() {
        for k in [0, 1] {
            for regla in [Regla::Izquierda, Regla::Derecha] {
                for b in BS {
                    let n = 4096.0;
                    let err = (suma(k, b, n as usize, regla) - integral(k, b)).abs();
                    let limite = ((f(k, b) - f(k, 0.0)) * b / 2.0).abs();
                    assert!((err * n - limite).abs() < 0.01 * limite, "k={k} {regla:?} b={b}");
                }
            }
        }
    }

    /// R4: el punto medio falla como 1/n²: error·n² → |f′(b) − f′(0)|·b²/24.
    /// Un orden entero más rápido, sólo por dónde se mide la altura.
    #[test]
    fn r4_el_punto_medio_falla_como_uno_sobre_n_cuadrado() {
        for k in [0, 1] {
            for b in BS {
                let n = 4096.0;
                let err = (suma(k, b, n as usize, Regla::Medio) - integral(k, b)).abs();
                let limite = ((df(k, b) - df(k, 0.0)) * b * b / 24.0).abs();
                assert!((err * n * n - limite).abs() < 0.01 * limite, "k={k} b={b}");
            }
        }
    }

    /// R5: con √x el punto medio pierde su ventaja — su pendiente no es finita
    /// en 0, y error·n² ya no se estabiliza: sigue creciendo con n.
    #[test]
    fn r5_con_raiz_el_punto_medio_pierde_su_ventaja() {
        let b = 2.0;
        let en = |n: usize| (suma(2, b, n, Regla::Medio) - integral(2, b)).abs() * (n * n) as f64;
        assert!(en(4096) > 4.0 * en(64), "error·n² keeps growing: {} vs {}", en(4096), en(64));
    }

    #[test]
    fn el_deslizador_siempre_da_un_n_y_una_regla_validos() {
        for p in [-3.0, 0.0, 0.4, 59.6, 60.0, 99.0, f64::NAN] {
            let n = rectangulos(p);
            assert!((1..=60).contains(&n), "p={p}");
        }
        assert_eq!(Regla::de(1.4), Regla::Derecha);
        assert_eq!(Regla::de(9.0), Regla::Medio);
    }
}
