//! Dominada contra jalón: la misma fuerza en la mano, otra palanca.
//!
//! Del reel 42 de fisicobuenfisico (`tools/reel42.py` allá), portado con
//! sus mismos supuestos y probado contra sus números. La pregunta era «qué
//! diferencia hay entre hacer dominadas pesando 78 kg y jalar 78 kg en
//! polea», y la respuesta del modelo tiene dos mitades:
//!
//! 1. **En la mano, ninguna.** La barra te sostiene con tu peso completo
//!    (los brazos incluidos), y la placa jala con el suyo: 765 N los dos.
//!    Es cierto que del hombro cuelga solo el 90 % de tu masa (los brazos
//!    son el otro 10 %), pero eso es la fuerza que cruza la articulación,
//!    no la que sostiene la mano.
//! 2. **En la palanca, mucha.** En la dominada la barra es el ÚNICO apoyo,
//!    así que el centro de masa de todo el cuerpo tiene que quedar en su
//!    vertical: no es técnica, es la condición de equilibrio. Con eso el
//!    hombro también anda bajo la barra, casi sin palanca, y la única
//!    forma de dársela es reclinarse. En el jalón las rodilleras son un
//!    segundo apoyo, y la barra cuelga delante del hombro toda la bajada.
//!
//! # Lo que se resuelve y lo que se declara
//!
//! Se declara cuánto se reclina el cuerpo (de 0 colgado a
//! [`Dominada::reclinado_max_rad`] arriba, en proporción a la subida) y
//! hasta dónde sube (la barbilla a la barra). La posición horizontal del
//! hombro **se resuelve**: la que pone el centro de masa de todo el cuerpo
//! en la vertical de la barra, por bisección. Por eso el colgado da 0 N·m
//! en el hombro sin que nadie lo pida.
//!
//! # Dos cuentas para el mismo número
//!
//! El torque del hombro se puede contar por el lado del BRAZO (la reacción
//! de la barra en la mano menos los pesos del propio brazo) o por el lado
//! del TRONCO (el cuerpo sin brazos por su palanca al reclinarse). Son
//! datos distintos y tienen que dar lo mismo: [`Dominada::tau_tronco`] y
//! la prueba `dos_cuentas_para_el_hombro`.
//!
//! # Convención
//!
//! Lo que se compara es el τ de la CARGA sola (la fuerza en la mano), sin
//! el peso del propio brazo, igual que en [`crate::jalon`]: por eso el
//! jalón da los 118 N·m del reel 26 con 50 kg. El balance de energía sí
//! usa todos los pesos, porque ahí no se puede tirar nada. Los números son
//! de las DOS manos juntas: un solo brazo en el plano sagital con la masa
//! de los dos.
//!
//! # El jalón reclinado
//!
//! [`JalonReclinado`] es el mismo jalón con el torso echado atrás (el reel 29
//! de fisicobuenfisico, «¿derecho o reclinado?»). Ahí el cable deja de colgar
//! a plomo y apunta a la polea, así que la fuerza de la mano lleva su línea
//! ([`Repeticion::polea`]) y la palanca es la distancia a ESA línea
//! ([`palanca`]). La dominada y el jalón derecho no la usan: sus números no
//! cambian.
//!
//! El modelo es 2D, cuasi-estático, con el cuerpo de la dominada rígido del
//! hombro al pie y las fracciones de Winter de [`crate::maquina_humana`].

use crate::maquina_humana::{centro, codo, largo, masa, Doblez, Persona};
use crate::palanca::{self, Linea, Punto};
use crate::G;

/// Winter: la altura de la barbilla, fracción de la estatura.
pub const ALTURA_BARBILLA: f64 = 0.870;
/// Winter: la altura del acromion (el hombro), fracción de la estatura.
pub const ALTURA_HOMBRO: f64 = 0.818;

/// Iteraciones de la bisección del hombro (las de reel42.py: 1.2 m / 2⁴⁶).
const BISECCION: usize = 46;

/// Una postura: dónde quedan las articulaciones, en metros.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    /// El hombro.
    pub hombro: Punto,
    /// El codo.
    pub codo: Punto,
    /// La mano (el agarre).
    pub mano: Punto,
    /// Cuánto se reclina el cuerpo contra la vertical, radianes.
    pub reclinado: f64,
    /// El eje del cuerpo, unitario, del hombro hacia la cadera.
    pub eje: Punto,
    /// L5/S1.
    pub cadera: Punto,
}

/// Qué articulación.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Articulacion {
    /// El hombro.
    Hombro,
    /// El codo.
    Codo,
}

/// |τ| en el hombro y en el codo, N·m.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Torques {
    /// En el hombro.
    pub hombro: f64,
    /// En el codo: solo ve lo que le cuelga (la mano y el antebrazo).
    pub codo: f64,
}

impl Torques {
    /// El de una articulación.
    #[must_use]
    pub fn de(&self, a: Articulacion) -> f64 {
        match a {
            Articulacion::Hombro => self.hombro,
            Articulacion::Codo => self.codo,
        }
    }
}

/// El cuerpo SIN brazos, segmento por segmento: (fracción de masa, cuánto
/// cuelga bajo el hombro a lo largo del eje del cuerpo, m). La cabeza va
/// en negativo porque está arriba del hombro.
#[must_use]
pub fn cuerpo_sin_brazos(p: &Persona) -> [(f64, f64); 6] {
    let (t, f, s) = (p.torso(), p.femur(), p.tibia());
    [
        (masa::CABEZA, -largo::CABEZA * p.estatura_m),
        (masa::TRONCO, (1.0 - centro::TRONCO) * t),
        (masa::PELVIS, t),
        (2.0 * masa::FEMUR, t + centro::FEMUR * f),
        (2.0 * masa::TIBIA, t + f + centro::TIBIA * s),
        (2.0 * masa::PIE, t + f + s),
    ]
}

