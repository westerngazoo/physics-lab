//! La carga en las llantas: cuánto se pasa al lado de afuera en una curva, y
//! a qué eje.
//!
//! En una curva el piso empuja las llantas hacia el centro, a ras del piso,
//! y el centro de masa va a una altura `h`. Esas dos cosas no están en la
//! misma línea, así que hacen girar al vehículo hacia afuera, y el piso lo
//! equilibra cargando más las llantas de afuera que las de adentro. Con una
//! vía `t` (la distancia entre llantas de un eje), el par que hay que
//! equilibrar es m a_y h y la carga que se pasa es
//!
//! ```text
//! ΔF_z = m a_y h / t.
//! ```
//!
//! **A qué eje se pasa** lo decide la rigidez a rodar de cada eje
//! ([`reparto_dos_nodos`]). En un kart, sin suspensión, las llantas y el
//! chasis son los resortes: adelante, las llantas en serie con el chasis que
//! se tuerce; atrás, las llantas sobre el eje sólido. El eje que se lleva más
//! transferencia descarga primero su llanta de adentro, y la levanta cuando
//! la transferencia iguala la carga que tenía ([`despegue_en_g`]).
//!
//! # El modelo, y lo que no es
//!
//! Cuasi-estático (una curva sostenida), piso plano, sin aerodinámica,
//! llantas como resortes lineales y el chasis como dos largueros en torsión
//! pura. No hay caster: el gato del caster, que en un kart es lo que más
//! levanta la trasera de adentro, es geometría de la dirección y va aparte.
//! Supone agarre suficiente: si μ < a_y/g, el vehículo desliza antes.

use std::f64::consts::TAU;

/// La carga vertical que pasa de las llantas de adentro a las de afuera
/// (N): m a_y h / t, con `m` en kg, `a_y` la aceleración lateral (m/s²), `h`
/// la altura del centro de masa (m) y `t` la vía (m). Es la de un eje que se
/// lleva todo el par; con dos ejes, cada uno se lleva su fracción.
pub fn transferencia_lateral(m: f64, a_y: f64, h: f64, t: f64) -> f64 {
    m * a_y * h / t
}

/// La rigidez a rodar de un par de llantas (N·m/rad): k t²/2. Cada llanta,
/// de rigidez vertical `k` (N/m), está a t/2 del centro: al rodar el eje un
/// ángulo θ, una baja y la otra sube t θ/2, y el par que devuelven es
/// 2 · k (t θ/2) · (t/2) = k t² θ/2.
pub fn rigidez_de_llantas(k: f64, t: f64) -> f64 {
    k * t * t / 2.0
}

/// La rigidez torsional de un chasis de dos largueros de tubo (N·m/rad):
/// 2 G J / L, con J = (τ/64)(D⁴ − d⁴) el momento polar del tubo de diámetro
/// exterior `d_ext` e interior `d_int` (m), `g_corte` el módulo de corte
/// (Pa) y `largo` la distancia entre ejes (m).
pub fn rigidez_de_chasis(g_corte: f64, d_ext: f64, d_int: f64, largo: f64) -> f64 {
    // J = ∫ r² dA sobre el anillo: lleva una vuelta completa, τ
    let j = TAU / 64.0 * (d_ext.powi(4) - d_int.powi(4));
    2.0 * g_corte * j / largo
}

/// Dos resortes en serie: 1/(1/a + 1/b). El más blando manda.
pub fn en_serie(a: f64, b: f64) -> f64 {
    a * b / (a + b)
}

/// Qué fracción del par de vuelco se lleva cada eje: `(adelante, atrás)`,
/// que suman 1.
///
/// El modelo de dos nodos: el nodo delantero apoya en las llantas de
/// adelante (`k_del`, de [`rigidez_de_llantas`]), el trasero en las de
/// atrás (`k_tras`), y el chasis (`k_chasis`) los une como un resorte de
/// torsión. Del par m a_y h, la fracción `alfa` entra por el nodo delantero y
/// el resto por el trasero (dónde va la masa). Cada nodo gira hasta
/// equilibrarse:
///
/// ```text
/// α T       = k_del θ_d + k_chasis (θ_d − θ_t)
/// (1 − α) T = k_tras θ_t + k_chasis (θ_t − θ_d)
/// ```
///
/// y cada eje se lleva k θ de su nodo. Resuelto, con
/// D = k_del k_tras + k_chasis (k_del + k_tras):
///
/// ```text
/// adelante = k_del (α k_tras + k_chasis) / D
/// atrás    = k_tras ((1 − α) k_del + k_chasis) / D
/// ```
///
/// Escrito así no resta números grandes: con un chasis muy rígido el
/// determinante de la forma matricial pierde todas sus cifras. Con el
/// chasis rígido el reparto es k_del : k_tras, entre donde entre el par;
/// con uno blando, cada eje se queda con lo que entra por su nodo. Con todo
/// el par atrás (α = 0), atrás = k_tras / (k_tras + k_del ‖ k_chasis): lo de
/// adelante pasa por las llantas y el chasis en serie ([`en_serie`]).
pub fn reparto_dos_nodos(k_del: f64, k_tras: f64, k_chasis: f64, alfa: f64) -> (f64, f64) {
    let d = k_del * k_tras + k_chasis * (k_del + k_tras);
    (
        k_del * (alfa * k_tras + k_chasis) / d,
        k_tras * ((1.0 - alfa) * k_del + k_chasis) / d,
    )
}

