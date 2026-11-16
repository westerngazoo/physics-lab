//! `difraccion` — la luz detrás de un borde recto.
//!
//! Las integrales de Fresnel son
//!
//! ```text
//! C(x) = ∫₀ˣ cos(π t²/2) dt,    S(x) = ∫₀ˣ sin(π t²/2) dt,
//! ```
//!
//! y con ellas la intensidad detrás de un borde recto, opaco y delgado:
//!
//! ```text
//! I(w) / I₀ = ½ [(½ + C(w))² + (½ + S(w))²]
//! ```
//!
//! I₀ es la intensidad sin el borde y `w` la distancia al borde de la sombra
//! geométrica en **unidades de Fresnel** ([`unidades_de_fresnel`]); `w < 0`
//! es la sombra. En el borde llega la mitad de la amplitud, un cuarto de la
//! intensidad. Dentro de la sombra la luz baja sin oscilar y nunca llega a
//! cero; del lado iluminado oscila y tiende a 1.
//!
//! # El modelo, y lo que no es
//!
//! Onda plana (fuente muy lejana), un solo color, teoría escalar,
//! aproximación paraxial (`|x| ≪ z`). No hay polarización ni borde con
//! grosor: es la difracción de Fresnel de libro, y nada más.
//!
//! # Cómo se calcula
//!
//! En forma cerrada, sin pasos de integración: la serie de potencias para
//! `|x| ≤ 1.5` y, más allá, la fracción continua de erfc evaluada con el
//! método de Lentz (Press et al., *Numerical Recipes*, §6.8). Las dos
//! convergen a la precisión de `f64` en unas decenas de términos, así que una
//! página puede pedir cientos de valores por cuadro. Puro: sin estado ni
//! azar, el mismo `x` da los mismos bits.

#![forbid(unsafe_code)]

use std::f64::consts::{FRAC_PI_2, PI};

/// Hasta aquí manda la serie; de aquí en adelante, la fracción continua.
/// Es el corte de *Numerical Recipes*: las dos ramas convergen bien ahí.
const CORTE: f64 = 1.5;

/// Tolerancia relativa de las dos ramas: unos cuantos ulp de `f64`.
const EPS: f64 = 1e-16;

/// Tope de términos. Ninguna rama se le acerca (ver las pruebas), pero un
/// bucle sin tope no es un cálculo, es una esperanza.
const MAXIMO: usize = 300;

/// C(x) y S(x), las integrales de Fresnel. Son impares: C(−x) = −C(x).
pub fn fresnel(x: f64) -> (f64, f64) {
    let ax = x.abs();
    let (c, s) = if ax <= CORTE { serie(ax) } else { fraccion_continua(ax) };
    if x < 0.0 {
        (-c, -s)
    } else {
        (c, s)
    }
}

/// C + iS = Σₖ (iπ/2)ᵏ x^(2k+1) / (k! (2k+1)): los términos pares son de C y
/// los impares de S, con el signo de iᵏ.
fn serie(x: f64) -> (f64, f64) {
    let t = FRAC_PI_2 * x * x;
    let (mut c, mut s) = (0.0_f64, 0.0_f64);
    let mut termino = x; // tᵏ x / k!
    for k in 0..MAXIMO {
        let parte = termino / (2 * k + 1) as f64;
        match k % 4 {
            0 => c += parte,
            1 => s += parte,
            2 => c -= parte,
            _ => s -= parte,
        }
        termino *= t / (k + 1) as f64;
        // pasado el pico (k > t) los términos solo bajan
        if k as f64 > t && termino < EPS * (c.abs() + s.abs()) {
            break;
        }
    }
    (c, s)
}

/// Un complejo mínimo: lo justo para la fracción continua.
#[derive(Clone, Copy)]
struct Z(f64, f64);

impl Z {
    fn mas(self, o: Z) -> Z {
        Z(self.0 + o.0, self.1 + o.1)
    }
    fn por(self, o: Z) -> Z {
        Z(self.0 * o.0 - self.1 * o.1, self.0 * o.1 + self.1 * o.0)
    }
    fn escala(self, k: f64) -> Z {
        Z(self.0 * k, self.1 * k)
    }
    fn inverso(self) -> Z {
        let n = self.0 * self.0 + self.1 * self.1;
        Z(self.0 / n, -self.1 / n)
    }
}

