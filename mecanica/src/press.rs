//! Press de hombro: barra libre contra Smith.
//!
//! Del reel 39. La pregunta que contesta es si el riel de la Smith de
//! verdad da "tensión constante", y la respuesta es que reacciona
//! horizontal, no vertical: el riel te quita el camino en S de la barra
//! libre, y con él se va la parte del recorrido donde el hombro cobra.
//!
//! LA TRAYECTORIA NO ES UN AJUSTE. En la barra libre la mano sube recto
//! desde la clavícula el alto del claro, y de ahí va en recta a quedar
//! sobre el hombro con el brazo casi estirado; se recorre por LONGITUD DE
//! CAMINO, no interpolando la coordenada x. En la Smith sube vertical y
//! ya. Las dos salen de tres puntos y una raíz cuadrada.
//!
//! Esto vivió un tiempo como cuatro nudos interpolados con dieciséis
//! dígitos y un `alcance - 0.008` sin explicación, ajustados a los propios
//! valores dorados. Un ajuste a los dorados pasa los dorados siempre, y no
//! describe nada.

use crate::maquina_humana::{centro, codo, largo, masa, Doblez, Persona};
use crate::palanca::{self, Linea, Punto};

/// Cuál de los dos.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Variante {
    /// Barra libre: la mano elige su camino.
    Militar,
    /// Smith: el riel lo elige por ti.
    Smith,
}

/// Dónde está cada articulación en el plano sagital.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    /// L5/S1.
    pub l5: Punto,
    /// El hombro.
    pub hombro: Punto,
    /// El codo.
    pub codo: Punto,
    /// El centro del agarre.
    pub mano: Punto,
}

/// Un press de hombro, de pie.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Press {
    /// Quién lo hace.
    pub persona: Persona,
    /// La barra, kilogramos.
    pub carga_kg: f64,
    /// Altura de L5/S1 de pie, metros.
    pub cadera_m: f64,
    /// Cuánto al frente del hombro arranca la barra libre.
    pub x_militar: f64,
    /// Cuánto sube recto antes de irse en diagonal.
    pub claro: f64,
    /// Cuánto al frente del hombro corre el riel de la Smith.
    pub x_smith: f64,
    /// Fracción del brazo estirado que se alcanza arriba.
    pub extension: f64,
}

impl Press {
    /// El montaje del reel 39.
    #[must_use]
    pub fn reel39(persona: Persona, carga_kg: f64) -> Self {
        Press {
            persona,
            carga_kg,
            cadera_m: 0.95,
            x_militar: 0.12,
            claro: 0.26,
            x_smith: 0.10,
            extension: 0.97,
        }
    }

    /// Hasta dónde llega la mano con el brazo casi estirado.
    #[must_use]
    pub fn alcance(&self) -> f64 {
        self.persona.brazo() * self.extension
    }

    /// L5/S1.
    #[must_use]
    pub fn l5(&self) -> Punto {
        (0.0, self.cadera_m)
    }

    /// El hombro.
    #[must_use]
    pub fn hombro(&self) -> Punto {
        (0.0, self.cadera_m + self.persona.torso())
    }

    /// La mano en el avance `u`.
    ///
    /// La barra libre va por longitud de camino sobre dos tramos, que es
    /// lo que hace que la x se mueva despacio al principio y rápido
    /// después. Interpolar la x directamente da otra curva.
    #[must_use]
    pub fn mano(&self, variante: Variante, u: f64) -> Punto {
        let h = self.hombro();
        match variante {
            Variante::Smith => {
                let x = h.0 + self.x_smith;
                let alto = (self.alcance().powi(2) - self.x_smith.powi(2))
                    .max(0.0)
                    .sqrt();
                (x, h.1 + alto * u)
            }
            Variante::Militar => {
                let p0 = (h.0 + self.x_militar, h.1);
                let p1 = (h.0 + self.x_militar, h.1 + self.claro);
                let p2 = (h.0, h.1 + self.alcance());
                let l1 = self.claro;
                let l2 = (p2.0 - p1.0).hypot(p2.1 - p1.1);
                let s = u * (l1 + l2);
                if s <= l1 {
                    let t = if l1 > 0.0 { s / l1 } else { 0.0 };
                    (p0.0 + (p1.0 - p0.0) * t, p0.1 + (p1.1 - p0.1) * t)
                } else {
                    let t = (s - l1) / l2;
                    (p1.0 + (p2.0 - p1.0) * t, p1.1 + (p2.1 - p1.1) * t)
                }
            }
        }
    }

