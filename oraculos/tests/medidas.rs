//! Oráculo de las medidas: un archivo cuyas respuestas salen a mano.
//!
//! La regla del crate vale aquí: del lado esperado solo hay literales, y al
//! lado la cuenta que los produjo. Las alturas se escogieron para que los
//! segmentos sean redondos y el tronco quede con seno exactamente ½:
//!
//! | referencia | altura (cm) | segmento |
//! |---|---|---|
//! | maléolo | 7 | |
//! | rodilla | 50 | tibia = 50 − 7 = 43 cm |
//! | trocánter | 95.5 | fémur = 95.5 − 50 = 45.5 cm |
//! | hombro | 151.5 | tronco = 151.5 − 95.5 = 56 cm |
//!
//! Tibia a 30° (sen 30° = ½). El modelo pone el tobillo en x = −0.04 m y la
//! barra en x = 0 (sobre el medio pie):
//!
//! - rodilla: x = −0.04 + 0.43 · ½ = 0.175 m;
//! - cadera: x = 0.175 − 0.455 = −0.28 m (muslo paralelo);
//! - tronco: sen θ = 0.28 / 0.56 = ½, θ = 30°;
//! - brazo de cadera 0.28 m: 100 kg · 9.81 · 0.28 = **274.68 N·m**;
//! - brazo de rodilla 0.175 m: 100 kg · 9.81 · 0.175 = **171.675 N·m**.

use mecanica::medidas::{informe, revisar, Campo};

const ARCHIVO: &str = r#"
formato = "medidas-sentadilla/1"
persona = "P-0RACULO1"
fecha = "2026-10-18"

[estatura]
fuente = "medido"
valor_cm = 180.0
fecha = "2026-10-18"
como = "oráculo"

[masa]
fuente = "medido"
valor_kg = 80.0
fecha = "2026-10-18"
como = "oráculo"

[altura_hombro]
fuente = "medido"
valor_cm = 151.5
fecha = "2026-10-18"
como = "oráculo"

[altura_trocanter]
fuente = "medido"
valor_cm = 95.5
fecha = "2026-10-18"
como = "oráculo"

[altura_rodilla]
fuente = "medido"
valor_cm = 50.0
fecha = "2026-10-18"
como = "oráculo"

[altura_maleolo]
fuente = "medido"
valor_cm = 7.0
fecha = "2026-10-18"
como = "oráculo"

[largo_pie]
fuente = "estimado"

[altura_barra]
fuente = "estimado"

[angulo_tibia]
fuente = "medido"
valor_grados = 30.0
fecha = "2026-10-18"
como = "oráculo"
"#;

fn cerca(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-9, "{a} ≠ {b}");
}

#[test]
fn las_medidas_redondas_dan_las_cuentas_a_mano() {
    let r = revisar(ARCHIVO).expect("el oráculo pasa la revisión");
    assert!(r.avisos.is_empty(), "{:?}", r.avisos);
    let i = informe::resolver(&r, Some(100.0)).unwrap();

    cerca(i.entradas.tibia_m, 0.43);
    cerca(i.entradas.femur_m, 0.455);
    cerca(i.entradas.torso_m, 0.56);
    cerca(i.postura.knee.0, 0.175);
    cerca(i.postura.hip.0, -0.28);
    cerca(i.postura.torso_rad.sin(), 0.5);
    cerca(i.tronco_grados(), 30.0);
    cerca(i.brazos_m[0], 0.28);
    cerca(i.brazos_m[1], 0.175);
    let [cadera, rodilla] = i.momentos_nm().unwrap();
    cerca(cadera, 274.68);
    cerca(rodilla, 171.675);
}

/// Un centímetro más de trocánter es un centímetro más de fémur, y el
/// fémur entero se suma al brazo de la cadera (la cadera se va 1 cm hacia
/// atrás) sin tocar la rodilla: +0.01 m de brazo, +9.81 N·m con 100 kg.
#[test]
fn un_centimetro_de_trocanter_es_un_centimetro_de_brazo() {
    let i = informe::resolver(&revisar(ARCHIVO).unwrap(), Some(100.0)).unwrap();
    let h = i
        .huellas
        .iter()
        .find(|h| h.campo == Campo::AlturaTrocanter)
        .unwrap();
    cerca(h.por_unidad[0], 0.01);
    cerca(h.por_unidad[1], 0.0);
}
