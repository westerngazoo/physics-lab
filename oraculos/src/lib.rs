//! Oráculos: respuestas que **no salen del motor**.
//!
//! Los tests de cada crate comprueban que sus piezas sean coherentes entre
//! ellas. Eso es necesario y no basta: si el error está en la base, todo lo
//! que se apoya en ella lo confirma con entusiasmo. Un motor no puede ser
//! su propio testigo.
//!
//! Este crate existe por esa razón, y tiene **una sola regla**:
//!
//! > El valor esperado es un **literal**, y al lado está la cuenta o la
//! > cita que lo produjo. **Ninguna función del motor puede aparecer del
//! > lado esperado de una comparación.**
//!
//! Por eso los casos son deliberadamente simples: un triángulo 3-4-5, una
//! palanca de un metro, un ángulo cuyo seno es exactamente un medio. Si un
//! caso necesita el motor para saber su respuesta, no sirve como oráculo —
//! por más realista que sea.
//!
//! De dónde sale cada respuesta:
//!
//! - **Aritmética a mano**, escrita en el comentario para que se pueda
//!   rehacer sin computadora.
//! - **Geometría clásica** (el 3-4-5 es el triángulo rectángulo de
//!   enteros más viejo que hay).
//! - **Tablas publicadas** (Winter, *Biomechanics and Motor Control of
//!   Human Movement*), citadas.
//! - **Leyes de conservación**, que no dependen del camino de integración.

#![forbid(unsafe_code)]

/// Gravedad estándar, la misma que usa el motor. Es un dato del mundo, no
/// un resultado suyo, así que compartirlo no rompe la regla.
pub const G: f64 = 9.81;

#[cfg(test)]
mod tests {
    use super::G;
    use garust::tree::{Tree, TreeLink};
    use garust::twolink::two_link_joint;
    use garust::{chain::ChainJoint, Motor};
    use garust_core::Pga3;
    use mecanica::maquina_humana::{largo, masa, Persona};
    use mecanica::{gluteo, peak, work, Lift};

    fn eje_z() -> Pga3 {
        Pga3::point(0.0, 0.0, 0.0).line_through(&Pga3::point(0.0, 0.0, 1.0))
    }

    /// **Palanca de manual.** Una fuerza de 100 N hacia abajo, aplicada a
    /// 3 m del pivote: el momento es 3 × 100 = 300 N·m. No hay más.
    ///
    /// Si esto falla, nada de lo demás importa.
    #[test]
    fn la_palanca_mas_simple_que_existe() {
        use garust::physics::load::{Load, Weight};
        use garust::physics::multibody::joint_torque;

        let links = [TreeLink {
            parent: None,
            offset: Motor::identity(),
            joint: ChainJoint::Revolute(eje_z()),
        }];
        let tree = Tree::new(&links);
        let mut poses = [Motor::identity(); 1];
        tree.fk(&[0.0], &mut poses);
        // 100 N son 100/9.81 kg bajo esta gravedad; se pone la masa para
        // que la FUERZA sea exactamente 100 N.
        let w = Weight {
            link: 0,
            offset: [3.0, 0.0, 0.0],
            mass: 100.0 / G,
            gravity: [0.0, -G, 0.0],
        };
        let loads: [&dyn Load; 1] = [&w];
        let tau = joint_torque(&tree, &poses, &loads, 0);
        assert!((tau[2] + 300.0).abs() < 1e-9, "3 m × 100 N = 300, dio {tau:?}");
    }

