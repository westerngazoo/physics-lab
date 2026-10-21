//! Remo con mancuerna a una mano contra remo en máquina con apoyo: la misma
//! carga, la misma mano, otra línea de fuerza.
//!
//! Del reel 44 de fisicobuenfisico (`tools/reel44.py` allá), portado con sus
//! mismos supuestos y probado contra sus números. El hallazgo no es cuál pesa
//! más: es que **la mancuerna no da nada con el brazo estirado**, y no es un
//! supuesto del modelo, es geometría. Abajo del todo el brazo cuelga y la
//! mancuerna cuelga con él, así que la carga queda a plomo del hombro: su
//! línea de acción PASA POR LA ARTICULACIÓN. Brazo de momento cero, torque
//! cero, se incline lo que se incline el torso (la mancuerna siempre cuelga
//! vertical; [`RemoMancuerna::inclina_rad`] no cambia ese cero).
//!
//! # La misma cuenta girada 90°
//!
//! Con la mancuerna la fuerza es vertical ([`Linea::Vertical`]), así que la
//! palanca del hombro es la distancia HORIZONTAL a la mano, y abajo vale cero.
//! Con la máquina la fuerza es horizontal ([`Linea::Horizontal`]), así que la
//! palanca es la distancia VERTICAL, cuánto quedan las manos por debajo del
//! hombro, y eso no se muere en ningún punto del recorrido. Son perfiles
//! espejo: la mancuerna arranca en nada y lo da todo al final; la máquina
//! carga desde el primer centímetro y sube poco.
//!
//! # Convención
//!
//! El τ es el de la CARGA sola, sin el peso del propio brazo, igual que en
//! [`crate::dominada`]; y es de UN brazo, porque el remo con mancuerna es a
//! una mano y compararlo contra las dos de la máquina sería hacer trampa. El
//! modelo es 2D, cuasi-estático, con las fracciones de Winter de
//! [`crate::maquina_humana`] y el codo resuelto por [`crate::maquina_humana::codo`].

use crate::maquina_humana::{codo, Doblez, Persona};
use crate::palanca::{self, Linea, Punto};

/// Dónde llega la mano al cerrar: a esta fracción del torso desde el hombro
/// (la misma del reel 38 de fisicobuenfisico, a propósito: dos piezas de la
/// misma serie miden el mismo punto).
pub const TOQUE: f64 = 0.45;
/// Y este medio grosor de tronco afuera del eje del torso, m.
pub const GROSOR: f64 = 0.12;
/// El brazo abajo, casi estirado: fracción del brazo.
pub const EXTENSION: f64 = 0.99;
/// La máquina: cuánto quedan las manos POR DEBAJO del hombro al arrancar y al
/// cerrar, m. Es geometría de máquina, o sea un supuesto, y por eso se prueba
/// con variantes: la conclusión tiene que aguantar.
pub const PROFUNDIDAD: (f64, f64) = (0.15, 0.20);
/// En la máquina la mano cierra a esta fracción de [`TOQUE`] · torso delante
/// del hombro (el apoyo del pecho estorba).
pub const CIERRE_MAQUINA: f64 = 0.6;

/// Iteraciones de las bisecciones (sobran: 2⁻⁶⁰ del avance).
const BISECCION: usize = 60;

/// Una postura: dónde quedan las articulaciones, en metros, con el hombro en
/// el origen (y hacia arriba).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Postura {
    pub hombro: Punto,
    pub codo: Punto,
    pub mano: Punto,
    /// El eje del torso, unitario, del hombro hacia la cadera.
    pub eje: Punto,
}

/// Un remo: una postura por avance `u` (0: el brazo estirado, abajo; 1: el
/// cierre) y por dónde jala la carga.
pub trait Remo {
    fn persona(&self) -> &Persona;
    /// La carga en la mano, kg.
    fn carga_kg(&self) -> f64;
    fn postura(&self, u: f64) -> Postura;
    /// La línea por la que jala la carga.
    fn linea(&self) -> Linea;
    /// El peso de la carga, N.
    fn peso(&self) -> f64 {
        palanca::peso(self.carga_kg())
    }
    /// Lo que viaja la carga en su línea de acción hasta el avance `u`, m.
    fn recorrido(&self, u: f64) -> f64 {
        let (a, b) = (self.postura(0.0).mano, self.postura(u).mano);
        let (ux, uy) = self.linea().direccion(b);
        ((b.0 - a.0) * ux + (b.1 - a.1) * uy).abs()
    }
    /// El trabajo de la carga por repetición, J.
    fn trabajo(&self) -> f64 {
        self.peso() * self.recorrido(1.0)
    }
}

