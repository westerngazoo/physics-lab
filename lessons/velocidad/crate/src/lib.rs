//! La derivada es una velocidad — la primera lección que se ESCRIBE.
//!
//! El lector escribe `x(t)` como en Desmos, y un coche se mueve con eso.
//! Al lado, la gráfica de la posición: en el instante `t`, su recta
//! TANGENTE tiene una pendiente, y esa pendiente es la velocidad del
//! coche en ese instante — no "se parece": es el mismo número, y la
//! flecha amarilla sobre el coche mide eso.
//!
//! Tres ideas, una por vista:
//!
//! 1. **El coche** (con su cinta registradora: dónde estaba cada medio
//!    segundo). Puntos apretados, coche lento; puntos separados, coche
//!    rápido. La cinta ES la derivada antes de tener nombre.
//! 2. **La posición x(t)**, con la tangente en `t` y la SECANTE entre dos
//!    instantes separados por `Δt`. La secante es la definición:
//!    `(x(t+Δt) − x(t))/Δt`. Encoge `Δt` y se acuesta sobre la tangente.
//! 3. **La velocidad v(t) = x′(t)**, calculada EXACTA (diferenciación
//!    automática, ver `formulas::Jet`), con la secante como un punto rojo
//!    a su lado: la distancia entre los dos es el error, en pantalla.
//!
//! Y si el lector escribe también `y(t)`, el coche sale al plano: la
//! velocidad se vuelve un vector tangente al camino, la cuerda entre
//! `r(t)` y `r(t+Δt)` dividida entre `Δt` tiende a él, y la aceleración
//! se parte en dos — `v·a` cambia la rapidez, `v∧a` dobla el camino —
//! con la curvatura `κ = |v∧a|/|v|³` como producto cuña de garust.
//!
//! La ley de la casa: la aproximación (la secante) pone su error en
//! pantalla, con un control que la rompe a propósito. Aquí el control es
//! `Δt`, y se puede bajar hasta `10⁻¹²`: por debajo de `~10⁻⁸` la secante
//! EMPEORA, porque `x(t+Δt) − x(t)` resta dos números casi iguales y el
//! redondeo se come la diferencia. La derivada exacta no tiene ese piso.
//!
//! Afirmaciones (cada una una prueba, abajo):
//!
//! - **D1** las derivadas son exactas (contra derivadas hechas a mano);
//! - **D2** la secante hacia adelante converge con orden 1, la centrada
//!   con orden 2;
//! - **D3** por debajo de `Δt ≈ 10⁻⁸` la secante empeora (el piso del
//!   redondeo);
//! - **D4** la tangente DIBUJADA tiene pendiente `v(t)`, medida sobre el
//!   dibujo mismo;
//! - **D5** el movimiento de arranque se detiene en `t = 0` y `t = 10`,
//!   y va más rápido justo en su punto de inflexión, donde `a = 0`;
//! - **D6** en el círculo, `v ⊥ r`, `|a| = |v|²/R` y `κ = 1/R`;
//! - **D7** nada que el lector escriba tumba la página: ni pánico, ni
//!   buffer desbordado, ni una coordenada que no sea un número.

use formulas::{compila, Formula, Jet};
use garust::Vga2;
use lessons_common::{Diagnosticos, Prims, Readouts};

// ---- el manifiesto, por índice ------------------------------------------

/// Parámetros, en el orden de `lesson.json` (una prueba lo comprueba).
pub const P_T: usize = 0;
/// `log₁₀ Δt`: el deslizador recorre órdenes de magnitud.
pub const P_K: usize = 1;
/// 0 hacia adelante · 1 centrada.
pub const P_CENTRADA: usize = 2;
pub const P_A: usize = 3;
pub const P_B: usize = 4;
pub const P_C: usize = 5;
/// Los nombres de los parámetros, en el orden de arriba.
pub const PARAMS: [&str; 6] = ["t", "k", "centrada", "a", "b", "c"];

/// Lo que una fórmula puede nombrar…
pub const VARIABLES: [&str; 4] = ["t", "a", "b", "c"];
/// …y qué parámetro del manifiesto es cada nombre.
const PARAM_DE: [usize; 4] = [P_T, P_A, P_B, P_C];

/// La fórmula con la que abre la página (la misma que `lesson.json`).
pub const ARRANQUE: &str = "3t² − 0.2t³";

/// La ventana de tiempo, fija como una pista: de 0 a 10 s.
pub const T0: f64 = 0.0;
pub const T1: f64 = 10.0;
/// Muestras por curva.
pub const MUESTRAS: usize = 200;
/// Cada cuánto deja un punto la cinta registradora, s.
pub const CINTA: f64 = 0.5;

// Estilos: índices en `styles`.
const EJES: usize = 0;
const CURVA: usize = 1;
const TANGENTE: usize = 2;
const SECANTE: usize = 3;
const PUNTO: usize = 4;
const VEL: usize = 5;
const ACEL: usize = 6;
const CARRO: usize = 7;
const RUEDA: usize = 8;
const TENUE: usize = 9;
const CURVA_Y: usize = 10;
const MARCA: usize = 11;
const MARCA_GRANDE: usize = 12;
const ROTULO: usize = 13;
const ROTULO_CHICO: usize = 14;
const TRIANGULO: usize = 15;
const RAPIDEZ: usize = 16;
const CINTA_PUNTO: usize = 17;

// Rótulos: índices en `labels`.
const R_V: usize = 0;
const R_A: usize = 1;
const R_DT: usize = 2;
const R_DX: usize = 3;
const R_T: usize = 4;
const R_X: usize = 5;
const R_XY: usize = 6;
const R_VEL: usize = 7;
const R_VXY: usize = 8;
const R_CUERDA: usize = 9;
const R_CINTA: usize = 10;

// ---- el modelo -------------------------------------------------------------

/// Un movimiento escrito por el lector: `x(t)` y, si la hay, `y(t)`.
pub struct Movimiento {
    x: Formula,
    y: Option<Formula>,
    /// Valores de `t, a, b, c` — el orden de [`VARIABLES`].
    vals: [f64; 4],
}

