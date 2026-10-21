//! Hip thrust contra peso muerto rumano: el torque en la cadera (y en la
//! rodilla) a lo largo de la repetición, con la barra y con el cuerpo.
//!
//! Del reel 08 de fisicobuenfisico, rehecho. [`crate::gluteo`] reproduce los
//! números del reel viejo (494 y 451 N·m con 100 kg), pero los dos lados
//! salían de una sola palanca: en el rumano el torso entero (la barra a plomo
//! bajo el hombro, que con 100 kg te tira hacia adelante) y en el hip thrust el
//! fémur entero (la barra completa en el pie, como si el banco no cargara
//! nada). Aquí cada uno se resuelve con su equilibrio.
//!
//! # El rumano
//!
//! La barra baja pegada a las piernas, por la vertical del medio pie: con
//! 100 kg en las manos es la única forma de no irte hacia adelante. Las
//! rodillas van suaves con una flexión fija, el torso se inclina (el avance de
//! la repetición) y el ángulo de la espinilla **se resuelve**: el que pone el
//! centro de masa del cuerpo también sobre el medio pie (bisección). Por eso
//! la cadera se va hacia atrás sin que nadie lo pida. La palanca de la barra
//! es la distancia horizontal de la cadera al medio pie.
//!
//! # El hip thrust
//!
//! La espalda en el banco, los pies en el piso y la barra sobre la cadera. Es
//! un puente con dos apoyos: el banco (en la línea del hombro) y el medio pie.
//! Si los dos solo empujan hacia arriba (sin fricción: la espalda rueda sobre
//! el canto del banco), la barra se reparte como en una viga simplemente
//! apoyada: con `a` la distancia horizontal del banco a la cadera y `b` la de
//! la cadera al pie, el pie empuja `F = W · a / (a + b)` y el torque en la
//! cadera es `τ = F · b = W · a · b / (a + b)`: el τ = F · d de siempre, con la
//! fuerza del pie y su distancia horizontal a la cadera. Torso y pierna forman
//! un cuadrilátero articulado (banco–cadera–rodilla–tobillo, con el banco y el
//! tobillo fijos): la flexión de la cadera lo mueve y el resto se resuelve con
//! [`garust`]. Con la cadera extendida la espinilla queda vertical: así se
//! colocan los pies.
//!
//! # Convención
//!
//! [`Cargas`] es lo de la BARRA sola; [`Bisagra::cuerpo`] suma el peso del
//! propio cuerpo (Winter, segmento por segmento), que en el rumano es el torso
//! que cuelga delante de la cadera. Positivo en la cadera es lo que tienen que
//! vencer sus extensores. En la rodilla, positivo pide a los extensores y
//! negativo a los flexores (el apoyo delante de la rodilla). 2D,
//! cuasi-estático, las dos piernas juntas, con las fracciones de
//! [`crate::maquina_humana`]. La cadera es la articulación (trocánter): el
//! torso va de ella al hombro.

use garust::twolink::two_link_joint;

use crate::maquina_humana::{centro, largo, masa, Persona};
use crate::palanca::Punto;
use crate::G;

/// El medio pie: delante del tobillo, fracción de la estatura (supuesto: la
/// mitad de un pie de 0.152 H que arranca 0.030 H detrás del tobillo).
pub const MEDIO_PIE: f64 = 0.046;
/// El centro de masa del brazo, del hombro hacia la mano (fracción; supuesto).
pub const CENTRO_BRAZO: f64 = 0.5;

/// Iteraciones de las bisecciones (sobran: 2⁻⁶⁰ de cualquier rango humano).
const BISECCION: usize = 60;

/// Una postura: dónde quedan las articulaciones y la carga, en metros (x hacia
/// las puntas de los pies, y hacia arriba).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Postura {
    pub tobillo: Punto,
    pub rodilla: Punto,
    pub cadera: Punto,
    pub hombro: Punto,
    /// El centro de la barra.
    pub barra: Punto,
    /// Dónde empuja el piso: el medio pie (en el piso).
    pub pie: Punto,
    /// La flexión de la cadera (0: torso y muslo en línea), radianes.
    pub flexion: f64,
}