fn entre(a: Punto, b: Punto, t: f64) -> Punto {
    (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t)
}

fn codo_de(p: &Persona, mano: Punto, hacia: Doblez) -> Punto {
    let c = codo(p, [0.0, 0.0], [mano.0, mano.1], hacia).expect("el codo del remo tiene lado");
    (c[0], c[1])
}

/// El remo con mancuerna a una mano: el torso apoyado, casi horizontal, y la
/// mancuerna COLGANDO. El hombro en el origen, +x hacia la cadera.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RemoMancuerna {
    pub persona: Persona,
    pub carga_kg: f64,
    /// Cuánto se levanta el eje del torso sobre la horizontal (positivo: la
    /// cadera más alta que el hombro), radianes. Da igual para el hallazgo.
    pub inclina_rad: f64,
}

impl RemoMancuerna {
    /// El eje del torso, unitario, del hombro hacia la cadera.
    #[must_use]
    pub fn eje(&self) -> Punto {
        (self.inclina_rad.cos(), self.inclina_rad.sin())
    }
    /// La mano abajo: cuelga vertical, siempre.
    #[must_use]
    pub fn abajo(&self) -> Punto {
        (0.0, -self.persona.brazo() * EXTENSION)
    }
    /// La mano al cerrar: a [`TOQUE`] del torso hacia la cadera y [`GROSOR`] afuera.
    #[must_use]
    pub fn cierre(&self) -> Punto {
        let e = self.eje();
        let fuera = (e.1, -e.0);
        let t = TOQUE * self.persona.torso();
        (e.0 * t + fuera.0 * GROSOR, e.1 * t + fuera.1 * GROSOR)
    }
}

impl Remo for RemoMancuerna {
    fn persona(&self) -> &Persona {
        &self.persona
    }
    fn carga_kg(&self) -> f64 {
        self.carga_kg
    }
    fn postura(&self, u: f64) -> Postura {
        let mano = entre(self.abajo(), self.cierre(), u);
        // el codo sube y va hacia la cadera al cerrar
        let codo = codo_de(&self.persona, mano, Doblez::Hacia([1.0, 0.5]));
        Postura { hombro: (0.0, 0.0), codo, mano, eje: self.eje() }
    }
    fn linea(&self) -> Linea {
        Linea::Vertical
    }
}

/// El remo en máquina con apoyo en el pecho: el torso vertical, la máquina
/// jala horizontal. El hombro en el origen, +x al frente.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RemoMaquina {
    pub persona: Persona,
    pub carga_kg: f64,
    /// Cuánto quedan las manos por debajo del hombro (al arrancar, al cerrar), m.
    pub profundidad: (f64, f64),
}

impl RemoMaquina {
    /// La mano abajo (estirado): el brazo casi estirado al frente, a
    /// [`EXTENSION`] del brazo del hombro, con la mano a su profundidad.
    ///
    /// reel44.py ponía aquí la x a 0.99 del brazo Y la mano 15 cm abajo, o sea
    /// la mano 1 cm más allá de donde llega el brazo (0.672 m con un brazo de
    /// 0.662), y estiraba el antebrazo para alcanzarla. Aquí el brazo mide lo
    /// que mide. Ningún número del hombro cambia (la palanca de la máquina es
    /// la vertical, y la y es la misma); lo que cambia es el recorrido de la
    /// mano (0.500 m en vez de 0.517) y el τ del codo, que el reel no dice.
    #[must_use]
    pub fn abajo(&self) -> Punto {
        let l = self.persona.brazo() * EXTENSION;
        ((l * l - self.profundidad.0 * self.profundidad.0).sqrt(), -self.profundidad.0)
    }
    /// La mano al cerrar.
    #[must_use]
    pub fn cierre(&self) -> Punto {
        (TOQUE * self.persona.torso() * CIERRE_MAQUINA, -self.profundidad.1)
    }
}

impl Remo for RemoMaquina {
    fn persona(&self) -> &Persona {
        &self.persona
    }
    fn carga_kg(&self) -> f64 {
        self.carga_kg
    }
    fn postura(&self, u: f64) -> Postura {
        let mano = entre(self.abajo(), self.cierre(), u);
        // el codo va hacia atrás y abajo al cerrar
        let codo = codo_de(&self.persona, mano, Doblez::Hacia([-1.0, -0.4]));
        Postura { hombro: (0.0, 0.0), codo, mano, eje: (0.0, -1.0) }
    }
    fn linea(&self) -> Linea {
        Linea::Horizontal
    }
}