impl Movimiento {
    #[must_use]
    pub fn nuevo(x: Formula, y: Option<Formula>, coef: [f64; 3]) -> Self {
        Movimiento {
            x,
            y,
            vals: [0.0, coef[0], coef[1], coef[2]],
        }
    }

    /// ¿Se mueve en el plano, o sobre una recta?
    #[must_use]
    pub fn plano(&self) -> bool {
        self.y.is_some()
    }

    /// `x` y `y` en `t`, cada una con su velocidad y su aceleración.
    /// En una dimensión, `y` es la constante cero.
    #[must_use]
    pub fn en(&self, t: f64) -> (Jet, Jet) {
        let mut v = self.vals;
        v[0] = t;
        let y = self
            .y
            .as_ref()
            .map_or(Jet::constante(0.0), |f| f.jet(&v, 0));
        (self.x.jet(&v, 0), y)
    }

    /// El movimiento muestreado en toda la ventana: `(t, x, y)`.
    #[must_use]
    pub fn muestras(&self) -> Vec<(f64, Jet, Jet)> {
        (0..=MUESTRAS)
            .map(|i| {
                let t = T0 + (T1 - T0) * i as f64 / MUESTRAS as f64;
                let (x, y) = self.en(t);
                (t, x, y)
            })
            .collect()
    }
}

/// Los dos instantes que une la secante: `[t, t+Δt]` hacia adelante,
/// `[t−Δt/2, t+Δt/2]` centrada.
#[must_use]
pub fn instantes(t: f64, dt: f64, centrada: bool) -> (f64, f64) {
    if centrada {
        (t - dt / 2.0, t + dt / 2.0)
    } else {
        (t, t + dt)
    }
}

/// La pendiente de la secante — la definición de derivada sin el límite
/// — para `x` y para `y`.
///
/// Se divide entre la distancia REAL entre los dos instantes, `tb − ta`,
/// no entre el `Δt` pedido: es la pendiente de la recta que se dibuja.
#[must_use]
pub fn secante(m: &Movimiento, t: f64, dt: f64, centrada: bool) -> (f64, f64) {
    let (ta, tb) = instantes(t, dt, centrada);
    let ((xa, ya), (xb, yb)) = (m.en(ta), m.en(tb));
    let h = tb - ta;
    ((xb.v - xa.v) / h, (yb.v - ya.v) / h)
}

/// La curvatura del camino, `κ = |v∧a| / |v|³`.
///
/// `v∧a` es el área orientada del paralelogramo que forman la velocidad y
/// la aceleración: un bivector, múltiplo de `e₁₂`. Si la aceleración es
/// paralela a la velocidad, el área es cero y el camino no se dobla —
/// sólo cambia la rapidez. Toda la curvatura vive en la parte que NO es
/// paralela, y el producto cuña es exactamente esa parte.
#[must_use]
pub fn curvatura(v: (f64, f64), a: (f64, f64)) -> f64 {
    let vv = Vga2::basis(1) * v.0 + Vga2::basis(2) * v.1;
    let aa = Vga2::basis(1) * a.0 + Vga2::basis(2) * a.1;
    let area = vv.wedge(&aa).coeffs[3];
    let rapidez = v.0.hypot(v.1);
    area.abs() / (rapidez * rapidez * rapidez)
}

// ---- la escala: de unidades del mundo a la caja de la vista ---------------

/// Un intervalo de valores, mapeado a `[0, 1]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rango {
    pub lo: f64,
    pub hi: f64,
}

impl Rango {
    /// El rango de las muestras finitas, con margen.
    ///
    /// Si las colas son asíntotas (`tan`, `1/t`), el mínimo y el máximo
    /// son basura de ±10¹⁶ que aplastaría la curva contra una línea; en
    /// ese caso se usa lo que la curva hace el 96 % del tiempo, y lo que
    /// sale de la caja simplemente no se dibuja — como en una calculadora.
    #[must_use]
    pub fn de(muestras: impl IntoIterator<Item = f64>) -> Rango {
        let mut v: Vec<f64> = muestras.into_iter().filter(|x| x.is_finite()).collect();
        if v.is_empty() {
            return Rango { lo: -1.0, hi: 1.0 };
        }
        v.sort_by(f64::total_cmp);
        let n = v.len();
        let (mut lo, mut hi) = (v[0], v[n - 1]);
        let (p02, p98) = (v[n * 2 / 100], v[(n * 98 / 100).min(n - 1)]);
        if p98 > p02 && hi - lo > 4.0 * (p98 - p02) {
            (lo, hi) = (p02, p98);
        }
        let ancho = hi - lo;
        if ancho <= 1e-9 * lo.abs().max(1.0) {
            let pad = 0.1 * lo.abs().max(1.0);
            return Rango {
                lo: lo - pad,
                hi: hi + pad,
            };
        }
        Rango {
            lo: lo - 0.08 * ancho,
            hi: hi + 0.08 * ancho,
        }
    }
    #[must_use]
    pub fn a01(&self, x: f64) -> f64 {
        (x - self.lo) / (self.hi - self.lo)
    }
}

/// Marcas "redondas" (1, 2 o 5 × 10ᵏ) dentro de `r`, y cuántos decimales
/// piden para escribirse sin mentir ni sobrar.
#[must_use]
pub fn marcas(r: Rango, cuantas: f64) -> (Vec<f64>, usize) {
    let bruto = (r.hi - r.lo) / cuantas;
    if !bruto.is_finite() || bruto <= 0.0 {
        return (Vec::new(), 0);
    }
    let mag = 10f64.powf(bruto.log10().floor());
    let paso = mag
        * match bruto / mag {
            q if q < 1.5 => 1.0,
            q if q < 3.5 => 2.0,
            q if q < 7.5 => 5.0,
            _ => 10.0,
        };
    let mut out = Vec::new();
    let mut k = (r.lo / paso).ceil();
    while k * paso <= r.hi && out.len() < 16 {
        let v = k * paso;
        out.push(if v.abs() < paso * 1e-9 { 0.0 } else { v });
        k += 1.0;
    }
    let dec = (-paso.log10().floor()).clamp(0.0, 6.0) as usize;
    (out, dec)
}