impl Postura {
    /// La cabeza (su centro de masa): más allá del hombro, a lo largo del eje
    /// cadera → hombro.
    #[must_use]
    pub fn cabeza(&self, p: &Persona) -> Punto {
        let k = largo::CABEZA * p.estatura_m / p.torso();
        (self.hombro.0 + (self.hombro.0 - self.cadera.0) * k, self.hombro.1 + (self.hombro.1 - self.cadera.1) * k)
    }
}

/// Lo que la barra le pide a cada articulación en una postura.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cargas {
    /// La fuerza del τ = F · d de la cadera, N: el peso de la barra (rumano) o
    /// lo que empuja el pie (hip thrust).
    pub fuerza: f64,
    /// La palanca de esa fuerza respecto a la cadera (horizontal), m.
    pub palanca: f64,
    /// τ en la cadera, N·m.
    pub cadera: f64,
    /// τ en la rodilla, N·m (positivo: extensores; negativo: flexores).
    pub rodilla: f64,
    /// Lo que carga el banco, N (cero en el rumano).
    pub banco: f64,
}

/// Una de las dos: una postura por avance `u` (0: la cadera extendida, de pie
/// o en el bloqueo; 1: abajo) y lo que pide la carga en ella.
pub trait Bisagra {
    fn postura(&self, u: f64) -> Postura;
    /// Lo de la barra sola.
    fn cargas(&self, u: f64) -> Cargas;
    /// Lo que pide el peso del propio cuerpo, N·m: (cadera, rodilla).
    fn cuerpo(&self, u: f64) -> (f64, f64);
    /// El peso de la barra, N.
    fn peso(&self) -> f64;
    /// Barra y cuerpo, N·m: (cadera, rodilla).
    fn total(&self, u: f64) -> (f64, f64) {
        let (c, (bc, br)) = (self.cargas(u), self.cuerpo(u));
        (c.cadera + bc, c.rodilla + br)
    }
}

/// Los pesos del cuerpo en una postura: (N, x en m), los pares dos veces:
/// pies, piernas, muslos, pelvis, tronco, cabeza y brazos (en `brazos`).
fn pesos(p: &Persona, q: &Postura, brazos: f64) -> [(f64, f64); 7] {
    let mg = p.masa_kg * G;
    let en = |a: Punto, b: Punto, f: f64| a.0 + (b.0 - a.0) * f;
    [
        (2.0 * masa::PIE * mg, q.pie.0),
        (2.0 * masa::TIBIA * mg, en(q.rodilla, q.tobillo, centro::TIBIA)),
        (2.0 * masa::FEMUR * mg, en(q.cadera, q.rodilla, centro::FEMUR)),
        (masa::PELVIS * mg, q.cadera.0),
        (masa::TRONCO * mg, en(q.cadera, q.hombro, centro::TRONCO)),
        (masa::CABEZA * mg, q.cabeza(p).0),
        (2.0 * masa::BRAZO * mg, brazos),
    ]
}

/// Un punto a `l` de `p` en la dirección que forma `ang` con la vertical
/// hacia arriba (positivo hacia +x).
fn desde(p: Punto, l: f64, ang: f64) -> Punto {
    (p.0 + l * ang.sin(), p.1 + l * ang.cos())
}

// ------------------------------------------------------------------ el rumano

/// El peso muerto rumano.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rumano {
    pub persona: Persona,
    pub carga_kg: f64,
    /// Lo que se inclina el torso abajo, contra la vertical, radianes.
    pub torso_max_rad: f64,
    /// La flexión de la rodilla, fija (rodillas suaves), radianes.
    pub rodilla_rad: f64,
}

