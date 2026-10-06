//! ¿Qué tan rápido entra una curva? — el círculo de fricción.
//!
//! Un kart entra a una curva de radio `r` a rapidez `v`. Para seguirla
//! necesita una aceleración hacia el centro de v²/r, y la llanta da como
//! mucho μ g: el círculo de fricción. La masa se cancela (μ·m·g / m), así que
//! la página no la pide.
//!
//! Si v²/r ≤ μ g, el kart sigue la curva. Si no, no se sale en línea recta:
//! la llanta sigue dando todo lo que tiene, μ g, y el kart gira en un círculo
//! más abierto, de radio v²/(μ g), tangente a donde entró. La rapidez tope
//! es donde las dos cosas se igualan, v = √(μ g r).
//!
//! Todo sale de `vehiculo`, el crate de dominio, con sus oráculos y su
//! comprobación en Python. Aquí no se integra nada: la trayectoria es un
//! arco de círculo, en forma cerrada.

use std::f64::consts::{FRAC_PI_2, TAU};

use lessons_common::{Prims, Readouts};
use vehiculo::{agarre, centro_de_giro, radio_minimo, v_tope, G_ESTANDAR as G};

/// Lo que la página necesita de un caso: la física, sin dibujo.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Caso {
    /// La rapidez tope de la curva, m/s.
    pub tope: f64,
    /// La aceleración que pide la curva, m/s².
    pub pide: f64,
    /// La que da la llanta, m/s².
    pub da: f64,
    /// El radio que de verdad sigue el kart, m.
    pub radio: f64,
}

/// La física de un caso: agarre μ, radio r (m) y rapidez v (m/s).
pub fn caso(mu: f64, r: f64, v: f64) -> Caso {
    let pide = v * v / r;
    let da = agarre([pide, 0.0], mu, G)[0];
    let radio = if pide <= mu * G { r } else { radio_minimo(v, mu, G) };
    Caso {
        tope: v_tope(mu, G, r),
        pide,
        da,
        radio,
    }
}

// ---- the wasm boundary --------------------------------------------------

// estilos (índices en lesson.json)
const EJES: usize = 0;
const CURVA: usize = 1;
const CAMINO: usize = 2;
const NO_ALCANZA: usize = 3;
const TEXTO: usize = 4;
const CIRCULO: usize = 5;
const ALCANZA: usize = 6;
// etiquetas (índices en lesson.json → labels)
const L_CENTRO: usize = 0;
const L_MU: usize = 1;
const L_PIDE: usize = 2;
const L_DA: usize = 3;

/// Hasta dónde se dibuja una flecha en el círculo de fricción, en g: más
/// allá se sale de la vista. La lectura dice el valor completo.
const FLECHA_MAX: f64 = 1.72;

/// Parámetros en orden de manifiesto: μ, r (m), v (m/s; la página la
/// muestra en km/h y la escala).
fn draw(p: &[f64], out: &mut Prims, read: &mut Readouts) {
    let (mu, r, v) = (p[0], p[1], p[2]);
    let c = caso(mu, r, v);

    // ---- vista 0: la curva desde arriba, en radios de la curva ----
    // Unidades naturales: toda longitud entre r. La curva siempre mide 1 y
    // el círculo que de verdad sigue el kart, R/r; así se ve la razón, que
    // es lo que importa, y no se pierde una curva chica en una vista grande.
    // El centro de la curva en el origen; el kart entra en (1, 0) hacia +y.
    out.view(0);
    out.segment(1.0, -0.3, 1.0, 0.0, EJES); // la recta de entrada
    out.curve(0.0, FRAC_PI_2, 64, CURVA, |a| (a.cos(), a.sin()));
    // por donde va: el mismo recorrido, π/2, sobre el círculo que puede seguir
    let rr = c.radio / r;
    let cg = centro_de_giro([1.0, 0.0], [0.0, v.max(1e-9)], rr, [0.0, 0.0]);
    out.curve(0.0, FRAC_PI_2 / rr, 96, CAMINO, |a| {
        (cg[0] + rr * a.cos(), cg[1] + rr * a.sin())
    });
    out.point(0.0, 0.0, EJES);
    out.label(0.06, -0.12, L_CENTRO, TEXTO);
    out.point(1.0, 0.0, CAMINO);

    // ---- vista 1: el círculo de fricción (en g) ----
    out.view(1);
    out.segment(-1.8, 0.0, 1.8, 0.0, EJES);
    out.segment(0.0, -1.8, 0.0, 1.8, EJES);
    out.curve(0.0, TAU, 96, CIRCULO, |a| (mu * a.cos(), mu * a.sin()));
    out.label(0.12, mu + 0.12, L_MU, TEXTO);
    // hacia el centro de la curva, que en la vista 0 queda a la izquierda
    let (pide_g, da_g) = (c.pide / G, c.da / G);
    let estilo = if c.pide > mu * G { NO_ALCANZA } else { ALCANZA };
    out.arrow(0.0, 0.1, -pide_g.min(FLECHA_MAX), 0.1, estilo);
    out.arrow(0.0, -0.1, -da_g, -0.1, CAMINO);
    // los nombres, centrados sobre su flecha (el texto se centra en x)
    out.label(-pide_g.min(FLECHA_MAX) / 2.0, 0.3, L_PIDE, TEXTO);
    out.label(-da_g / 2.0, -0.4, L_DA, TEXTO);

    read.set(0, c.tope * 3.6);
    read.set(1, pide_g);
    read.set(2, da_g);
    read.set(3, 100.0 * c.pide / (mu * G));
    read.set(4, c.radio);
}