/// La fracción de masa del cuerpo sin brazos (0.9: los brazos son el 10 %).
#[must_use]
pub fn masa_sin_brazos(p: &Persona) -> f64 {
    cuerpo_sin_brazos(p).iter().map(|(m, _)| m).sum()
}

/// Cuánto cuelga bajo el hombro el centro de masa del cuerpo sin brazos, m.
#[must_use]
pub fn centro_sin_brazos(p: &Persona) -> f64 {
    let c = cuerpo_sin_brazos(p);
    c.iter().map(|(m, d)| m * d).sum::<f64>() / masa_sin_brazos(p)
}

/// La mitad de un segmento.
fn medio(a: Punto, b: Punto) -> Punto {
    ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0)
}

/// El codo de la persona, con su doblez.
fn codo_de(p: &Persona, hombro: Punto, mano: Punto, hacia: Doblez) -> Punto {
    let c = codo(p, [hombro.0, hombro.1], [mano.0, mano.1], hacia)
        .expect("el codo de la dominada y del jalón tiene lado");
    (c[0], c[1])
}

/// Una repetición: cualquiera de las dos, vista igual por las cuentas.
pub trait Repeticion {
    /// La persona que la hace.
    fn persona(&self) -> &Persona;
    /// La postura en el avance `u` (0 al arrancar, 1 al final de la subida
    /// o de la bajada).
    fn pose(&self, u: f64) -> Pose;
    /// La fuerza en la mano, N, vertical y hacia arriba: la barra que te
    /// sostiene o el cable que jala la placa.
    fn fuerza_mano(&self) -> f64;
    /// Lo que se ha jalado en el avance `u`, m: lo que sube el hombro en la
    /// dominada, lo que baja la barra en el jalón.
    fn recorrido(&self, u: f64) -> f64;
    /// La energía potencial de todo lo que se mueve, J (para el balance).
    fn potencial(&self, u: f64) -> f64;
    /// El trabajo de la carga por repetición, J: la carga por lo que viaja
    /// en su línea de acción.
    fn trabajo(&self) -> f64;
    /// La polea hacia la que jala la mano, si su cable NO cuelga a plomo (el
    /// jalón reclinado). `None`: la fuerza de la mano es vertical (la barra de
    /// la dominada, o el cable a plomo del jalón derecho).
    fn polea(&self) -> Option<Punto> {
        None
    }
}

/// La dominada. La barra está en el origen y es el ÚNICO apoyo.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dominada {
    /// Quién cuelga: su peso es la carga.
    pub persona: Persona,
    /// Cuánto se reclina arriba, radianes (0 colgado; crece con la subida).
    pub reclinado_max_rad: f64,
}

impl Dominada {
    /// Lo que sube el hombro, m: del brazo estirado a la barbilla en la barra.
    #[must_use]
    pub fn subida(&self) -> f64 {
        self.persona.brazo() - (ALTURA_BARBILLA - ALTURA_HOMBRO) * self.persona.estatura_m
    }

    /// La posición horizontal del centro de masa de TODO el cuerpo, con el
    /// hombro en `(xs, ys)` y el cuerpo reclinado `lam`, en fracciones de
    /// masa por metro. La barra está en el origen: esto tiene que valer cero.
    fn com_x(&self, xs: f64, ys: f64, lam: f64) -> f64 {
        let p = &self.persona;
        let cod = codo_de(p, (xs, ys), (0.0, 0.0), Doblez::Adelante);
        let cuerpo = masa_sin_brazos(p) * (xs + centro_sin_brazos(p) * lam.sin());
        let brazos = masa::BRAZO * (xs + cod.0) / 2.0 + masa::BRAZO * cod.0 / 2.0;
        cuerpo + brazos
    }

    /// El MISMO torque del hombro (con los pesos del brazo), contado por el
    /// lado del tronco: el cuerpo sin brazos por su palanca al reclinarse.
    #[must_use]
    pub fn tau_tronco(&self, pose: &Pose) -> f64 {
        let p = &self.persona;
        masa_sin_brazos(p) * p.masa_kg * G * centro_sin_brazos(p) * pose.reclinado.sin()
    }

    /// La altura del centro de masa de todo el cuerpo en el avance `u`, m.
    #[must_use]
    pub fn centro_y(&self, u: f64) -> f64 {
        let pose = self.pose(u);
        let cy: f64 = cuerpo_sin_brazos(&self.persona)
            .iter()
            .map(|(m, d)| m * (pose.hombro.1 - d * pose.reclinado.cos()))
            .sum();
        cy + masa::BRAZO * medio(pose.hombro, pose.codo).1
            + masa::BRAZO * medio(pose.codo, pose.mano).1
    }

    /// A cuántos grados de reclinado arriba el hombro de la dominada iguala,
    /// en su punto más alto, al del jalón (los dos de la carga sola), por
    /// bisección. reel42.py daba 34.8° comparando el torque contado por el
    /// tronco (con los pesos del brazo) contra el del jalón sin ellos: dos
    /// convenciones en una cuenta. Esta usa una sola.
    #[must_use]
    pub fn reclinado_de_cruce(&self, jalon: &JalonPlomo) -> f64 {
        let meta = tau(jalon, Articulacion::Hombro, 0.0);
        let f = |grados: f64| {
            tau(
                &Dominada {
                    reclinado_max_rad: grados.to_radians(),
                    ..*self
                },
                Articulacion::Hombro,
                1.0,
            ) - meta
        };
        let (mut lo, mut hi) = (0.0_f64, 80.0_f64);
        assert!(f(lo) < 0.0 && f(hi) > 0.0, "sin cruce entre 0° y 80°");
        for _ in 0..60 {
            let mid = 0.5 * (lo + hi);
            if f(mid) < 0.0 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        0.5 * (lo + hi)
    }
}

impl Repeticion for Dominada {
    fn persona(&self) -> &Persona {
        &self.persona
    }