impl Rumano {
    /// El tobillo: el origen en x.
    #[must_use]
    pub fn tobillo(&self) -> Punto {
        (0.0, largo::TOBILLO * self.persona.estatura_m)
    }
    /// La línea de la barra: la vertical del medio pie (x, m).
    #[must_use]
    pub fn medio_pie(&self) -> f64 {
        self.tobillo().0 + MEDIO_PIE * self.persona.estatura_m
    }
    /// La postura con el torso a `beta` de la vertical y la espinilla a
    /// `tibia` (hacia adelante si es positivo). La barra baja por la vertical
    /// del medio pie, al largo de los brazos desde el hombro.
    ///
    /// # Panics
    /// Si el hombro queda tan lejos del medio pie que los brazos no alcanzan.
    #[must_use]
    pub fn con(&self, beta: f64, tibia: f64) -> Postura {
        let p = &self.persona;
        let tob = self.tobillo();
        let rod = desde(tob, p.tibia(), tibia);
        let muslo = self.rodilla_rad - tibia;
        let cad = desde(rod, p.femur(), -muslo);
        let hom = desde(cad, p.torso(), beta);
        let x = self.medio_pie();
        let dx = hom.0 - x;
        assert!(dx.abs() < p.brazo(), "la barra no alcanza el medio pie");
        let barra = (x, hom.1 - (p.brazo() * p.brazo() - dx * dx).sqrt());
        Postura { tobillo: tob, rodilla: rod, cadera: cad, hombro: hom, barra, pie: (x, 0.0), flexion: beta + muslo }
    }
    fn brazos(q: &Postura) -> f64 {
        q.hombro.0 + (q.barra.0 - q.hombro.0) * CENTRO_BRAZO
    }
    /// La x del centro de masa del cuerpo, sin la barra (m).
    #[must_use]
    pub fn centro_x(&self, q: &Postura) -> f64 {
        let w = pesos(&self.persona, q, Self::brazos(q));
        w.iter().map(|(f, x)| f * x).sum::<f64>() / w.iter().map(|(f, _)| f).sum::<f64>()
    }
}

impl Bisagra for Rumano {
    /// El torso a `u · torso_max` de la vertical; la espinilla, la que pone el
    /// centro de masa del cuerpo sobre el medio pie, como la barra: así el de
    /// cuerpo y barra también queda ahí.
    fn postura(&self, u: f64) -> Postura {
        let beta = u.clamp(0.0, 1.0) * self.torso_max_rad;
        let meta = self.medio_pie();
        // la espinilla adelante lleva todo adelante: el centro de masa sube en x
        let (mut lo, mut hi) = ((-25.0_f64).to_radians(), 25.0_f64.to_radians());
        let f = |s: f64| self.centro_x(&self.con(beta, s)) - meta;
        assert!(f(lo) < 0.0 && f(hi) > 0.0, "el rumano no se equilibra con el torso a {:.1}°", beta.to_degrees());
        for _ in 0..BISECCION {
            let mid = 0.5 * (lo + hi);
            if f(mid) < 0.0 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        self.con(beta, 0.5 * (lo + hi))
    }
    fn cargas(&self, u: f64) -> Cargas {
        let q = self.postura(u);
        let w = self.peso();
        let palanca = q.barra.0 - q.cadera.0;
        Cargas { fuerza: w, palanca, cadera: w * palanca, rodilla: w * (q.rodilla.0 - q.barra.0), banco: 0.0 }
    }
    /// Lo de arriba de cada articulación por su distancia horizontal a ella:
    /// de la cadera, el tronco, la cabeza y los brazos; de la rodilla, además,
    /// la pelvis y los muslos.
    fn cuerpo(&self, u: f64) -> (f64, f64) {
        let q = self.postura(u);
        let w = pesos(&self.persona, &q, Self::brazos(&q));
        let cad: f64 = w[4..].iter().map(|(f, x)| f * (x - q.cadera.0)).sum();
        let rod: f64 = w[2..].iter().map(|(f, x)| f * (q.rodilla.0 - x)).sum();
        (cad, rod)
    }
    fn peso(&self) -> f64 {
        self.carga_kg * G
    }
}

// ------------------------------------------------------------------ el hip thrust

/// El hip thrust.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HipThrust {
    pub persona: Persona,
    pub carga_kg: f64,
    /// La altura del apoyo en el banco (la línea del hombro), m.
    pub banco_m: f64,
    /// La flexión de la cadera abajo, radianes.
    pub flexion_max_rad: f64,
}

/// El reparto del puente: (lo que empuja el pie, lo que empuja el banco), N.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Puente {
    pub pie: f64,
    pub banco: f64,
}