    /// La postura completa.
    ///
    /// El codo va `Hacia([1, -0.3])`, abajo y al frente. No es un gusto:
    /// al arrancar la barra queda al frente del hombro y a la MISMA
    /// altura, así que el eje hombro-mano es horizontal y "adelante" no
    /// nombra ningún lado de él.
    ///
    /// # Errors
    /// Si la mano cae fuera del alcance del brazo.
    pub fn pose(&self, variante: Variante, u: f64) -> Result<Pose, garust::twolink::TwoLinkError> {
        let hombro = self.hombro();
        let mano = self.mano(variante, u);
        let c = codo(
            &self.persona,
            [hombro.0, hombro.1],
            [mano.0, mano.1],
            Doblez::Hacia([1.0, -0.3]),
        )?;
        Ok(Pose {
            l5: self.l5(),
            hombro,
            codo: (c[0], c[1]),
            mano,
        })
    }

    /// Lo que pesa la barra, newton.
    #[must_use]
    pub fn fuerza(&self) -> f64 {
        palanca::peso(self.carga_kg)
    }

    /// Torque en el hombro por la carga, N·m.
    #[must_use]
    pub fn tau_hombro(&self, p: &Pose) -> f64 {
        palanca::momento(p.hombro, p.mano, self.fuerza(), Linea::Vertical).abs()
    }

    /// Torque en el codo por la carga, N·m.
    #[must_use]
    pub fn tau_codo(&self, p: &Pose) -> f64 {
        palanca::momento(p.codo, p.mano, self.fuerza(), Linea::Vertical).abs()
    }

