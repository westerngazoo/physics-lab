//! El péndulo simple — el movimiento exacto y el de ángulos pequeños, lado
//! a lado.
//!
//! Una masa puntual en un hilo sin masa de largo `L`, bajo una gravedad `g`,
//! soltada desde el reposo a un ángulo `θ₀`. La ecuación es
//! `θ̈ = −(g/L)·sen θ`, y la página dibuja dos respuestas a la vez:
//!
//! - **la exacta**, en forma cerrada con las funciones elípticas de Jacobi:
//!   `sen(θ/2) = k·sn(K − ω₀t, k)` con `k = sen(θ₀/2)`, `ω₀ = √(g/L)` y `K`
//!   la integral elíptica completa de primera especie. Su periodo es
//!   `T = 4K/ω₀`, que es lo mismo que `T = T₀ / AGM(1, cos(θ₀/2))`;
//! - **la de ángulos pequeños**, `θ(t) = θ₀·cos(ω₀t)`, con periodo
//!   `T₀ = τ·√(L/g)` (el `2π√(L/g)` de los libros).
//!
//! Nada se integra paso a paso: `K`, `sn` y `cn` salen de la media
//! aritmético-geométrica (A&S 16.4), que converge cuadráticamente, así que
//! cada cuadro es una función pura de `(θ₀, L, g, t)` y el tiempo se puede
//! mover hacia atrás sin acumular error (RFC-001: forma cerrada cuando
//! existe). Las pruebas la comparan contra una integración RK4 independiente
//! (P1), igual que la ecuación de onda se compara contra Verlet.
//!
//! # Parámetros (orden del manifiesto)
//!
//! `p[0]` θ₀ en radianes (el deslizador va en grados; la `scale` τ/360
//! convierte), `p[1]` L en m, `p[2]` g en m/s², `p[3]` el tiempo en
//! periodos de ángulos pequeños `T₀` (así cada número entero del deslizador
//! es una vuelta del modelo, y lo que el exacto se atrasa se ve a simple
//! vista).
//!
//! # Lecturas — ranuras estables
//!
//! Otras herramientas (Akademos) construyen predicciones y retos sobre
//! estos números: no se renumeran.
//!
//! | ranura | qué | unidad |
//! |---|---|---|
//! | 0 | θ(t) exacto | ° |
//! | 1 | θ(t) de ángulos pequeños | ° |
//! | 2 | periodo exacto `T` | s |
//! | 3 | periodo de ángulos pequeños `T₀ = τ√(L/g)` | s |
//! | 4 | diferencia `(T − T₀)/T₀` | % |
//! | 5 | el tiempo `t` | s |
//! | 6, 7 | sin usar (reservadas) | — |

use std::f64::consts::TAU;

/// Vueltas máximas de la media aritmético-geométrica. Converge
/// cuadráticamente: con `θ₀ ≤ 80°` le bastan cinco o seis.
const AGM_MAX: usize = 32;

/// La media aritmético-geométrica de `a` y `b` (ambos positivos).
pub fn agm(a: f64, b: f64) -> f64 {
    let (mut a, mut b) = (a, b);
    for _ in 0..AGM_MAX {
        if (a - b).abs() <= 1e-15 * a {
            break;
        }
        let media = 0.5 * (a + b);
        b = (a * b).sqrt();
        a = media;
    }
    0.5 * (a + b)
}

/// `T/T₀` para una amplitud `θ₀` (radianes, `0 ≤ θ₀ < τ/2`):
/// `1 / AGM(1, cos(θ₀/2))`. No depende ni de `L` ni de `g`.
pub fn periodo_relativo(theta0: f64) -> f64 {
    1.0 / agm(1.0, (0.5 * theta0).cos())
}

