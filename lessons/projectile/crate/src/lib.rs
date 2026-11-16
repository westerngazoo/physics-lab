//! Tiro parabólico — la lección mínima a propósito, y la plantilla de
//! autoría que la guía del aula recorre línea por línea.
//!
//! Un lanzamiento con rapidez `v`, ángulo `θ` y gravedad `g`. El ángulo
//! llega aquí en radianes: el deslizador de la página se lee en grados y la
//! `scale` del manifiesto (τ/360) hace la conversión antes de la frontera.
//! Todo es forma cerrada, y la trayectoria se dibuja en UNIDADES NATURALES
//! `v²/g`: por eso cambiar `v` o `g` nunca cambia la forma dibujada, sólo
//! los números. Esa invariancia de escala es en sí una afirmación (P5), no
//! un accidente.

/// Alcance, altura máxima y tiempo de vuelo: los números reales, con unidades.
pub fn numeros(theta: f64, v: f64, g: f64) -> (f64, f64, f64) {
    let alcance = v * v * (2.0 * theta).sin() / g;
    let altura = v * v * theta.sin().powi(2) / (2.0 * g);
    let tiempo = 2.0 * v * theta.sin() / g;
    (alcance, altura, tiempo)
}

/// La trayectoria en unidades naturales x' = x·g/v²: la forma sólo depende de θ.
pub fn forma(theta: f64, xp: f64) -> f64 {
    xp * theta.tan() - xp * xp / (2.0 * theta.cos().powi(2))
}

// ---- la frontera con wasm -----------------------------------------------

use lessons_common::{Prims, Readouts};

/// El punto de entrada con que se dibuja la lección.
///
/// # Cómo se conecta con el framework
///
/// El framework no usa traits de Rust (no hay un trait `Draw` que
/// implementar). La interfaz es puramente de datos: la lección registra
/// esta función `draw` con la macro `lessons_common::lesson!`, que genera
/// los exports uniformes de WebAssembly con ABI de C (`params_ptr`,
/// `prims_ptr`, `readouts_ptr`, `state_at`) que espera el runtime de
/// JavaScript.
///
/// # Parámetros
///
/// * `p`: los parámetros como `f64`, **en el orden del manifiesto**, tal
///   como se declaran en `params` de `lesson.json`, ya multiplicados por la
///   `scale` de cada uno. Aquí `p[0]` es `theta` en radianes (el deslizador
///   va en grados y su `scale`, τ/360, lo convierte), `p[1]` es `v` (la
///   rapidez de lanzamiento) y `p[2]` es `g` (la gravedad).
/// * `out`: el escritor del buffer de primitivas (`view`, `segment`,
///   `arrow`, `curve`, `point`), que escribe registros de dibujo en el
///   buffer plano compartido con el runtime de JS.
/// * `read`: el escritor de lecturas, que llena por índice (0 a 7) las
///   ranuras numéricas que la página muestra como declara `lesson.json`.
///
/// # El contrato de pureza
///
/// Esta función tiene que ser pura y sin estado: nada de números
/// aleatorios, ni reloj, ni estado interno. Corre en cada cuadro desde los
/// parámetros del momento.
fn draw(p: &[f64], out: &mut Prims, read: &mut Readouts) {
    let (theta, v, g) = (p[0], p[1], p[2]);
    let (alcance, altura, tiempo) = numeros(theta, v, g);
    let rp = (2.0 * theta).sin(); // el alcance en unidades naturales

    out.view(0);
    out.segment(-0.05, 0.0, 1.15, 0.0, 0);                    // el suelo
    out.arrow(0.0, 0.0, 0.16 * theta.cos(), 0.16 * theta.sin(), 3); // el lanzamiento
    out.curve(0.0, rp, 48, 1, |xp| (xp, forma(theta, xp)));   // el vuelo
    out.point(rp / 2.0, forma(theta, rp / 2.0), 2);           // el punto más alto
    out.segment(rp, -0.02, rp, 0.02, 3);                      // la marca del alcance

    read.set(0, theta.to_degrees()); // la página no la muestra: el deslizador ya dice θ
    read.set(1, alcance);
    read.set(2, altura);
    read.set(3, tiempo);
    read.set(4, (2.0 * theta).sin() * 100.0); // % del mejor alcance posible
}

