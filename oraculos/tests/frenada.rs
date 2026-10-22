//! Oráculo de la frenada con freno solo atrás: respuestas que salen a mano.
//!
//! La regla del crate vale aquí: del lado esperado solo hay literales, y al
//! lado la cuenta que los produjo. Un vehículo de 100 kg con g = 10 m/s²,
//! μ = 1, el centro de masa a b = 0.6 m del eje delantero, a h = 0.2 m del
//! piso, y una distancia entre ejes de 1 m:
//!
//! - sin frenar, atrás: 100 × 10 × 0.6 / 1 = **600 N**; adelante, **400 N**;
//! - frenando a 5 m/s², pasan 100 × 5 × 0.2 / 1 = 100 N adelante: atrás
//!   **500 N**, adelante **500 N**;
//! - el tope con freno solo atrás: 1 × 10 × 0.6 / (1 + 1 × 0.2) = 6 / 1.2 =
//!   **5 m/s²**, y ahí el agarre de atrás, 1 × 500 N, es justo m a = 500 N.
//!   Con las cuatro frenaría a μ g = 10: la mitad.

use vehiculo::frenada::{cargas_al_frenar, frenada_solo_atras};

#[test]
fn las_cargas_al_frenar_a_mano() {
    let (d, t) = cargas_al_frenar(100.0, 10.0, 0.0, 0.6, 1.0, 0.2);
    assert!(
        (d - 400.0).abs() < 1e-9 && (t - 600.0).abs() < 1e-9,
        "{d}, {t}"
    );
    let (d, t) = cargas_al_frenar(100.0, 10.0, 5.0, 0.6, 1.0, 0.2);
    assert!(
        (d - 500.0).abs() < 1e-9 && (t - 500.0).abs() < 1e-9,
        "{d}, {t}"
    );
}

#[test]
fn la_frenada_solo_atras_a_mano() {
    assert!((frenada_solo_atras(1.0, 10.0, 0.6, 1.0, 0.2) - 5.0).abs() < 1e-12);
}