/// `sn(u, k)` y `cn(u, k)` por la transformación descendente de Landen
/// (Abramowitz y Stegun 16.4). `kp = √(1 − k²)` se pasa aparte para no
/// perder precisión cuando `k` es chico.
pub fn sn_cn(u: f64, k: f64, kp: f64) -> (f64, f64) {
    let mut a = [0.0; AGM_MAX + 1];
    let mut c = [0.0; AGM_MAX + 1];
    a[0] = 1.0;
    c[0] = k;
    let mut b = kp;
    let mut n = 0;
    while n < AGM_MAX && c[n].abs() > 1e-16 {
        a[n + 1] = 0.5 * (a[n] + b);
        c[n + 1] = 0.5 * (a[n] - b);
        b = (a[n] * b).sqrt();
        n += 1;
    }
    let mut phi = 2f64.powi(n as i32) * a[n] * u;
    for j in (1..=n).rev() {
        phi = 0.5 * (phi + (c[j] / a[j] * phi.sin()).asin());
    }
    (phi.sin(), phi.cos())
}

/// Un péndulo soltado desde el reposo a `θ₀`: todo lo que hace falta para
/// evaluar el movimiento en cualquier instante, sin estado que avance.
#[derive(Clone, Copy, Debug)]
pub struct Pendulo {
    /// La amplitud, en radianes.
    pub theta0: f64,
    /// `ω₀ = √(g/L)`, en rad/s.
    pub w0: f64,
    /// `k = sen(θ₀/2)`, el módulo elíptico.
    k: f64,
    /// `k′ = cos(θ₀/2)`.
    kp: f64,
    /// `K(k)`, la integral elíptica completa de primera especie.
    kk: f64,
}

impl Pendulo {
    /// `theta0` en radianes, `l` en m, `g` en m/s². Válido para
    /// `0 ≤ θ₀ < τ/2` (por debajo de la vertical invertida).
    pub fn new(theta0: f64, l: f64, g: f64) -> Self {
        let (k, kp) = ((0.5 * theta0).sin(), (0.5 * theta0).cos());
        Pendulo {
            theta0,
            w0: (g / l).sqrt(),
            k,
            kp,
            kk: TAU / (4.0 * agm(1.0, kp)),
        }
    }

    /// El periodo exacto, `T = 4K/ω₀`, en segundos.
    pub fn periodo(&self) -> f64 {
        4.0 * self.kk / self.w0
    }

    /// El periodo de ángulos pequeños, `T₀ = τ/ω₀ = τ√(L/g)`, en segundos.
    pub fn periodo_pequenos(&self) -> f64 {
        TAU / self.w0
    }

    /// El movimiento exacto en `t` (s): `(θ, θ̇)` en rad y rad/s.
    ///
    /// `θ = 2·arcsen(k·sn(K − ω₀t))` y `θ̇ = −2kω₀·cn(K − ω₀t)`.
    pub fn estado(&self, t: f64) -> (f64, f64) {
        let (sn, cn) = sn_cn(self.kk - self.w0 * t, self.k, self.kp);
        (2.0 * (self.k * sn).asin(), -2.0 * self.k * self.w0 * cn)
    }

    /// El modelo de ángulos pequeños en `t` (s): `θ₀·cos(ω₀t)`.
    pub fn pequenos(&self, t: f64) -> f64 {
        self.theta0 * (self.w0 * t).cos()
    }
}

/// La energía por unidad de masa, `½L²θ̇² + gL(1 − cos θ)`, en J/kg.
pub fn energia(theta: f64, omega: f64, l: f64, g: f64) -> f64 {
    0.5 * l * l * omega * omega + g * l * (1.0 - theta.cos())
}

// ---- la frontera con wasm -----------------------------------------------

use lessons_common::{Prims, Readouts};

/// El tope del deslizador del tiempo, en periodos `T₀`; la gráfica θ(t)
/// cubre exactamente ese rango.
const T_MAX: f64 = 5.0;

/// El radio con que se dibuja la lenteja, en unidades de `L`.
const LENTEJA: f64 = 0.055;

/// Una lenteja: un círculo y su centro.
fn lenteja(out: &mut Prims, x: f64, y: f64, style: usize) {
    out.curve(0.0, TAU, 24, style, |a| {
        (x + LENTEJA * a.cos(), y + LENTEJA * a.sin())
    });
    out.point(x, y, style);
}