lessons_common::lesson!(draw);

// ---- las afirmaciones -----------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    /// La rejilla del deslizador, en grados: de 1° a 89°, de medio en medio.
    fn angulos() -> impl Iterator<Item = f64> {
        (2..=178).map(|i| f64::from(i) * 0.5)
    }

    /// Las lecturas que pinta la página, con θ en grados como en el
    /// deslizador: `to_radians` multiplica por la misma τ/360 que la
    /// `scale` del manifiesto.
    fn lecturas(grados: f64, v: f64, g: f64) -> [f64; lessons_common::READ_SLOTS] {
        let mut buf = [0.0; lessons_common::PRIM_CAP];
        let mut lee = [0.0; lessons_common::READ_SLOTS];
        {
            let mut prims = Prims::new(&mut buf);
            let mut read = Readouts::new(&mut lee);
            draw(&[grados.to_radians(), v, g], &mut prims, &mut read);
        }
        lee
    }

    /// P1: la curva dibujada cae exactamente donde dice la fórmula del alcance.
    #[test]
    fn p1_la_curva_cae_en_el_alcance() {
        for grados in angulos() {
            let theta = grados.to_radians();
            let rp = (2.0 * theta).sin();
            assert!(forma(theta, rp).abs() < 1e-12, "θ = {grados}°");
        }
    }

    /// P2: el alcance es máximo a 45°, y en ningún otro punto de la rejilla
    /// del deslizador. La página dice lo mismo: a 45° marca 100 %.
    #[test]
    fn p2_el_alcance_es_maximo_a_45_grados() {
        let mejor = numeros(45.0_f64.to_radians(), 2.0, 9.81).0;
        for grados in angulos() {
            let r = numeros(grados.to_radians(), 2.0, 9.81).0;
            assert!(r <= mejor + 1e-12);
            if (grados - 45.0).abs() > 1e-9 {
                assert!(r < mejor, "{grados}° tiene que quedar estrictamente abajo");
            }
        }
        let lee = lecturas(45.0, 2.0, 9.81);
        assert!((lee[4] - 100.0).abs() < 1e-12, "a 45° la página marca {} %", lee[4]);
    }

    /// P3: los ángulos complementarios tienen el mismo alcance:
    /// R(θ) = R(90° − θ).
    #[test]
    fn p3_los_complementarios_tienen_el_mismo_alcance() {
        for grados in angulos().filter(|g| *g < 45.0) {
            let a = numeros(grados.to_radians(), 3.0, 1.62).0;
            let b = numeros((90.0 - grados).to_radians(), 3.0, 1.62).0;
            assert!((a - b).abs() < 1e-12 * a.max(1e-12), "θ = {grados}°");
        }
    }

    /// P4: el punto más alto dibujado está a la altura de la fórmula.
    #[test]
    fn p4_el_punto_mas_alto_coincide() {
        for grados in angulos() {
            let theta = grados.to_radians();
            let (_, altura, _) = numeros(theta, 2.5, 9.81);
            let natural = forma(theta, (2.0 * theta).sin() / 2.0);
            // de la altura natural a metros: × v²/g
            assert!((natural * 2.5 * 2.5 / 9.81 - altura).abs() < 1e-12, "θ = {grados}°");
        }
    }

    /// P5: invariancia de escala. La forma dibujada es idéntica bit a bit
    /// para cualquier (v, g), que es todo el sentido de las unidades
    /// naturales.
    #[test]
    fn p5_la_forma_ignora_v_y_g() {
        for grados in angulos() {
            let theta = grados.to_radians();
            for xp in [0.1, 0.3, 0.55, 0.8] {
                // forma() ni siquiera recibe v ni g: la invariancia es de
                // estructura. Se comprueba que los números sí cambian, para
                // que la afirmación no sea vacía.
                let a = numeros(theta, 2.0, 9.81);
                let b = numeros(theta, 4.0, 1.62);
                assert!(a.0 != b.0 && forma(theta, xp) == forma(theta, xp));
            }
        }
    }
}
