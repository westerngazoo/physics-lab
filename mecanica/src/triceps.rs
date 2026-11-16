//! Extensión de tríceps en polea alta: el codo quieto, la mano baja en un
//! arco, y el cable jala hacia la polea.
//!
//! Del reel 31 de @fisicobuenfisico («pegado y recto, o atrás e inclinado»).
//! Misma polea, misma carga, mismo recorrido del codo: lo único que cambia
//! entre las dos variantes es DÓNDE queda el codo respecto a la vertical de
//! la polea, y con eso por dónde pasa la línea del cable.
//!
//! La palanca del codo es la perpendicular del codo a la LÍNEA del cable
//! ([`crate::palanca::brazo`] con [`Linea::Cable`]), no la distancia a la
//! mano. Con el codo pegado a la polea, el cable baja casi a plomo y, al
//! extender, su línea pasa casi por el codo: la palanca se muere en el
//! bloqueo. Con el codo lejos, el cable llega inclinado y la línea nunca
//! pasa tan cerca. Las dos pican en lo mismo, `F · antebrazo`, cuando el
//! cable queda perpendicular al antebrazo: no cambia cuánto, cambia hasta
//! dónde dura. Todo sale de [`crate::palanca`]; aquí no hay trigonometría
//! propia.

use crate::palanca::{self, Linea, Punto};
use crate::Lift;

/// Una extensión de tríceps en polea. El marco es el piso bajo la polea:
/// la polea en `polea` (x = 0 es su vertical), el codo en `codo`, los dos
/// en metros; la persona está del lado de las x positivas y la mano va
/// hacia la polea al doblar el codo.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExtensionPolea {
    /// Carga, kilogramos.
    pub carga_kg: f64,
    /// Dónde cuelga el cable: la polea, metros.
    pub polea: Punto,
    /// El codo, quieto toda la repetición, metros.
    pub codo: Punto,
    /// Codo → centro del agarre, metros.
    pub antebrazo_m: f64,
    /// De dónde arranca el codo, radianes de flexión: la repetición va de
    /// aquí al bloqueo (0).
    pub flexion_max_rad: f64,
}

impl ExtensionPolea {
    /// Dónde está la mano con el codo flexionado `phi` (radianes).
    ///
    /// `phi = 0` es el bloqueo: la mano a plomo bajo el codo. Al doblar, la
    /// mano sube y se va hacia la polea (x negativas).
    #[must_use]
    pub fn mano(&self, phi: f64) -> Punto {
        (self.codo.0 - self.antebrazo_m * phi.sin(), self.codo.1 - self.antebrazo_m * phi.cos())
    }

    /// Por dónde jala la carga: el cable, de la mano a la polea.
    #[must_use]
    pub fn linea(&self) -> Linea {
        Linea::Cable(self.polea)
    }

    /// La fuerza en el cable, N: la carga.
    #[must_use]
    pub fn fuerza(&self) -> f64 {
        palanca::peso(self.carga_kg)
    }

    /// La palanca del codo, metros: la distancia del codo a la línea del
    /// cable. Es lo que los reels dibujan como cota.
    #[must_use]
    pub fn brazo(&self, phi: f64) -> f64 {
        palanca::brazo(self.codo, self.mano(phi), self.linea())
    }

    /// El pie de la perpendicular del codo a la línea del cable, metros
    /// (para dibujar la palanca donde está).
    #[must_use]
    pub fn pie(&self, phi: f64) -> Punto {
        let m = self.mano(phi);
        let (ux, uy) = self.linea().direccion(m);
        let k = (self.codo.0 - m.0) * ux + (self.codo.1 - m.1) * uy;
        (m.0 + ux * k, m.1 + uy * k)
    }

    /// Cuánto se aparta el cable de la vertical, radianes, con signo:
    /// positivo cuando la polea queda del lado de la persona (la mano pasó la
    /// vertical de la polea), negativo cuando el cable llega inclinado desde
    /// la polea hacia la persona.
    #[must_use]
    pub fn cable_rad(&self, phi: f64) -> f64 {
        let m = self.mano(phi);
        (self.polea.0 - m.0).atan2(self.polea.1 - m.1)
    }