// ---- dibujo ------------------------------------------------------------------

/// Un rectángulo del mundo de una vista: lo que cae fuera no se dibuja.
#[derive(Clone, Copy)]
struct Caja {
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
}

impl Caja {
    fn tiene(&self, (x, y): (f64, f64)) -> bool {
        (self.x0..=self.x1).contains(&x) && (self.y0..=self.y1).contains(&y)
    }
}

/// Una polilínea que se corta donde la curva no existe (`NaN`, `∞`) o se
/// sale de la caja: los huecos se ven como huecos, no como rayas.
fn traza(out: &mut Prims, estilo: usize, caja: Caja, pts: impl Iterator<Item = (f64, f64)>) {
    let mut tramo: Vec<(f64, f64)> = Vec::with_capacity(MUESTRAS + 1);
    for p in pts {
        if caja.tiene(p) {
            tramo.push(p);
        } else {
            if tramo.len() >= 2 {
                out.polyline(tramo.drain(..), estilo);
            }
            tramo.clear();
        }
    }
    if tramo.len() >= 2 {
        out.polyline(tramo, estilo);
    }
}

/// Un segmento, sólo si sus dos puntas son números.
fn segmento(out: &mut Prims, a: (f64, f64), b: (f64, f64), estilo: usize) {
    if [a.0, a.1, b.0, b.1].iter().all(|v| v.is_finite()) {
        out.segment(a.0, a.1, b.0, b.1, estilo);
    }
}

/// Una flecha, sólo si mide algo: una flecha de largo cero no tiene
/// dirección y su punta saldría hacia cualquier lado. Dice si la dibujó,
/// para que su rótulo no quede flotando sobre nada.
fn flecha(out: &mut Prims, a: (f64, f64), b: (f64, f64), estilo: usize) -> bool {
    let largo = (b.0 - a.0).hypot(b.1 - a.1);
    let dibuja = largo.is_finite() && largo > 1e-4 && a.0.is_finite() && a.1.is_finite();
    if dibuja {
        out.arrow(a.0, a.1, b.0, b.1, estilo);
    }
    dibuja
}

fn punto(out: &mut Prims, p: (f64, f64), estilo: usize, caja: Caja) {
    if caja.tiene(p) {
        out.point(p.0, p.1, estilo);
    }
}

fn rotulo(out: &mut Prims, p: (f64, f64), idx: usize, estilo: usize) {
    if p.0.is_finite() && p.1.is_finite() {
        out.label(p.0, p.1, idx, estilo);
    }
}

/// La caja donde viven las gráficas (vistas 1 y 2): `t` en `[0, 1]` a lo
/// ancho y el valor en `[0, 1]` a lo alto, con margen para números.
const CAJA_GRAFICA: Caja = Caja {
    x0: -0.01,
    x1: 1.01,
    y0: -0.02,
    y1: 1.02,
};

/// Una gráfica contra el tiempo.
struct Grafica {
    r: Rango,
}

impl Grafica {
    fn p(&self, t: f64, v: f64) -> (f64, f64) {
        ((t - T0) / (T1 - T0), self.r.a01(v))
    }

    fn ejes(&self, out: &mut Prims, rotulo_y: usize) {
        segmento(out, (0.0, 0.0), (1.0, 0.0), EJES);
        segmento(out, (0.0, 0.0), (0.0, 1.0), EJES);
        let cero = self.r.a01(0.0);
        if cero > 0.02 && cero < 1.0 {
            segmento(out, (0.0, cero), (1.0, cero), TENUE);
        }
        let (tt, _) = marcas(Rango { lo: T0, hi: T1 }, 5.0);
        for t in tt {
            let x = (t - T0) / (T1 - T0);
            segmento(out, (x, 0.0), (x, -0.025), EJES);
            out.numero(x, -0.075, t, 0, MARCA);
        }
        let (vs, dec) = marcas(self.r, 4.0);
        for v in vs {
            let y = self.r.a01(v);
            if (0.0..=1.0).contains(&y) {
                segmento(out, (0.0, y), (-0.02, y), EJES);
                out.numero(-0.075, y, v, dec, MARCA);
            }
        }
        rotulo(out, (1.035, -0.075), R_T, ROTULO_CHICO);
        rotulo(out, (0.06, 1.04), rotulo_y, ROTULO_CHICO);
    }

    fn curva(&self, out: &mut Prims, estilo: usize, pts: impl Iterator<Item = (f64, f64)>) {
        traza(out, estilo, CAJA_GRAFICA, pts.map(|(t, v)| self.p(t, v)));
    }

    /// La recta de pendiente `m` (en unidades del mundo) por `(t, v)`,
    /// con el mismo largo en pantalla sea cual sea la pendiente.
    fn recta(&self, out: &mut Prims, t: f64, v: f64, m: f64, largo: f64, estilo: usize) {
        let c = self.p(t, v);
        let d = (1.0 / (T1 - T0), m / (self.r.hi - self.r.lo));
        let n = d.0.hypot(d.1);
        if !n.is_finite() || n == 0.0 || !CAJA_GRAFICA.tiene(c) {
            return;
        }
        let (ux, uy) = (d.0 / n * largo, d.1 / n * largo);
        segmento(out, (c.0 - ux, c.1 - uy), (c.0 + ux, c.1 + uy), estilo);
    }
}

// ---- la escena ---------------------------------------------------------------

/// Todo lo que un cuadro necesita, calculado una vez.
struct Escena<'m> {
    m: &'m Movimiento,
    t: f64,
    dt: f64,
    centrada: bool,
    muestras: Vec<(f64, Jet, Jet)>,
    /// El instante `t`: posición, velocidad, aceleración.
    ahora: (Jet, Jet),
}

