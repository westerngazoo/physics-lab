//! El jet: un número que carga consigo sus dos primeras derivadas.
//!
//! Hay tres maneras de sacarle la derivada a una fórmula que escribió
//! alguien más, y sólo una es honesta aquí:
//!
//! 1. **La secante**, `(f(t+Δt) − f(t))/Δt`. Es la definición, y por eso
//!    la lección la dibuja — pero como *aproximación*, con su error en
//!    pantalla. Usarla también como "la derivada verdadera" sería medir
//!    una regla con ella misma.
//! 2. **Derivación simbólica**: reescribir el árbol. Da una fórmula
//!    bonita, pero hay que simplificarla para que se pueda leer, y un
//!    simplificador es un programa entero con sus propios errores.
//! 3. **Diferenciación automática** — esto. Cada número viaja con sus
//!    derivadas, y cada operación aplica la regla de la cadena en el
//!    momento. No aproxima nada: el resultado tiene el mismo redondeo que
//!    el valor mismo, sea cual sea la fórmula.
//!
//! La idea es la de los números duales, `a + b·ε` con `ε² = 0`, llevada
//! un paso más lejos (`ε³ = 0`) para cargar también la segunda derivada:
//! la velocidad Y la aceleración de lo que el lector escriba.

use core::ops::{Add, Div, Mul, Neg, Sub};

/// Un valor y sus dos primeras derivadas respecto a UNA variable (en las
/// lecciones, el tiempo).
///
/// Es un paquete de datos, no un invariante: los campos son públicos
/// porque cualquier terna `(v, d1, d2)` es un jet válido.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Jet {
    /// `f(t)`
    pub v: f64,
    /// `f′(t)`
    pub d1: f64,
    /// `f″(t)`
    pub d2: f64,
}

impl Jet {
    /// Una constante: no cambia, así que sus derivadas son cero.
    #[must_use]
    pub const fn constante(c: f64) -> Self {
        Jet {
            v: c,
            d1: 0.0,
            d2: 0.0,
        }
    }

    /// LA variable, valiendo `t`: su derivada respecto a sí misma es 1.
    #[must_use]
    pub const fn variable(t: f64) -> Self {
        Jet {
            v: t,
            d1: 1.0,
            d2: 0.0,
        }
    }

    /// ¿Es una constante? Exactamente, no "casi".
    #[must_use]
    pub fn es_constante(self) -> bool {
        self.d1 == 0.0 && self.d2 == 0.0
    }

    /// La regla de la cadena de segundo orden: `h = φ(f)`, dados `φ`,
    /// `φ′` y `φ″` evaluadas en `f.v`.
    ///
    /// `h′ = φ′·f′` y `h″ = φ″·f′² + φ′·f″`.
    ///
    /// Una constante sale constante sin pasar por las derivadas: así
    /// `sqrt(0)` vale 0 con derivada 0, en vez de `∞·0 = NaN`.
    #[must_use]
    pub fn cadena(self, phi: f64, phi1: f64, phi2: f64) -> Self {
        if self.es_constante() {
            return Jet::constante(phi);
        }
        Jet {
            v: phi,
            d1: phi1 * self.d1,
            d2: phi2 * self.d1 * self.d1 + phi1 * self.d2,
        }
    }

    /// `f^n` con `n` una constante real.
    ///
    /// Los exponentes enteros van por multiplicación repetida (`powi`):
    /// son el caso común (`t²`, `t³`) y así `(−2)³` da `−8`, que `powf`
    /// también da, pero sin preguntarse si −2 tiene logaritmo.
    #[must_use]
    pub fn potencia_constante(self, n: f64) -> Self {
        let entero = n == n.trunc() && n.abs() <= 64.0;
        let pow = |x: f64, k: f64| {
            if entero {
                x.powi(k as i32)
            } else {
                x.powf(k)
            }
        };
        if n == 0.0 {
            return Jet::constante(1.0);
        }
        // El coeficiente de φ″ es n(n−1): cero para n = 1, y ahí no hay
        // que evaluar x⁻¹ — en x = 0 daría 0·∞ = NaN para la recta t¹.
        let phi2 = if n == 1.0 {
            0.0
        } else {
            n * (n - 1.0) * pow(self.v, n - 2.0)
        };
        self.cadena(pow(self.v, n), n * pow(self.v, n - 1.0), phi2)
    }