    /// Lo que se alarga el tramo de cable de la mano a la polea desde el
    /// arranque, metros: lo que sube la placa (sin fricción, una caída).
    #[must_use]
    pub fn recorrido(&self, phi: f64) -> f64 {
        let largo = |p: f64| {
            let m = self.mano(p);
            (self.polea.0 - m.0).hypot(self.polea.1 - m.1)
        };
        largo(phi) - largo(self.flexion_max_rad)
    }
}

impl Lift for ExtensionPolea {
    /// `phi` es la flexión del codo en radianes. Positivo resiste la
    /// extensión (el cable quiere doblar el codo); negativo la ayudaría.
    fn tau(&self, phi: f64) -> f64 {
        -palanca::momento(self.codo, self.mano(phi), self.fuerza(), self.linea())
    }

    /// Del codo doblado al bloqueo.
    fn range(&self) -> (f64, f64) {
        (self.flexion_max_rad, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{peak, work_over_range, zero_crossing};

    /// El montaje publicado del reel 31: 30 kg, la polea a 2.20 m, el
    /// antebrazo de 0.28 m, el codo arranca en 100° de flexión.
    fn con_el_codo_en(codo: Punto) -> ExtensionPolea {
        ExtensionPolea {
            carga_kg: 30.0,
            polea: (0.0, 2.20),
            codo,
            antebrazo_m: 0.28,
            flexion_max_rad: 100.0_f64.to_radians(),
        }
    }

    /// Pegado a la máquina: el codo a 10 cm de la vertical de la polea.
    fn pegado() -> ExtensionPolea {
        con_el_codo_en((0.10, 1.13))
    }

    /// Un paso atrás e inclinado: el codo a 45 cm.
    fn inclinado() -> ExtensionPolea {
        con_el_codo_en((0.45, 1.06))
    }

    fn grados(g: f64) -> f64 {
        g.to_radians()
    }

    /// Los números del reel 31 (tools/reel31.py de fbf, su propia cuenta,
    /// reproducida aparte): τ en el codo a 100°, 40° y en el bloqueo.
    #[test]
    fn los_numeros_del_reel_31() {
        let esperado = [
            (pegado(), [(100.0, 82.403), (40.0, 48.943), (0.0, 6.087)]),
            (inclinado(), [(100.0, 77.881), (40.0, 64.287), (0.0, 24.894)]),
        ];
        for (x, puntos) in esperado {
            for (g, t) in puntos {
                let dio = x.tau(grados(g));
                assert!((dio - t).abs() < 2e-3, "a {g}°: {dio} contra {t}");
            }
        }
        // en el bloqueo, cuatro veces más inclinado que pegado
        let r = inclinado().tau(0.0) / pegado().tau(0.0);
        assert!((r - 4.089).abs() < 2e-3, "dio {r}");
    }

    /// El trabajo por repetición del reel 31: 93 contra 113 J. Su cuenta era
    /// una suma por la izquierda de 600 tramos (93.27 y 113.05); la de aquí,
    /// por el punto medio, cae a 0.2 J y redondea igual.
    #[test]
    fn el_trabajo_del_reel_31() {
        let (wp, wi) = (work_over_range(&pegado()), work_over_range(&inclinado()));
        assert!((wp - 93.27).abs() < 0.2, "pegado: {wp}");
        assert!((wi - 113.05).abs() < 0.2, "inclinado: {wi}");
        assert!(((wi / wp) - 1.212).abs() < 2e-3, "razón {}", wi / wp);
    }

    /// El pico de las dos es la fuerza por el antebrazo, `F · L`: ocurre
    /// cuando el cable queda perpendicular al antebrazo, y la palanca nunca
    /// pasa de ahí. Es la cuenta de mano que mata cualquier ajuste de curva.
    #[test]
    fn el_pico_es_la_fuerza_por_el_antebrazo() {
        for x in [pegado(), inclinado()] {
            let (phi, t) = peak(&x);
            let tope = x.fuerza() * x.antebrazo_m;
            assert!((t - tope).abs() < 0.01, "pico {t} contra {tope}");
            assert!((x.brazo(phi) - x.antebrazo_m).abs() < 1e-4, "la palanca en el pico es el antebrazo");
            for i in 0..=100 {
                let p = x.flexion_max_rad * f64::from(i) / 100.0;
                assert!(x.brazo(p) <= x.antebrazo_m + 1e-12);
            }
        }
        // pegado pica al arrancar (100°); inclinado, a 82°
        assert!((peak(&pegado()).0.to_degrees() - 99.78).abs() < 0.3);
        assert!((peak(&inclinado()).0.to_degrees() - 81.67).abs() < 0.3);
    }

    /// En el bloqueo, con la mano a plomo bajo el codo, la palanca es
    /// `L · sen θ`, con θ lo que se aparta el cable de la vertical:
    /// `tan θ = x_codo / (y_polea − y_codo + L)`. Trigonometría de mano.
    #[test]
    fn el_bloqueo_por_trigonometria_de_mano() {
        for (x, d_esperado, theta_esperado) in [(pegado(), 0.02068, 4.236), (inclinado(), 0.08459, 17.583)] {
            let theta = (x.codo.0 / (x.polea.1 - x.codo.1 + x.antebrazo_m)).atan();
            let d = x.antebrazo_m * theta.sin();
            assert!((d - d_esperado).abs() < 1e-5, "dio {d}");
            assert!((x.brazo(0.0) - d).abs() < 1e-12, "{} contra {d}", x.brazo(0.0));
            assert!((x.cable_rad(0.0).abs().to_degrees() - theta_esperado).abs() < 1e-3);
            assert!((x.tau(0.0) - x.fuerza() * d).abs() < 1e-9);
        }
    }

    /// La palanca por otra cuenta: el doble del área del triángulo
    /// codo-mano-polea entre su base (la mano a la polea).
    #[test]
    fn la_palanca_por_el_area() {
        for x in [pegado(), inclinado()] {
            for g in [100.0, 70.0, 40.0, 10.0, 0.0] {
                let phi = grados(g);
                let (c, m, p) = (x.codo, x.mano(phi), x.polea);
                let doble_area = ((m.0 - c.0) * (p.1 - c.1) - (m.1 - c.1) * (p.0 - c.0)).abs();
                let base = (p.0 - m.0).hypot(p.1 - m.1);
                assert!((x.brazo(phi) - doble_area / base).abs() < 1e-12, "a {g}°");
            }
        }
    }

    /// El pie de la perpendicular cae sobre la línea del cable y a la
    /// distancia de la palanca del codo.
    #[test]
    fn el_pie_esta_en_la_linea_del_cable() {
        for x in [pegado(), inclinado()] {
            for g in [100.0, 50.0, 0.0] {
                let phi = grados(g);
                let (pie, m) = (x.pie(phi), x.mano(phi));
                let (ux, uy) = x.linea().direccion(m);
                let fuera = ((pie.0 - m.0) * uy - (pie.1 - m.1) * ux).abs();
                assert!(fuera < 1e-12, "el pie se salió de la línea: {fuera}");
                let d = (pie.0 - x.codo.0).hypot(pie.1 - x.codo.1);
                assert!((d - x.brazo(phi)).abs() < 1e-12);
            }
        }
    }

    /// El cable resiste toda la repetición en las dos: nunca cruza por
    /// cero (nunca ayuda a extender).
    #[test]
    fn el_cable_siempre_resiste() {
        for x in [pegado(), inclinado()] {
            assert!(zero_crossing(&x, 0.0, x.flexion_max_rad).is_none());
            assert!(x.tau(0.0) > 0.0 && x.tau(x.flexion_max_rad) > 0.0);
        }
    }

    /// Pegado, el cable baja casi a plomo en el bloqueo (4°); inclinado
    /// llega inclinado (18°). Y la placa sube lo que se alarga el cable: de
    /// 1.036 a 1.354 m pegado (31.7 cm) y de 1.105 a 1.490 m inclinado
    /// (38.4 cm), a mano.
    #[test]
    fn el_cable_a_plomo_contra_inclinado() {
        assert!(pegado().cable_rad(0.0).abs().to_degrees() < 5.0);
        assert!(inclinado().cable_rad(0.0).abs().to_degrees() > 17.0);
        assert!((pegado().recorrido(0.0) - (1.35370 - 1.03639)).abs() < 5e-5, "{}", pegado().recorrido(0.0));
        assert!((inclinado().recorrido(0.0) - (1.48960 - 1.10520)).abs() < 5e-5, "{}", inclinado().recorrido(0.0));
        assert!(pegado().recorrido(pegado().flexion_max_rad).abs() < 1e-12);
    }
}