/// Params: `[θ₀ (rad), L (m), g (m/s²), t (en periodos T₀)]`.
fn draw(p: &[f64], out: &mut Prims, read: &mut Readouts) {
    let (theta0, l, g, tp) = (p[0], p[1], p[2], p[3]);
    let pen = Pendulo::new(theta0, l, g);
    let t0 = pen.periodo_pequenos();
    let t = tp * t0;
    let (theta, _) = pen.estado(t);
    let theta_peq = pen.pequenos(t);

    // ---- vista 0: el péndulo, en unidades de L ---------------------------
    out.view(0);
    out.segment(-0.3, 0.0, 0.3, 0.0, 0); // el techo
    out.segment(0.0, 0.0, 0.0, -1.12, 1); // la vertical
    out.curve(-theta0, theta0, 48, 1, |a| (a.sin(), -a.cos())); // el arco
    let (xs, ys) = (theta_peq.sin(), -theta_peq.cos());
    out.segment(0.0, 0.0, xs, ys, 3); // el modelo, debajo
    lenteja(out, xs, ys, 3);
    let (xe, ye) = (theta.sin(), -theta.cos());
    out.segment(0.0, 0.0, xe, ye, 2); // el exacto, encima
    lenteja(out, xe, ye, 2);
    out.point(0.0, 0.0, 0); // el pivote

    // ---- vista 1: T/T₀ contra θ₀ (grados) ---------------------------------
    out.view(1);
    out.segment(0.0, 0.99, 90.0, 0.99, 0); // eje de θ₀
    out.segment(0.0, 0.99, 0.0, 1.2, 0); // eje de T/T₀
    for d in [20.0, 40.0, 60.0, 80.0] {
        out.segment(d, 0.99, d, 0.983, 0);
        out.numero(d, 0.975, d, 0, 5);
    }
    for r in [1.0, 1.05, 1.1, 1.15] {
        out.segment(0.0, r, -1.2, r, 0);
        out.numero(-6.5, r, r, 2, 5);
    }
    out.label(45.0, 0.951, 0, 5); // θ₀ (°)
    out.label(9.0, 1.195, 1, 5); // T/T₀
    out.segment(0.0, 1.0, 90.0, 1.0, 3); // ángulos pequeños: T = T₀ siempre
    out.curve(0.0, 89.5, 90, 2, |d| (d, periodo_relativo(d.to_radians())));
    let rel = periodo_relativo(theta0);
    let grados = theta0.to_degrees();
    out.segment(grados, 0.99, grados, rel, 4);
    out.point(grados, rel, 4);

    // ---- vista 2: θ(t)/θ₀, exacto contra ángulos pequeños -----------------
    out.view(2);
    out.segment(0.0, 0.0, T_MAX, 0.0, 0); // el eje del tiempo
    out.segment(0.0, -1.1, 0.0, 1.1, 0);
    for n in 1..=5 {
        let x = f64::from(n);
        out.segment(x, -0.05, x, 0.05, 0);
        out.numero(x, -1.25, x, 0, 6);
    }
    for y in [-1.0, 1.0] {
        out.segment(0.0, y, -0.05, y, 0);
        out.numero(-0.17, y, y, 0, 6);
    }
    out.label(2.5, -1.48, 2, 6); // t/T₀
    out.label(0.3, 1.24, 3, 6); // θ/θ₀
    out.curve(0.0, T_MAX, 400, 3, |x| (x, (TAU * x).cos()));
    out.curve(0.0, T_MAX, 400, 2, |x| (x, pen.estado(x * t0).0 / theta0));
    out.segment(tp, -1.12, tp, 1.12, 4); // el instante t
    out.point(tp, theta_peq / theta0, 3);
    out.point(tp, theta / theta0, 2);

    read.set(0, theta.to_degrees());
    read.set(1, theta_peq.to_degrees());
    read.set(2, pen.periodo());
    read.set(3, t0);
    read.set(4, (pen.periodo() / t0 - 1.0) * 100.0);
    read.set(5, t);
}

lessons_common::lesson!(draw);

