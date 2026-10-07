//! Un flexor a lo largo de su flexión: largo, palanca, y nada más.
//!
//! El trayecto es **sintético**. No es la anatomía de nadie y no sale de
//! ningún artículo: es la geometría más simple que reproduce lo que hace
//! un flexor — una palanca que nace en cero, pasa por un máximo y vuelve a
//! caer. Las coordenadas con fuente citada son el hueco que `RFC-002 §7.5`
//! deja abierto, y fingir que ya lo llenamos sería peor que dejarlo.
//!
//! Fíjate en lo que NO imprime: newtons. Ese es el punto del módulo.
//!
//! ```text
//! cargo run -p mecanica --example flexor
//! ```

use core::f64::consts::TAU;

use mecanica::musculo::{acortamiento, brazo, Musculo, Recto};

fn main() {
    // Sintético, y elegido para que la palanca se parezca en forma a la de
    // un flexor de codo: nace chica, pica a media flexión, y cae.
    let flexor = Recto {
        origen: 0.130,
        insercion: 0.045,
        abierto: TAU * 0.47,
        rango: (0.0, TAU * 0.40),
    };

    println!("TRAYECTO SINTÉTICO — no es la anatomía de nadie\n");
    println!(" flexión   largo (cm)   palanca (cm)   estirado");
    for g in [0, 20, 40, 60, 90, 120, 145] {
        let th = f64::from(g) * TAU / 360.0;
        println!(
            "  {g:4}°      {:5.2}         {:4.2}          {:.0}%",
            flexor.largo(th) * 100.0,
            brazo(&flexor, th) * 100.0,
            flexor.normalizado(th) * 100.0
        );
    }

    let (a, b) = flexor.rango();
    println!(
        "\n  se acorta {:.1} cm en toda la flexión",
        acortamiento(&flexor, a, b) * 100.0
    );

    // Dónde pica la palanca. Ese pico es geometría pura, y es la mitad de
    // la explicación de por qué un levantamiento se atora en algún lado:
    // la otra mitad es la demanda, que ya vive en los módulos de ejercicio.
    let n = 400;
    let (mut mejor, mut donde) = (f64::MIN, 0.0);
    for i in 0..=n {
        let th = b * f64::from(i) / f64::from(n);
        let r = brazo(&flexor, th);
        if r > mejor {
            (mejor, donde) = (r, th);
        }
    }
    println!(
        "  la palanca pica en {:.0}°, con {:.2} cm",
        donde * 360.0 / TAU,
        mejor * 100.0
    );
    println!("\n  (y aquí no hay un número de fuerza: RFC-002 §7.3)");
}