/// C + iS = ½(1+i) erf(z) con z = (√π/2)(1−i)x, y erfc(z) por su fracción
/// continua (Lentz modificado). Converge para x > 1.5 en pocos términos.
fn fraccion_continua(x: f64) -> (f64, f64) {
    let fase2 = PI * x * x;
    let mut b = Z(1.0, -fase2);
    let mut cc = Z(1e300, 0.0);
    let mut d = b.inverso();
    let mut h = d;
    let mut n = -1.0;
    for _ in 2..MAXIMO {
        n += 2.0;
        let a = -n * (n + 1.0);
        b = b.mas(Z(4.0, 0.0));
        d = d.escala(a).mas(b).inverso();
        cc = b.mas(cc.inverso().escala(a));
        let del = cc.por(d);
        h = h.por(del);
        if (del.0 - 1.0).abs() + del.1.abs() < EPS {
            break;
        }
    }
    let h = Z(x, -x).por(h);
    let giro = Z((0.5 * fase2).cos(), (0.5 * fase2).sin());
    let cs = Z(0.5, 0.5).por(Z(1.0, 0.0).mas(giro.por(h).escala(-1.0)));
    (cs.0, cs.1)
}

/// I(w)/I₀ detrás de un borde recto: `w` en unidades de Fresnel, `w < 0` en
/// la sombra.
pub fn borde_recto(w: f64) -> f64 {
    let (c, s) = fresnel(w);
    0.5 * ((0.5 + c).powi(2) + (0.5 + s).powi(2))
}

/// dI/dw: (½ + C) C′ + (½ + S) S′, con C′ = cos(πw²/2) y S′ = sin(πw²/2).
/// Cero en cada franja, sin derivar nada numéricamente.
pub fn pendiente(w: f64) -> f64 {
    let (c, s) = fresnel(w);
    let fase = FRAC_PI_2 * w * w;
    (0.5 + c) * fase.cos() + (0.5 + s) * fase.sin()
}

/// La franja `k` del lado iluminado: `(w, I)` del extremo número `k`, desde
/// 0. Las pares son brillantes (I > 1) y las impares oscuras (I < 1).
///
/// Lejos del borde, I ≈ 1 + √2 sin(πw²/2 − π/4)/(πw), cuyos extremos caen
/// en w² = 2(k + ¾). Eso da el intervalo: entre w² = 2(k + ¼) y
/// w² = 2(k + 5/4) la pendiente cambia de signo exactamente una vez, y la
/// bisección la encuentra a la precisión de `f64`.
pub fn franja(k: usize) -> (f64, f64) {
    let kf = k as f64;
    let (mut lo, mut hi) = ((2.0 * (kf + 0.25)).sqrt(), (2.0 * (kf + 1.25)).sqrt());
    let signo_lo = pendiente(lo) > 0.0;
    for _ in 0..200 {
        let m = 0.5 * (lo + hi);
        if m <= lo || m >= hi {
            break; // ya no cabe otro f64 entre los dos
        }
        if (pendiente(m) > 0.0) == signo_lo {
            lo = m;
        } else {
            hi = m;
        }
    }
    let w = 0.5 * (lo + hi);
    (w, borde_recto(w))
}

