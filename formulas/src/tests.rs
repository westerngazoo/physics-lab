//! Las afirmaciones del crate, como pruebas.

use super::*;
use core::f64::consts::{E, LN_2, PI, TAU};

const VARS: [&str; 4] = ["t", "a", "b", "c"];

fn val(src: &str, t: f64, a: f64) -> f64 {
    compila(src, &VARS)
        .unwrap_or_else(|e| panic!("{src:?} no compiló: {e:?}"))
        .valor(&[t, a, 0.0, 0.0])
}

fn err(src: &str) -> (Codigo, usize) {
    match compila(src, &VARS) {
        Ok(f) => panic!("{src:?} compiló y no debía: {f:?}"),
        Err(e) => (e.codigo, e.columna),
    }
}

fn cerca(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-12 * (1.0 + b.abs())
}

/// La gramática decide lo que el pizarrón decide.
#[test]
fn precedencia_como_en_el_pizarron() {
    assert_eq!(val("2+3*4", 0.0, 0.0), 14.0);
    assert_eq!(val("(2+3)*4", 0.0, 0.0), 20.0);
    assert_eq!(val("-t^2", 3.0, 0.0), -9.0); // −(t²), no (−t)²
    assert_eq!(val("2^3^2", 0.0, 0.0), 512.0); // a la derecha: 2^(3^2)
    assert_eq!(val("2^-1", 0.0, 0.0), 0.5);
    assert_eq!(val("1/2t", 4.0, 0.0), 2.0); // (1/2)·t, documentado
    assert_eq!(val("2t", 3.0, 0.0), 6.0);
    assert_eq!(val("(t+1)(t-1)", 3.0, 0.0), 8.0);
    assert_eq!(val("3sin(t)", 0.0, 0.0), 0.0);
    assert_eq!(val("t²", 3.0, 0.0), 9.0);
    assert_eq!(val("t³", 2.0, 0.0), 8.0);
}

/// `at²` es a·t², no (a·t)²: el exponente se pega a la última letra.
#[test]
fn nombres_partidos_como_en_desmos() {
    assert_eq!(val("at²", 3.0, 2.0), 18.0);
    assert_eq!(val("at^2", 3.0, 2.0), 18.0);
    assert_eq!(val("at", 3.0, 2.0), 6.0);
    assert!(cerca(val("ae^t", 1.0, 2.0), 2.0 * E));
}

/// Los símbolos que el lector copia de la propia lección.
#[test]
fn unicode_del_pizarron() {
    assert!(cerca(val("τ", 0.0, 0.0), TAU));
    assert!(cerca(val("2πt", 1.0, 0.0), 2.0 * PI));
    assert_eq!(val("−t", 2.0, 0.0), -2.0); // U+2212, el menos de verdad
    assert_eq!(val("3·t", 2.0, 0.0), 6.0);
    assert_eq!(val("t×t", 3.0, 0.0), 9.0);
    assert_eq!(val("t÷2", 3.0, 0.0), 1.5);
    assert_eq!(val("sen(t)", 1.0, 0.0), 1.0f64.sin());
}

/// `e^t` es exp(t) exacto, no ln(2.718281828459045…) disfrazado.
#[test]
fn e_a_la_t_es_exp_exacto() {
    for i in 0..50 {
        let t = -3.0 + 0.13 * i as f64;
        assert_eq!(val("e^t", t, 0.0), t.exp());
    }
    assert_eq!(val("e", 0.0, 0.0), E);
}