    /// `f^g` cuando el exponente también cambia: `e^(g·ln f)`.
    ///
    /// Sólo existe para `f > 0` en los reales, y aquí no se finge lo
    /// contrario: con base negativa el resultado es `NaN` y la curva
    /// tiene un hueco, que es exactamente lo que es.
    #[must_use]
    pub fn potencia(self, g: Jet) -> Self {
        if g.es_constante() {
            return self.potencia_constante(g.v);
        }
        (g * self.ln()).exp()
    }

    #[must_use]
    pub fn sin(self) -> Self {
        let (s, c) = self.v.sin_cos();
        self.cadena(s, c, -s)
    }
    #[must_use]
    pub fn cos(self) -> Self {
        let (s, c) = self.v.sin_cos();
        self.cadena(c, -s, -c)
    }
    #[must_use]
    pub fn tan(self) -> Self {
        let t = self.v.tan();
        let sec2 = 1.0 + t * t;
        self.cadena(t, sec2, 2.0 * t * sec2)
    }
    #[must_use]
    pub fn exp(self) -> Self {
        let e = self.v.exp();
        self.cadena(e, e, e)
    }
    /// Logaritmo natural.
    #[must_use]
    pub fn ln(self) -> Self {
        let x = self.v;
        self.cadena(x.ln(), 1.0 / x, -1.0 / (x * x))
    }
    /// Logaritmo base 10.
    #[must_use]
    pub fn log10(self) -> Self {
        let x = self.v;
        let k = core::f64::consts::LN_10;
        self.cadena(x.log10(), 1.0 / (x * k), -1.0 / (x * x * k))
    }
    #[must_use]
    pub fn sqrt(self) -> Self {
        let r = self.v.sqrt();
        self.cadena(r, 0.5 / r, -0.25 / (r * r * r))
    }
    /// Valor absoluto. En el cero no hay derivada (hay una esquina), y
    /// el jet lo dice con `NaN` en vez de inventar un signo.
    #[must_use]
    pub fn abs(self) -> Self {
        let x = self.v;
        let signo = if x > 0.0 {
            1.0
        } else if x < 0.0 {
            -1.0
        } else {
            f64::NAN
        };
        self.cadena(x.abs(), signo, 0.0)
    }
    #[must_use]
    pub fn sinh(self) -> Self {
        let (s, c) = (self.v.sinh(), self.v.cosh());
        self.cadena(s, c, s)
    }
    #[must_use]
    pub fn cosh(self) -> Self {
        let (s, c) = (self.v.sinh(), self.v.cosh());
        self.cadena(c, s, c)
    }
    #[must_use]
    pub fn tanh(self) -> Self {
        let t = self.v.tanh();
        let u = 1.0 - t * t;
        self.cadena(t, u, -2.0 * t * u)
    }
    #[must_use]
    pub fn atan(self) -> Self {
        let x = self.v;
        let u = 1.0 + x * x;
        self.cadena(x.atan(), 1.0 / u, -2.0 * x / (u * u))
    }
    #[must_use]
    pub fn asin(self) -> Self {
        let x = self.v;
        let u = 1.0 - x * x;
        self.cadena(x.asin(), 1.0 / u.sqrt(), x / (u * u.sqrt()))
    }
    #[must_use]
    pub fn acos(self) -> Self {
        let x = self.v;
        let u = 1.0 - x * x;
        self.cadena(x.acos(), -1.0 / u.sqrt(), -x / (u * u.sqrt()))
    }
}

impl Add for Jet {
    type Output = Jet;
    fn add(self, o: Jet) -> Jet {
        Jet {
            v: self.v + o.v,
            d1: self.d1 + o.d1,
            d2: self.d2 + o.d2,
        }
    }
}

impl Sub for Jet {
    type Output = Jet;
    fn sub(self, o: Jet) -> Jet {
        Jet {
            v: self.v - o.v,
            d1: self.d1 - o.d1,
            d2: self.d2 - o.d2,
        }
    }
}

