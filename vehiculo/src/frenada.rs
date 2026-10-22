//! La frenada con freno solo atrás: por qué un kart frena peor que lo que
//! agarra.
//!
//! Al frenar, el piso jala las llantas hacia atrás a ras del piso, y el centro
//! de masa va a una altura `h`. Esas dos cosas no están en la misma línea, así
//! que tienden a irse de nariz, y el piso lo equilibra cargando más las llantas
//! de adelante y menos las de atrás: la transferencia de carga a lo largo,
//! m a h / L. Con el centro de masa a `b` del eje delantero y a L − b del
//! trasero, en una distancia entre ejes `L`:
//!
//! ```text
//! F_z,atrás    = m (g b − a h) / L
//! F_z,adelante = m (g (L − b) + a h) / L
//! ```
//!
//! ([`cargas_al_frenar`]). Si solo frenan las de atrás, la frenada que dan es,
//! como mucho, μ veces la carga que les queda, y esa carga baja al frenar más.
//! El tope es donde las dos cosas se encuentran, μ F_z,atrás = m a:
//!
//! ```text
//! a = μ g b / (L + μ h)
//! ```
//!
//! ([`frenada_solo_atras`]), menos que el μ g que daría frenar con las cuatro.
//! Dos cosas lo bajan: las de atrás solo llevan la parte b/L del peso, y al
//! frenar pierden todavía más (el μ h del denominador). La masa se cancela.
//!
//! # El modelo, y lo que no es
//!
//! Cuasi-estático (una frenada sostenida), piso plano, en línea recta, sin
//! aerodinámica, un solo μ constante, sin brincos ni cabeceo del chasis, y las
//! dos traseras con la misma carga (sin curva). Supone que el freno puede
//! bloquear las de atrás: el tope lo pone el agarre, no el freno.

/// Las cargas verticales (N) en las dos llantas de adelante y en las dos de
/// atrás, `(adelante, atrás)`, de un vehículo de masa `m` (kg) que frena a `a`
/// (m/s², positiva al frenar) con el centro de masa a `b` m del eje delantero,
/// a `h` m del piso y una distancia entre ejes `l` (m). Suman m g: frenar no
/// cambia la carga total, cambia el reparto.
pub fn cargas_al_frenar(m: f64, g: f64, a: f64, b: f64, l: f64, h: f64) -> (f64, f64) {
    let atras = m * (g * b - a * h) / l;
    let adelante = m * (g * (l - b) + a * h) / l;
    (adelante, atras)
}

/// La frenada más fuerte (m/s²) de un vehículo que solo frena con las llantas
/// de atrás: a = μ g b / (L + μ h), con `b` la distancia del centro de masa al
/// eje delantero, `l` la distancia entre ejes y `h` la altura del centro de
/// masa (m). Siempre menos que μ g si h > 0 o b < L.
pub fn frenada_solo_atras(mu: f64, g: f64, b: f64, l: f64, h: f64) -> f64 {
    mu * g * b / (l + mu * h)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::G_ESTANDAR;

    /// El kart de la casa (valores supuestos, modelados): 145 kg, 58 % del
    /// peso atrás, distancia entre ejes 1.04 m, h = 0.28 m, μ = 1.2.
    const M: f64 = 145.0;
    const L: f64 = 1.04;
    const B: f64 = 0.58 * L;
    const H: f64 = 0.28;
    const MU: f64 = 1.2;

    /// F1: las cargas suman m g con cualquier frenada; sin frenar, las de atrás
    /// llevan b/L del peso; al frenar, lo que pierden atrás lo ganan adelante,
    /// m a h / L.
    #[test]
    fn f1_frenar_cambia_el_reparto_no_la_suma() {
        let peso = M * G_ESTANDAR;
        for a in [0.0, 2.0, 5.16, 9.0] {
            let (d, t) = cargas_al_frenar(M, G_ESTANDAR, a, B, L, H);
            assert!((d + t - peso).abs() < 1e-9, "a = {a}: {d} + {t}");
        }
        let (_, t0) = cargas_al_frenar(M, G_ESTANDAR, 0.0, B, L, H);
        assert!((t0 / peso - 0.58).abs() < 1e-12);
        let (_, t1) = cargas_al_frenar(M, G_ESTANDAR, 3.0, B, L, H);
        assert!((t0 - t1 - M * 3.0 * H / L).abs() < 1e-9);
    }

    /// F2: en el tope, el agarre de las de atrás (μ por la carga que les queda)
    /// es justo la fuerza que pide la frenada, m a. Un poco más ya no cabe.
    #[test]
    fn f2_en_el_tope_el_agarre_de_atras_alcanza_justo() {
        let a = frenada_solo_atras(MU, G_ESTANDAR, B, L, H);
        let (_, t) = cargas_al_frenar(M, G_ESTANDAR, a, B, L, H);
        assert!((MU * t - M * a).abs() < 1e-9, "{} vs {}", MU * t, M * a);
        let (_, t) = cargas_al_frenar(M, G_ESTANDAR, a * 1.001, B, L, H);
        assert!(MU * t < M * a * 1.001);
    }

    /// F3: con el centro de masa en el piso no hay transferencia y queda μ g b/L;
    /// más alto, frena menos; y nunca llega a μ g.
    #[test]
    fn f3_mas_alto_frena_menos_y_nunca_llega_a_mu_g() {
        let a0 = frenada_solo_atras(MU, G_ESTANDAR, B, L, 0.0);
        assert!((a0 - MU * G_ESTANDAR * B / L).abs() < 1e-12);
        let mut antes = a0;
        for h in [0.1, 0.28, 0.5, 1.0] {
            let a = frenada_solo_atras(MU, G_ESTANDAR, B, L, h);
            assert!(a < antes && a < MU * G_ESTANDAR, "h = {h}: {a}");
            antes = a;
        }
    }

    /// F4: los números del kart de la casa (modelados): 0.526 g, el 44 % de μ g.
    /// La masa no entra.
    #[test]
    fn f4_el_kart_de_la_casa() {
        let a = frenada_solo_atras(MU, G_ESTANDAR, B, L, H);
        assert!((a / G_ESTANDAR - 0.526).abs() < 5e-4, "{}", a / G_ESTANDAR);
        assert_eq!((100.0 * a / (MU * G_ESTANDAR)).round(), 44.0);
        // a ese tope, las de atrás llevan el 44 % del peso (de 58 % sin frenar):
        // la misma fracción, porque a / (μ g) = F_z,atrás / (m g) en el tope
        let (_, t) = cargas_al_frenar(M, G_ESTANDAR, a, B, L, H);
        assert!(
            (t / (M * G_ESTANDAR) - 0.438).abs() < 5e-4,
            "{}",
            t / (M * G_ESTANDAR)
        );
    }
}