    /// Torque en L5/S1, N·m: la carga MÁS los segmentos que cuelgan.
    ///
    /// Aquí sí entra el cuerpo. En el hombro y el codo no, porque el peso
    /// del propio brazo llega a cambiar de signo a media repetición y
    /// mete un cruce por cero que la carga no tiene.
    #[must_use]
    pub fn tau_l5s1(&self, p: &Pose) -> f64 {
        let w_brazo = palanca::peso(self.persona.masa_de(masa::BRAZO));
        let w_tronco = palanca::peso(self.persona.masa_de(masa::TRONCO));
        let w_cabeza = palanca::peso(self.persona.masa_de(masa::CABEZA));
        let medio = |a: Punto, b: Punto| ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
        let lerp = |a: Punto, b: Punto, t: f64| (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
        let torso = self.persona.torso();
        let cabeza_t = 1.0 + largo::CABEZA * self.persona.estatura_m / torso;
        let cargas = [
            (p.mano, self.fuerza()),
            (medio(p.codo, p.mano), w_brazo),
            (medio(p.hombro, p.codo), w_brazo),
            (lerp(p.l5, p.hombro, centro::TRONCO), w_tronco),
            (lerp(p.l5, p.hombro, cabeza_t), w_cabeza),
        ];
        cargas
            .iter()
            .map(|&(donde, w)| palanca::momento(p.l5, donde, w, Linea::Vertical))
            .sum::<f64>()
            .abs()
    }

    /// Ángulo del hombro, grados: 0 con el húmero pegado al torso hacia
    /// la cadera, 90 al frente, 180 arriba.
    #[must_use]
    pub fn angulo_hombro(&self, p: &Pose) -> f64 {
        let a = (p.codo.1 - p.hombro.1).atan2(p.codo.0 - p.hombro.0);
        let c = (p.l5.1 - p.hombro.1).atan2(p.l5.0 - p.hombro.0);
        envuelve(a - c).to_degrees()
    }

    /// Ángulo del codo, grados: 180 con el brazo estirado.
    #[must_use]
    pub fn angulo_codo(&self, p: &Pose) -> f64 {
        let a = (p.hombro.1 - p.codo.1).atan2(p.hombro.0 - p.codo.0);
        let b = (p.mano.1 - p.codo.1).atan2(p.mano.0 - p.codo.0);
        envuelve(a - b).to_degrees().abs()
    }

    /// Trabajo de la carga en su línea vertical, joules.
    #[must_use]
    pub fn trabajo(&self, variante: Variante) -> f64 {
        let a = self.mano(variante, 0.0);
        let b = self.mano(variante, 1.0);
        self.fuerza() * (b.1 - a.1)
    }

    /// El mayor torque de hombro de toda la repetición, N·m.
    #[must_use]
    pub fn pico_hombro(&self, variante: Variante, muestras: usize) -> f64 {
        (0..=muestras)
            .filter_map(|i| {
                let u = i as f64 / muestras as f64;
                self.pose(variante, u).ok().map(|p| self.tau_hombro(&p))
            })
            .fold(f64::NEG_INFINITY, f64::max)
    }

    /// Balance de energía: `(músculos, carga)` en joules.
    ///
    /// Los músculos hacen `∫τ dφ` en hombro y codo sobre los ángulos
    /// RELATIVOS, con signo; la carga hace fuerza por lo que sube. Que
    /// las dos cuentas cierren es la prueba fuerte de la pieza: sólo
    /// cuadra si la postura, los torques y los ángulos cuentan la misma
    /// historia. Ninguna tabla de nudos ajustada a un valor guardado
    /// pasa esto por accidente.
    ///
    /// # Errors
    /// Si alguna postura del recorrido cae fuera de alcance.
    pub fn balance(
        &self,
        variante: Variante,
        muestras: usize,
    ) -> Result<(f64, f64), garust::twolink::TwoLinkError> {
        let mut musculos = 0.0;
        let mut previo: Option<(f64, f64, f64, f64)> = None;
        for i in 0..=muestras {
            let u = i as f64 / muestras as f64;
            let p = self.pose(variante, u)?;
            let f = self.fuerza();
            let th = palanca::momento(p.hombro, p.mano, f, Linea::Vertical);
            let tc = palanca::momento(p.codo, p.mano, f, Linea::Vertical);
            let fh = (p.codo.1 - p.hombro.1).atan2(p.codo.0 - p.hombro.0);
            let fc = (p.mano.1 - p.codo.1).atan2(p.mano.0 - p.codo.0);
            if let Some((th0, tc0, fh0, fc0)) = previo {
                let dfh = envuelve(fh - fh0);
                let dfc = envuelve(fc - fc0);
                musculos -= 0.5 * (th + th0) * dfh + 0.5 * (tc + tc0) * (dfc - dfh);
            }
            previo = Some((th, tc, fh, fc));
        }
        Ok((musculos, self.trabajo(variante)))
    }
}

/// Envuelve un ángulo a `(−π, π]`.
fn envuelve(x: f64) -> f64 {
    use core::f64::consts::PI;
    (x + PI).rem_euclid(2.0 * PI) - PI
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press() -> Press {
        Press::reel39(
            Persona {
                estatura_m: 1.75,
                masa_kg: 80.0,
            },
            40.0,
        )
    }

    /// El alcance sale de Winter por la estatura, no de una constante.
    #[test]
    fn el_alcance_sale_de_la_persona() {
        assert!((press().alcance() - 0.631_47).abs() < 1e-9);
    }

    /// El hombro de pie: cadera más torso.
    #[test]
    fn el_hombro_sale_de_la_persona() {
        assert!((press().hombro().1 - 1.454).abs() < 1e-9);
    }

    /// Arriba del todo, la Smith llega a `sqrt(alcance² − x²)`, que es lo
    /// que un `alcance - 0.008` ajustado a mano aproximaba.
    #[test]
    fn el_tope_de_la_smith_es_una_raiz() {
        let p = press();
        let esperado = (p.alcance().powi(2) - 0.10_f64.powi(2)).sqrt();
        assert!((p.mano(Variante::Smith, 1.0).1 - (1.454 + esperado)).abs() < 1e-9);
    }

    /// Al arrancar, el hombro carga la barra a 12 cm: aritmética de mano.
    #[test]
    fn el_arranque_contra_aritmetica_de_mano() {
        let p = press();
        let pose = p.pose(Variante::Militar, 0.0).unwrap();
        assert!((p.tau_hombro(&pose) - 40.0 * 9.81 * 0.12).abs() < 1e-9);
    }

    /// La barra libre acaba sobre el hombro: sin palanca, CERO.
    #[test]
    fn la_barra_libre_cierra_en_cero() {
        let p = press();
        let pose = p.pose(Variante::Militar, 1.0).unwrap();
        assert!(p.tau_hombro(&pose) < 1e-9);
    }

    /// Y la Smith no, porque el riel no la deja meterse encima.
    #[test]
    fn la_smith_no_cierra_en_cero() {
        let p = press();
        let pose = p.pose(Variante::Smith, 1.0).unwrap();
        assert!((p.tau_hombro(&pose) - 40.0 * 9.81 * 0.10).abs() < 1e-9);
    }

    /// El camino de la barra libre se recorre por longitud, así que a
    /// mitad de avance la mano NO va a mitad de la x.
    #[test]
    fn el_camino_va_por_longitud_no_por_x() {
        let p = press();
        let x_medio = p.mano(Variante::Militar, 0.5).0;
        assert!(
            x_medio > 0.06,
            "interpolando la x daría 0.06; por camino da {x_medio}"
        );
    }
}