impl HipThrust {
    /// El hombro, apoyado en el banco: el origen en x.
    #[must_use]
    pub fn hombro(&self) -> Punto {
        (0.0, self.banco_m)
    }
    /// El tobillo, fijo: donde la espinilla queda vertical con la cadera
    /// extendida (torso y muslo en línea).
    #[must_use]
    pub fn tobillo(&self) -> Punto {
        let p = &self.persona;
        let y_t = largo::TOBILLO * p.estatura_m;
        let l = p.torso() + p.femur();
        let sube = y_t + p.tibia() - self.banco_m;
        ((l * l - sube * sube).sqrt(), y_t)
    }
    /// La postura con la cadera doblada `phi` (radianes).
    ///
    /// # Panics
    /// Si el cuadrilátero no cierra (una flexión fuera de su alcance).
    #[must_use]
    pub fn con_flexion(&self, phi: f64) -> Postura {
        let p = &self.persona;
        let (s, a) = (self.hombro(), self.tobillo());
        let (lt, lf) = (p.torso(), p.femur());
        // banco → rodilla: el ángulo interior en la cadera es π − φ
        let sk = (lt * lt + lf * lf + 2.0 * lt * lf * phi.cos()).sqrt();
        let r = two_link_joint([s.0, s.1, 0.0], [a.0, a.1, 0.0], sk, p.tibia(), [0.0, 1.0, 0.0]).expect("la rodilla");
        let rod = (r[0], r[1]);
        let c = two_link_joint([s.0, s.1, 0.0], [rod.0, rod.1, 0.0], lt, lf, [0.0, -1.0, 0.0]).expect("la cadera");
        let cad = (c[0], c[1]);
        Postura { tobillo: a, rodilla: rod, cadera: cad, hombro: s, barra: cad, pie: (a.0 + MEDIO_PIE * p.estatura_m, 0.0), flexion: phi }
    }
    /// El reparto de una carga `w` puesta en la cadera, con los pesos del
    /// cuerpo si `con_cuerpo`: momentos alrededor de la cadera de cada lado.
    #[must_use]
    pub fn puente(&self, q: &Postura, w: f64, con_cuerpo: bool) -> Puente {
        let (a, b) = (q.cadera.0 - q.hombro.0, q.pie.0 - q.cadera.0);
        let (total, banco_lado, pie_lado) = if con_cuerpo {
            let p = pesos(&self.persona, q, Self::brazos(q));
            (
                w + p.iter().map(|(f, _)| f).sum::<f64>(),
                p[4..].iter().map(|(f, x)| f * (q.cadera.0 - x)).sum::<f64>(),
                p[..3].iter().map(|(f, x)| f * (x - q.cadera.0)).sum::<f64>(),
            )
        } else {
            (w, 0.0, 0.0)
        };
        let pie = (total * a - banco_lado + pie_lado) / (a + b);
        Puente { pie, banco: total - pie }
    }
    /// Los brazos, de las manos en la barra (sobre la cadera) al hombro.
    fn brazos(q: &Postura) -> f64 {
        0.5 * (q.hombro.0 + q.cadera.0)
    }
}

impl Bisagra for HipThrust {
    fn postura(&self, u: f64) -> Postura {
        self.con_flexion(u.clamp(0.0, 1.0) * self.flexion_max_rad)
    }
    fn cargas(&self, u: f64) -> Cargas {
        let q = self.postura(u);
        let pt = self.puente(&q, self.peso(), false);
        let b = q.pie.0 - q.cadera.0;
        Cargas { fuerza: pt.pie, palanca: b, cadera: pt.pie * b, rodilla: pt.pie * (q.rodilla.0 - q.pie.0), banco: pt.banco }
    }
    /// El puente con los pesos del cuerpo y sin la barra: del lado del banco
    /// el tronco, la cabeza y los brazos; del lado del pie, los muslos, las
    /// piernas y los pies. La cadera, con lo del lado del pie; la rodilla, con
    /// lo de abajo de ella (el pie y su pierna).
    fn cuerpo(&self, u: f64) -> (f64, f64) {
        let q = self.postura(u);
        let w = pesos(&self.persona, &q, Self::brazos(&q));
        let pie = self.puente(&q, 0.0, true).pie;
        let cad = pie * (q.pie.0 - q.cadera.0) - w[..3].iter().map(|(f, x)| f * (x - q.cadera.0)).sum::<f64>();
        let rod = pie * (q.rodilla.0 - q.pie.0) - w[..2].iter().map(|(f, x)| f * (q.rodilla.0 - x)).sum::<f64>();
        (cad, rod)
    }
    fn peso(&self) -> f64 {
        self.carga_kg * G
    }
}