/// La aceleración lateral, en g, a la que un eje levanta su llanta de
/// adentro: cuando la transferencia que se lleva, f · m a_y h / t, iguala la
/// carga estática de esa llanta, r m g / 2. Despejando,
///
/// ```text
/// a_y / g = r t / (2 f h),
/// ```
///
/// con `reparto` (r) la fracción del peso sobre el eje, `t` su vía (m),
/// `fraccion` (f) la del par que se lleva ([`reparto_dos_nodos`]) y `h` la
/// altura del centro de masa (m). La masa se cancela.
pub fn despegue_en_g(reparto: f64, t: f64, fraccion: f64, h: f64) -> f64 {
    reparto * t / (2.0 * fraccion * h)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// El kart por defecto del reel 1.3 (valores supuestos, modelados):
    /// 145 kg, h = 0.28 m, 58 % atrás, batalla 1.04 m, vías 0.975 y 1.205 m,
    /// llantas de 100 kN/m, chasis de tubo 4130 de 30 × 2 mm (G = 80 GPa).
    fn kart() -> (f64, f64, f64) {
        let k_ch = rigidez_de_chasis(80e9, 0.030, 0.026, 1.04);
        (
            rigidez_de_llantas(100e3, 0.975),
            rigidez_de_llantas(100e3, 1.205),
            k_ch,
        )
    }

    /// T1: los repartos suman 1, entre donde entre el par.
    #[test]
    fn t1_los_dos_ejes_se_llevan_todo_el_par() {
        let (kd, kt, kc) = kart();
        for alfa in [0.0, 0.2, 0.42, 0.7, 1.0] {
            let (d, t) = reparto_dos_nodos(kd, kt, kc, alfa);
            assert!((d + t - 1.0).abs() < 1e-12, "α = {alfa}: {d} + {t}");
        }
    }

    /// T2: con un chasis rígido el reparto es el de las rigideces de las
    /// llantas, y ya no importa dónde entra el par.
    #[test]
    fn t2_con_chasis_rigido_manda_la_rigidez() {
        for alfa in [0.0, 0.5, 1.0] {
            let (d, t) = reparto_dos_nodos(1.0, 3.0, 1e15, alfa);
            assert!(
                (d - 0.25).abs() < 1e-9 && (t - 0.75).abs() < 1e-9,
                "α = {alfa}: {d}, {t}"
            );
        }
    }

    /// T3: con todo el par entrando atrás, el reparto trasero es
    /// k_tras/(k_tras + k_del): lo de adelante pasa por el chasis en serie.
    #[test]
    fn t3_todo_atras_es_la_formula_de_serie() {
        let (kd, kt, kc) = kart();
        let (_, t) = reparto_dos_nodos(kd, kt, kc, 0.0);
        assert!((t - kt / (kt + en_serie(kd, kc))).abs() < 1e-12);
    }

    /// T4: los números del kart del reel 1.3 (modelados): chasis 5 332 N·m/rad,
    /// llantas 47 531 adelante (4 794 en serie con el chasis) y 72 601 atrás;
    /// 94 % atrás y la trasera a 1.33 g con la masa atrás; 58 % atrás y la
    /// delantera a 1.76 g con la masa repartida por peso.
    #[test]
    fn t4_el_kart_del_reel() {
        let (kd, kt, kc) = kart();
        assert_eq!(kc.round(), 5332.0);
        assert_eq!(kd.round(), 47531.0);
        assert_eq!(en_serie(kd, kc).round(), 4794.0);
        assert_eq!(kt.round(), 72601.0);
        let (_, atras) = reparto_dos_nodos(kd, kt, kc, 0.0);
        assert!((atras - 0.938).abs() < 5e-4, "{atras}");
        let g_tras = despegue_en_g(0.58, 1.205, atras, 0.28);
        assert!((g_tras - 1.33).abs() < 5e-3, "{g_tras}");
        let (del, atras) = reparto_dos_nodos(kd, kt, kc, 0.42);
        assert!((atras - 0.584).abs() < 5e-4, "{atras}");
        let g_del = despegue_en_g(0.42, 0.975, del, 0.28);
        assert!((g_del - 1.76).abs() < 5e-3, "{g_del}");
        assert!(
            despegue_en_g(0.58, 1.205, atras, 0.28) > g_del,
            "la delantera primero"
        );
    }

    /// T5: la transferencia crece con h y con a_y, baja con la vía, y el
    /// umbral no depende de la masa.
    #[test]
    fn t5_lo_que_mueve_la_transferencia() {
        let base = transferencia_lateral(145.0, 9.8, 0.28, 1.2);
        assert!((transferencia_lateral(145.0, 9.8, 0.56, 1.2) / base - 2.0).abs() < 1e-12);
        assert!((transferencia_lateral(145.0, 9.8, 0.28, 2.4) / base - 0.5).abs() < 1e-12);
        // la llanta de adentro se queda sin carga justo en despegue_en_g
        let m = 145.0;
        let g = 9.806_65;
        let ay = despegue_en_g(0.5, 1.2, 1.0, 0.28) * g;
        let estatica = 0.5 * m * g / 2.0;
        assert!((transferencia_lateral(m, ay, 0.28, 1.2) - estatica).abs() < 1e-9);
    }
}