impl Escena<'_> {
    fn vmax(&self) -> f64 {
        maximo(self.muestras.iter().map(|(_, x, y)| x.d1.hypot(y.d1)))
    }
    fn amax(&self) -> f64 {
        maximo(self.muestras.iter().map(|(_, x, y)| x.d2.hypot(y.d2)))
    }
}

/// El máximo de lo finito; 1 si no hay nada que medir (así una escala
/// dividida entre esto nunca es infinita).
fn maximo(it: impl Iterator<Item = f64>) -> f64 {
    let m = it.filter(|v| v.is_finite()).fold(0.0f64, f64::max);
    if m > 0.0 {
        m
    } else {
        1.0
    }
}

/// Largo en pantalla de la flecha de la velocidad más grande de la
/// ventana, y de la aceleración más grande. Cada una a escala CONSIGO
/// MISMA: se pueden comparar dos velocidades, no una velocidad con una
/// aceleración (son unidades distintas, y el dibujo no lo esconde).
const LARGO_V: f64 = 0.2;
const LARGO_A: f64 = 0.13;
/// En el plano las flechas salen del camino en cualquier dirección, y
/// tienen que caber arriba y abajo: son más cortas.
const LARGO_V_PLANO: f64 = 0.1;
const LARGO_A_PLANO: f64 = 0.08;

/// La dirección de `v`, si tiene una.
fn unitario((x, y): (f64, f64)) -> Option<(f64, f64)> {
    let n = x.hypot(y);
    (n.is_finite() && n > 1e-12).then(|| (x / n, y / n))
}

/// Vista 0 en una dimensión: la carretera, el coche, la cinta.
fn coche(e: &Escena, out: &mut Prims) {
    out.view(0);
    let rx = Rango::de(e.muestras.iter().map(|(_, x, _)| x.v));
    let caja = Caja {
        x0: -0.05,
        x1: 1.05,
        y0: -0.2,
        y1: 0.5,
    };
    // La carretera y sus marcas en metros.
    segmento(out, (-0.05, 0.0), (1.05, 0.0), EJES);
    let (xs, dec) = marcas(rx, 6.0);
    for x in xs {
        let u = rx.a01(x);
        segmento(out, (u, 0.0), (u, -0.02), EJES);
        out.numero(u, -0.055, x, dec, MARCA_GRANDE);
    }
    rotulo(out, (1.03, -0.055), R_X, ROTULO);
    // La cinta registradora: una tira bajo la carretera, y un punto donde
    // estaba el coche cada CINTA segundos.
    segmento(out, (-0.05, -0.1), (1.05, -0.1), TENUE);
    rotulo(out, (0.035, -0.127), R_CINTA, ROTULO_CHICO);
    let mut k = 0.0;
    while T0 + k * CINTA <= e.t + 1e-12 && k < 64.0 {
        let (x, _) = e.m.en(T0 + k * CINTA);
        punto(out, (rx.a01(x.v), -0.1), CINTA_PUNTO, caja);
        k += 1.0;
    }
    let (x, _) = e.ahora;
    let cx = rx.a01(x.v);
    if !cx.is_finite() {
        return;
    }
    // Las ruedas son rotores: el ángulo que giran es el camino entre el
    // radio. Ruedan sin patinar sobre la carretera DIBUJADA.
    const R: f64 = 0.022;
    let x0 = rx.a01(e.m.en(T0).0.v);
    let giro = -(cx - x0) / R;
    for dx in [-0.036, 0.036] {
        let c = (cx + dx, R);
        out.curve(0.0, std::f64::consts::TAU, 20, RUEDA, |s| {
            (c.0 + R * s.cos(), c.1 + R * s.sin())
        });
        for j in 0..2 {
            let s = giro + j as f64 * std::f64::consts::TAU / 4.0;
            segmento(out, (c.0 - R * s.cos(), c.1 - R * s.sin()), (c.0 + R * s.cos(), c.1 + R * s.sin()), RUEDA);
        }
    }
    let cuerpo = [
        (cx - 0.066, R * 0.8),
        (cx + 0.066, R * 0.8),
        (cx + 0.066, R + 0.034),
        (cx + 0.044, R + 0.034),
        (cx + 0.026, R + 0.068),
        (cx - 0.032, R + 0.068),
        (cx - 0.046, R + 0.034),
        (cx - 0.066, R + 0.034),
        (cx - 0.066, R * 0.8),
    ];
    out.polyline(cuerpo, CARRO);
    // Velocidad y aceleración, cada una a su propia escala.
    let (sv, sa) = (LARGO_V / e.vmax(), LARGO_A / e.amax());
    let yv = R + 0.14;
    if flecha(out, (cx, yv), (cx + sv * x.d1, yv), VEL) {
        rotulo(out, (cx + sv * x.d1 + 0.03 * x.d1.signum(), yv + 0.025), R_V, ROTULO);
    }
    let ya = yv + 0.09;
    if flecha(out, (cx, ya), (cx + sa * x.d2, ya), ACEL) {
        rotulo(out, (cx + sa * x.d2 + 0.03 * x.d2.signum(), ya + 0.025), R_A, ROTULO);
    }
}