/// La distancia `x` al borde de la sombra (m), en unidades de Fresnel, para
/// luz de longitud de onda `lambda` (m) y la pantalla a `z` (m) del borde:
/// w = x √(2/(λz)).
///
/// Con una fuente puntual a una distancia `a` del borde, la escala es
/// √(2a/(λz(a + z))); la onda plana es el límite a → ∞.
pub fn unidades_de_fresnel(x: f64, lambda: f64, z: f64) -> f64 {
    x * (2.0 / (lambda * z)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Simpson compuesto con un paso que sigue a la fase: otra forma de
    /// calcular lo mismo, lenta y sin nada en común con la serie ni con la
    /// fracción continua. Si las dos coinciden, ninguna se equivocó sola.
    fn simpson(x: f64) -> (f64, f64) {
        let n = ((PI * x * x / 0.005).ceil() as usize).max(256);
        let n = n + n % 2;
        let h = x / n as f64;
        let f = |t: f64| {
            let fase = FRAC_PI_2 * t * t;
            (fase.cos(), fase.sin())
        };
        let (c0, s0) = f(0.0);
        let (cn, sn) = f(x);
        let (mut c, mut s) = (c0 + cn, s0 + sn);
        for k in 1..n {
            let peso = if k % 2 == 1 { 4.0 } else { 2.0 };
            let (ck, sk) = f(k as f64 * h);
            c += peso * ck;
            s += peso * sk;
        }
        (c * h / 3.0, s * h / 3.0)
    }

    /// F1: la forma cerrada y la cuadratura coinciden, a los dos lados del
    /// corte y lejos de él.
    #[test]
    fn f1_la_serie_y_la_fraccion_coinciden_con_la_cuadratura() {
        for x in [0.1, 0.5, 1.0, 1.4999, 1.5, 1.5001, 2.0, 3.3, 5.0, 8.0] {
            let (c, s) = fresnel(x);
            let (cq, sq) = simpson(x);
            assert!((c - cq).abs() < 1e-11, "C({x}) = {c}, cuadratura {cq}");
            assert!((s - sq).abs() < 1e-11, "S({x}) = {s}, cuadratura {sq}");
        }
    }

    /// F2: el corte no deja escalón: a un lado y otro de 1.5 la función es
    /// continua al último dígito que importa.
    #[test]
    fn f2_el_corte_entre_las_dos_ramas_es_invisible() {
        let (cs, ss) = serie(CORTE);
        let (cf, sf) = fraccion_continua(CORTE);
        assert!((cs - cf).abs() < 1e-14 && (ss - sf).abs() < 1e-14, "{cs} {cf} · {ss} {sf}");
    }

    /// F3: impares, y de vuelta al origen.
    #[test]
    fn f3_son_impares() {
        assert_eq!(fresnel(0.0), (0.0, 0.0));
        for x in [0.3, 1.7, 4.2] {
            let (c, s) = fresnel(x);
            assert_eq!(fresnel(-x), (-c, -s));
        }
    }

    /// F4: lejos del origen siguen la asintótica, con un error del orden del
    /// término siguiente, 3/(π³x⁵).
    #[test]
    fn f4_lejos_siguen_la_asintotica() {
        for x in [6.0, 8.5, 10.0, 40.0] {
            let (c, s) = fresnel(x);
            let fase = FRAC_PI_2 * x * x;
            let (f, g) = (1.0 / (PI * x), 1.0 / (PI * PI * x * x * x));
            let ca = 0.5 + f * fase.sin() - g * fase.cos();
            let sa = 0.5 - f * fase.cos() - g * fase.sin();
            let cota = 3.0 / (PI.powi(3) * x.powi(5)) + 1e-12;
            assert!((c - ca).abs() < cota, "C({x}) = {c}, asintótica {ca}");
            assert!((s - sa).abs() < cota, "S({x}) = {s}, asintótica {sa}");
        }
    }

    /// B1: dentro de la sombra la luz sube hacia el borde sin oscilar, y
    /// lejos del borde la cola es 1/(2π²w²): nunca es cero.
    #[test]
    fn b1_la_sombra_no_oscila_ni_llega_a_cero() {
        let mut antes = borde_recto(-8.0);
        assert!(antes > 0.0);
        for k in 1..=800 {
            let w = -8.0 + k as f64 * 0.01;
            let i = borde_recto(w);
            assert!(i > antes, "en la sombra I sube hacia el borde: w = {w}");
            antes = i;
        }
        let w: f64 = -8.0;
        let cola = 1.0 / (2.0 * PI * PI * w * w);
        assert!((borde_recto(w) - cola).abs() / cola < 0.01);
    }

    /// B2: las franjas alternan, se juntan y se apagan hacia 1, y cada una
    /// cae donde la asintótica la pone: w² → 2(k + ¾).
    #[test]
    fn b2_las_franjas_alternan_se_juntan_y_se_apagan() {
        let fr: Vec<(f64, f64)> = (0..40).map(franja).collect();
        for (k, &(w, i)) in fr.iter().enumerate() {
            assert!(pendiente(w).abs() < 1e-12, "franja {k}: pendiente {}", pendiente(w));
            assert_eq!(i > 1.0, k % 2 == 0, "franja {k}: I = {i}");
            let amplitud = 2f64.sqrt() / (PI * w);
            assert!(((i - 1.0).abs() - amplitud).abs() < 0.2 * amplitud, "franja {k}");
        }
        for t in fr.windows(3) {
            assert!(t[1].0 - t[0].0 > t[2].0 - t[1].0, "se juntan");
            assert!((t[0].1 - 1.0).abs() > (t[2].1 - 1.0).abs(), "se apagan");
        }
        let (w39, _) = fr[39];
        assert!((w39 * w39 - 2.0 * (39.0 + 0.75)).abs() < 0.01, "w² = {}", w39 * w39);
    }

    /// B3: la pendiente es la derivada de la intensidad (contra diferencias
    /// centradas, que no comparten nada con la fórmula).
    #[test]
    fn b3_la_pendiente_es_la_derivada() {
        for w in [-2.0, -0.4, 0.0, 0.9, 1.6, 3.1] {
            let h = 1e-5;
            let num = (borde_recto(w + h) - borde_recto(w - h)) / (2.0 * h);
            assert!((num - pendiente(w)).abs() < 1e-8, "w = {w}: {num} vs {}", pendiente(w));
        }
    }

    /// U1: una franja de laboratorio: luz roja de 633 nm, pantalla a 1 m. La
    /// primera franja brillante cae a 0.68 mm del borde de la sombra.
    #[test]
    fn u1_la_primera_franja_de_un_laser_rojo_a_un_metro() {
        let (w0, _) = franja(0);
        let x = w0 / unidades_de_fresnel(1.0, 633e-9, 1.0);
        assert!((x * 1e3 - 0.685).abs() < 0.001, "x = {} mm", x * 1e3);
    }
}
