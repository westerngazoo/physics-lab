//! `vehiculo` — el vehículo en la curva, en forma cerrada.
//!
//! Una llanta empuja de lado con, como mucho, μ veces la fuerza que la
//! aprieta contra el piso: el **círculo de fricción**. En piso plano y sin
//! alerones esa fuerza es m g, así que el tope de la aceleración es μ g y la
//! masa se cancela. Todo aquí es aceleración, rapidez y geometría del plano.
//!
//! Lo que sale de eso:
//!
//! - [`agarre`]: la aceleración que la llanta sí da, de la que se pide.
//! - [`v_tope`]: la rapidez más alta en una curva de radio r, √(μ g r).
//! - [`radio_minimo`]: el giro más cerrado a rapidez v, v²/(μ g). Pasado el
//!   tope el vehículo no se sale en línea recta: sigue este círculo, más
//!   abierto que la curva ([`centro_de_giro`] dice dónde).
//! - [`trazada`]: el radio de la trazada que abre la curva (de afuera al
//!   vértice y de vuelta afuera), r + w/(1 − cos(θ/2)).
//! - [`aceleracion_en_curva`]: la ley que un integrador necesita para mover
//!   un vehículo que quiere seguir un círculo. Aquí no se integra nada
//!   (las páginas no dan pasos de tiempo); quien la integre puede comparar
//!   contra [`centro_de_giro`] y [`radio_minimo`], que son exactos.
//!
//! # El modelo, y lo que no es
//!
//! Un punto con un solo μ, en piso plano, sin alerones, sin transferencia de
//! carga entre llantas, sin ángulo de deriva ni curva de la llanta. Es el
//! límite que todos los refinamientos corrigen, no un modelo de llanta.
//!
//! Unidades SI (m, s, rad); el plano es (x, y) con y hacia arriba.

#![forbid(unsafe_code)]

/// La gravedad estándar, 9.806 65 m/s², exacta por definición (CGPM, 1901).
pub const G_ESTANDAR: f64 = 9.806_65;

fn largo(a: [f64; 2]) -> f64 {
    (a[0] * a[0] + a[1] * a[1]).sqrt()
}

/// El círculo de fricción: la aceleración `pedida` (m/s²), recortada a μ g
/// en su largo y con su misma dirección.
pub fn agarre(pedida: [f64; 2], mu: f64, g: f64) -> [f64; 2] {
    let l = largo(pedida);
    let tope = mu * g;
    if l <= tope {
        pedida
    } else {
        [pedida[0] * tope / l, pedida[1] * tope / l]
    }
}

/// La rapidez más alta para una curva de radio `r`: donde la aceleración que
/// pide la curva, v²/r, llega al tope μ g. v = √(μ g r): el doble de radio
/// no da el doble de rapidez, da √2.
pub fn v_tope(mu: f64, g: f64, r: f64) -> f64 {
    (mu * g * r).sqrt()
}

/// El giro más cerrado posible a rapidez `v`: r = v²/(μ g). Es también el
/// círculo que sigue un vehículo que entró a una curva más cerrada demasiado
/// rápido: la llanta da todo lo que tiene y no alcanza.
pub fn radio_minimo(v: f64, mu: f64, g: f64) -> f64 {
    v * v / (mu * g)
}

/// El centro del círculo de radio `radio` que pasa por `x` tangente a la
/// velocidad `v`, del lado de `hacia` (el centro de la curva que se quería
/// seguir). Con [`radio_minimo`], es la trayectoria exacta de un vehículo
/// que perdió el agarre en `x`.
///
/// Con `v = 0` no hay tangente y devuelve `x`.
pub fn centro_de_giro(x: [f64; 2], v: [f64; 2], radio: f64, hacia: [f64; 2]) -> [f64; 2] {
    let n = normal_hacia(v, [hacia[0] - x[0], hacia[1] - x[1]]);
    [x[0] + radio * n[0], x[1] + radio * n[1]]
}

/// La normal unitaria a `v` del lado de `d`; cero si `v` es cero.
fn normal_hacia(v: [f64; 2], d: [f64; 2]) -> [f64; 2] {
    let lv = largo(v);
    if lv == 0.0 {
        return [0.0, 0.0];
    }
    let n = [-v[1] / lv, v[0] / lv];
    if n[0] * d[0] + n[1] * d[1] < 0.0 {
        [-n[0], -n[1]]
    } else {
        n
    }
}