// ---- las afirmaciones -----------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    /// La rejilla del deslizador de θ₀, en grados: de 1° a 80°, de medio en
    /// medio.
    fn angulos() -> impl Iterator<Item = f64> {
        (2..=160).map(|i| f64::from(i) * 0.5)
    }

    /// Las lecturas que pinta la página, con θ₀ en grados como en el
    /// deslizador y `tp` en periodos `T₀`.
    fn lecturas(grados: f64, l: f64, g: f64, tp: f64) -> [f64; lessons_common::READ_SLOTS] {
        let mut buf = [0.0; lessons_common::PRIM_CAP];
        let mut lee = [0.0; lessons_common::READ_SLOTS];
        {
            let mut prims = Prims::new(&mut buf);
            let mut read = Readouts::new(&mut lee);
            draw(&[grados.to_radians(), l, g, tp], &mut prims, &mut read);
        }
        lee
    }

    /// Una integración RK4 independiente de `m·L·θ̈ = −m·g·sen θ`, con la
    /// masa explícita: la fuerza tangencial entre la masa da la aceleración.
    /// Llama a `visita(t, θ, θ̇)` en cada paso y devuelve los cruces de θ
    /// por cero hacia abajo (de + a −), interpolados.
    fn rk4(
        theta0: f64,
        l: f64,
        g: f64,
        m: f64,
        dt: f64,
        pasos: usize,
        mut visita: impl FnMut(f64, f64, f64),
    ) -> Vec<f64> {
        let acel = |th: f64| (-m * g * th.sin()) / (m * l);
        let (mut th, mut om) = (theta0, 0.0);
        let mut cruces = Vec::new();
        for i in 0..pasos {
            let k1 = (om, acel(th));
            let k2 = (om + 0.5 * dt * k1.1, acel(th + 0.5 * dt * k1.0));
            let k3 = (om + 0.5 * dt * k2.1, acel(th + 0.5 * dt * k2.0));
            let k4 = (om + dt * k3.1, acel(th + dt * k3.0));
            let th1 = th + dt / 6.0 * (k1.0 + 2.0 * k2.0 + 2.0 * k3.0 + k4.0);
            let om1 = om + dt / 6.0 * (k1.1 + 2.0 * k2.1 + 2.0 * k3.1 + k4.1);
            let t1 = (i + 1) as f64 * dt;
            if th > 0.0 && th1 <= 0.0 {
                cruces.push(t1 - dt + dt * th / (th - th1));
            }
            th = th1;
            om = om1;
            visita(t1, th, om);
        }
        cruces
    }

    /// El periodo medido en una integración: la distancia entre dos cruces
    /// por cero en el mismo sentido.
    fn periodo_medido(cruces: &[f64]) -> f64 {
        assert!(
            cruces.len() >= 2,
            "hacen falta dos cruces; hubo {}",
            cruces.len()
        );
        cruces[1] - cruces[0]
    }

    /// P1: la forma cerrada coincide con una integración RK4 independiente
    /// —ángulo y velocidad a 1e-9 durante tres periodos— y el periodo medido
    /// en esa integración es el que marca la página (ranura 2), a 1e-9.
    #[test]
    fn p1_coincide_con_una_integracion_independiente() {
        let (l, g) = (1.0, 9.81);
        for grados in [1.0, 10.0, 30.0, 45.0, 60.0, 80.0] {
            let pen = Pendulo::new(f64::to_radians(grados), l, g);
            let periodo = pen.periodo();
            let dt = periodo / 20_000.0;
            let mut peor: f64 = 0.0;
            let cruces = rk4(pen.theta0, l, g, 1.0, dt, 60_000, |t, th, om| {
                let (te, oe) = pen.estado(t);
                peor = peor.max((te - th).abs()).max((oe - om).abs() / pen.w0);
            });
            assert!(
                peor < 1e-9,
                "θ₀ = {grados}°: la forma cerrada se aparta {peor:e}"
            );
            let medido = periodo_medido(&cruces);
            let pagina = lecturas(grados, l, g, 0.0)[2];
            assert!(
                (medido / pagina - 1.0).abs() < 1e-9,
                "θ₀ = {grados}°: {medido} contra {pagina}"
            );
        }
    }

    /// P2: el periodo no depende de la masa. La masa se cancela en
    /// `m·L·θ̈ = −m·g·sen θ`; la integración la lleva explícita, de 50 g a
    /// 200 kg, y mide el mismo periodo —el de la página— cada vez. `draw`
    /// ni siquiera recibe una masa: la página no tiene deslizador para ella.
    #[test]
    fn p2_el_periodo_no_depende_de_la_masa() {
        let (l, g, grados) = (0.8, 9.81, 40.0);
        let pagina = lecturas(grados, l, g, 0.0)[2];
        let dt = pagina / 20_000.0;
        for m in [0.05, 1.0, 200.0] {
            let medido =
                periodo_medido(&rk4(grados.to_radians(), l, g, m, dt, 50_000, |_, _, _| {}));
            assert!(
                (medido / pagina - 1.0).abs() < 1e-9,
                "m = {m} kg: {medido} contra {pagina}"
            );
        }
    }

    /// P3: con ángulos pequeños el periodo exacto tiende a `T₀ = τ√(L/g)`.
    /// A 1° la página marca menos de 0,002 % de diferencia y las dos curvas
    /// no se separan ni 0,02 % de θ₀ en el primer periodo; y el exceso
    /// `T/T₀ − 1` va como `θ₀²/16` cuando θ₀ → 0.
    #[test]
    fn p3_con_angulos_pequenos_tiende_a_t0() {
        let lee = lecturas(1.0, 1.0, 9.81, 0.0);
        assert!((lee[3] - TAU * (1.0_f64 / 9.81).sqrt()).abs() < 1e-15);
        assert!(lee[4] > 0.0 && lee[4] < 0.002, "a 1° difieren {} %", lee[4]);

        let pen = Pendulo::new(1.0_f64.to_radians(), 1.0, 9.81);
        let t0 = pen.periodo_pequenos();
        let peor = (0..=1000)
            .map(|i| f64::from(i) / 1000.0 * t0)
            .map(|t| (pen.estado(t).0 - pen.pequenos(t)).abs())
            .fold(0.0, f64::max);
        assert!(peor < 2e-4 * pen.theta0, "a 1° se separan {peor:e} rad");

        for grados in [2.0, 1.0, 0.5, 0.1] {
            let th: f64 = f64::to_radians(grados);
            let cociente = (periodo_relativo(th) - 1.0) / (th * th / 16.0);
            assert!((cociente - 1.0).abs() < 1e-3, "θ₀ = {grados}°: {cociente}");
        }
    }

    /// P4: el periodo crece con θ₀, estrictamente, en toda la rejilla del
    /// deslizador, y lo hace como la serie exacta
    /// `T/T₀ = 1 + θ₀²/16 + 11θ₀⁴/3072 + 173θ₀⁶/737280 + …`: todos sus
    /// términos son positivos, así que tres términos quedan por debajo y lo
    /// que falta no pasa de 1,2 veces el cuarto. Y a 80° el exacto termina
    /// fuera de fase con el modelo: se separan más que θ₀/2 en cinco T₀.
    #[test]
    fn p4_el_periodo_crece_con_el_angulo() {
        let mut antes = 1.0;
        for grados in angulos() {
            let th: f64 = grados.to_radians();
            let r = periodo_relativo(th);
            assert!(r > antes, "θ₀ = {grados}°: {r} no crece");
            antes = r;
            let tres = 1.0 + th.powi(2) / 16.0 + 11.0 * th.powi(4) / 3072.0;
            let cuarto = 173.0 * th.powi(6) / 737_280.0;
            assert!(r > tres - 1e-15, "θ₀ = {grados}°");
            assert!(r < tres + 1.2 * cuarto + 1e-15, "θ₀ = {grados}°");
            // la página dice lo mismo, en %
            let lee = lecturas(grados, 1.3, 3.71, 0.0);
            assert!((lee[4] - (r - 1.0) * 100.0).abs() < 1e-12, "θ₀ = {grados}°");
        }

        let pen = Pendulo::new(80.0_f64.to_radians(), 1.0, 9.81);
        let t0 = pen.periodo_pequenos();
        let peor = (0..=5000)
            .map(|i| f64::from(i) / 1000.0 * t0)
            .map(|t| (pen.estado(t).0 - pen.pequenos(t)).abs())
            .fold(0.0, f64::max);
        assert!(peor > 0.5 * pen.theta0, "a 80° sólo se separan {peor} rad");
    }

    /// P5: duplicar L multiplica el periodo por √2, a cualquier ángulo —
    /// exacto y de ángulos pequeños— porque el tiempo sólo entra como
    /// `√(L/g)`. Por eso la diferencia en % no se mueve con L ni con g.
    #[test]
    fn p5_duplicar_l_multiplica_el_periodo_por_raiz_de_dos() {
        for grados in angulos() {
            for (l, g) in [(0.25, 9.81), (1.0, 1.62), (1.5, 24.79)] {
                let a = lecturas(grados, l, g, 0.0);
                let b = lecturas(grados, 2.0 * l, g, 0.0);
                assert!(
                    (b[2] / a[2] - 2f64.sqrt()).abs() < 1e-14,
                    "θ₀ = {grados}°, L = {l}"
                );
                assert!(
                    (b[3] / a[3] - 2f64.sqrt()).abs() < 1e-14,
                    "θ₀ = {grados}°, L = {l}"
                );
                assert!((b[4] - a[4]).abs() < 1e-11, "θ₀ = {grados}°, L = {l}");
            }
        }
    }

    /// P6: la energía se conserva. En la forma cerrada,
    /// `½L²θ̇² + gL(1 − cos θ)` vale `gL(1 − cos θ₀)` en todo instante a
    /// 1e-10 relativo; la integración RK4 independiente también la conserva,
    /// a 1e-9 en tres periodos.
    #[test]
    fn p6_la_energia_se_conserva() {
        let (l, g) = (1.2, 9.81);
        for grados in angulos() {
            let pen = Pendulo::new(grados.to_radians(), l, g);
            let e0 = energia(pen.theta0, 0.0, l, g);
            let periodo = pen.periodo();
            for i in 0..=997 {
                let t = f64::from(i) / 997.0 * 3.0 * periodo;
                let (th, om) = pen.estado(t);
                let e = energia(th, om, l, g);
                assert!(
                    (e / e0 - 1.0).abs() < 1e-10,
                    "θ₀ = {grados}°, t = {t}: {e} contra {e0}"
                );
            }
        }
        for grados in [5.0, 45.0, 80.0] {
            let th0: f64 = f64::to_radians(grados);
            let e0 = energia(th0, 0.0, l, g);
            let dt = Pendulo::new(th0, l, g).periodo() / 20_000.0;
            let mut peor: f64 = 0.0;
            rk4(th0, l, g, 1.0, dt, 60_000, |_, th, om| {
                peor = peor.max((energia(th, om, l, g) / e0 - 1.0).abs());
            });
            assert!(peor < 1e-9, "RK4 a {grados}°: la energía deriva {peor:e}");
        }
    }

    /// Las ranuras son estables: en t = 0 los dos péndulos están en θ₀, y
    /// en cada entero del deslizador el modelo vuelve a θ₀ con `t = n·T₀`.
    #[test]
    fn las_lecturas_son_las_documentadas() {
        for grados in [1.0, 10.0, 47.5, 80.0] {
            let lee = lecturas(grados, 0.7, 9.81, 0.0);
            assert!((lee[0] - grados).abs() < 1e-9, "θ(0) = {}", lee[0]);
            assert!((lee[1] - grados).abs() < 1e-12);
            assert!(lee[5] == 0.0);
            for n in 1..=5 {
                let lee = lecturas(grados, 0.7, 9.81, f64::from(n));
                assert!((lee[1] - grados).abs() < 1e-9, "θ₀ = {grados}°, n = {n}");
                assert!((lee[5] - f64::from(n) * lee[3]).abs() < 1e-12);
            }
        }
    }

    /// El dibujo cabe en el buffer, en el peor caso del deslizador.
    #[test]
    fn el_dibujo_cabe() {
        let mut buf = [0.0; lessons_common::PRIM_CAP];
        let mut lee = [0.0; lessons_common::READ_SLOTS];
        let n = {
            let mut prims = Prims::new(&mut buf);
            let mut read = Readouts::new(&mut lee);
            draw(
                &[80.0_f64.to_radians(), 3.0, 1.62, 5.0],
                &mut prims,
                &mut read,
            );
            prims.len()
        };
        assert!(n < lessons_common::PRIM_CAP);
        assert_eq!(lessons_common::cuenta(&buf, n, 9), 3, "tres vistas");
    }
}