    /// **El triángulo 3-4-5.** Extremos separados 5, eslabones de 3 y 4:
    /// el codo cae en (1.8, 2.4).
    ///
    /// A mano: `a = (5² + 3² − 4²) / (2·5) = (25+9−16)/10 = 1.8`, y
    /// `h = √(3² − 1.8²) = √(9 − 3.24) = √5.76 = 2.4`. Números exactos en
    /// binario, así que la tolerancia puede ser brutal.
    #[test]
    fn el_codo_del_triangulo_3_4_5() {
        let j = two_link_joint([0.0; 3], [5.0, 0.0, 0.0], 3.0, 4.0,
                               [0.0, 1.0, 0.0])
            .expect("resuelve");
        assert!((j[0] - 1.8).abs() < 1e-12, "x = {}", j[0]);
        assert!((j[1] - 2.4).abs() < 1e-12, "y = {}", j[1]);
        assert!(j[2].abs() < 1e-12, "sale del plano: {}", j[2]);
    }

    /// **Cinemática directa de un brazo de dos eslabones en L.**
    ///
    /// Dos eslabones de 1 m: el primero gira 0, el segundo 90°. La punta
    /// queda a un metro del segundo eje, perpendicular al primer eslabón.
    ///
    /// **Este oráculo encontró algo.** La punta cae en (1, **−1**), no en
    /// (1, +1): sobre el eje +z, un ángulo positivo lleva +x hacia −y, al
    /// revés de la regla de la mano derecha que uno supone. La convención
    /// estaba en la fórmula (`exp(−½θL̂)`) pero su consecuencia no estaba
    /// escrita en ningún lado, y ninguno de los cientos de tests del motor
    /// la exponía: todos comparaban magnitudes, o comparaban el motor
    /// contra sí mismo. Exactamente para esto existe este crate.
    ///
    /// Se fija el comportamiento REAL, no el que yo esperaba.
    #[test]
    fn el_brazo_en_ele() {
        let links = [
            TreeLink { parent: None, offset: Motor::identity(),
                       joint: ChainJoint::Revolute(eje_z()) },
            TreeLink { parent: Some(0), offset: Motor::translator(1.0, 0.0, 0.0),
                       joint: ChainJoint::Revolute(eje_z()) },
        ];
        let tree = Tree::new(&links);
        let mut poses = [Motor::identity(); 2];
        tree.fk(&[0.0, core::f64::consts::FRAC_PI_2], &mut poses);
        // la punta está a 1 m del segundo eje, en el eje x de SU marco
        let punta = poses[1].apply(&Pga3::point(1.0, 0.0, 0.0));
        let w = punta.coeffs[7];
        let (x, y, z) = (-punta.coeffs[14] / w, punta.coeffs[13] / w,
                         -punta.coeffs[11] / w);
        assert!((x - 1.0).abs() < 1e-12, "x = {x}");
        assert!((y + 1.0).abs() < 1e-12, "y = {y}, no +1: ver la nota");
        assert!(z.abs() < 1e-12, "z = {z}");
    }

    /// **Un ángulo cuyo seno es exactamente ½.**
    ///
    /// `τ = m·g·L·sen φ` con 100 kg, 0.53 m y φ = 30°:
    /// `100 × 9.81 × 0.53 × 0.5 = 259.965 N·m`. Se hace con lápiz.
    #[test]
    fn el_rumano_a_treinta_grados() {
        let r = gluteo::Rumano {
            load_kg: 100.0,
            torso_m: 0.53,
            bottom_rad: 72.0_f64.to_radians(),
        };
        let t = r.tau(30.0_f64.to_radians());
        assert!((t - 259.965).abs() < 1e-6, "dio {t}");
    }

    /// **El hip thrust en el bloqueo**, donde el coseno vale 1:
    /// `100 × 9.81 × 0.46 = 451.26 N·m`, sin funciones trigonométricas de
    /// por medio.
    #[test]
    fn el_hip_thrust_en_el_bloqueo() {
        let h = gluteo::HipThrust {
            load_kg: 100.0,
            femur_m: 0.46,
            bottom_rad: 40.0_f64.to_radians(),
        };
        let t = h.tau(0.0);
        assert!((t - 451.26).abs() < 1e-9, "dio {t}");
        // y es el máximo del recorrido, porque el coseno no pasa de 1
        assert!((peak(&h).1 - 451.26).abs() < 1e-9);
    }

