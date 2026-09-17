//! La palanca: el brazo de momento de una carga alrededor de una
//! articulación.
//!
//! Es la cuenta que TODAS las piezas de comparación hacen, y por eso vive
//! aquí y no en cada una. Cuatro reels la traían escrita cuatro veces, con
//! su propia `G`, su propio producto cruz y su propia proyección; tres de
//! esas cuatro copias acabaron siendo ajustes de curva sobre los valores
//! dorados en vez de geometría, que es lo que pasa cuando la cuenta no
//! tiene dueño.
//!
//! Dos líneas de acción cubren todo el gimnasio: la gravedad, que siempre
//! apunta abajo, y un cable, que apunta a su polea. La diferencia entre
//! las dos ES el reel 04 y ES el reel 38, así que no es un detalle de
//! implementación: es el contenido.

use crate::G;

/// Un punto del plano sagital, en metros.
pub type Punto = (f64, f64);

/// Por dónde jala la carga.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Linea {
    /// Peso libre. La gravedad, hacia abajo, sin importar dónde esté la
    /// mano: por eso el brazo de momento de una barra es una distancia
    /// HORIZONTAL y nada más.
    Vertical,
    /// Cable. De la mano hacia la polea, así que la línea se mueve con la
    /// repetición. Ésa es toda la diferencia con la barra.
    Cable(Punto),
}

impl Linea {
    /// Vector unitario de la fuerza aplicada en `mano`.
    ///
    /// Para el cable apunta de la mano HACIA la polea, que es hacia donde
    /// jala: el cable tira, no empuja.
    #[must_use]
    pub fn direccion(&self, mano: Punto) -> Punto {
        match *self {
            Linea::Vertical => (0.0, -1.0),
            Linea::Cable(polea) => {
                let (dx, dy) = (polea.0 - mano.0, polea.1 - mano.1);
                let n = dx.hypot(dy);
                if n < 1e-12 {
                    (0.0, 0.0)
                } else {
                    (dx / n, dy / n)
                }
            }
        }
    }
}

/// El brazo de momento: distancia perpendicular del pivote a la línea de
/// acción, en metros.
///
/// Esto es el `d` que los reels dibujan punteado. No es la distancia del
/// pivote a la mano: es la distancia a la RECTA por la que jala la carga,
/// y se muere cuando la recta pasa por el pivote, que es el momento en
/// que el ejercicio deja de pedir nada a esa articulación.
#[must_use]
pub fn brazo(pivote: Punto, mano: Punto, linea: Linea) -> f64 {
    let (ux, uy) = linea.direccion(mano);
    let (rx, ry) = (mano.0 - pivote.0, mano.1 - pivote.1);
    (rx * uy - ry * ux).abs()
}

/// Momento de la carga alrededor del pivote, con signo, en newton-metro.
///
/// El signo es la componente en z de `r × F`, con z saliendo del papel.
/// Va con signo y no en valor absoluto por la misma razón que [`crate::Lift::tau`]:
/// un cable que deja de estorbar y empieza a ayudar cruza por cero, y en
/// valor absoluto ese cruce es invisible.
#[must_use]
pub fn momento(pivote: Punto, mano: Punto, fuerza_n: f64, linea: Linea) -> f64 {
    let (ux, uy) = linea.direccion(mano);
    let (rx, ry) = (mano.0 - pivote.0, mano.1 - pivote.1);
    fuerza_n * (rx * uy - ry * ux)
}

/// Lo que pesa una carga, en newton.
#[must_use]
pub fn peso(kg: f64) -> f64 {
    kg * G
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Palanca de tres metros, cien newton, colgando: 300 N·m. Aritmética
    /// de mano, no salida del motor.
    #[test]
    fn la_palanca_del_oraculo() {
        let t = momento((0.0, 0.0), (3.0, 0.0), 100.0, Linea::Vertical);
        assert!((t.abs() - 300.0).abs() < 1e-9, "dio {t}");
    }

    /// Con la carga a plomo del pivote no hay palanca, y da CERO exacto.
    #[test]
    fn a_plomo_no_hay_palanca() {
        assert!(brazo((0.0, 0.0), (0.0, -0.5), Linea::Vertical) < 1e-15);
    }

    /// Un cable que pasa por el pivote tampoco tiene palanca, aunque la
    /// mano esté lejísimos. Esto es lo que la versión vertical no ve.
    #[test]
    fn el_cable_por_el_pivote_tampoco() {
        let d = brazo((0.0, 0.0), (0.6, 0.0), Linea::Cable((1.4, 0.0)));
        assert!(d < 1e-12, "dio {d}");
    }

    /// Misma mano, dos líneas, dos palancas distintas: eso es el reel 04.
    #[test]
    fn la_linea_cambia_la_palanca() {
        let mano = (0.0, -0.32);
        let vertical = brazo((0.0, 0.0), mano, Linea::Vertical);
        let cable = brazo((0.0, 0.0), mano, Linea::Cable((0.42, -0.85)));
        assert!(vertical < 1e-15);
        assert!(cable > 0.1, "el cable sí tiene palanca ahí: {cable}");
    }

    /// El signo se voltea al pasar la carga del otro lado del pivote.
    #[test]
    fn el_signo_cuenta_de_que_lado_esta() {
        let a = momento((0.0, 0.0), (0.3, 0.0), 100.0, Linea::Vertical);
        let b = momento((0.0, 0.0), (-0.3, 0.0), 100.0, Linea::Vertical);
        assert!(a * b < 0.0, "{a} y {b} deberían tener signos opuestos");
    }
}