    /// Lo declarado: cuánto se reclina y cuánto sube. Lo demás se resuelve:
    /// el hombro va donde pone el centro de masa en la vertical de la barra.
    fn pose(&self, u: f64) -> Pose {
        let p = &self.persona;
        let lam = self.reclinado_max_rad * u;
        let ys = -p.brazo() + self.subida() * u;
        let (mut lo, mut hi) = (-0.60_f64, 0.60_f64);
        let mut f_lo = self.com_x(lo, ys, lam);
        for _ in 0..BISECCION {
            let mid = 0.5 * (lo + hi);
            let f_mid = self.com_x(mid, ys, lam);
            if f_lo * f_mid <= 0.0 {
                hi = mid;
            } else {
                lo = mid;
                f_lo = f_mid;
            }
        }
        let hombro = (0.5 * (lo + hi), ys);
        let eje = (lam.sin(), -lam.cos());
        Pose {
            hombro,
            codo: codo_de(p, hombro, (0.0, 0.0), Doblez::Adelante),
            mano: (0.0, 0.0),
            reclinado: lam,
            eje,
            cadera: (hombro.0 + eje.0 * p.torso(), hombro.1 + eje.1 * p.torso()),
        }
    }

    fn fuerza_mano(&self) -> f64 {
        self.persona.masa_kg * G
    }

    fn recorrido(&self, u: f64) -> f64 {
        self.pose(u).hombro.1 - self.pose(0.0).hombro.1
    }

    fn potencial(&self, u: f64) -> f64 {
        let pose = self.pose(u);
        let p = &self.persona;
        let w = masa::BRAZO * p.masa_kg * G;
        let brazo = w * medio(pose.hombro, pose.codo).1 + w * medio(pose.codo, pose.mano).1;
        brazo
            + cuerpo_sin_brazos(p)
                .iter()
                .map(|(m, d)| m * p.masa_kg * G * (pose.hombro.1 - d * pose.reclinado.cos()))
                .sum::<f64>()
    }

    fn trabajo(&self) -> f64 {
        self.fuerza_mano() * (self.centro_y(1.0) - self.centro_y(0.0))
    }
}

/// El jalón sentado, con el torso derecho y el cable a plomo: L5/S1 en el
/// origen. La barra baja en la vertical de la polea, así que la palanca del
/// hombro es la distancia horizontal hombro-barra y no se mueve.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JalonPlomo {
    /// Quién jala (su brazo pesa en el balance).
    pub persona: Persona,
    /// La placa, kg.
    pub carga_kg: f64,
    /// La polea, en el marco de L5/S1.
    pub polea: Punto,
    /// Dónde arranca la barra (arriba) y dónde acaba, alturas sobre L5/S1, m.
    pub y_alta: f64,
    /// Ídem, abajo.
    pub y_baja: f64,
}

impl JalonPlomo {
    /// El hombro, a plomo de L5/S1.
    #[must_use]
    pub fn hombro(&self) -> Punto {
        (0.0, self.persona.torso())
    }
}

impl Repeticion for JalonPlomo {
    fn persona(&self) -> &Persona {
        &self.persona
    }

    fn pose(&self, u: f64) -> Pose {
        let hombro = self.hombro();
        let mano = (self.polea.0, self.y_alta + (self.y_baja - self.y_alta) * u);
        Pose {
            hombro,
            codo: codo_de(&self.persona, hombro, mano, Doblez::Abajo),
            mano,
            reclinado: 0.0,
            eje: (0.0, -1.0),
            cadera: (0.0, 0.0),
        }
    }

    fn fuerza_mano(&self) -> f64 {
        palanca::peso(self.carga_kg)
    }

    fn recorrido(&self, u: f64) -> f64 {
        self.y_alta - self.pose(u).mano.1
    }

    fn potencial(&self, u: f64) -> f64 {
        let pose = self.pose(u);
        let w = masa::BRAZO * self.persona.masa_kg * G;
        // la placa sube lo que baja la barra
        w * medio(pose.hombro, pose.codo).1 + w * medio(pose.codo, pose.mano).1
            - self.fuerza_mano() * pose.mano.1
    }

    fn trabajo(&self) -> f64 {
        self.fuerza_mano() * (self.y_alta - self.y_baja)
    }
}

/// El jalón con el torso RECLINADO hacia atrás un ángulo fijo, en la misma
/// máquina que [`JalonPlomo`] (la misma persona, la misma placa, la misma
/// polea). Es la pregunta del reel 29 de fisicobuenfisico («¿derecho o
/// reclinado?»), con el modelo de este módulo.
///
/// Lo declarado: el torso gira atrás `reclinado_rad` sobre L5/S1 y se queda
/// así toda la repetición. Arriba la barra CUELGA bajo la polea, con el brazo
/// tan extendido como en el jalón derecho (la misma distancia hombro-mano).
/// Abajo la barra llega al MISMO punto de tu pecho que en el derecho, que al
/// reclinarte se va contigo: la barra viaja contigo, no se queda clavada en el
/// aire. Entre los dos, en línea recta.
///
/// Lo que sale: el cable ya no cuelga a plomo, apunta de la mano a la polea, y
/// su línea cambia con la bajada. La palanca del hombro es la distancia de
/// la articulación a ESA línea ([`palanca`]), y deja de ser constante.
///
/// Con `reclinado_rad = 0` es el [`JalonPlomo`] (prueba `derecho_es_el_plomo`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JalonReclinado {
    /// La máquina, la persona, la placa y el recorrido del jalón derecho.
    pub base: JalonPlomo,
    /// Cuánto se reclina el torso hacia atrás, radianes, fijo.
    pub reclinado_rad: f64,
}

impl JalonReclinado {
    /// Un punto fijo al torso (en el marco del torso derecho) llevado al
    /// torso reclinado: el giro sobre L5/S1.
    #[must_use]
    pub fn con_el_torso(&self, q: Punto) -> Punto {
        let (s, c) = self.reclinado_rad.sin_cos();
        (q.0 * c - q.1 * s, q.0 * s + q.1 * c)
    }