    /// **Una integral que se resuelve de memoria.**
    ///
    /// `∫₀^π sen φ dφ = 2`. Con `τ = 10·sen φ`, el trabajo es 20 J.
    /// Es el oráculo del integrador: no depende de ningún modelo.
    #[test]
    fn la_integral_del_seno_vale_dos() {
        struct Seno;
        impl Lift for Seno {
            fn tau(&self, phi: f64) -> f64 {
                10.0 * phi.sin()
            }
            fn range(&self) -> (f64, f64) {
                (0.0, core::f64::consts::PI)
            }
        }
        let w = work(&Seno, 0.0, core::f64::consts::PI);
        // El método es punto medio con 900 muestras, así que el error de
        // discretización es del orden de 1e-5 y NO se puede pedir menos.
        // La tolerancia dice cuánta exactitud tiene el integrador, que es
        // información, no una concesión: si un día empeora, esto falla.
        assert!((w - 20.0).abs() < 1e-4, "dio {w}, debía dar 20");
        assert!((w - 20.0).abs() > 1e-9,
                "si ahora da exacto, alguien cambió el integrador y este \
                 oráculo tiene que decir cuánto mejoró");
    }

    /// **Las fracciones de Winter, contra la tabla publicada.**
    ///
    /// Winter, *Biomechanics and Motor Control of Human Movement*, tabla
    /// 4.1: el tronco entre L5/S1 y el hombro es 0.288 de la estatura, y
    /// el muslo 0.245. Las masas: tronco 0.355 del peso corporal, cabeza
    /// y cuello 0.081, un brazo entero 0.050.
    ///
    /// Esto no comprueba aritmética: comprueba que no le hayamos cambiado
    /// un número a la fuente sin darnos cuenta.
    #[test]
    fn las_fracciones_son_las_de_winter() {
        assert_eq!(largo::TORSO, 0.288);
        assert_eq!(largo::FEMUR, 0.245);
        assert_eq!(largo::TIBIA, 0.246);
        assert_eq!(masa::TRONCO, 0.355);
        assert_eq!(masa::CABEZA, 0.081);
        assert_eq!(masa::BRAZO, 0.050);
        // y una multiplicación que se hace en la cabeza
        let p = Persona { estatura_m: 2.0, masa_kg: 100.0 };
        assert!((p.torso() - 0.576).abs() < 1e-12, "0.288 × 2 = 0.576");
        assert!((p.masa_de(masa::TRONCO) - 35.5).abs() < 1e-12);
    }

    /// **Conservación de energía en una caída**, que no depende de cómo se
    /// integre: `v = √(2gh)`. Desde 1 m, `v = √(2 × 9.81) = 4.4294...` m/s.
    ///
    /// Se comprueba contra el literal, no contra lo que el integrador diga
    /// de sí mismo.
    #[test]
    fn una_caida_de_un_metro() {
        use garust::physics::world::Body;
        use garust::physics::World;

        let mundo = World::default();
        // una bola cualquiera: en caída libre la masa y el radio no entran
        // en la respuesta, y que NO entren es parte de lo que se comprueba
        let mut bodies = [Body::ball(2.5, 0.1)];
        bodies[0].rigid.position = [0.0, 1.0, 0.0];
        // hasta tocar y=0
        let dt = 1e-5;
        let mut pasos = 0;
        while bodies[0].rigid.position[1] > 0.0 && pasos < 2_000_000 {
            mundo.step(&mut bodies, &[], dt);
            pasos += 1;
        }
        let v = -bodies[0].rigid.linear_momentum[1] / bodies[0].rigid.mass;
        let esperado = (2.0_f64 * 9.81).sqrt();      // 4.42944...
        assert!(
            (v / esperado - 1.0).abs() < 1e-3,
            "cayó a {v:.5} m/s, la conservación dice {esperado:.5}"
        );
    }
}