// ------------------------------------------------------------------ la comparación

/// El máximo de `f(u)` en `n + 1` muestras: (u, valor).
pub fn maximo(f: impl Fn(f64) -> f64, n: usize) -> (f64, f64) {
    (0..=n).map(|i| i as f64 / n as f64).map(|u| (u, f(u))).fold((0.0, f64::MIN), |m, x| if x.1 > m.1 { x } else { m })
}

/// El mínimo de `f(u)` en `n + 1` muestras: (u, valor).
pub fn minimo(f: impl Fn(f64) -> f64, n: usize) -> (f64, f64) {
    let (u, v) = maximo(|u| -f(u), n);
    (u, -v)
}

/// El primer avance `u` donde `f`, que sube, pasa de `umbral` (bisección);
/// `None` si no llega.
pub fn donde_pasa(f: impl Fn(f64) -> f64, umbral: f64) -> Option<f64> {
    if f(1.0) < umbral {
        return None;
    }
    let (mut lo, mut hi) = (0.0, 1.0);
    for _ in 0..BISECCION {
        let mid = 0.5 * (lo + hi);
        if f(mid) < umbral {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some(0.5 * (lo + hi))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn persona() -> Persona {
        Persona { estatura_m: 1.78, masa_kg: 78.0 }
    }
    fn rumano() -> Rumano {
        Rumano { persona: persona(), carga_kg: 100.0, torso_max_rad: 75f64.to_radians(), rodilla_rad: 20f64.to_radians() }
    }
    fn hip() -> HipThrust {
        HipThrust { persona: persona(), carga_kg: 100.0, banco_m: 0.45, flexion_max_rad: 75f64.to_radians() }
    }
    fn us() -> impl Iterator<Item = f64> {
        (0..=40).map(|i| f64::from(i) / 40.0)
    }

    /// Los largos salen de Winter y no se estiran: el cuadrilátero y la
    /// cadena del rumano los respetan en toda la repetición.
    #[test]
    fn los_segmentos_no_se_estiran() {
        let p = persona();
        let l = |a: Punto, b: Punto| (a.0 - b.0).hypot(a.1 - b.1);
        for u in us() {
            for q in [rumano().postura(u), hip().postura(u)] {
                assert!((l(q.tobillo, q.rodilla) - p.tibia()).abs() < 1e-9);
                assert!((l(q.rodilla, q.cadera) - p.femur()).abs() < 1e-9);
                assert!((l(q.cadera, q.hombro) - p.torso()).abs() < 1e-9);
            }
            let q = rumano().postura(u);
            assert!((l(q.hombro, q.barra) - p.brazo()).abs() < 1e-9, "los brazos llegan a la barra");
        }
    }

    /// La flexión de la cadera es el ángulo entre el torso y el muslo, medido
    /// en la postura (no la que se pidió).
    #[test]
    fn la_flexion_se_mide_en_la_postura() {
        for u in us() {
            for q in [rumano().postura(u), hip().postura(u)] {
                let t = (q.hombro.0 - q.cadera.0, q.hombro.1 - q.cadera.1);
                let m = (q.cadera.0 - q.rodilla.0, q.cadera.1 - q.rodilla.1);
                let ang = (t.0 * m.1 - t.1 * m.0).atan2(t.0 * m.0 + t.1 * m.1).abs();
                assert!((ang - q.flexion).abs() < 1e-9, "u = {u}: {ang} contra {}", q.flexion);
            }
        }
    }

    /// El rumano se equilibra: el centro de masa de cuerpo y barra sobre el
    /// medio pie, y la barra en su vertical.
    #[test]
    fn el_rumano_se_equilibra() {
        let r = rumano();
        for u in us() {
            let q = r.postura(u);
            let m = persona().masa_kg;
            let todo = (m * r.centro_x(&q) + r.carga_kg * q.barra.0) / (m + r.carga_kg);
            assert!((todo - r.medio_pie()).abs() < 1e-9);
            assert!((q.barra.0 - r.medio_pie()).abs() < 1e-12);
        }
    }

    /// De pie, la barra casi pasa por la cadera: el torque es casi cero.
    #[test]
    fn de_pie_casi_nada() {
        assert!(rumano().cargas(0.0).cadera.abs() < 5.0);
        assert!(rumano().total(0.0).0.abs() < 5.0);
    }

    /// Dos cuentas para la cadera del rumano: por arriba (barra, tronco,
    /// cabeza y brazos) y por abajo (el piso en el medio pie menos las
    /// piernas). Datos distintos, el mismo número.
    #[test]
    fn dos_cuentas_para_la_cadera_del_rumano() {
        let r = rumano();
        for u in us() {
            let q = r.postura(u);
            let w = pesos(&r.persona, &q, Rumano::brazos(&q));
            let todo = r.peso() + w.iter().map(|(f, _)| f).sum::<f64>();
            let abajo = todo * (q.pie.0 - q.cadera.0) - w[..3].iter().map(|(f, x)| f * (x - q.cadera.0)).sum::<f64>();
            assert!((abajo - r.total(u).0).abs() < 1e-6, "u = {u}");
        }
    }

    /// El puente cierra: fuerzas y momentos de todo el cuerpo en equilibrio,
    /// y las dos cuentas de la cadera (lado del banco, lado del pie) iguales.
    #[test]
    fn el_puente_cierra() {
        let h = hip();
        for u in us() {
            let q = h.postura(u);
            let w = pesos(&h.persona, &q, HipThrust::brazos(&q));
            let pt = h.puente(&q, h.peso(), true);
            let peso: f64 = h.peso() + w.iter().map(|(f, _)| f).sum::<f64>();
            assert!((pt.pie + pt.banco - peso).abs() < 1e-9);
            let m = pt.pie * q.pie.0 + pt.banco * q.hombro.0 - h.peso() * q.cadera.0 - w.iter().map(|(f, x)| f * x).sum::<f64>();
            assert!(m.abs() < 1e-9, "momento neto {m}");
            let banco_lado = pt.banco * (q.cadera.0 - q.hombro.0) - w[4..].iter().map(|(f, x)| f * (q.cadera.0 - x)).sum::<f64>();
            assert!((banco_lado - h.total(u).0).abs() < 1e-6, "u = {u}");
        }
    }

    /// El oráculo de la viga: con la barra sola, τ = W · a · b / (a + b), y la
    /// fuerza del pie por su palanca es lo mismo que la del banco por la suya.
    #[test]
    fn la_viga_simplemente_apoyada() {
        let h = hip();
        for u in us() {
            let (q, c) = (h.postura(u), h.cargas(u));
            let (a, b) = (q.cadera.0 - q.hombro.0, q.pie.0 - q.cadera.0);
            assert!((c.cadera - h.peso() * a * b / (a + b)).abs() < 1e-9);
            assert!((c.banco * a - c.fuerza * c.palanca).abs() < 1e-9);
        }
    }

    /// Lo que el reel nuevo corrige del viejo: con los dos apoyos, el hip
    /// thrust da mucho menos que la barra entera en el fémur (451 N·m).
    #[test]
    fn menos_que_el_femur_entero() {
        let (_, pico) = maximo(|u| hip().cargas(u).cadera, 300);
        assert!(pico < 0.6 * 451.0, "{pico}");
    }

    /// La forma de las curvas: el rumano sube de cero hasta abajo; el hip
    /// thrust casi no cambia (menos de 3 % con barra y cuerpo).
    #[test]
    fn la_forma_de_las_curvas() {
        let (r, h) = (rumano(), hip());
        for w in us().collect::<Vec<_>>().windows(2) {
            assert!(r.total(w[1]).0 > r.total(w[0]).0, "el rumano sube");
        }
        let (_, alto) = maximo(|u| h.total(u).0, 300);
        let (_, bajo) = minimo(|u| h.total(u).0, 300);
        assert!(alto / bajo < 1.03, "{alto} / {bajo}");
    }
}
