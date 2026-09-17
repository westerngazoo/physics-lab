//! Jalón al pecho en polea: derecho o reclinado.
//!
//! De los reels 26 y 29, que son el mismo ejercicio mirado dos veces. Se
//! modela una sola vez aquí porque es un solo ejercicio, y porque tenerlo
//! dos veces fue justo lo que dejó que uno de los dos se convirtiera en
//! un ajuste de curva sin que nadie lo notara.
//!
//! LA SUPOSICIÓN QUE DECIDE TODO. El final del jalón no es un punto fijo
//! del mundo: es TU esternón, y al reclinarte se va contigo. Dejar la
//! mano clavada en el aire mientras el torso se echa atrás infla el
//! torque más del doble, y es mentira: la barra viaja contigo.
//!
//! Con el torso derecho el cable cae a plomo y el brazo de momento es la
//! distancia horizontal hombro-barra, que no cambia en toda la bajada: el
//! torque sale CONSTANTE. Eso no es una curiosidad del modelo, es el
//! hallazgo que publicó el reel 26.
//!
//! El marco tiene L5/S1 en el origen.

use crate::maquina_humana::{codo, Doblez, Persona};
use crate::palanca::{self, Linea, Punto};

/// Un jalón al pecho.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Jalon {
    /// La placa, kilogramos.
    pub carga_kg: f64,
    /// Cadera → hombro, metros.
    pub torso_m: f64,
    /// Húmero, metros.
    pub humero_m: f64,
    /// Antebrazo, metros.
    pub antebrazo_m: f64,
    /// Del hombro al esternón: al frente, y hacia abajo.
    ///
    /// No son a ojo. El 0.24 está calibrado para que, sentado derecho, el
    /// esternón caiga justo bajo la barra y el modelo devuelva los mismos
    /// N·m que la otra pieza publicó. Calibrar aquí es lo que evita que
    /// dos reels se contradigan.
    pub esternon_fwd: f64,
    /// Ídem, hacia abajo desde el hombro.
    pub esternon_up: f64,
    /// La polea, en el marco de L5/S1.
    pub polea: Punto,
    /// Cuánto se reclina el torso, radianes.
    pub reclinado_rad: f64,
    /// Qué fracción del brazo estirado se alcanza arriba. Un brazo
    /// bloqueado al 100 % es una singularidad, no una postura.
    pub extension: f64,
}

impl Jalon {
    /// Construye el jalón para una persona, con la geometría de máquina
    /// declarada aparte. Los largos salen de Winter, no del aire.
    #[must_use]
    pub fn para(p: &Persona, carga_kg: f64, polea: Punto) -> Self {
        Jalon {
            carga_kg,
            torso_m: p.torso(),
            humero_m: p.humero(),
            antebrazo_m: p.antebrazo(),
            esternon_fwd: 0.24,
            esternon_up: 0.15,
            polea,
            reclinado_rad: 0.0,
            extension: 0.97,
        }
    }

    /// Los ejes del tronco reclinado: arriba y al frente.
    fn ejes(&self) -> (Punto, Punto) {
        let (s, c) = self.reclinado_rad.sin_cos();
        ((-s, c), (c, s))
    }

    /// El hombro.
    #[must_use]
    pub fn hombro(&self) -> Punto {
        let (up, _) = self.ejes();
        (self.torso_m * up.0, self.torso_m * up.1)
    }

    /// El esternón: donde termina el jalón, y viaja con el torso.
    #[must_use]
    pub fn esternon(&self) -> Punto {
        let (up, fw) = self.ejes();
        let s = self.hombro();
        (
            s.0 - self.esternon_up * up.0 + self.esternon_fwd * fw.0,
            s.1 - self.esternon_up * up.1 + self.esternon_fwd * fw.1,
        )
    }

    /// Arriba del todo: la barra CUELGA, así que la mano está en la
    /// vertical de la polea, tan alto como alcance el brazo casi estirado
    /// desde ese hombro.
    ///
    /// Esto es geometría de una línea. Vivió durante un tiempo como una
    /// constante de dieciséis dígitos despejada de un valor dorado, que
    /// es la forma de que un modelo pase su propia prueba sin describir
    /// nada.
    #[must_use]
    pub fn arranque(&self) -> Punto {
        let s = self.hombro();
        let largo = (self.humero_m + self.antebrazo_m) * self.extension;
        let dx = self.polea.0 - s.0;
        let alto = (largo * largo - dx * dx).max(0.0).sqrt();
        (self.polea.0, s.1 + alto)
    }