/// Todo error dice qué y dónde — en columnas de CARACTERES.
#[test]
fn errores_con_su_columna() {
    assert_eq!(err(""), (Codigo::Vacia, 1));
    assert_eq!(err("   "), (Codigo::Vacia, 1));
    assert_eq!(err("2+"), (Codigo::FaltaOperando, 3));
    assert_eq!(err("*3"), (Codigo::FaltaOperando, 1));
    assert_eq!(err("sin()"), (Codigo::FaltaOperando, 5));
    assert_eq!(err("sin(t"), (Codigo::FaltaParentesis, 4));
    assert_eq!(err("2*(t+1"), (Codigo::FaltaParentesis, 3));
    assert_eq!(err("t)"), (Codigo::ParentesisDeMas, 2));
    assert_eq!(err("sin t"), (Codigo::FuncionSinParentesis, 1));
    assert_eq!(err("x"), (Codigo::NombreDesconocido, 1));
    assert_eq!(err("2+sint"), (Codigo::NombreDesconocido, 3));
    assert_eq!(err("t2"), (Codigo::NumeroSuelto, 2));
    assert_eq!(err("1.2.3"), (Codigo::NumeroMalFormado, 1));
    assert_eq!(err("."), (Codigo::NumeroMalFormado, 1));
    assert_eq!(err("0,5"), (Codigo::Coma, 2));
    assert_eq!(err("t#"), (Codigo::CaracterInesperado, 2));
    // τ ocupa dos bytes y UNA columna: el # está en la 3, no en la 5.
    assert_eq!(err("ττ#"), (Codigo::CaracterInesperado, 3));
    assert_eq!(err("θ"), (Codigo::NombreDesconocido, 1));
}

/// Anidar sin fin no desborda nada: se rechaza con su código.
#[test]
fn anidamiento_acotado() {
    let mil = format!("{}t{}", "(".repeat(1000), ")".repeat(1000));
    assert_eq!(err(&mil).0, Codigo::DemasiadoProfunda);
    assert_eq!(err(&format!("{}t", "-".repeat(1000))).0, Codigo::DemasiadoProfunda);
    assert_eq!(err(&format!("{}t", "t^".repeat(500))).0, Codigo::DemasiadoProfunda);
    // Lo largo pero plano sí pasa: la pila de t+t+…+t mide dos.
    let larga = vec!["t"; 1000].join("+");
    assert_eq!(val(&larga, 1.0, 0.0), 1000.0);
}

/// La máscara dice qué variables nombra la fórmula.
#[test]
fn mascara_de_variables() {
    let f = compila("a t² + c", &VARS).unwrap();
    assert_eq!(f.mascara(), 0b1011);
    assert_eq!(compila("3", &VARS).unwrap().mascara(), 0);
}

/// Derivadas exactas contra derivadas hechas a mano, en una batería de
/// fórmulas que cubre cada regla: producto, cociente, cadena, potencia
/// constante y variable, y cada función.
#[test]
fn derivadas_contra_formas_cerradas() {
    type F = fn(f64) -> f64;
    let casos: [(&str, F, F, F); 11] = [
        ("3t² − 0.2t³", |t| 3.0 * t * t - 0.2 * t.powi(3), |t| 6.0 * t - 0.6 * t * t, |t| 6.0 - 1.2 * t),
        ("sin(2t)", |t| (2.0 * t).sin(), |t| 2.0 * (2.0 * t).cos(), |t| -4.0 * (2.0 * t).sin()),
        ("ln(1+t²)", |t| (1.0 + t * t).ln(), |t| 2.0 * t / (1.0 + t * t), |t| (2.0 - 2.0 * t * t) / (1.0 + t * t).powi(2)),
        ("sqrt(1+t)", |t| (1.0 + t).sqrt(), |t| 0.5 / (1.0 + t).sqrt(), |t| -0.25 / (1.0 + t).powf(1.5)),
        ("t/(1+t)", |t| t / (1.0 + t), |t| 1.0 / (1.0 + t).powi(2), |t| -2.0 / (1.0 + t).powi(3)),
        ("atan(t)", |t| t.atan(), |t| 1.0 / (1.0 + t * t), |t| -2.0 * t / (1.0 + t * t).powi(2)),
        ("tanh(t)", |t| t.tanh(), |t| 1.0 - t.tanh().powi(2), |t| -2.0 * t.tanh() * (1.0 - t.tanh().powi(2))),
        ("2^t", |t| 2f64.powf(t), |t| 2f64.powf(t) * LN_2, |t| 2f64.powf(t) * LN_2 * LN_2),
        ("e^(−t/2)cos(3t)", |t| (-t / 2.0).exp() * (3.0 * t).cos(),
            |t| (-t / 2.0).exp() * (-0.5 * (3.0 * t).cos() - 3.0 * (3.0 * t).sin()),
            |t| (-t / 2.0).exp() * (-8.75 * (3.0 * t).cos() + 3.0 * (3.0 * t).sin())),
        ("cosh(t) − sinh(t)", |t| (-t).exp(), |t| -(-t).exp(), |t| (-t).exp()),
        ("log(t)", |t| t.log10(), |t| 1.0 / (t * core::f64::consts::LN_10), |t| -1.0 / (t * t * core::f64::consts::LN_10)),
    ];
    for (src, f, f1, f2) in casos {
        let prog = compila(src, &VARS).unwrap();
        for i in 1..60 {
            let t = 0.05 + 0.13 * i as f64;
            let j = prog.jet(&[t, 0.0, 0.0, 0.0], 0);
            let tol = |x: f64, y: f64| (x - y).abs() <= 1e-11 * (1.0 + y.abs());
            assert!(tol(j.v, f(t)), "{src} f({t}) = {} vs {}", j.v, f(t));
            assert!(tol(j.d1, f1(t)), "{src} f'({t}) = {} vs {}", j.d1, f1(t));
            assert!(tol(j.d2, f2(t)), "{src} f''({t}) = {} vs {}", j.d2, f2(t));
        }
    }
}