impl Neg for Jet {
    type Output = Jet;
    fn neg(self) -> Jet {
        Jet {
            v: -self.v,
            d1: -self.d1,
            d2: -self.d2,
        }
    }
}

/// La regla del producto, de segundo orden:
/// `(fg)″ = f″g + 2f′g′ + fg″`.
impl Mul for Jet {
    type Output = Jet;
    fn mul(self, o: Jet) -> Jet {
        Jet {
            v: self.v * o.v,
            d1: self.d1 * o.v + self.v * o.d1,
            d2: self.d2 * o.v + 2.0 * self.d1 * o.d1 + self.v * o.d2,
        }
    }
}

/// El cociente, derivado de `q·g = f`: `q′ = (f′ − q·g′)/g` y
/// `q″ = (f″ − 2q′g′ − q·g″)/g`. Así no aparece nunca `g²` ni `g³`, que
/// es donde un cociente pierde precisión antes que el valor.
impl Div for Jet {
    type Output = Jet;
    fn div(self, g: Jet) -> Jet {
        let q = self.v / g.v;
        if g.es_constante() {
            return Jet {
                v: q,
                d1: self.d1 / g.v,
                d2: self.d2 / g.v,
            };
        }
        let q1 = (self.d1 - q * g.d1) / g.v;
        let q2 = (self.d2 - 2.0 * q1 * g.d1 - q * g.d2) / g.v;
        Jet { v: q, d1: q1, d2: q2 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cerca(a: f64, b: f64) -> bool {
        (a - b).abs() <= 1e-12 * (1.0 + b.abs())
    }

    fn igual(j: Jet, v: f64, d1: f64, d2: f64) -> bool {
        cerca(j.v, v) && cerca(j.d1, d1) && cerca(j.d2, d2)
    }

    /// Las reglas elementales, contra derivadas hechas a mano.
    #[test]
    fn reglas_contra_derivadas_a_mano() {
        for i in 1..40 {
            let t = i as f64 * 0.137;
            let x = Jet::variable(t);
            // t³: 3t², 6t
            assert!(igual(x * x * x, t * t * t, 3.0 * t * t, 6.0 * t));
            // 1/t: −1/t², 2/t³
            let uno = Jet::constante(1.0);
            assert!(igual(uno / x, 1.0 / t, -1.0 / (t * t), 2.0 / (t * t * t)));
            // sin(t²): 2t·cos t², 2cos t² − 4t² sin t²
            let s = (x * x).sin();
            let u = t * t;
            assert!(igual(s, u.sin(), 2.0 * t * u.cos(), 2.0 * u.cos() - 4.0 * u * u.sin()));
            // e^(−t)·cos(t): e^(−t)(−cos − sin), e^(−t)(2 sin t)
            let d = (-x).exp() * x.cos();
            let e = (-t).exp();
            assert!(igual(d, e * t.cos(), e * (-t.cos() - t.sin()), e * 2.0 * t.sin()));
            // t^t = e^(t ln t): t^t(ln t + 1), t^t((ln t + 1)² + 1/t)
            let tt = x.potencia(x);
            let k = t.ln() + 1.0;
            let p = t.powf(t);
            assert!(igual(tt, p, p * k, p * (k * k + 1.0 / t)));
        }
    }

    /// Una constante no tiene derivadas, ni siquiera donde φ′ es infinita.
    #[test]
    fn constantes_no_fabrican_nan() {
        let cero = Jet::constante(0.0);
        let r = cero.sqrt();
        assert_eq!((r.v, r.d1, r.d2), (0.0, 0.0, 0.0));
        let p = cero.potencia_constante(0.5);
        assert_eq!((p.v, p.d1, p.d2), (0.0, 0.0, 0.0));
    }

    /// Base negativa con exponente entero: existe, y vale lo que debe.
    #[test]
    fn base_negativa_exponente_entero() {
        let x = Jet::variable(-2.0);
        let c = x.potencia_constante(3.0);
        assert!(igual(c, -8.0, 12.0, -12.0));
        // …y con exponente fraccionario no existe en los reales: NaN.
        assert!(x.potencia_constante(0.5).v.is_nan());
    }
}