/// Vista 0 en el plano: el camino, la cinta, la velocidad tangente, la
/// aceleración, y la cuerda que se vuelve velocidad cuando `Δt → 0`.
fn plano(e: &Escena, out: &mut Prims) {
    out.view(0);
    let finitos = || {
        e.muestras
            .iter()
            .filter(|(_, x, y)| x.v.is_finite() && y.v.is_finite())
    };
    let (mut x0, mut x1, mut y0, mut y1) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
    for (_, x, y) in finitos() {
        (x0, x1, y0, y1) = (x0.min(x.v), x1.max(x.v), y0.min(y.v), y1.max(y.v));
    }
    if x0 > x1 {
        (x0, x1, y0, y1) = (-1.0, 1.0, -1.0, 1.0);
    }
    // Un solo factor para los dos ejes: en el plano, un metro mide lo
    // mismo de ancho que de alto, o los ángulos mienten. El camino cabe
    // en una caja más chica que la vista, para que las flechas que salen
    // de su borde también quepan.
    let (wx, wy) = ((x1 - x0) * 1.05, (y1 - y0) * 1.05);
    let s = match (wx > 1e-12, wy > 1e-12) {
        (true, true) => (0.8 / wx).min(0.34 / wy),
        (true, false) => 0.8 / wx,
        (false, true) => 0.34 / wy,
        (false, false) => 1.0,
    };
    let (cx, cy) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
    let map = |x: f64, y: f64| (0.5 + (x - cx) * s, 0.13 + (y - cy) * s);
    let caja = Caja {
        x0: -0.06,
        x1: 1.06,
        y0: -0.14,
        y1: 0.42,
    };
    // Los ejes del plano, si pasan por la vista.
    let o = map(0.0, 0.0);
    if caja.tiene((o.0, 0.15)) {
        segmento(out, (o.0, -0.12), (o.0, 0.41), TENUE);
    }
    if caja.tiene((0.5, o.1)) {
        segmento(out, (-0.05, o.1), (1.05, o.1), TENUE);
    }
    traza(out, TENUE, caja, e.muestras.iter().map(|(_, x, y)| map(x.v, y.v)));
    traza(
        out,
        CURVA,
        caja,
        e.muestras
            .iter()
            .filter(|(t, _, _)| *t <= e.t)
            .map(|(_, x, y)| map(x.v, y.v))
            .chain(core::iter::once(map(e.ahora.0.v, e.ahora.1.v))),
    );
    let mut k = 0.0;
    while T0 + k * CINTA <= e.t + 1e-12 && k < 64.0 {
        let (x, y) = e.m.en(T0 + k * CINTA);
        punto(out, map(x.v, y.v), CINTA_PUNTO, caja);
        k += 1.0;
    }
    let (x, y) = e.ahora;
    let r = map(x.v, y.v);
    if !caja.tiene(r) {
        return;
    }
    // La cuerda: de r(ta) a r(tb), sobre el camino.
    let (ta, tb) = instantes(e.t, e.dt, e.centrada);
    let ((xa, ya), (xb, yb)) = (e.m.en(ta), e.m.en(tb));
    segmento(out, map(xa.v, ya.v), map(xb.v, yb.v), SECANTE);
    // El coche, como una punta de flecha que mira hacia donde va.
    let (ux, uy) = unitario((x.d1, y.d1)).or_else(|| unitario((x.d2, y.d2))).unwrap_or((1.0, 0.0));
    let (nx, ny) = (-uy, ux);
    out.polyline(
        [
            (r.0 + 0.03 * ux, r.1 + 0.03 * uy),
            (r.0 - 0.018 * ux + 0.016 * nx, r.1 - 0.018 * uy + 0.016 * ny),
            (r.0 - 0.018 * ux - 0.016 * nx, r.1 - 0.018 * uy - 0.016 * ny),
            (r.0 + 0.03 * ux, r.1 + 0.03 * uy),
        ],
        CARRO,
    );
    // Velocidad, y la cuerda entre Δt A LA MISMA ESCALA: comparables.
    let (sv, sa) = (LARGO_V_PLANO / e.vmax(), LARGO_A_PLANO / e.amax());
    let (mx, my) = secante(e.m, e.t, e.dt, e.centrada);
    let cuerda = flecha(out, r, (r.0 + sv * mx, r.1 + sv * my), SECANTE);
    if flecha(out, r, (r.0 + sv * x.d1, r.1 + sv * y.d1), VEL) {
        rotulo(out, (r.0 + sv * x.d1 * 1.12, r.1 + sv * y.d1 * 1.12 + 0.02), R_V, ROTULO);
    }
    if flecha(out, r, (r.0 + sa * x.d2, r.1 + sa * y.d2), ACEL) {
        rotulo(out, (r.0 + sa * x.d2 * 1.15, r.1 + sa * y.d2 * 1.15 + 0.02), R_A, ROTULO);
    }
    // El rótulo de la cuerda va a un costado de su mitad, no en su punta:
    // cuando Δt es chico la cuerda y v son casi paralelas, y sus puntas
    // (con sus rótulos) se enciman.
    if let (true, Some((cx, cy))) = (cuerda && e.dt > 0.3, unitario((mx, my))) {
        let medio = (r.0 + sv * mx * 0.5, r.1 + sv * my * 0.5);
        rotulo(out, (medio.0 + 0.05 * cy, medio.1 - 0.05 * cx), R_CUERDA, ROTULO);
    }
}

/// Vista 1: la posición contra el tiempo, con tangente y secante.
fn posicion(e: &Escena, out: &mut Prims) {
    out.view(1);
    let (x, y) = e.ahora;
    let r = Rango::de(
        e.muestras
            .iter()
            .flat_map(|(_, x, y)| [x.v, if e.m.plano() { y.v } else { f64::NAN }]),
    );
    let g = Grafica { r };
    g.ejes(out, if e.m.plano() { R_XY } else { R_X });
    g.curva(out, CURVA, e.muestras.iter().map(|(t, x, _)| (*t, x.v)));
    if e.m.plano() {
        g.curva(out, CURVA_Y, e.muestras.iter().map(|(t, _, y)| (*t, y.v)));
        g.recta(out, e.t, y.v, y.d1, 0.14, TANGENTE);
        punto(out, g.p(e.t, y.v), PUNTO, CAJA_GRAFICA);
    }
    // La secante, prolongada más allá de sus dos puntos, y su escalón
    // Δt–Δx: la pendiente de ESTA recta es el cociente de los catetos.
    let (ta, tb) = instantes(e.t, e.dt, e.centrada);
    let (xa, xb) = (e.m.en(ta).0.v, e.m.en(tb).0.v);
    let (a, b) = (g.p(ta, xa), g.p(tb, xb));
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let n = dx.hypot(dy);
    if n.is_finite() && n > 1e-9 {
        let (ux, uy) = (dx / n * 0.08, dy / n * 0.08);
        segmento(out, (a.0 - ux, a.1 - uy), (b.0 + ux, b.1 + uy), SECANTE);
    }
    if dx > 0.04 {
        segmento(out, a, (b.0, a.1), TRIANGULO);
        segmento(out, (b.0, a.1), b, TRIANGULO);
        rotulo(out, ((a.0 + b.0) / 2.0, a.1 - 0.045), R_DT, ROTULO_CHICO);
        rotulo(out, (b.0 + 0.045, (a.1 + b.1) / 2.0), R_DX, ROTULO_CHICO);
    }
    punto(out, a, SECANTE, CAJA_GRAFICA);
    punto(out, b, SECANTE, CAJA_GRAFICA);
    g.recta(out, e.t, x.v, x.d1, 0.16, TANGENTE);
    punto(out, g.p(e.t, x.v), PUNTO, CAJA_GRAFICA);
}