/// La recta y la parábola en el cero, donde una fórmula ingenua de la
/// regla de la potencia fabrica 0·∞.
#[test]
fn potencias_en_el_origen() {
    let recta = compila("t^1", &VARS).unwrap().jet(&[0.0], 0);
    assert_eq!((recta.v, recta.d1, recta.d2), (0.0, 1.0, 0.0));
    let parabola = compila("t^2", &VARS).unwrap().jet(&[0.0], 0);
    assert_eq!((parabola.v, parabola.d1, parabola.d2), (0.0, 0.0, 2.0));
}

/// Un generador determinista (xorshift): la misma "aleatoriedad" en cada
/// corrida, para que un fallo se pueda repetir.
struct Xorshift(u64);
impl Xorshift {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

/// Ninguna entrada, por absurda que sea, provoca un pánico — que en la
/// página sería una trampa de wasm y una lección muerta.
#[test]
fn ninguna_entrada_provoca_panico() {
    let piezas = [
        "t", "a", "b", "c", "e", "τ", "π", "sin", "cos", "sen", "exp", "ln",
        "sqrt", "abs", "tan", "(", ")", "+", "-", "−", "*", "/", "^", "²", "³",
        "·", "1", "2.5", ".", "0", ",", "#", " ", "x", "ñ", "∫", "😀", "\u{0}",
        "e^", "at", "tt", "1e5", "..", "((", "))",
    ];
    let mut rng = Xorshift(0x9e37_79b9_7f4a_7c15);
    for _ in 0..20_000 {
        let n = rng.below(24);
        let src: String = (0..n).map(|_| piezas[rng.below(piezas.len())]).collect();
        comprueba(&src);
    }
    // Y bytes crudos, pasados por la misma puerta UTF-8 que usa el ABI.
    for _ in 0..5_000 {
        let n = rng.below(40);
        let bytes: Vec<u8> = (0..n).map(|_| rng.next() as u8).collect();
        comprueba(&String::from_utf8_lossy(&bytes));
    }
}

fn comprueba(src: &str) {
    match compila(src, &VARS) {
        Ok(f) => {
            for t in [-2.0, 0.0, 0.5, 3.0] {
                let _ = f.jet(&[t, 1.0, 2.0, 3.0], 0);
            }
        }
        Err(e) => {
            let largo = src.chars().count();
            assert!(e.columna >= 1 && e.columna <= largo + 1, "{src:?}: {e:?}");
        }
    }
}

/// La tabla de mensajes de la página es ESTA tabla. El código viaja como
/// número por el ABI; si alguien cambia un mensaje de un lado y no del
/// otro, esta prueba lo dice antes que un lector confundido.
#[test]
fn la_pagina_tiene_los_mismos_mensajes() {
    let ruta = concat!(env!("CARGO_MANIFEST_DIR"), "/../public/js/runtime.js");
    let js = std::fs::read_to_string(ruta).expect("runtime.js");
    for c in Codigo::TODOS {
        let linea = format!("{}: \"{}\"", c as u8, c.mensaje());
        assert!(js.contains(&linea), "runtime.js no tiene {linea}");
    }
}