    /// El hombro: el del jalón derecho, girado con el torso.
    #[must_use]
    pub fn hombro(&self) -> Punto {
        self.con_el_torso(self.base.hombro())
    }

    /// Arriba: la barra cuelga bajo la polea, con la misma distancia
    /// hombro-mano que en el jalón derecho.
    #[must_use]
    pub fn arranque(&self) -> Punto {
        let b = &self.base;
        let h0 = b.hombro();
        let largo = (b.polea.0 - h0.0).hypot(b.y_alta - h0.1);
        let h = self.hombro();
        let dx = b.polea.0 - h.0;
        assert!(largo > dx, "el brazo no alcanza a la barra bajo la polea");
        (b.polea.0, h.1 + (largo * largo - dx * dx).sqrt())
    }

    /// Abajo: el punto del pecho donde acaba el jalón derecho, que se va con
    /// el torso.
    #[must_use]
    pub fn final_(&self) -> Punto {
        self.con_el_torso((self.base.polea.0, self.base.y_baja))
    }

    /// La mano en el avance `u`.
    #[must_use]
    pub fn mano(&self, u: f64) -> Punto {
        let (a, b) = (self.arranque(), self.final_());
        (a.0 + (b.0 - a.0) * u, a.1 + (b.1 - a.1) * u)
    }

    /// El tramo de cable de la mano a la polea, m.
    #[must_use]
    pub fn largo_cable(&self, u: f64) -> f64 {
        let (m, p) = (self.mano(u), self.base.polea);
        (p.0 - m.0).hypot(p.1 - m.1)
    }

    /// Cuántos grados se sale el cable de la vertical en el avance `u`.
    #[must_use]
    pub fn cable_grados(&self, u: f64) -> f64 {
        let (m, p) = (self.mano(u), self.base.polea);
        (p.0 - m.0).atan2(p.1 - m.1).to_degrees()
    }
}

impl Repeticion for JalonReclinado {
    fn persona(&self) -> &Persona {
        &self.base.persona
    }

    fn pose(&self, u: f64) -> Pose {
        let hombro = self.hombro();
        let mano = self.mano(u);
        let (s, c) = self.reclinado_rad.sin_cos();
        Pose {
            hombro,
            codo: codo_de(&self.base.persona, hombro, mano, Doblez::Abajo),
            mano,
            reclinado: self.reclinado_rad,
            eje: (s, -c),
            cadera: (0.0, 0.0),
        }
    }

    fn fuerza_mano(&self) -> f64 {
        self.base.fuerza_mano()
    }

    /// Lo que sube la placa: lo que se alarga el cable de la mano a la polea.
    fn recorrido(&self, u: f64) -> f64 {
        self.largo_cable(u) - self.largo_cable(0.0)
    }

    fn potencial(&self, u: f64) -> f64 {
        let pose = self.pose(u);
        let w = masa::BRAZO * self.base.persona.masa_kg * G;
        // la placa sube lo que se alarga el cable
        w * medio(pose.hombro, pose.codo).1
            + w * medio(pose.codo, pose.mano).1
            + self.fuerza_mano() * self.largo_cable(u)
    }

    fn trabajo(&self) -> f64 {
        self.fuerza_mano() * self.recorrido(1.0)
    }

    fn polea(&self) -> Option<Punto> {
        Some(self.base.polea)
    }
}

/// Las fuerzas sobre el brazo: (fuerza, N, dónde, su línea). La primera es
/// la de la mano: vertical hacia arriba (negativa sobre la línea vertical,
/// que apunta abajo) o, si hay [`Repeticion::polea`], por el cable hacia la
/// polea. Las otras, los pesos del húmero y del antebrazo, solo con
/// `con_brazos`.
fn cargas<R: Repeticion + ?Sized>(
    r: &R,
    pose: &Pose,
    con_brazos: bool,
) -> Vec<(f64, Punto, Linea)> {
    let mano = match r.polea() {
        None => (-r.fuerza_mano(), pose.mano, Linea::Vertical),
        Some(p) => (r.fuerza_mano(), pose.mano, Linea::Cable(p)),
    };
    let mut c = vec![mano];
    if con_brazos {
        let w = masa::BRAZO * r.persona().masa_kg * G;
        c.push((w, medio(pose.hombro, pose.codo), Linea::Vertical));
        c.push((w, medio(pose.codo, pose.mano), Linea::Vertical));
    }
    c
}

fn momento(piv: Punto, cargas: &[(f64, Punto, Linea)]) -> f64 {
    cargas
        .iter()
        .map(|&(f, q, l)| palanca::momento(piv, q, f, l))
        .sum()
}

/// Los momentos con signo (hombro, codo) de las cargas sobre el brazo.
fn momentos<R: Repeticion + ?Sized>(r: &R, pose: &Pose, con_brazos: bool) -> (f64, f64) {
    let c = cargas(r, pose, con_brazos);
    // el codo solo ve la mano y el antebrazo
    let del_codo: Vec<(f64, Punto, Linea)> = c
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != 1)
        .map(|(_, x)| *x)
        .collect();
    (momento(pose.hombro, &c), momento(pose.codo, &del_codo))
}

/// |τ| en hombro y codo en una postura. Sin `con_brazos`, el de la carga
/// sola (la convención del módulo).
#[must_use]
pub fn torques<R: Repeticion + ?Sized>(r: &R, pose: &Pose, con_brazos: bool) -> Torques {
    let (h, c) = momentos(r, pose, con_brazos);
    Torques {
        hombro: h.abs(),
        codo: c.abs(),
    }
}

/// |τ| de la carga en el avance `u`.
#[must_use]
pub fn tau<R: Repeticion + ?Sized>(r: &R, a: Articulacion, u: f64) -> f64 {
    torques(r, &r.pose(u), false).de(a)
}

/// La palanca del hombro, m: la fuerza de la mano es vertical en las dos,
/// así que es la distancia horizontal del hombro a la mano.
#[must_use]
pub fn palanca_hombro(pose: &Pose) -> f64 {
    (pose.mano.0 - pose.hombro.0).abs()
}

