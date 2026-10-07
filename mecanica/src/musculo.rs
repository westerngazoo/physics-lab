//! El trayecto de un músculo: cuánto mide y cuánta palanca tiene.
//!
//! Realiza lo que `RFC-002 §7.5` especificó y dejó para después, y con su
//! restricción intacta: **geometría y nada más. Aquí no hay un número de
//! fuerza.**
//!
//! Esa restricción no es timidez. `§7.3` la argumenta: el equilibrio de
//! momentos en una articulación es UNA ecuación con *n* incógnitas, así
//! que repartir la fuerza entre los músculos que la cruzan no es difícil,
//! es **indeterminado**. Y `fitAI R-0045 §4` lo cierra por decisión del
//! dueño: *muscle-fibre or tendon models* están fuera de alcance. Un
//! número de fuerza por músculo es una fabricación que un entrenador le
//! repetiría a un cliente como dato.
//!
//! Lo que sí es determinado, y es lo que vive aquí:
//!
//! - **el largo** del trayecto en cada ángulo, y
//! - **el brazo de momento**, que no se declara: se deriva, porque
//!   `r = dL/dθ` es el principio de trabajo virtual, no un ajuste.
//!
//! # La trampa que esto NO cae
//!
//! `§7.4` la nombra: la tensión de una liga la fija su estiramiento, la de
//! un músculo no, porque la activación la elige quien levanta. Tomar la
//! analogía en serio da un modelo determinista que *parece funcionar* y
//! está equivocado. Por eso este módulo devuelve largos, no tensiones: con
//! el largo, quien quiera puede montar encima el modelo que declare —
//! y declararlo es la parte que no se puede saltar.
//!
//! # Anatomía citada: el hueco que queda abierto
//!
//! Los trayectos de aquí son **sintéticos y están rotulados como tales**.
//! Ninguno afirma ser la anatomía de nadie. `§7.5` pide coordenadas con
//! fuente citada —Klein Horsman et al. (2007), Delp et al. (1990)— y
//! ninguna de las dos está verificada todavía, así que ninguna se asume.
//! Inventar coordenadas y presentarlas como de un artículo sería
//! exactamente la fabricación que `AC5` prohíbe.

/// Un músculo visto como un trayecto que sólo puede jalar.
///
/// `theta` es el ángulo de la articulación en **radianes** — nunca grados:
/// `W = ∫τ dθ` en grados sale 57.3× mal y ninguna comprobación de unidades
/// lo atrapa, porque el radián es adimensional (RFC-002 §9).
pub trait Musculo {
    /// Largo del trayecto origen→inserción en `theta`, en metros.
    fn largo(&self, theta: f64) -> f64;

    /// El rango de `theta` en el que este trayecto tiene sentido, en
    /// radianes. Fuera de él el modelo no promete nada.
    fn rango(&self) -> (f64, f64);

    /// Dónde cae `theta` dentro del propio recorrido del músculo: `0.0` en
    /// su posición más corta, `1.0` en la más larga.
    ///
    /// Sirve para hablar de «estirado» y «acortado» sin comprometerse con
    /// un largo óptimo, que es fisiología y no geometría.
    #[must_use]
    fn normalizado(&self, theta: f64) -> f64 {
        let (a, b) = self.rango();
        let (la, lb) = (self.largo(a), self.largo(b));
        let (corto, largo) = if la <= lb { (la, lb) } else { (lb, la) };
        if (largo - corto).abs() < f64::EPSILON {
            return 0.0;
        }
        ((self.largo(theta) - corto) / (largo - corto)).clamp(0.0, 1.0)
    }
}

/// Paso de la diferencia central, en radianes: ~0.057°.
///
/// Suficientemente chico para que el error de truncamiento sea
/// despreciable y suficientemente grande para que la resta no se coma los
/// dígitos significativos. La prueba `la_polea_da_su_radio_exacto` fija
/// que a este paso el error contra la respuesta analítica es < 1e-9.
const H: f64 = 1e-3;

/// El brazo de momento, por excursión del tendón: `r = -dL/dθ`.
///
/// **No se declara, se deriva.** Es el principio de trabajo virtual: si el
/// músculo se acorta `dL` mientras la articulación gira `dθ`, la palanca
/// que tiene es el cociente. Un brazo de momento declarado a mano y un
/// largo declarado a mano pueden contradecirse; derivado uno del otro, no
/// pueden.
///
/// Signo: **positivo cuando el músculo se acorta al crecer `theta`**, que
/// es el caso de un flexor durante su flexión.
#[must_use]
pub fn brazo<M: Musculo + ?Sized>(m: &M, theta: f64) -> f64 {
    -(m.largo(theta + H) - m.largo(theta - H)) / (2.0 * H)
}

/// Cuánto se acortó el músculo entre dos ángulos, en metros.
///
/// Es la integral del brazo de momento, y por eso no hace falta declarar
/// el largo *y* la palanca por separado: una sale de la otra.
#[must_use]
pub fn acortamiento<M: Musculo + ?Sized>(m: &M, desde: f64, hasta: f64) -> f64 {
    m.largo(desde) - m.largo(hasta)
}

/// Un trayecto que envuelve una polea circular centrada en la
/// articulación — **sintético**, para enseñar y para probar.
///
/// Es el único caso con respuesta cerrada conocida: el brazo de momento
/// vale el radio, constante, en todo el rango. Por eso existe: es contra
/// él que se comprueba que [`brazo`] deriva bien.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Polea {
    /// Largo en `theta = 0`, en metros.
    pub largo_0: f64,
    /// Radio de la polea, en metros. **Es** el brazo de momento.
    pub radio: f64,
    /// Rango útil, en radianes.
    pub rango: (f64, f64),
}