/// El radio de la trazada abierta en una curva de `giro` radianes (θ, en
/// (0, π]), radio interior `r_interior` y ancho útil `ancho` (el de la pista
/// menos el del vehículo).
///
/// Es el círculo más grande que cabe: tangente al borde exterior de la recta
/// de entrada y al de la de salida, y tangente por dentro al borde interior
/// en el vértice. Con su centro sobre la bisectriz a una distancia d del
/// centro de la curva, las dos tangencias piden R = r + w + d cos(θ/2) y
/// R = r + d, de donde
///
/// ```text
/// R = r + w / (1 − cos(θ/2)).
/// ```
///
/// A 90°, R = r + (2 + √2) w; en una horquilla (180°), R = r + w. Con
/// `giro` = 0 no hay curva y el radio es infinito; fuera de (0, π] la
/// geometría es otra y devuelve NaN.
pub fn trazada(r_interior: f64, ancho: f64, giro: f64) -> f64 {
    if giro == 0.0 {
        return f64::INFINITY;
    }
    if !(0.0..=std::f64::consts::PI).contains(&giro) {
        return f64::NAN;
    }
    r_interior + ancho / (1.0 - (giro / 2.0).cos())
}

/// La aceleración de un vehículo en `x` con velocidad `v` que quiere seguir
/// el círculo de centro `centro` y radio `r` a rapidez constante: pide v²/r,
/// perpendicular a la velocidad y del lado del centro, y recibe lo que el
/// [`agarre`] le dé.
///
/// Si pide más que μ g se abre: integrada, esta ley da un círculo de radio
/// [`radio_minimo`] con centro en [`centro_de_giro`]. Con `v = 0` no pide
/// nada.
pub fn aceleracion_en_curva(
    x: [f64; 2],
    v: [f64; 2],
    centro: [f64; 2],
    r: f64,
    mu: f64,
    g: f64,
) -> [f64; 2] {
    let n = normal_hacia(v, [centro[0] - x[0], centro[1] - x[1]]);
    let v2 = v[0] * v[0] + v[1] * v[1];
    agarre([n[0] * v2 / r, n[1] * v2 / r], mu, g)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::{FRAC_PI_2, PI, SQRT_2};

    const MU: f64 = 1.2;

    /// A1: lo que cabe en el círculo pasa entero; lo que no, queda en μ g con
    /// la misma dirección.
    #[test]
    fn a1_el_agarre_recorta_al_circulo_sin_girar() {
        assert_eq!(agarre([3.0, 4.0], MU, G_ESTANDAR), [3.0, 4.0]);
        let a = agarre([30.0, -40.0], MU, G_ESTANDAR);
        assert!((largo(a) - MU * G_ESTANDAR).abs() < 1e-12);
        assert!((a[1] / a[0] + 40.0 / 30.0).abs() < 1e-12);
    }

    /// A2: la masa no aparece: μ (m g) / m es μ g para cualquier m. La firma
    /// de [`agarre`] ya lo dice; esto lo dice en números.
    #[test]
    fn a2_la_masa_se_cancela() {
        for m in [80.0, 145.0, 1800.0] {
            let tope = MU * (m * G_ESTANDAR) / m;
            let a = agarre([100.0, 0.0], MU, G_ESTANDAR);
            assert!((a[0] - tope).abs() < 1e-12);
        }
    }

    /// V1: el doble de radio da √2 veces la rapidez, no el doble.
    #[test]
    fn v1_doble_radio_no_es_doble_rapidez() {
        let razon = v_tope(MU, G_ESTANDAR, 40.0) / v_tope(MU, G_ESTANDAR, 20.0);
        assert!((razon - SQRT_2).abs() < 1e-12);
        assert_eq!((v_tope(MU, G_ESTANDAR, 20.0) * 3.6).round(), 55.0);
        assert_eq!((v_tope(MU, G_ESTANDAR, 40.0) * 3.6).round(), 78.0);
    }

    /// V2: v_tope y radio_minimo son la misma ley leída al revés.
    #[test]
    fn v2_tope_y_radio_minimo_son_inversos() {
        for r in [5.0, 11.0, 37.3, 200.0] {
            let v = v_tope(MU, G_ESTANDAR, r);
            assert!((radio_minimo(v, MU, G_ESTANDAR) - r).abs() < 1e-12 * r);
        }
    }

    /// V3: 60 km/h en una curva de 20 m con μ = 1.2 no cabe: el vehículo se
    /// abre a un círculo de 23.6 m, tangente a donde perdió el agarre.
    #[test]
    fn v3_pasado_el_tope_se_abre_a_un_circulo_tangente() {
        let v = 60.0 / 3.6;
        assert!(v > v_tope(MU, G_ESTANDAR, 20.0));
        let rm = radio_minimo(v, MU, G_ESTANDAR);
        assert!((rm - 23.6).abs() < 0.01, "{rm}");
        // en (20, 0) yendo hacia +y, con la curva centrada en el origen
        let c = centro_de_giro([20.0, 0.0], [0.0, v], rm, [0.0, 0.0]);
        assert!((c[0] - (20.0 - rm)).abs() < 1e-12 && c[1] == 0.0, "{c:?}");
    }

    /// T1: la trazada de 90° es r + (2 + √2) w; la del reel, 37.3 m.
    #[test]
    fn t1_la_trazada_a_noventa_grados() {
        let r = trazada(10.0, 8.0, FRAC_PI_2);
        assert!((r - (10.0 + (2.0 + SQRT_2) * 8.0)).abs() < 1e-12);
        assert!((r - 37.3).abs() < 0.05, "{r}");
    }

    /// T2: la trazada es tangente a las dos rectas exteriores y al borde
    /// interior, comprobado con distancias y no con la fórmula.
    #[test]
    fn t2_la_trazada_toca_los_tres_bordes() {
        for giro in [0.4, FRAC_PI_2, 2.0, PI] {
            let (ri, w) = (10.0, 8.0);
            let rr = trazada(ri, w, giro);
            // normales de las dos rectas exteriores, a ±θ/2 de la bisectriz +x
            let u = [1.0, 0.0];
            let d = rr - ri;
            let c = [-d * u[0], -d * u[1]];
            for s in [-1.0, 1.0] {
                let n = [(s * giro / 2.0).cos(), (s * giro / 2.0).sin()];
                let distancia = (ri + w) - (n[0] * c[0] + n[1] * c[1]);
                assert!((distancia - rr).abs() < 1e-9, "θ = {giro}: {distancia} vs {rr}");
            }
            assert!((largo(c) + ri - rr).abs() < 1e-9, "tangente por dentro al vértice");
        }
    }

    /// T3: los extremos: una horquilla da r + w; sin giro, infinito; fuera de
    /// (0, π], NaN.
    #[test]
    fn t3_horquilla_recta_y_fuera_de_dominio() {
        assert!((trazada(10.0, 8.0, PI) - 18.0).abs() < 1e-12);
        assert_eq!(trazada(10.0, 8.0, 0.0), f64::INFINITY);
        assert!(trazada(10.0, 8.0, -0.1).is_nan());
        assert!(trazada(10.0, 8.0, 3.5).is_nan());
    }

    /// C1: la ley pide v²/r perpendicular a v y del lado del centro, y el
    /// agarre la recorta cuando no cabe.
    #[test]
    fn c1_la_curva_pide_perpendicular_y_hacia_adentro() {
        let (x, centro) = ([20.0, 0.0], [0.0, 0.0]);
        let v = [0.0, 50.0 / 3.6];
        let a = aceleracion_en_curva(x, v, centro, 20.0, MU, G_ESTANDAR);
        assert!(a[1].abs() < 1e-15, "perpendicular a v");
        assert!((a[0] + v[1] * v[1] / 20.0).abs() < 1e-12, "v²/r hacia el centro");
        let rapido = [0.0, 60.0 / 3.6];
        let b = aceleracion_en_curva(x, rapido, centro, 20.0, MU, G_ESTANDAR);
        assert!((b[0] + MU * G_ESTANDAR).abs() < 1e-12, "recortada a μ g");
        assert_eq!(aceleracion_en_curva(x, [0.0, 0.0], centro, 20.0, MU, G_ESTANDAR), [0.0, 0.0]);
    }
}
