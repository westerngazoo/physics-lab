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
//! - **Tablas de funciones especiales** (Abramowitz y Stegun, *Handbook of
//!   Mathematical Functions*), citadas con su número de tabla.

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

    /// **Las integrales de Fresnel en 1**, de la tabla: C(1) = 0.7798934 y
    /// S(1) = 0.4382591 (Abramowitz y Stegun, tabla 7.7, a siete cifras).
    #[test]
    fn fresnel_en_uno_como_la_tabla() {
        let (c, s) = difraccion::fresnel(1.0);
        assert!((c - 0.779_893_4).abs() < 1e-7, "C(1) = {c}");
        assert!((s - 0.438_259_1).abs() < 1e-7, "S(1) = {s}");
    }

    /// **En el borde de la sombra llega un cuarto.** C(0) = S(0) = 0, así
    /// que I/I₀ = ½ (¼ + ¼) = ¼: la mitad de la amplitud, un cuarto de la
    /// intensidad. Sin tolerancia: es aritmética de potencias de dos.
    #[test]
    fn en_el_borde_llega_un_cuarto() {
        assert_eq!(difraccion::borde_recto(0.0), 0.25);
    }

    /// **Lejos del borde, toda la luz.** ∫₀^∞ cos(πt²/2) dt = ½ (y lo mismo
    /// con el seno): con C = S = ½, I/I₀ = ½ (1 + 1) = 1. A w = 1000 la franja
    /// que queda mide √2/(1000π) ≈ 4.5 × 10⁻⁴.
    #[test]
    fn lejos_del_borde_llega_toda_la_luz() {
        let i = difraccion::borde_recto(1000.0);
        assert!((i - 1.0).abs() < 5e-4, "I(1000) = {i}");
        assert!(difraccion::borde_recto(-1000.0) < 1e-6, "y del otro lado, la sombra");
    }

    /// **La primera franja brillante pasa de la luz sin borde:** 1.37 veces
    /// (Hecht, *Optics*, §10.3, la difracción de Fresnel en un borde recto).
    #[test]
    fn la_primera_franja_brilla_un_37_por_ciento_mas() {
        let (_, i) = difraccion::franja(0);
        assert!((i - 1.37).abs() < 0.005, "I = {i}");
    }

    /// **El círculo de fricción con un 3-4-5.** Se piden (30, 40) m/s² y el
    /// tope es μ g = 1 × 10 = 10: el largo pedido es 50, así que se escala por
    /// 10/50 = 1/5 y quedan (6, 8). Con la misma dirección, exacto.
    #[test]
    fn el_circulo_de_friccion_con_un_3_4_5() {
        let a = vehiculo::agarre([30.0, 40.0], 1.0, 10.0);
        assert!((a[0] - 6.0).abs() < 1e-12 && (a[1] - 8.0).abs() < 1e-12, "{a:?}");
    }

    /// **La rapidez tope y el radio mínimo, a mano.** μ = 1, g = 10, r = 10:
    /// v = √(1 × 10 × 10) = 10 m/s. Y a 20 m/s el giro más cerrado es
    /// 20²/(1 × 10) = 40 m.
    #[test]
    fn la_rapidez_tope_y_el_radio_minimo_a_mano() {
        assert!((vehiculo::v_tope(1.0, 10.0, 10.0) - 10.0).abs() < 1e-12);
        assert!((vehiculo::radio_minimo(20.0, 1.0, 10.0) - 40.0).abs() < 1e-12);
        // el que perdió el agarre en (20, 0) yendo hacia +y gira alrededor
        // de (15, 0) con radio 5: el centro queda del lado de la curva
        let c = vehiculo::centro_de_giro([20.0, 0.0], [0.0, 10.0], 5.0, [0.0, 0.0]);
        assert!((c[0] - 15.0).abs() < 1e-12 && c[1].abs() < 1e-12, "{c:?}");
    }

    /// **La trazada abierta en tres curvas de libro.** R = r + w/(1 − cos(θ/2)):
    ///
    /// - 120°: cos 60° = ½ (el triángulo equilátero), R = 10 + 5/½ = 20.
    /// - 180°, la horquilla: cos 90° = 0, R = 10 + 8 = 18.
    /// - 90°: 1 − 1/√2 da R = 10 + (2 + √2) × 8 = 10 + 27.31371 = 37.31371.
    #[test]
    fn la_trazada_en_tres_curvas_de_libro() {
        use std::f64::consts::PI;
        assert!((vehiculo::trazada(10.0, 5.0, 2.0 * PI / 3.0) - 20.0).abs() < 1e-12);
        assert!((vehiculo::trazada(10.0, 8.0, PI) - 18.0).abs() < 1e-12);
        assert!((vehiculo::trazada(10.0, 8.0, PI / 2.0) - 37.313_708_5).abs() < 1e-7);
    }

    /// **La transferencia de carga, a mano.** 100 kg a 10 m/s² de lado, el
    /// centro de masa a 0.5 m y una vía de 1 m: 100 × 10 × 0.5 / 1 = 500 N
    /// pasan de la llanta de adentro a la de afuera. Y la llanta de adentro de
    /// un eje con la mitad del peso a cuestas (0.5 de reparto), vía 1 m y
    /// h = 0.25 m se levanta a 0.5 × 1 / (2 × 1 × 0.25) = 1 g.
    #[test]
    fn la_transferencia_de_carga_a_mano() {
        use vehiculo::carga::{despegue_en_g, transferencia_lateral};
        assert!((transferencia_lateral(100.0, 10.0, 0.5, 1.0) - 500.0).abs() < 1e-12);
        assert!((despegue_en_g(0.5, 1.0, 1.0, 0.25) - 1.0).abs() < 1e-12);
    }

    /// **Los resortes del kart, a mano.** Dos llantas de 100 kN/m a 1 m:
    /// k t²/2 = 100 000 × 1 / 2 = 50 000 N·m/rad. Un resorte de 3 y uno de 6
    /// en serie: 3 × 6 / 9 = 2.
    #[test]
    fn los_resortes_del_kart_a_mano() {
        use vehiculo::carga::{en_serie, rigidez_de_llantas};
        assert!((rigidez_de_llantas(100e3, 1.0) - 50_000.0).abs() < 1e-9);
        assert!((en_serie(3.0, 6.0) - 2.0).abs() < 1e-12);
    }

    /// **El modelo de dos nodos, a mano.** Llantas de 2 adelante y 1 atrás,
    /// chasis de 2. D = 2 × 1 + 2 × (2 + 1) = 8.
    ///
    /// - Todo el par atrás (α = 0): adelante 2 × (0 + 2)/8 = 0.5, atrás
    ///   1 × (2 + 2)/8 = 0.5.
    /// - Todo adelante (α = 1): adelante 2 × (1 + 2)/8 = 0.75, atrás
    ///   1 × (0 + 2)/8 = 0.25.
    #[test]
    fn el_modelo_de_dos_nodos_a_mano() {
        use vehiculo::carga::reparto_dos_nodos;
        let (d, t) = reparto_dos_nodos(2.0, 1.0, 2.0, 0.0);
        assert!((d - 0.5).abs() < 1e-12 && (t - 0.5).abs() < 1e-12, "{d}, {t}");
        let (d, t) = reparto_dos_nodos(2.0, 1.0, 2.0, 1.0);
        assert!((d - 0.75).abs() < 1e-12 && (t - 0.25).abs() < 1e-12, "{d}, {t}");
    }
}