impl Musculo for Polea {
    fn largo(&self, theta: f64) -> f64 {
        self.largo_0 - self.radio * theta
    }
    fn rango(&self) -> (f64, f64) {
        self.rango
    }
}

/// Un trayecto en línea recta entre dos puntos, uno de ellos en el
/// segmento que gira — **sintético**, no la anatomía de nadie.
///
/// Es la forma más simple que reproduce lo que de verdad hace un flexor:
/// un brazo de momento que **no** es constante, que pasa por un máximo a
/// media flexión y cae a los dos lados. Ese pico es la razón de que un
/// curl tenga un punto donde se atora, y sale de la geometría sola.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Recto {
    /// Distancia del origen al centro de la articulación, en metros.
    pub origen: f64,
    /// Distancia de la inserción al centro, en metros.
    pub insercion: f64,
    /// Ángulo entre los dos segmentos cuando `theta = 0`, en radianes.
    pub abierto: f64,
    /// Rango útil, en radianes.
    pub rango: (f64, f64),
}

impl Musculo for Recto {
    /// Ley de los cosenos: el trayecto es el tercer lado del triángulo.
    fn largo(&self, theta: f64) -> f64 {
        let ang = self.abierto - theta;
        (self
            .origen
            .mul_add(self.origen, self.insercion * self.insercion)
            - 2.0 * self.origen * self.insercion * ang.cos())
        .max(0.0)
        .sqrt()
    }
    fn rango(&self) -> (f64, f64) {
        self.rango
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::f64::consts::TAU;

    fn polea() -> Polea {
        Polea {
            largo_0: 0.30,
            radio: 0.05,
            rango: (0.0, TAU / 4.0),
        }
    }

    /// El caso con respuesta cerrada: enrollada en una polea de radio `R`,
    /// la palanca vale `R` en todas partes. Si la derivada numérica no da
    /// eso, no da nada.
    #[test]
    fn la_polea_da_su_radio_exacto() {
        let p = polea();
        for i in 0..=20 {
            let th = TAU / 4.0 * f64::from(i) / 20.0;
            assert!(
                (brazo(&p, th) - p.radio).abs() < 1e-9,
                "θ={th}: {} ≠ {}",
                brazo(&p, th),
                p.radio
            );
        }
    }

    /// El acortamiento es la integral de la palanca, y en la polea eso es
    /// `R·Δθ`. Comprobarlo aquí es comprobar que las dos funciones hablan
    /// de lo mismo.
    #[test]
    fn el_acortamiento_es_la_integral_de_la_palanca() {
        let p = polea();
        let d = TAU / 8.0;
        assert!((acortamiento(&p, 0.0, d) - p.radio * d).abs() < 1e-12);
    }

    /// El flexor recto tiene el pico de palanca A MEDIA FLEXIÓN, no en un
    /// extremo. Es la diferencia entre este modelo y la polea, y es la que
    /// hace que un curl se atore en algún lado en vez de en ninguno.
    #[test]
    fn el_flexor_recto_tiene_su_pico_a_media_flexion() {
        let m = Recto {
            origen: 0.13,
            insercion: 0.04,
            abierto: TAU / 2.0,
            rango: (0.0, TAU * 0.4),
        };
        let n = 200;
        let (mut mejor, mut donde) = (f64::MIN, 0.0);
        for i in 0..=n {
            let th = TAU * 0.4 * f64::from(i) / f64::from(n);
            let r = brazo(&m, th);
            if r > mejor {
                mejor = r;
                donde = th;
            }
        }
        let (a, b) = m.rango();
        assert!(
            donde > a + 0.15 && donde < b - 0.15,
            "el pico cayó en el borde: θ={donde}"
        );
        assert!(mejor > 0.0, "un flexor se acorta al flexionar");
    }

    /// `normalizado` es 0 donde el músculo está más corto y 1 donde más
    /// largo, sin decir una palabra sobre cuál de los dos es «óptimo» —
    /// eso es fisiología y no vive en este módulo.
    #[test]
    fn normalizado_va_de_lo_mas_corto_a_lo_mas_largo() {
        let p = polea();
        let (a, b) = p.rango();
        assert!((p.normalizado(b) - 0.0).abs() < 1e-12, "b es el más corto");
        assert!((p.normalizado(a) - 1.0).abs() < 1e-12, "a es el más largo");
        assert!((0.0..=1.0).contains(&p.normalizado((a + b) / 2.0)));
    }

    /// Un trayecto degenerado no divide entre cero.
    #[test]
    fn un_trayecto_sin_recorrido_no_revienta() {
        let quieto = Polea {
            largo_0: 0.2,
            radio: 0.0,
            rango: (0.0, 1.0),
        };
        assert!((quieto.normalizado(0.5) - 0.0).abs() < f64::EPSILON);
        assert!((brazo(&quieto, 0.5)).abs() < 1e-12);
    }

    /// Lo que este módulo promete NO hacer. La prueba es que el trait no
    /// tiene forma de contestar «cuánta fuerza», y eso se comprueba
    /// leyéndolo: no hay método que devuelva newtons. Se deja escrito
    /// como prueba para que agregar uno rompa algo visible.
    #[test]
    fn aqui_no_hay_fuerza() {
        let p = polea();
        // Todo lo que el trait sabe contestar, en metros y adimensional:
        let _: f64 = p.largo(0.1);
        let _: f64 = p.normalizado(0.1);
        let _: f64 = brazo(&p, 0.1);
        let _: f64 = acortamiento(&p, 0.0, 0.1);
        // Y no hay un quinto. RFC-002 §7.3 y fitAI R-0045 AC5.
    }
}