/// Vista 2: la velocidad contra el tiempo — exacta — con la secante como
/// un punto rojo y su error como la distancia entre los dos.
fn velocidad(e: &Escena, out: &mut Prims) {
    out.view(2);
    let (x, y) = e.ahora;
    let plano = e.m.plano();
    let r = Rango::de(e.muestras.iter().flat_map(|(_, x, y)| {
        if plano {
            [x.d1, y.d1, x.d1.hypot(y.d1)]
        } else {
            [x.d1, f64::NAN, f64::NAN]
        }
    }));
    let g = Grafica { r };
    g.ejes(out, if plano { R_VXY } else { R_VEL });
    g.curva(out, CURVA, e.muestras.iter().map(|(t, x, _)| (*t, x.d1)));
    if plano {
        g.curva(out, CURVA_Y, e.muestras.iter().map(|(t, _, y)| (*t, y.d1)));
        g.curva(out, RAPIDEZ, e.muestras.iter().map(|(t, x, y)| (*t, x.d1.hypot(y.d1))));
        punto(out, g.p(e.t, y.d1), PUNTO, CAJA_GRAFICA);
    }
    // La pendiente de ESTA gráfica es la aceleración.
    g.recta(out, e.t, x.d1, x.d2, 0.12, TANGENTE);
    let (m, _) = secante(e.m, e.t, e.dt, e.centrada);
    let (pv, ps) = (g.p(e.t, x.d1), g.p(e.t, m));
    if CAJA_GRAFICA.tiene(pv) && CAJA_GRAFICA.tiene(ps) {
        segmento(out, pv, ps, SECANTE);
    }
    punto(out, ps, SECANTE, CAJA_GRAFICA);
    punto(out, pv, PUNTO, CAJA_GRAFICA);
}

fn escena(m: &Movimiento, t: f64, dt: f64, centrada: bool, out: &mut Prims, read: &mut Readouts) {
    let e = Escena {
        m,
        t,
        dt,
        centrada,
        muestras: m.muestras(),
        ahora: m.en(t),
    };
    if m.plano() {
        plano(&e, out);
    } else {
        coche(&e, out);
    }
    posicion(&e, out);
    velocidad(&e, out);

    let (x, y) = e.ahora;
    let (sx, _) = secante(m, t, dt, centrada);
    let (ta, tb) = instantes(t, dt, centrada);
    read.set(0, x.v);
    read.set(1, x.d1);
    read.set(2, sx);
    read.set(3, (sx - x.d1).abs());
    read.set(4, tb - ta);
    read.set(5, x.d2);
    read.set(6, x.d1.hypot(y.d1));
    read.set(7, if m.plano() { curvatura((x.d1, y.d1), (x.d2, y.d2)) } else { f64::NAN });
}

/// Una fórmula que no se pudo leer: la página ya dice dónde; aquí sólo
/// quedan los marcos vacíos, sin inventar un movimiento que nadie pidió.
fn vacia(out: &mut Prims, read: &mut Readouts) {
    out.view(0);
    segmento(out, (-0.05, 0.0), (1.05, 0.0), EJES);
    for v in [1, 2] {
        out.view(v);
        segmento(out, (0.0, 0.0), (1.0, 0.0), EJES);
        segmento(out, (0.0, 0.0), (0.0, 1.0), EJES);
    }
    for s in 0..8 {
        read.set(s, f64::NAN);
    }
}

/// La máscara de VARIABLES, traducida a parámetros del manifiesto.
fn mascara_de_parametros(m: u64) -> u64 {
    PARAM_DE
        .iter()
        .enumerate()
        .filter(|(i, _)| m & (1 << i) != 0)
        .fold(0, |acc, (_, &p)| acc | (1 << p))
}

fn lee(fuente: &str, linea: usize, diag: &mut Diagnosticos) -> Option<Formula> {
    match compila(fuente, &VARIABLES) {
        Ok(f) => {
            diag.ok(linea, mascara_de_parametros(f.mascara()));
            Some(f)
        }
        Err(e) => {
            diag.error(linea, e.codigo as u8, e.columna);
            None
        }
    }
}

// ---- la frontera wasm ---------------------------------------------------------

/// Params (orden del manifiesto): `[t, k, centrada, a, b, c]`.
/// Texto: `x(t)` en el primer renglón, `y(t)` (opcional) en el segundo.
fn draw(p: &[f64], texto: &str, out: &mut Prims, read: &mut Readouts, diag: &mut Diagnosticos) {
    let param = |i: usize| p.get(i).copied().unwrap_or(0.0);
    let (t, dt, centrada) = (param(P_T), 10f64.powf(param(P_K)), param(P_CENTRADA) >= 0.5);
    let coef = [param(P_A), param(P_B), param(P_C)];
    let mut lineas = texto.split('\n');
    let fx = lineas.next().unwrap_or("");
    let fy = lineas.next().unwrap_or("");
    let x = lee(fx, 0, diag);
    let y = if fy.trim().is_empty() {
        diag.ok(1, 0);
        Some(None)
    } else {
        lee(fy, 1, diag).map(Some)
    };
    match (x, y) {
        (Some(x), Some(y)) => escena(&Movimiento::nuevo(x, y, coef), t, dt, centrada, out, read),
        _ => vacia(out, read),
    }
}

lessons_common::lesson!(draw, texto);