/// La palanca del hombro, m: la distancia del hombro a la LÍNEA por la que
/// jala la carga ([`palanca::brazo`]). Con la mancuerna es la horizontal a la
/// mano; con la máquina, la vertical.
#[must_use]
pub fn palanca_hombro<R: Remo + ?Sized>(r: &R, q: &Postura) -> f64 {
    palanca::brazo(q.hombro, q.mano, r.linea())
}

/// La palanca del codo, m.
#[must_use]
pub fn palanca_codo<R: Remo + ?Sized>(r: &R, q: &Postura) -> f64 {
    palanca::brazo(q.codo, q.mano, r.linea())
}

/// |τ| de la carga en el hombro en el avance `u`, N·m.
#[must_use]
pub fn tau<R: Remo + ?Sized>(r: &R, u: f64) -> f64 {
    r.peso() * palanca_hombro(r, &r.postura(u))
}

/// |τ| de la carga en el codo en el avance `u`, N·m.
#[must_use]
pub fn tau_codo<R: Remo + ?Sized>(r: &R, u: f64) -> f64 {
    r.peso() * palanca_codo(r, &r.postura(u))
}

/// El pico de τ en el hombro en `n + 1` muestras.
#[must_use]
pub fn pico<R: Remo + ?Sized>(r: &R, n: usize) -> f64 {
    (0..=n).map(|i| tau(r, i as f64 / n as f64)).fold(f64::MIN, f64::max)
}

/// El promedio de τ en el hombro en `n + 1` muestras.
#[must_use]
pub fn media<R: Remo + ?Sized>(r: &R, n: usize) -> f64 {
    (0..=n).map(|i| tau(r, i as f64 / n as f64)).sum::<f64>() / (n + 1) as f64
}

/// Qué fracción del avance (en `n + 1` muestras) pasa `r` por debajo de
/// `umbral` N·m. Con el umbral en lo que la máquina da estirada, es cuánto
/// del remo con mancuerna va por debajo del arranque de la máquina.
#[must_use]
pub fn fraccion_por_debajo<R: Remo + ?Sized>(r: &R, umbral: f64, n: usize) -> f64 {
    (0..=n).filter(|&i| tau(r, i as f64 / n as f64) < umbral).count() as f64 / (n + 1) as f64
}