/// La palanca del codo, m: la distancia horizontal del codo a la mano.
#[must_use]
pub fn palanca_codo(pose: &Pose) -> f64 {
    (pose.mano.0 - pose.codo.0).abs()
}

/// La palanca de la fuerza de la mano en una articulación, m: la distancia
/// de la articulación a la LÍNEA por la que jala (la vertical por la mano, o
/// el cable hacia la polea). Con la línea vertical es [`palanca_hombro`] o
/// [`palanca_codo`].
#[must_use]
pub fn palanca<R: Repeticion + ?Sized>(r: &R, pose: &Pose, a: Articulacion) -> f64 {
    let piv = match a {
        Articulacion::Hombro => pose.hombro,
        Articulacion::Codo => pose.codo,
    };
    let linea = r.polea().map_or(Linea::Vertical, Linea::Cable);
    palanca::brazo(piv, pose.mano, linea)
}

/// El ángulo del hombro, grados: 0 con el húmero pegado al cuerpo, 180
/// estirado en la línea del cuerpo (colgado de la barra).
#[must_use]
pub fn angulo_hombro(pose: &Pose) -> f64 {
    let a = (pose.codo.1 - pose.hombro.1).atan2(pose.codo.0 - pose.hombro.0);
    let c = pose.eje.1.atan2(pose.eje.0);
    let tau = std::f64::consts::TAU;
    ((a - c + std::f64::consts::PI).rem_euclid(tau) - std::f64::consts::PI)
        .abs()
        .to_degrees()
}

/// El pico de |τ| en `n + 1` muestras de la repetición.
#[must_use]
pub fn pico<R: Repeticion + ?Sized>(r: &R, a: Articulacion, n: usize) -> f64 {
    (0..=n)
        .map(|i| tau(r, a, i as f64 / n as f64))
        .fold(f64::MIN, f64::max)
}

/// El promedio de |τ| en `n + 1` muestras de la repetición. Es el número
/// honesto para comparar un torque plano contra una rampa: el pico de la
/// dominada solo existe en el último centímetro.
#[must_use]
pub fn media<R: Repeticion + ?Sized>(r: &R, a: Articulacion, n: usize) -> f64 {
    (0..=n).map(|i| tau(r, a, i as f64 / n as f64)).sum::<f64>() / (n + 1) as f64
}

