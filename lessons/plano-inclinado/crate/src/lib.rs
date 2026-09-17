//! El plano inclinado — el diagrama de cuerpo libre canónico.
//!
//! Existe por dos razones. La primera es que es la lección que todo mundo
//! ve primero y casi siempre mal dibujada: con la normal y el peso del
//! mismo largo, aunque uno valga el doble que el otro.
//!
//! La segunda es que es el ejemplo trabajado de [`lessons_common::dcl`].
//! Esta lección no dibuja una sola flecha a mano: entrega las tres
//! fuerzas ya calculadas y el diagrama se encarga de que su largo
//! signifique su magnitud, de que su color signifique su tensión, y de
//! ponerles nombre. Si alguna sale mal, sale mal para todas y se ve.
//!
//! LA FÍSICA, que es de una línea y aun así tiene una trampa. El peso se
//! parte en dos: `m·g·sen θ` cuesta abajo y `m·g·cos θ` contra el plano.
//! La normal iguala la segunda. La fricción iguala la primera **mientras
//! pueda**: su tope es `μ·N`, y ahí está la trampa. Pasado el ángulo en
//! que `tan θ = μ`, la fricción ya no alcanza y el bloque arranca. Ese
//! ángulo no depende de la masa, y ver que el bloque se suelta en el
//! mismo punto con 1 kg y con 100 es la mitad de la lección.

use lessons_common::dcl::{self, Estilo, Fuerza};
use lessons_common::{Prims, Readouts};

const G: f64 = 9.81;

/// Etiquetas, por índice en `lesson.json`.
const PESO: usize = 0;
const NORMAL: usize = 1;
const FRICCION: usize = 2;
const NETA: usize = 3;

/// Cómo se ve el diagrama. Cuatro bandas de frío a caliente.
const DIAGRAMA: Estilo = Estilo {
    largo_max: 0.34,
    calor: &[1, 2, 3, 4],
    palanca: 0,
    margen: 0.05,
};

/// Medio lado del bloque, en unidades del mundo.
const BLOQUE: f64 = 0.07;

/// Las tres fuerzas sobre el bloque, y si se desliza.
///
/// Devuelve `(peso, normal, fricción, desliza)`.
#[must_use]
pub fn fuerzas(theta: f64, masa: f64, mu: f64) -> (f64, f64, f64, bool) {
    let w = masa * G;
    let normal = w * theta.cos();
    let cuesta_abajo = w * theta.sin();
    let tope = mu * normal;
    // La fricción estática iguala lo que la empuja, PERO no puede pasar
    // de su tope. Olvidar el tope es el error clásico: da un bloque que
    // nunca se mueve por empinado que esté el plano.
    let desliza = cuesta_abajo > tope;
    let friccion = if desliza { tope } else { cuesta_abajo };
    (w, normal, friccion, desliza)
}

/// La aceleración cuesta abajo, m/s². Cero si no se desliza.
#[must_use]
pub fn aceleracion(theta: f64, masa: f64, mu: f64) -> f64 {
    let (_, _, fr, desliza) = fuerzas(theta, masa, mu);
    if desliza {
        (masa * G * theta.sin() - fr) / masa
    } else {
        0.0
    }
}

/// El ángulo en que el bloque se suelta. **No depende de la masa.**
#[must_use]
pub fn angulo_critico(mu: f64) -> f64 {
    mu.atan()
}