/// El avance en que el τ de `a` alcanza al de `b` (bisección sobre la
/// diferencia). `None` si no se cruzan entre 0 y 1.
#[must_use]
pub fn cruce<A: Remo + ?Sized, B: Remo + ?Sized>(a: &A, b: &B) -> Option<f64> {
    let d = |u: f64| tau(a, u) - tau(b, u);
    let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
    if (d(lo) < 0.0) == (d(hi) < 0.0) {
        return None;
    }
    let sube = d(lo) < 0.0;
    for _ in 0..BISECCION {
        let m = (lo + hi) / 2.0;
        if (d(m) < 0.0) == sube {
            lo = m;
        } else {
            hi = m;
        }
    }
    Some((lo + hi) / 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn persona() -> Persona {
        Persona { estatura_m: 1.78, masa_kg: 78.0 }
    }
    fn mancuerna() -> RemoMancuerna {
        RemoMancuerna { persona: persona(), carga_kg: 30.0, inclina_rad: 0.0 }
    }
    fn maquina() -> RemoMaquina {
        RemoMaquina { persona: persona(), carga_kg: 30.0, profundidad: PROFUNDIDAD }
    }
    fn cerca(a: f64, b: f64, tol: f64, que: &str) {
        assert!((a - b).abs() < tol, "{que}: {a} ≠ {b}");
    }

    /// Las longitudes y el peso de reel44.py.
    #[test]
    fn las_constantes_de_reel44_py() {
        let p = persona();
        cerca(p.torso(), 0.512_640, 1e-6, "LTS");
        cerca(p.brazo(), 0.662_160, 1e-6, "BRAZO");
        cerca(p.humero(), 0.344_323, 1e-6, "LA");
        cerca(p.antebrazo(), 0.317_837, 1e-6, "LF");
        cerca(mancuerna().peso(), 294.3, 1e-9, "W_CARGA");
        cerca(maquina().peso(), 294.3, 1e-9, "W_CARGA");
    }

    /// Las posturas de reel44.py (`pose_mancuerna`, `pose_maquina`, `codo`,
    /// `palanca`, `torques`), en cinco puntos de la repetición.
    #[test]
    fn las_posturas_de_reel44_py() {
        // (u, mano, codo, d, τ hombro, τ codo)
        let man = [
            (0.00, (0.000_000, -0.655_538), (0.046_666, -0.341_146), 0.000_000, 0.0000, 13.7339),
            (0.25, (0.057_672, -0.521_654), (0.231_065, -0.255_280), 0.057_672, 16.9729, 51.0294),
            (0.50, (0.115_344, -0.387_769), (0.314_535, -0.140_094), 0.115_344, 33.9457, 58.6219),
            (0.75, (0.173_016, -0.253_885), (0.344_038, 0.014_018), 0.173_016, 50.9186, 50.3317),
            (1.00, (0.230_688, -0.120_000), (0.285_044, 0.193_154), 0.230_688, 67.8915, 15.9968),
        ];
        let a = mancuerna();
        for &(u, mano, codo, d, th, tc) in &man {
            let q = a.postura(u);
            cerca(q.mano.0, mano.0, 1e-6, "mano.x");
            cerca(q.mano.1, mano.1, 1e-6, "mano.y");
            cerca(q.codo.0, codo.0, 1e-6, "codo.x");
            cerca(q.codo.1, codo.1, 1e-6, "codo.y");
            cerca(palanca_hombro(&a, &q), d, 1e-6, "d");
            cerca(tau(&a, u), th, 1e-3, "τ hombro");
            cerca(tau_codo(&a, u), tc, 1e-3, "τ codo");
        }
        // La máquina: reel44.py arrancaba con la mano 1 cm más allá del brazo (ver
        // `RemoMaquina::abajo`), así que de su oráculo valen la y de la mano, la
        // palanca y el τ del hombro (que solo dependen de la y); la x, el codo y
        // el τ del codo son los del brazo honesto.
        let maq = [(0.00, -0.150_000, 44.1450), (0.25, -0.162_500, 47.8238), (0.50, -0.175_000, 51.5025), (0.75, -0.187_500, 55.1813), (1.00, -0.200_000, 58.8600)];
        let b = maquina();
        for &(u, y, th) in &maq {
            let q = b.postura(u);
            cerca(q.mano.1, y, 1e-6, "mano.y");
            cerca(palanca_hombro(&b, &q), -y, 1e-6, "d");
            cerca(tau(&b, u), th, 1e-3, "τ hombro");
        }
        cerca(b.postura(1.0).mano.0, 0.138_413, 1e-6, "el cierre de la máquina es el de reel44.py");
        cerca(b.postura(0.0).mano.0, 0.638_146, 1e-5, "el arranque, 1.7 cm más cerca que el de reel44.py (0.655538)");
        assert!(b.postura(0.0).mano.0 < 0.655_538);
    }

    /// Los resúmenes de reel44.py: pico, media, trabajo, recorrido y la
    /// fracción floja (lo que la mancuerna va por debajo de la máquina estirada).
    #[test]
    fn los_resumenes_de_reel44_py() {
        let (a, b) = (mancuerna(), maquina());
        cerca(pico(&a, 300), 67.8915, 1e-3, "pico mancuerna");
        cerca(media(&a, 300), 33.9457, 1e-3, "media mancuerna");
        cerca(a.trabajo(), 157.6090, 1e-3, "W mancuerna");
        cerca(a.recorrido(1.0), 0.535_538, 1e-6, "recorrido mancuerna");
        cerca(pico(&b, 300), 58.8600, 1e-3, "pico máquina");
        cerca(media(&b, 300), 51.5025, 1e-3, "media máquina");
        // el recorrido de la máquina es el del brazo honesto (reel44.py: 0.517 m, 152.2 J)
        cerca(b.recorrido(1.0), 0.638_146 - 0.138_413, 1e-5, "recorrido máquina");
        cerca(b.trabajo(), 294.3 * (0.638_146 - 0.138_413), 1e-2, "W máquina");
        assert!((b.trabajo() - 152.1901).abs() < 6.0, "a menos de 6 J del reel viejo");
        cerca(fraccion_por_debajo(&a, tau(&b, 0.0), 1000), 0.6503, 1e-4, "FRAC_FLOJA");
    }

    /// El corazón: la mancuerna da CERO exacto abajo, se incline lo que se
    /// incline el torso (las variantes de reel44.py y alguna más).
    #[test]
    fn a_plomo_del_hombro_no_hay_palanca() {
        for g in [-30.0, -15.0, 0.0, 10.0, 20.0, 35.0] {
            let r = RemoMancuerna { inclina_rad: f64::to_radians(g), ..mancuerna() };
            assert!(tau(&r, 0.0) < 1e-9, "a {g}°: {}", tau(&r, 0.0));
            assert!(palanca_hombro(&r, &r.postura(0.0)) < 1e-15);
            // y la mano queda exactamente bajo el hombro
            assert!(r.postura(0.0).mano.0.abs() < 1e-15);
        }
    }

    /// La máquina con otras profundidades (las variantes de reel44.py): sigue
    /// cargando desde el arranque, y la conclusión aguanta.
    #[test]
    fn la_maquina_no_se_muere_con_otra_profundidad() {
        let esperado = [((0.10, 0.15), 29.43, 44.145), ((0.20, 0.25), 58.86, 73.575), ((0.12, 0.22), 35.316, 64.746), ((0.18, 0.18), 52.974, 52.974)];
        for (prof, estirado, cierre) in esperado {
            let r = RemoMaquina { profundidad: prof, ..maquina() };
            cerca(tau(&r, 0.0), estirado, 1e-3, "estirado");
            cerca(tau(&r, 1.0), cierre, 1e-3, "cierre");
            assert!(tau(&r, 0.0) > 10.0, "carga desde el primer centímetro");
        }
    }

    /// Dos cuentas para la palanca: la de [`palanca::brazo`] y la geometría a
    /// mano (horizontal con la mancuerna, vertical con la máquina).
    #[test]
    fn dos_cuentas_para_la_palanca() {
        let (a, b) = (mancuerna(), maquina());
        for i in 0..=20 {
            let u = f64::from(i) / 20.0;
            let qa = a.postura(u);
            cerca(palanca_hombro(&a, &qa), qa.mano.0.abs(), 1e-12, "mancuerna: la horizontal");
            let qb = b.postura(u);
            cerca(palanca_hombro(&b, &qb), qb.mano.1.abs(), 1e-12, "máquina: la vertical");
            // y los segmentos no se estiran
            for q in [qa, qb] {
                cerca((q.codo.0).hypot(q.codo.1), persona().humero(), 1e-9, "húmero");
                cerca((q.mano.0 - q.codo.0).hypot(q.mano.1 - q.codo.1), persona().antebrazo(), 1e-9, "antebrazo");
            }
        }
    }

    /// Lo que afirma el reel: perfiles espejo (una rampa desde cero, la otra
    /// casi plana), el cruce solo al final, y lo que va por debajo.
    #[test]
    fn lo_que_afirma_el_reel() {
        let (a, b) = (mancuerna(), maquina());
        assert!(tau(&a, 0.0) < 1e-9 && tau(&b, 0.0) > 40.0);
        assert!(tau(&a, 1.0) > tau(&b, 1.0), "al cerrar manda la mancuerna");
        for i in 0..100 {
            let (u0, u1) = (f64::from(i) / 100.0, f64::from(i + 1) / 100.0);
            assert!(tau(&a, u1) > tau(&a, u0) && tau(&b, u1) > tau(&b, u0), "las dos suben");
        }
        // la máquina sube poco: menos de un tercio y medio sobre su arranque
        assert!(tau(&b, 1.0) / tau(&b, 0.0) < 1.4);
        // la mancuerna pasa a la máquina en el último sexto del jalón
        let u = cruce(&a, &b).expect("se cruzan");
        cerca(u, 0.830_160, 1e-5, "cruce (bisección sobre las mismas funciones de reel44.py)");
        cerca(tau(&a, u), 56.3608, 1e-3, "τ en el cruce");
        assert!(((1.0 - u) - 1.0 / 6.0).abs() < 0.01, "el último sexto: {}", 1.0 - u);
        // y no se cruzan al revés
        assert!(cruce(&b, &a).is_some());
        // en promedio, la máquina le pide al hombro una vez y media lo que la mancuerna
        let razon = media(&b, 300) / media(&a, 300);
        assert!((razon - 1.5).abs() < 0.03, "{razon}");
        // el trabajo por repetición, parecido: menos de 8 % aparte
        assert!((a.trabajo() / b.trabajo() - 1.0).abs() < 0.08);
    }
}