// ---- las afirmaciones ------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use lessons_common::{recorre, DIAG_ANCHO, DIAG_SLOTS, PRIM_CAP, READ_SLOTS};

    fn mov(x: &str, y: &str) -> Movimiento {
        let fx = compila(x, &VARIABLES).expect("x compila");
        let fy = (!y.is_empty()).then(|| compila(y, &VARIABLES).expect("y compila"));
        Movimiento::nuevo(fx, fy, [1.0, 1.0, 0.0])
    }

    /// Corre `draw` como lo corre la página y devuelve el buffer entero.
    fn dibuja(p: &[f64], texto: &str) -> (Vec<f64>, usize, Vec<f64>, Vec<f64>) {
        let mut prims = vec![0.0; PRIM_CAP];
        let mut rd = vec![0.0; READ_SLOTS];
        let mut dg = vec![0.0; DIAG_SLOTS * DIAG_ANCHO];
        let n = {
            let mut out = Prims::new(&mut prims);
            let mut read = Readouts::new(&mut rd);
            let mut diag = Diagnosticos::new(&mut dg);
            draw(p, texto, &mut out, &mut read, &mut diag);
            out.len()
        };
        (prims, n, rd, dg)
    }

    /// Los valores con que abre la página.
    const INICIO: [f64; 6] = [2.5, -0.3, 0.0, 1.0, 1.0, 0.0];

    /// D1: la velocidad y la aceleración que dibuja la lección son las
    /// derivadas de la fórmula escrita, exactas, contra la cuenta a mano.
    #[test]
    fn d1_derivadas_exactas() {
        let m = mov(ARRANQUE, "");
        for i in 0..=100 {
            let t = i as f64 * 0.1;
            let (x, _) = m.en(t);
            assert!((x.v - (3.0 * t * t - 0.2 * t * t * t)).abs() < 1e-12 * (1.0 + x.v.abs()));
            assert!((x.d1 - (6.0 * t - 0.6 * t * t)).abs() < 1e-12 * (1.0 + x.d1.abs()));
            assert!((x.d2 - (6.0 - 1.2 * t)).abs() < 1e-12 * (1.0 + x.d2.abs()));
        }
    }

    fn error_secante(m: &Movimiento, t: f64, dt: f64, centrada: bool) -> f64 {
        (secante(m, t, dt, centrada).0 - m.en(t).0.d1).abs()
    }

    /// D2: la secante tiende a la tangente — hacia adelante con orden 1
    /// (Δt diez veces menor, error diez veces menor), centrada con orden 2
    /// (cien veces menor). Medido, no supuesto.
    #[test]
    fn d2_orden_de_la_secante() {
        let m = mov("sin(t)", "");
        for t in [0.7, 1.9, 4.2] {
            let orden = |c: bool| {
                (error_secante(&m, t, 1e-2, c) / error_secante(&m, t, 1e-3, c)).log10()
            };
            let (adelante, centrada) = (orden(false), orden(true));
            assert!((adelante - 1.0).abs() < 0.05, "t={t}: orden {adelante}");
            assert!((centrada - 2.0).abs() < 0.05, "t={t}: orden {centrada}");
        }
    }

    /// D3: bajar Δt no mejora para siempre. Por debajo de ~10⁻⁸ la resta
    /// `x(t+Δt) − x(t)` pierde sus cifras en el redondeo, y la secante
    /// EMPEORA — mientras la derivada exacta no se entera.
    #[test]
    fn d3_el_piso_del_redondeo() {
        let m = mov("sin(t)", "");
        for t in [0.7, 1.9, 4.2] {
            let mejor = error_secante(&m, t, 1e-8, false);
            let peor = error_secante(&m, t, 1e-12, false);
            assert!(peor > 100.0 * mejor, "t={t}: {peor} vs {mejor}");
        }
    }

    /// D4: la tangente que se DIBUJA tiene pendiente v(t). Se mide sobre
    /// el dibujo: la escala se recupera de la curva dibujada, no de la
    /// función que la dibujó.
    #[test]
    fn d4_la_tangente_dibujada_mide_la_velocidad() {
        for (texto, t) in [(ARRANQUE, 2.5), (ARRANQUE, 7.0), ("4sin(t) + t", 3.3)] {
            let mut p = INICIO;
            p[P_T] = t;
            let (buf, n, _, _) = dibuja(&p, texto);
            let (mut vista, mut curva, mut tangente) = (0, None, None);
            for (tag, rec) in recorre(&buf, n) {
                match tag {
                    9 => vista = rec[1] as usize,
                    2 if vista == 1 && rec[rec.len() - 1] as usize == CURVA => {
                        curva = Some(rec[2..rec.len() - 1].to_vec());
                    }
                    1 if vista == 1 && rec[5] as usize == TANGENTE => tangente = Some(rec.to_vec()),
                    _ => {}
                }
            }
            let curva = curva.expect("la curva x(t)");
            let tg = tangente.expect("la tangente");
            // Dos puntos dibujados de la curva dan la escala vertical.
            let m = mov(texto, "");
            let (u1, y1) = (curva[20], curva[21]);
            let (u2, y2) = (curva[360], curva[361]);
            let (x1, x2) = (m.en(u1 * T1).0.v, m.en(u2 * T1).0.v);
            let alfa = (y2 - y1) / (x2 - x1);
            let pendiente_dibujada = (tg[4] - tg[2]) / (tg[3] - tg[1]);
            let v_medida = pendiente_dibujada / (alfa * (T1 - T0));
            let v = m.en(t).0.d1;
            assert!((v_medida - v).abs() < 1e-9 * (1.0 + v.abs()), "{texto} t={t}: {v_medida} vs {v}");
            // …y pasa por el punto de la curva en t.
            let medio = ((tg[1] + tg[3]) / 2.0, (tg[2] + tg[4]) / 2.0);
            let y_t = y1 + alfa * (m.en(t).0.v - x1);
            assert!((medio.0 - t / T1).abs() < 1e-12 && (medio.1 - y_t).abs() < 1e-9);
        }
    }

    /// D5: el movimiento de arranque sale del reposo y vuelve al reposo,
    /// y va más rápido exactamente en su punto de inflexión — donde la
    /// aceleración cruza el cero.
    #[test]
    fn d5_arranca_frena_y_la_inflexion_es_la_maxima() {
        let m = mov(ARRANQUE, "");
        assert!(m.en(0.0).0.d1.abs() < 1e-12);
        assert!(m.en(10.0).0.d1.abs() < 1e-12);
        let (x5, _) = m.en(5.0);
        assert!(x5.d2.abs() < 1e-12);
        assert!((x5.d1 - 15.0).abs() < 1e-12);
        for i in 0..=1000 {
            let v = m.en(i as f64 * 0.01).0.d1;
            assert!(v <= 15.0 + 1e-12);
        }
    }

    /// D6: en el círculo de radio R la velocidad es perpendicular al
    /// radio, `|a| = |v|²/R`, y la curvatura del producto cuña es `1/R`.
    #[test]
    fn d6_el_circulo() {
        let m = mov("3cos(2t)", "3sin(2t)");
        for i in 0..50 {
            let t = i as f64 * 0.21;
            let (x, y) = m.en(t);
            let v2 = x.d1 * x.d1 + y.d1 * y.d1;
            assert!((v2.sqrt() - 6.0).abs() < 1e-12);
            assert!((x.v * x.d1 + y.v * y.d1).abs() < 1e-12);
            assert!((x.d2.hypot(y.d2) - v2 / 3.0).abs() < 1e-11);
            let k = curvatura((x.d1, y.d1), (x.d2, y.d2));
            assert!((k - 1.0 / 3.0).abs() < 1e-14, "κ = {k}");
        }
        // Una recta recorrida acelerando: toda la aceleración es paralela
        // a la velocidad, el área v∧a es cero, y el camino no se dobla.
        let recta = mov("t²", "2t²");
        let (x, y) = recta.en(1.3);
        assert_eq!(curvatura((x.d1, y.d1), (x.d2, y.d2)), 0.0);
    }

    struct Xorshift(u64);
    impl Xorshift {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }
        fn unit(&mut self) -> f64 {
            (self.next() >> 11) as f64 / (1u64 << 53) as f64
        }
    }

    /// D7: nada que el lector escriba tumba la página. Ni pánico (sería
    /// una trampa de wasm), ni más prims de los que caben, ni una sola
    /// coordenada que no sea un número (el SVG escribiría "NaN").
    #[test]
    fn d7_nada_tumba_la_pagina() {
        let piezas = [
            "t", "a", "b", "c", "e", "τ", "sin(", "cos(", "tan(", "ln(", "sqrt(",
            "exp(", "abs(", "1/", "(", ")", "+", "−", "*", "^", "²", "3", "0.5",
            "1e", "20", "t^t", "1/(t-5)", "tan(t)", "sqrt(sin(20t))", "#", " ",
            "\n", "10^300", "ln(0)", "0/0",
        ];
        let rangos = [(0.0, 10.0), (-12.0, 0.3), (0.0, 1.0), (-5.0, 5.0), (-5.0, 5.0), (-5.0, 5.0)];
        let mut rng = Xorshift(0x2545_f491_4f6c_dd1d);
        for i in 0..3000 {
            let n = (rng.next() % 12) as usize;
            let texto: String = (0..n).map(|_| piezas[(rng.next() % piezas.len() as u64) as usize]).collect();
            let p: Vec<f64> = rangos.iter().map(|(lo, hi)| lo + (hi - lo) * rng.unit()).collect();
            let (buf, len, _, dg) = dibuja(&p, &texto);
            assert!(len <= PRIM_CAP);
            for (_, rec) in recorre(&buf, len) {
                assert!(rec.iter().all(|v| v.is_finite()), "#{i} {texto:?}: {rec:?}");
            }
            for linea in 0..2 {
                let codigo = dg[linea * DIAG_ANCHO];
                assert!((0.0..=11.0).contains(&codigo), "{texto:?}");
            }
        }
    }

    /// El manifiesto y este archivo hablan del mismo orden de parámetros
    /// y de la misma fórmula de arranque. Es la costura más débil del
    /// framework (DESIGN.md §5): aquí, al menos, tiene una prueba.
    #[test]
    fn el_manifiesto_dice_lo_mismo() {
        let ruta = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../public/lessons/velocidad/lesson.json");
        let json = std::fs::read_to_string(ruta).expect("lesson.json");
        let params = json.find("\"params\"").expect("params");
        let mut ultimo = params;
        for k in PARAMS {
            let aqui = json[params..].find(&format!("\"{k}\": {{")).map(|i| i + params);
            let aqui = aqui.unwrap_or_else(|| panic!("falta el parámetro {k}"));
            assert!(aqui > ultimo, "{k} fuera de orden");
            ultimo = aqui;
        }
        assert!(json.contains(&format!("\"valor\": \"{ARRANQUE}\"")));
    }

    /// El arranque de la página se dibuja completo y sin errores.
    #[test]
    fn la_pagina_abre_limpia() {
        let (_, n, rd, dg) = dibuja(&INICIO, ARRANQUE);
        assert!(n > 100);
        assert_eq!(&dg[0..2], &[0.0, 0.0]);
        assert_eq!(dg[2], 1.0); // sólo nombra t
        assert!((rd[1] - (6.0 * 2.5 - 0.6 * 2.5 * 2.5)).abs() < 1e-12);
        assert!(rd[7].is_nan()); // la curvatura no existe en una dimensión
        // Una fórmula rota deja el renglón con su código y su columna.
        let (_, _, rd, dg) = dibuja(&INICIO, "3t² −");
        assert_eq!(dg[0], f64::from(formulas::Codigo::FaltaOperando as u8));
        assert_eq!(dg[1], 6.0);
        assert!(rd[1].is_nan());
        // Escribir `a` enciende el bit del parámetro a.
        let (_, _, _, dg) = dibuja(&INICIO, "a t²");
        assert_eq!(dg[2], ((1u64 << P_T) | (1 << P_A)) as f64);
    }
}