fn draw(p: &[f64], out: &mut Prims, read: &mut Readouts) {
    let (theta, masa, mu) = (p[0], p[1], p[2]);
    let (w, n, fr, desliza) = fuerzas(theta, masa, mu);

    let (c, s) = (theta.cos(), theta.sin());
    let cuesta = (c, s); // pendiente arriba
    let normal = (-s, c); // saliendo del plano

    out.view(0);
    // el plano y el suelo
    out.segment(-0.95 * c, -0.95 * s, 0.55 * c, 0.55 * s, 0);
    out.segment(-0.95 * c, -0.95 * s, 0.55 * c, -0.95 * s, 0);
    out.segment(0.55 * c, -0.95 * s, 0.55 * c, 0.55 * s, 0);

    // el bloque, apoyado en el plano
    let centro = (normal.0 * BLOQUE, normal.1 * BLOQUE);
    let esquina = |a: f64, b: f64| {
        (
            centro.0 + cuesta.0 * a + normal.0 * b,
            centro.1 + cuesta.1 * a + normal.1 * b,
        )
    };
    out.polyline(
        [
            esquina(-BLOQUE, -BLOQUE),
            esquina(BLOQUE, -BLOQUE),
            esquina(BLOQUE, BLOQUE),
            esquina(-BLOQUE, BLOQUE),
            esquina(-BLOQUE, -BLOQUE),
        ],
        0,
    );

    // EL DIAGRAMA. Ni una flecha a mano: se entregan las fuerzas y el
    // módulo se encarga del largo, del color y del nombre.
    let fs = [
        Fuerza::nueva(centro, (0.0, -w), PESO),
        Fuerza::nueva(centro, (normal.0 * n, normal.1 * n), NORMAL),
        // la fricción se opone a la bajada, así que apunta cuesta arriba
        Fuerza::nueva(centro, (cuesta.0 * fr, cuesta.1 * fr), FRICCION),
    ];
    dcl::fuerzas(out, &fs, &DIAGRAMA);

    // Y la resultante, sólo si existe. En reposo no se dibuja, que es
    // exactamente lo que hay que ver: las tres se cancelan.
    let r = dcl::resultante(&fs, centro, Some(NETA));
    if r.magnitud() > 1e-9 {
        dcl::fuerzas(out, &[r], &DIAGRAMA);
    }

    read.set(0, n);
    read.set(1, fr);
    read.set(2, r.magnitud());
    read.set(3, aceleracion(theta, masa, mu));
    read.set(4, angulo_critico(mu).to_degrees());
    read.set(5, if desliza { 1.0 } else { 0.0 });
}

lessons_common::lesson!(draw);

#[cfg(test)]
mod tests {
    use super::*;

    /// P1 · En reposo las tres fuerzas se cancelan. Si la resultante no
    /// es cero, el bloque quieto del dibujo es mentira.
    #[test]
    fn p1_en_reposo_las_tres_se_cancelan() {
        let theta = 10.0_f64.to_radians();
        let (w, n, fr, desliza) = fuerzas(theta, 5.0, 0.6);
        assert!(!desliza);
        let (c, s) = (theta.cos(), theta.sin());
        let rx = 0.0 + (-s) * n + c * fr;
        let ry = -w + c * n + s * fr;
        assert!(rx.hypot(ry) < 1e-9, "resultante {rx}, {ry}");
    }

    /// P2 · El ángulo en que se suelta NO depende de la masa. Es la
    /// mitad de la lección y se puede comprobar de un tirón.
    #[test]
    fn p2_el_angulo_critico_no_depende_de_la_masa() {
        let mu = 0.4;
        let critico = angulo_critico(mu);
        for masa in [0.5, 5.0, 50.0, 500.0] {
            let justo_antes = fuerzas(critico - 1e-6, masa, mu).3;
            let justo_despues = fuerzas(critico + 1e-6, masa, mu).3;
            assert!(!justo_antes && justo_despues, "con {masa} kg falla");
        }
    }

    /// P3 · Sin fricción, la aceleración es `g·sen θ` exacto. Aritmética
    /// de mano, no un valor guardado.
    #[test]
    fn p3_sin_friccion_es_g_por_seno() {
        let theta = 30.0_f64.to_radians();
        let a = aceleracion(theta, 3.0, 0.0);
        assert!((a - G * 0.5).abs() < 1e-9, "dio {a}");
    }

    /// P4 · La normal muere en la vertical: un plano a 90° no aprieta
    /// nada contra él. Es el límite que la geometría obliga.
    #[test]
    fn p4_la_normal_muere_en_la_vertical() {
        let n = fuerzas(90.0_f64.to_radians(), 10.0, 0.5).1;
        assert!(n.abs() < 1e-9, "dio {n}");
    }

    /// P5 · El diagrama dibuja tres flechas en reposo y cuatro cuando se
    /// desliza: la cuarta es la resultante, y aparecer es su trabajo.
    #[test]
    fn p5_la_resultante_aparece_solo_al_deslizar() {
        fn flechas(theta_grados: f64, mu: f64) -> usize {
            let mut buf = [0.0; lessons_common::PRIM_CAP];
            let mut prims = Prims::new(&mut buf);
            let mut read_buf = [0.0; lessons_common::READ_SLOTS];
            let mut read = Readouts::new(&mut read_buf);
            draw(&[theta_grados.to_radians(), 5.0, mu], &mut prims, &mut read);
            let n = prims.len();
            lessons_common::cuenta(&buf, n, 3)
        }
        assert_eq!(flechas(10.0, 0.6), 3, "en reposo, tres");
        assert_eq!(flechas(45.0, 0.2), 4, "deslizando, cuatro");
    }
}
