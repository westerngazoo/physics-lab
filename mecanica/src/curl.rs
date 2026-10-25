//! Curl de bíceps: barra contra polea baja.
//!
//! Del reel 04, que es el molde de toda la serie de comparaciones. El
//! codo está fijo y la mano describe un arco; lo único que cambia entre
//! las dos opciones es POR DÓNDE jala la carga, y eso cambia la palanca
//! sin cambiar un gramo de peso.
//!
//! Con barra la palanca es `L·sen φ`: cero abajo, máxima a la horizontal,
//! cero otra vez arriba. Con polea baja el cable nunca queda a plomo del
//! codo, así que abajo ya hay tensión y el perfil se aplana. Ésa es toda
//! la pieza, y sale de [`crate::palanca`] sin una línea de trigonometría
//! propia.

use crate::palanca::{self, Linea, Punto};
use crate::Lift;

/// De dónde cuelga la carga.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Agarre {
    /// Barra o mancuerna: la gravedad, a plomo.
    Barra,
    /// Polea baja, con su polea en `(x, y)` relativos al CODO.
    Polea(Punto),
}

impl Agarre {
    fn linea(self) -> Linea {
        match self {
            Agarre::Barra => Linea::Vertical,
            Agarre::Polea(p) => Linea::Cable(p),
        }
    }
}

/// Un curl. El origen del marco es el CODO.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Curl {
    /// Carga, kilogramos.
    pub carga_kg: f64,
    /// Codo → centro del agarre, metros.
    pub antebrazo_m: f64,
    /// Barra o polea.
    pub agarre: Agarre,
    /// Recorrido del codo, radianes, como `(desde, hasta)`: 0 es el brazo
    /// colgado y crece al flexionar.
    pub rango_rad: (f64, f64),
}

impl Curl {
    /// Dónde está la mano con el codo flexionado `phi`.
    ///
    /// `phi = 0` cuelga a plomo bajo el codo; crece hacia adelante.
    #[must_use]
    pub fn mano(&self, phi: f64) -> Punto {
        (self.antebrazo_m * phi.sin(), -self.antebrazo_m * phi.cos())
    }

    /// La palanca del codo, metros: lo que los reels dibujan punteado.
    #[must_use]
    pub fn brazo(&self, phi: f64) -> f64 {
        palanca::brazo((0.0, 0.0), self.mano(phi), self.agarre.linea())
    }
}

impl Lift for Curl {
    /// `phi` es la flexión del codo en radianes.
    ///
    /// Con signo, como pide [`Lift::tau`]: positivo si la carga resiste la
    /// flexión. La flexión gira la mano en sentido antihorario (z sale del
    /// papel), así que la carga que resiste da `r × F` negativo y aquí se
    /// le cambia el signo. Un cable que ayuda a subir da negativo.
    fn tau(&self, phi: f64) -> f64 {
        -palanca::momento(
            (0.0, 0.0),
            self.mano(phi),
            palanca::peso(self.carga_kg),
            self.agarre.linea(),
        )
    }

    fn range(&self) -> (f64, f64) {
        self.rango_rad
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::f64::consts::PI;

    /// El montaje publicado del reel 04.
    fn barra() -> Curl {
        Curl {
            carga_kg: 20.0,
            antebrazo_m: 0.32,
            agarre: Agarre::Barra,
            rango_rad: (0.0, PI / 2.0),
        }
    }

    fn polea() -> Curl {
        Curl {
            agarre: Agarre::Polea((0.42, -0.85)),
            ..barra()
        }
    }

    /// Con la barra colgando a plomo del codo no hay palanca: CERO, no
    /// "casi cero". Es la comprobación que mata cualquier ajuste de curva.
    #[test]
    fn la_barra_arranca_en_cero() {
        assert!(barra().tau(0.0).abs() < 1e-12);
    }

    /// Y la polea NO arranca en cero, pero abajo AYUDA: con la polea
    /// adelante y abajo, flexionar acerca la mano a la polea, así que el
    /// cable tira a favor de la subida. En valor absoluto esto se leía
    /// como 39 N·m de carga; con signo es −39.
    #[test]
    fn la_polea_abajo_ayuda() {
        let t = polea().tau(0.0);
        assert!(t < -30.0, "dio {t}");
    }

    /// La barra resiste en todo el recorrido: nunca cambia de signo.
    #[test]
    fn la_barra_siempre_resiste() {
        for i in 1..=20 {
            let phi = PI / 2.0 * f64::from(i) / 20.0;
            assert!(barra().tau(phi) > 0.0, "en {phi} dio {}", barra().tau(phi));
        }
    }

    /// La polea deja de ayudar y empieza a resistir cuando la línea del
    /// cable pasa por el codo. A mano: la mano (L sen φ, −L cos φ) queda
    /// en la recta codo–polea cuando tan φ = −x_p / y_p = 0.42 / 0.85,
    /// φ ≈ 26.3°. Ese cruce es el que el valor absoluto escondía.
    #[test]
    fn la_polea_cruza_donde_el_cable_apunta_al_codo() {
        let p = polea();
        let (mut a, mut b) = (0.0_f64, PI / 2.0);
        for _ in 0..60 {
            let m = 0.5 * (a + b);
            if p.tau(a) * p.tau(m) <= 0.0 {
                b = m;
            } else {
                a = m;
            }
        }
        let phi = 0.5 * (a + b);
        let esperado = (0.42_f64 / 0.85).atan();
        assert!((phi - esperado).abs() < 1e-9, "dio {phi}, no {esperado}");
    }

    /// `τ = F·L·sen φ` a mano, sin pasar por el motor.
    #[test]
    fn la_barra_contra_aritmetica_de_mano() {
        let esperado = 20.0 * 9.81 * 0.32 * (45.0_f64).to_radians().sin();
        assert!((barra().tau(PI / 4.0) - esperado).abs() < 1e-9);
    }

    /// El máximo de la barra cae en la horizontal, donde `sen φ = 1`.
    #[test]
    fn el_pico_de_la_barra_va_en_la_horizontal() {
        let (phi, _) = crate::peak(&barra());
        assert!((phi - PI / 2.0).abs() < 1e-2, "dio {phi}");
    }
}