/// El balance de energía: lo que hacen los músculos en hombro y codo
/// (∫τ dφ sobre los ángulos RELATIVOS, con todos los pesos) contra lo que
/// sube la energía potencial de todo el sistema. Cierra solo si la postura
/// resuelta, los torques y los ángulos cuentan la misma historia. Devuelve
/// (músculos, potencial), J.
#[must_use]
pub fn balance<R: Repeticion + ?Sized>(r: &R, n: usize) -> (f64, f64) {
    let envuelve = |x: f64| {
        (x + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI
    };
    let mut musc = 0.0;
    let mut prev: Option<(f64, f64, f64, f64, f64)> = None;
    for i in 0..=n {
        let pose = r.pose(i as f64 / n as f64);
        let (mh, mc) = momentos(r, &pose, true);
        let (h, c, m) = (pose.hombro, pose.codo, pose.mano);
        let a_hum = (c.1 - h.1).atan2(c.0 - h.0);
        let a_ant = (m.1 - c.1).atan2(m.0 - c.0);
        let a_tro = pose.eje.1.atan2(pose.eje.0);
        if let Some((mh0, mc0, h0, f0, t0)) = prev {
            let dh = envuelve(a_hum - a_tro - (h0 - t0));
            let dc = envuelve(a_ant - a_hum - (f0 - h0));
            musc -= 0.5 * (mh + mh0) * dh + 0.5 * (mc + mc0) * dc;
        }
        prev = Some((mh, mc, a_hum, a_ant, a_tro));
    }
    (musc, r.potencial(1.0) - r.potencial(0.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    const MASA: f64 = 78.0;

    fn persona() -> Persona {
        Persona {
            estatura_m: 1.78,
            masa_kg: MASA,
        }
    }

    /// La dominada del reel 42: se reclina 20° arriba.
    fn dominada(lean: f64) -> Dominada {
        Dominada {
            persona: persona(),
            reclinado_max_rad: lean.to_radians(),
        }
    }

    /// El jalón del reel 42: la geometría del reel 26 en el marco de L5/S1.
    fn jalon(polea: Punto) -> JalonPlomo {
        JalonPlomo {
            persona: persona(),
            carga_kg: MASA,
            polea,
            y_alta: 1.06,
            y_baja: 0.56,
        }
    }

    fn cerca(a: f64, b: f64, tol: f64, que: &str) {
        assert!((a - b).abs() <= tol, "{que}: {a} ≠ {b} (± {tol})");
    }

    /// Las constantes de reel42.py, corrido aparte (2026-10-20).
    #[test]
    fn las_constantes_de_reel42_py() {
        let p = persona();
        cerca(p.torso(), 0.51264, 1e-12, "LTS");
        cerca(p.brazo(), 0.66216, 1e-12, "BRAZO");
        cerca(p.humero(), 0.344_323_2, 1e-12, "LA");
        cerca(p.antebrazo(), 0.317_836_8, 1e-12, "LF");
        cerca(masa_sin_brazos(&p), 0.9, 1e-12, "M_CUERPO");
        cerca(
            centro_sin_brazos(&p),
            0.467_219_886_355_555_45,
            1e-12,
            "C_CUERPO",
        );
        cerca(
            p.brazo() - dominada(20.0).subida(),
            0.09256,
            1e-12,
            "BARBILLA",
        );
        cerca(dominada(20.0).fuerza_mano(), 765.18, 1e-9, "W_CARGA");
        cerca(jalon((0.24, 1.60)).fuerza_mano(), 765.18, 1e-9, "la placa");
    }

    /// Las posturas y los torques de reel42.py en cinco avances: (u, hombro,
    /// codo, |τ| hombro, |τ| codo, palanca, ángulo del hombro, recorrido cm).
    #[test]
    fn las_posturas_de_reel42_py() {
        type Fila = (f64, Punto, Punto, f64, f64, f64, f64, f64);
        let dom: [Fila; 5] = [
            (
                0.0,
                (0.0, -0.662160),
                (0.0, -0.317837),
                0.0,
                0.000005,
                0.0,
                180.0,
                0.0,
            ),
            (
                0.25,
                (-0.049317, -0.519760),
                (0.179394, -0.262370),
                37.736615,
                137.268701,
                0.049317,
                133.3764,
                14.24,
            ),
            (
                0.5,
                (-0.090834, -0.377360),
                (0.220051, -0.229342),
                69.504181,
                168.378475,
                0.090834,
                105.4600,
                28.48,
            ),
            (
                0.75,
                (-0.129281, -0.234960),
                (0.215041, -0.234046),
                98.923227,
                164.545072,
                0.129281,
                75.1521,
                42.72,
            ),
            (
                1.0,
                (-0.161758, -0.092560),
                (0.116143, -0.295856),
                123.773801,
                88.870670,
                0.161758,
                33.8130,
                56.96,
            ),
        ];
        let jal: [Fila; 5] = [
            (
                0.0,
                (0.0, 0.512640),
                (0.256292, 0.742581),
                183.6432,
                12.466312,
                0.24,
                131.8979,
                0.0,
            ),
            (
                0.25,
                (0.0, 0.512640),
                (0.324235, 0.628529),
                183.6432,
                64.454889,
                0.24,
                109.6680,
                12.5,
            ),
            (
                0.5,
                (0.0, 0.512640),
                (0.344311, 0.509768),
                183.6432,
                79.816859,
                0.24,
                89.5220,
                25.0,
            ),
            (
                0.75,
                (0.0, 0.512640),
                (0.316242, 0.376443),
                183.6432,
                58.338658,
                0.24,
                66.6997,
                37.5,
            ),
            (
                1.0,
                (0.0, 0.512640),
                (0.214381, 0.243197),
                183.6432,
                19.603262,
                0.24,
                38.5074,
                50.0,
            ),
        ];
        let (d, j) = (dominada(20.0), jalon((0.24, 1.60)));
        let filas: [(&dyn Repeticion, &[Fila; 5]); 2] = [(&d, &dom), (&j, &jal)];
        for (r, tabla) in filas {
            for &(u, h, c, th, tc, pal, ang, rec) in tabla {
                let pose = r.pose(u);
                let t = torques(r, &pose, false);
                cerca(pose.hombro.0, h.0, 1e-6, "hombro x");
                cerca(pose.hombro.1, h.1, 1e-6, "hombro y");
                cerca(pose.codo.0, c.0, 1e-6, "codo x");
                cerca(pose.codo.1, c.1, 1e-6, "codo y");
                cerca(t.hombro, th, 1e-5, &format!("τ hombro en u = {u}"));
                cerca(t.codo, tc, 1e-5, &format!("τ codo en u = {u}"));
                cerca(palanca_hombro(&pose), pal, 1e-6, "palanca");
                cerca(angulo_hombro(&pose), ang, 1e-4, "ángulo del hombro");
                cerca(r.recorrido(u) * 100.0, rec, 1e-6, "recorrido");
            }
        }
    }

    /// Lo que reel42.py imprime: medias, picos, palancas, trabajo y
    /// recorrido de cada una, y sus dos razones.
    #[test]
    fn los_resumenes_de_reel42_py() {
        use Articulacion::{Codo, Hombro};
        let (d, j) = (dominada(20.0), jalon((0.24, 1.60)));
        cerca(
            media(&d, Hombro, 300),
            67.623934,
            1e-5,
            "media hombro dominada",
        );
        cerca(
            pico(&d, Hombro, 300),
            123.773801,
            1e-5,
            "pico hombro dominada",
        );
        cerca(
            media(&d, Codo, 300),
            139.296207,
            1e-5,
            "media codo dominada",
        );
        cerca(pico(&d, Codo, 300), 171.625711, 1e-5, "pico codo dominada");
        cerca(
            palanca_hombro(&d.pose(1.0)),
            0.161758,
            1e-6,
            "palanca arriba dominada",
        );
        cerca(d.trabajo(), 423.403288, 1e-5, "W dominada");
        cerca(d.recorrido(1.0) * 100.0, 56.96, 1e-9, "recorrido dominada");
        cerca(media(&j, Hombro, 300), 183.6432, 1e-5, "media hombro jalón");
        cerca(pico(&j, Hombro, 300), 183.6432, 1e-5, "pico hombro jalón");
        cerca(media(&j, Codo, 300), 54.491250, 1e-5, "media codo jalón");
        cerca(pico(&j, Codo, 300), 79.825786, 1e-5, "pico codo jalón");
        cerca(j.trabajo(), 382.59, 1e-9, "W jalón");
        cerca(j.recorrido(1.0) * 100.0, 50.0, 1e-9, "recorrido jalón");
        cerca(
            media(&j, Hombro, 300) / media(&d, Hombro, 300),
            2.715654,
            1e-6,
            "RAZON_DORSAL",
        );
        cerca(
            media(&d, Codo, 300) / media(&j, Codo, 300),
            2.556304,
            1e-6,
            "RAZON_BICEPS",
        );
        // reel42.py (CRUCE) mezclaba convenciones: su 34.8° es el torque por el tronco (con los
        // pesos del brazo) contra el del jalón sin ellos
        let p = &d.persona;
        let mezcla = (j.fuerza_mano() * 0.24
            / (masa_sin_brazos(p) * p.masa_kg * G * centro_sin_brazos(p)))
        .asin()
        .to_degrees();
        cerca(mezcla, 34.802679, 1e-6, "CRUCE de reel42.py");
    }

    /// El reclinado es lo único declarado: de 10° a 30° (reel42.py,
    /// VARIANTES_LEAN) y la polea de 20 a 28 cm (VARIANTES_POLEA).
    #[test]
    fn las_variantes_de_reel42_py() {
        use Articulacion::{Codo, Hombro};
        for (lean, m, pk, mc) in [
            (10.0, 39.212762, 69.834890, 165.293753),
            (15.0, 53.517706, 97.005519, 152.536965),
            (25.0, 81.502788, 149.950930, 126.080091),
            (30.0, 95.113032, 175.278718, 113.143002),
            (0.0, 9.993716, f64::NAN, f64::NAN),
        ] {
            let d = dominada(lean);
            cerca(
                media(&d, Hombro, 300),
                m,
                1e-5,
                &format!("media hombro a {lean}°"),
            );
            if pk.is_finite() {
                cerca(
                    pico(&d, Hombro, 300),
                    pk,
                    1e-5,
                    &format!("pico hombro a {lean}°"),
                );
                cerca(
                    media(&d, Codo, 300),
                    mc,
                    1e-5,
                    &format!("media codo a {lean}°"),
                );
            }
        }
        for (polea, m, mc) in [
            ((0.20, 1.60), 153.036, 84.232212),
            ((0.28, 1.60), 214.2504, 30.400303),
            ((0.24, 1.50), 183.6432, 54.49125),
            ((0.24, 1.70), 183.6432, 54.49125),
        ] {
            let j = jalon(polea);
            cerca(media(&j, Hombro, 300), m, 1e-5, "media hombro jalón");
            cerca(media(&j, Codo, 300), mc, 1e-5, "media codo jalón");
        }
    }

    /// El balance de energía de reel42.py: músculos contra potencial.
    #[test]
    fn el_balance_de_energia() {
        let (d, j) = (dominada(20.0), jalon((0.24, 1.60)));
        let (m, e) = balance(&d, 1200);
        cerca(m, 423.403107, 1e-5, "músculos, dominada");
        cerca(e, 423.403288, 1e-5, "potencial, dominada");
        cerca(m, e, 1e-3, "la dominada cierra");
        // en la dominada el potencial es exactamente el trabajo de la carga
        cerca(
            e,
            d.trabajo(),
            1e-9,
            "W = peso · lo que sube el centro de masa",
        );
        let (m, e) = balance(&j, 1200);
        cerca(m, 353.919276, 1e-5, "músculos, jalón");
        cerca(e, 353.919331, 1e-5, "potencial, jalón");
        cerca(m, e, 1e-3, "el jalón cierra");
    }

    /// Dos cuentas independientes para el hombro de la dominada: por el
    /// brazo (la barra menos los pesos del brazo) y por el tronco.
    #[test]
    fn dos_cuentas_para_el_hombro() {
        let d = dominada(20.0);
        for i in 0..=20 {
            let pose = d.pose(f64::from(i) / 20.0);
            let por_brazo = torques(&d, &pose, true).hombro;
            cerca(por_brazo, d.tau_tronco(&pose), 1e-9, "brazo contra tronco");
        }
        // reel42.py: 28.042934, 55.872444, 110.047232 en u = 0.25, 0.5, 1
        cerca(
            d.tau_tronco(&d.pose(0.25)),
            28.042_933_834,
            1e-8,
            "u = 0.25",
        );
        cerca(d.tau_tronco(&d.pose(0.5)), 55.872_444_009, 1e-8, "u = 0.5");
        cerca(d.tau_tronco(&d.pose(1.0)), 110.047_232_079, 1e-8, "u = 1");
    }

    /// Lo que afirma el reel, con su prueba.
    #[test]
    fn lo_que_afirma_el_reel() {
        use Articulacion::{Codo, Hombro};
        let (d, j) = (dominada(20.0), jalon((0.24, 1.60)));
        // «en la mano, ninguna diferencia»: la misma fuerza, no parecida
        assert!((d.fuerza_mano() - j.fuerza_mano()).abs() < 1e-12);
        // «del hombro cuelga el 90 %»
        cerca(masa_sin_brazos(&d.persona), 0.9, 1e-12, "sin brazos");
        // colgado, todo a plomo: el hombro bajo la barra y 0 N·m (al micro: la raíz del
        // codo con el brazo estirado deja un ruido de 1e-9 m, como en reel42.py)
        let p0 = d.pose(0.0);
        assert!(
            p0.hombro.0.abs() < 1e-9 && tau(&d, Hombro, 0.0) < 1e-6,
            "colgado: {:?}",
            p0.hombro
        );
        // el centro de masa en la vertical de la barra en toda la subida
        for i in 0..=20 {
            let u = f64::from(i) / 20.0;
            let pose = d.pose(u);
            let cx = d.com_x(pose.hombro.0, pose.hombro.1, pose.reclinado);
            assert!(
                cx.abs() < 1e-12,
                "centro de masa fuera de la barra en u = {u}: {cx}"
            );
        }
        // el jalón: la palanca del hombro no se mueve
        for i in 0..=20 {
            cerca(
                palanca_hombro(&j.pose(f64::from(i) / 20.0)),
                0.24,
                1e-12,
                "palanca del jalón",
            );
        }
        // repartida al revés, en todo el rango declarado del reclinado
        for lean in [10.0, 15.0, 20.0, 25.0, 30.0] {
            let dl = dominada(lean);
            assert!(
                media(&j, Hombro, 300) / media(&dl, Hombro, 300) > 1.5,
                "hombro a {lean}°"
            );
            assert!(
                media(&dl, Codo, 300) / media(&j, Codo, 300) > 1.5,
                "codo a {lean}°"
            );
        }
        // y dónde se voltearía: en lo más alto, el hombro de la dominada iguala al del jalón
        let c = d.reclinado_de_cruce(&j);
        cerca(
            tau(&dominada(c), Hombro, 1.0),
            tau(&j, Hombro, 0.5),
            1e-6,
            "cruce",
        );
        assert!(c > 30.0, "ni a 30° se alcanza: {c}");
        // en promedio, ni a 30° la dominada alcanza al jalón
        assert!(media(&dominada(30.0), Hombro, 300) < 0.6 * media(&j, Hombro, 300));
    }

    /// El jalón reclinado del remake del reel 29: la máquina del reel 42.
    fn reclinado(grados: f64) -> JalonReclinado {
        JalonReclinado {
            base: jalon((0.24, 1.60)),
            reclinado_rad: grados.to_radians(),
        }
    }

    /// Sin reclinarse es el jalón a plomo: las mismas posturas, los mismos
    /// torques (por el cable, no por la vertical) y el mismo recorrido.
    #[test]
    fn derecho_es_el_plomo() {
        use Articulacion::{Codo, Hombro};
        let (r, j) = (reclinado(0.0), jalon((0.24, 1.60)));
        for i in 0..=20 {
            let u = f64::from(i) / 20.0;
            let (a, b) = (r.pose(u), j.pose(u));
            for (x, y) in [(a.hombro, b.hombro), (a.codo, b.codo), (a.mano, b.mano)] {
                cerca(x.0, y.0, 1e-12, "x");
                cerca(x.1, y.1, 1e-12, "y");
            }
            cerca(tau(&r, Hombro, u), tau(&j, Hombro, u), 1e-9, "τ hombro");
            cerca(tau(&r, Codo, u), tau(&j, Codo, u), 1e-9, "τ codo");
            cerca(
                palanca(&r, &a, Hombro),
                palanca_hombro(&b),
                1e-12,
                "palanca",
            );
            cerca(r.recorrido(u), j.recorrido(u), 1e-12, "recorrido");
        }
    }

    /// Arriba la barra cuelga bajo la polea: el cable va a plomo y la palanca
    /// es la distancia horizontal hombro-barra, aritmética de mano: 0.24 m
    /// más lo que se fue el hombro atrás, torso · sen 30°.
    #[test]
    fn reclinado_arriba_a_mano() {
        let r = reclinado(30.0);
        let p = r.pose(0.0);
        cerca(r.cable_grados(0.0), 0.0, 1e-12, "a plomo arriba");
        let d = 0.24 + persona().torso() * 0.5;
        cerca(
            palanca(&r, &p, Articulacion::Hombro),
            d,
            1e-12,
            "palanca arriba",
        );
        cerca(
            tau(&r, Articulacion::Hombro, 0.0),
            765.18 * d,
            1e-9,
            "τ = F · d arriba",
        );
        // la mano, a la distancia del jalón derecho
        let j = jalon((0.24, 1.60)).pose(0.0);
        let l0 = (j.mano.0 - j.hombro.0).hypot(j.mano.1 - j.hombro.1);
        cerca(
            (p.mano.0 - p.hombro.0).hypot(p.mano.1 - p.hombro.1),
            l0,
            1e-12,
            "el mismo brazo",
        );
    }

    /// La palanca contra el cable, por otra cuenta: el doble del área del
    /// triángulo hombro-mano-polea entre la base mano-polea.
    #[test]
    fn la_palanca_por_el_area() {
        let r = reclinado(30.0);
        for i in 0..=10 {
            let u = f64::from(i) / 10.0;
            let p = r.pose(u);
            let (h, m, q) = (p.hombro, p.mano, (0.24, 1.60));
            let area2 = ((m.0 - h.0) * (q.1 - h.1) - (m.1 - h.1) * (q.0 - h.0)).abs();
            let base = (q.0 - m.0).hypot(q.1 - m.1);
            cerca(
                palanca(&r, &p, Articulacion::Hombro),
                area2 / base,
                1e-12,
                "palanca",
            );
            cerca(
                tau(&r, Articulacion::Hombro, u),
                765.18 * area2 / base,
                1e-9,
                "τ = F · d",
            );
        }
    }

    /// El balance de energía cierra también con el cable inclinado: los
    /// músculos de hombro y codo contra lo que suben la placa y los brazos.
    #[test]
    fn el_balance_del_reclinado() {
        for g in [10.0, 20.0, 30.0] {
            let r = reclinado(g);
            let (m, e) = balance(&r, 1200);
            cerca(m, e, 1e-3, &format!("cierra a {g}°"));
        }
    }

    /// Lo que afirma el remake del reel 29, con su prueba: reclinarse pone
    /// la palanca más grande arriba y la quita en el pecho, la línea del
    /// cable se inclina en la bajada, y la barra viaja menos.
    #[test]
    fn reclinarse_cambia_donde_pide() {
        use Articulacion::Hombro;
        let (d, r) = (jalon((0.24, 1.60)), reclinado(30.0));
        cerca(
            tau(&r, Hombro, 0.0),
            765.18 * 0.496_32,
            1e-9,
            "arriba: 765.18 N · (0.24 + 0.51264 · sen 30°) m",
        );
        cerca(tau(&r, Hombro, 1.0), 97.6, 0.05, "en el pecho");
        assert!(
            tau(&r, Hombro, 0.0) > 2.0 * tau(&d, Hombro, 0.0),
            "más del doble arriba"
        );
        assert!(
            tau(&r, Hombro, 1.0) < tau(&d, Hombro, 1.0),
            "menos en el pecho"
        );
        // baja todo el camino, y el cable se va inclinando
        for i in 0..20 {
            let (u0, u1) = (f64::from(i) / 20.0, f64::from(i + 1) / 20.0);
            assert!(tau(&r, Hombro, u1) < tau(&r, Hombro, u0));
            assert!(r.cable_grados(u1) > r.cable_grados(u0));
        }
        assert!(
            r.recorrido(1.0) < 0.5 * d.recorrido(1.0),
            "menos de la mitad del recorrido"
        );
        assert!(media(&r, Hombro, 300) > media(&d, Hombro, 300));
    }
}