lessons_common::lesson!(draw);

// ---- the claims ---------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use lessons_common::{recorre, PRIM_CAP, READ_SLOTS};

    /// Lo que la página mostraría para (μ, r, v en km/h), con su buffer.
    fn pagina(mu: f64, r: f64, kmh: f64) -> ([f64; READ_SLOTS], Vec<f64>, usize) {
        let mut buf = vec![0.0; PRIM_CAP];
        let mut rd = [0.0; READ_SLOTS];
        let n = {
            let mut prims = Prims::new(&mut buf);
            draw(&[mu, r, kmh / 3.6], &mut prims, &mut Readouts::new(&mut rd));
            prims.len()
        };
        (rd, buf, n)
    }

    const MUS: [f64; 4] = [0.4, 0.8, 1.2, 1.6];
    const RADIOS: [f64; 4] = [5.0, 11.0, 20.0, 60.0];

    /// K1: la rapidez tope es √(μ g r): a esa rapidez se usa el 100 % del
    /// agarre, ni más ni menos.
    #[test]
    fn k1_al_tope_se_usa_todo_el_agarre() {
        for mu in MUS {
            for r in RADIOS {
                let (rd, _, _) = pagina(mu, r, 30.0);
                let tope_kmh = rd[0];
                assert!((tope_kmh / 3.6 - (mu * G * r).sqrt()).abs() < 1e-9);
                let (al_tope, _, _) = pagina(mu, r, tope_kmh);
                assert!((al_tope[3] - 100.0).abs() < 1e-9, "μ={mu} r={r}: {} %", al_tope[3]);
            }
        }
    }

    /// K2: el doble de radio no da el doble de rapidez: da √2.
    #[test]
    fn k2_doble_radio_da_raiz_de_dos() {
        for mu in MUS {
            let (a, _, _) = pagina(mu, 20.0, 50.0);
            let (b, _, _) = pagina(mu, 40.0, 50.0);
            assert!((b[0] / a[0] - 2f64.sqrt()).abs() < 1e-12, "μ={mu}");
        }
    }

    /// K3: debajo del tope, el kart sigue la curva: su radio es el de ella.
    #[test]
    fn k3_debajo_del_tope_sigue_la_curva() {
        for mu in MUS {
            for r in RADIOS {
                let (rd, _, _) = pagina(mu, r, 1.0);
                let kmh = 0.95 * rd[0];
                let (rd, _, _) = pagina(mu, r, kmh);
                assert_eq!(rd[4], r, "μ={mu} r={r}");
                assert!(rd[2] == rd[1], "la llanta da lo que pide");
            }
        }
    }

    /// K4: pasado el tope no se sale en línea recta: sigue un círculo de
    /// radio v²/(μ g), más abierto, y lo que dibuja la página está sobre él.
    #[test]
    fn k4_pasado_el_tope_se_abre_a_v2_entre_mu_g() {
        let (mu, r, kmh) = (1.2, 20.0, 60.0);
        let v: f64 = kmh / 3.6;
        let (rd, buf, n) = pagina(mu, r, kmh);
        let esperado = v * v / (mu * G);
        assert!((rd[4] - esperado).abs() < 1e-9 && rd[4] > r, "{}", rd[4]);
        assert!((rd[2] - mu).abs() < 1e-12, "la llanta da μ g, todo lo que tiene");
        // el camino dorado (el polyline de estilo CAMINO, en radios de la
        // curva) cae sobre ese círculo: radio R/r, centro en (1 − R/r, 0)
        let camino = recorre(&buf, n)
            .into_iter()
            .find(|(t, rec)| *t == 2 && rec[rec.len() - 1] as usize == CAMINO)
            .expect("el camino");
        let pts = &camino.1[2..camino.1.len() - 1];
        let rr = esperado / r;
        let c = (1.0 - rr, 0.0);
        for xy in pts.chunks(2) {
            let d = ((xy[0] - c.0).powi(2) + (xy[1] - c.1).powi(2)).sqrt();
            assert!((d - rr).abs() < 1e-12, "punto a {d} del centro de giro, en radios");
        }
    }

    /// K5: lo que pide la curva crece con v²: al doble de rapidez, cuatro
    /// veces la fracción del agarre.
    #[test]
    fn k5_al_doble_de_rapidez_pide_cuatro_veces() {
        for r in RADIOS {
            let (a, _, _) = pagina(1.2, r, 20.0);
            let (b, _, _) = pagina(1.2, r, 40.0);
            assert!((b[3] / a[3] - 4.0).abs() < 1e-9, "r={r}");
        }
    }

    /// K6: lo que da la llanta nunca pasa de μ g, pida lo que pida.
    #[test]
    fn k6_la_llanta_nunca_da_mas_que_mu_g() {
        for mu in MUS {
            for r in RADIOS {
                for kmh in [10.0, 40.0, 70.0, 100.0] {
                    let (rd, _, _) = pagina(mu, r, kmh);
                    assert!(rd[2] <= mu + 1e-12, "μ={mu} r={r} v={kmh}");
                }
            }
        }
    }

    /// El buffer cabe y la página siempre tiene sus dos vistas.
    #[test]
    fn el_dibujo_cabe_en_el_buffer() {
        let (_, buf, n) = pagina(0.3, 5.0, 100.0);
        assert!(n < PRIM_CAP);
        assert_eq!(recorre(&buf, n).iter().filter(|(t, _)| *t == 9).count(), 2);
    }
}