    /// La mano en el avance `u`: 0 arriba con el brazo estirado, 1 en el
    /// esternón.
    #[must_use]
    pub fn mano(&self, u: f64) -> Punto {
        let a = self.arranque();
        let b = self.esternon();
        (a.0 + (b.0 - a.0) * u, a.1 + (b.1 - a.1) * u)
    }

    /// El codo, que en el jalón CAE, no se dobla hacia arriba.
    ///
    /// # Errors
    /// Si la mano queda fuera del alcance del brazo.
    pub fn codo(&self, u: f64) -> Result<Punto, garust::twolink::TwoLinkError> {
        let p = Persona {
            estatura_m: (self.humero_m + self.antebrazo_m) / crate::maquina_humana::largo::AGARRE,
            masa_kg: 0.0,
        };
        let s = self.hombro();
        let m = self.mano(u);
        let c = codo(&p, [s.0, s.1], [m.0, m.1], Doblez::Abajo)?;
        Ok((c[0], c[1]))
    }

    /// Torque en el hombro, N·m, en el avance `u`.
    #[must_use]
    pub fn tau(&self, u: f64) -> f64 {
        palanca::momento(
            self.hombro(),
            self.mano(u),
            palanca::peso(self.carga_kg),
            Linea::Cable(self.polea),
        )
        .abs()
    }

    /// Largo del tramo polea-mano, metros.
    #[must_use]
    pub fn largo_cable(&self, u: f64) -> f64 {
        let m = self.mano(u);
        (self.polea.0 - m.0).hypot(self.polea.1 - m.1)
    }

    /// Lo que sube la placa en una repetición, metros.
    #[must_use]
    pub fn recorrido(&self) -> f64 {
        self.largo_cable(1.0) - self.largo_cable(0.0)
    }

    /// `W = tensión × recorrido`, joules.
    ///
    /// Aquí el área bajo la curva NO es el trabajo: el eje del jalón es
    /// un avance sin unidades, no un ángulo. El trabajo sale de lo que
    /// viaja la placa, que es lo que la máquina levanta de verdad.
    #[must_use]
    pub fn trabajo(&self) -> f64 {
        palanca::peso(self.carga_kg) * self.recorrido()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// El montaje publicado de los reels 26 y 29.
    fn derecho() -> Jalon {
        Jalon {
            carga_kg: 50.0,
            torso_m: 0.55,
            humero_m: 0.32,
            antebrazo_m: 0.28,
            esternon_fwd: 0.24,
            esternon_up: 0.15,
            polea: (0.24, 1.60),
            reclinado_rad: 0.0,
            extension: 0.97,
        }
    }

    /// El hallazgo del reel 26: con el torso derecho el torque NO se
    /// mueve en toda la bajada.
    #[test]
    fn derecho_el_torque_es_constante() {
        let j = derecho();
        let t0 = j.tau(0.0);
        for i in 0..=20 {
            let t = j.tau(f64::from(i) / 20.0);
            assert!((t - t0).abs() < 1e-9, "en u={i}/20 dio {t}, no {t0}");
        }
    }

    /// Y vale la tensión por la distancia horizontal hombro-barra, que es
    /// aritmética de mano: 50 × 9.81 × 0.24.
    #[test]
    fn derecho_contra_aritmetica_de_mano() {
        let esperado = 50.0 * 9.81 * 0.24;
        assert!((derecho().tau(0.5) - esperado).abs() < 1e-9);
    }

    /// El arranque se DERIVA, no se despeja de un valor guardado.
    #[test]
    fn el_arranque_sale_de_la_geometria() {
        let j = derecho();
        let largo = (0.32 + 0.28) * 0.97;
        let esperado = 0.55 + (largo * largo - 0.24_f64 * 0.24).sqrt();
        assert!((j.arranque().1 - esperado).abs() < 1e-12);
    }

    /// El esternón también.
    #[test]
    fn el_esternon_sale_de_la_geometria() {
        let e = derecho().esternon();
        assert!((e.0 - 0.24).abs() < 1e-12 && (e.1 - 0.40).abs() < 1e-12);
    }

    /// Reclinarse sube el pico y encoge el recorrido. Las dos cosas a la
    /// vez, que es lo que hace que no sea "mejor" sino otro ejercicio.
    #[test]
    fn reclinarse_sube_el_pico_y_encoge_el_recorrido() {
        let d = derecho();
        let r = Jalon {
            reclinado_rad: 30.0_f64.to_radians(),
            ..d
        };
        assert!(r.tau(0.0) > d.tau(0.0), "el reclinado arranca más alto");
        assert!(
            r.recorrido().abs() < d.recorrido().abs(),
            "y viaja menos: {} contra {}",
            r.recorrido().abs(),
            d.recorrido().abs()
        );
    }
}
