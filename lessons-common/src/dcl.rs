//! Diagramas de cuerpo libre: el cuerpo, sus fuerzas, y sus palancas.
//!
//! Existe porque cada lección que quería dibujar fuerzas las ponía a mano
//! con coordenadas fijas y un índice de estilo elegido a ojo, y ahí se
//! pierden las tres cosas que hacen que un diagrama enseñe algo:
//!
//! 1. **La longitud tiene que significar la magnitud.** Si la flecha de
//!    200 N mide lo mismo que la de 50, el dibujo miente aunque los
//!    números al lado estén bien. Aquí una sola escala manda sobre todas
//!    las flechas del diagrama, y la calcula el diagrama, no el autor.
//! 2. **El color tiene que significar la tensión.** Verde a rojo por
//!    magnitud, para poder ver dónde está el problema sin leer un número.
//! 3. **La palanca tiene que estar dibujada.** El brazo de momento es la
//!    perpendicular del eje a la LÍNEA de acción, no la distancia al
//!    punto donde se aplica, y esa diferencia es la mitad de lo que hay
//!    que enseñar. Se dibuja punteada, que es lo que la distingue de una
//!    fuerza.
//!
//! Nada de esto decide física. El diagrama recibe fuerzas ya calculadas y
//! sólo se encarga de que se vean como lo que son.

use crate::Prims;

/// Una fuerza sobre el cuerpo.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fuerza {
    /// Dónde se aplica, en unidades del mundo.
    pub en: (f64, f64),
    /// El vector, en newton. Su dirección es la línea de acción.
    pub vec: (f64, f64),
    /// Cuál etiqueta de la página la nombra, si alguna.
    pub etiqueta: Option<usize>,
}

impl Fuerza {
    /// Una fuerza con nombre.
    #[must_use]
    pub fn nueva(en: (f64, f64), vec: (f64, f64), etiqueta: usize) -> Self {
        Fuerza {
            en,
            vec,
            etiqueta: Some(etiqueta),
        }
    }
    /// Su magnitud, newton.
    #[must_use]
    pub fn magnitud(&self) -> f64 {
        self.vec.0.hypot(self.vec.1)
    }
    /// Su dirección, unitaria. `(0, 0)` si la fuerza es nula.
    #[must_use]
    pub fn direccion(&self) -> (f64, f64) {
        let n = self.magnitud();
        if n < EPS {
            (0.0, 0.0)
        } else {
            (self.vec.0 / n, self.vec.1 / n)
        }
    }
}

/// Por debajo de esto una fuerza no existe y no se dibuja.
const EPS: f64 = 1e-12;

/// Cómo se ve el diagrama.
#[derive(Clone, Copy, Debug)]
pub struct Estilo {
    /// Largo de la flecha MÁS grande, en unidades del mundo. Todas las
    /// demás salen proporcionales a ésta.
    pub largo_max: f64,
    /// Los estilos de la página, de frío a caliente. El diagrama reparte
    /// las magnitudes entre ellos.
    pub calor: &'static [usize],
    /// El estilo de las líneas punteadas de palanca.
    pub palanca: usize,
    /// Cuánto se separa la etiqueta de la punta de la flecha.
    pub margen: f64,
}

impl Estilo {
    /// Qué estilo le toca a una fuerza de magnitud `m`, con `max` como
    /// tope de la escala.
    ///
    /// La banda es proporcional a la magnitud, no a su posición en una
    /// lista ordenada: dos fuerzas casi iguales tienen que salir del
    /// mismo color aunque una sea la mayor.
    #[must_use]
    pub fn banda(&self, m: f64, max: f64) -> usize {
        if self.calor.is_empty() {
            return 0;
        }
        if max < EPS {
            return self.calor[0];
        }
        let t = (m / max).clamp(0.0, 1.0);
        let n = self.calor.len();
        let i = ((t * n as f64) as usize).min(n - 1);
        self.calor[i]
    }
}

/// El brazo de momento de una fuerza alrededor de un eje, y el pie de esa
/// perpendicular sobre la línea de acción.
///
/// Devuelve `(brazo, pie)`. Una fuerza nula no tiene línea de acción y
/// devuelve brazo cero en el propio eje.
#[must_use]
pub fn palanca(eje: (f64, f64), f: &Fuerza) -> (f64, (f64, f64)) {
    let (ux, uy) = f.direccion();
    if ux == 0.0 && uy == 0.0 {
        return (0.0, eje);
    }
    let (rx, ry) = (eje.0 - f.en.0, eje.1 - f.en.1);
    let t = rx * ux + ry * uy;
    let pie = (f.en.0 + t * ux, f.en.1 + t * uy);
    ((eje.0 - pie.0).hypot(eje.1 - pie.1), pie)
}

/// La escala del diagrama: unidades del mundo por newton.
///
/// Sale de la fuerza MÁS grande, así que todas las flechas del diagrama
/// comparten una sola regla. Un diagrama sin escala común es un dibujo.
#[must_use]
pub fn escala(fuerzas: &[Fuerza], est: &Estilo) -> f64 {
    let max = maxima(fuerzas);
    if max < EPS {
        0.0
    } else {
        est.largo_max / max
    }
}

/// La mayor magnitud del conjunto.
#[must_use]
pub fn maxima(fuerzas: &[Fuerza]) -> f64 {
    fuerzas.iter().map(Fuerza::magnitud).fold(0.0, f64::max)
}

/// Dibuja las fuerzas: cada una a escala, coloreada por su magnitud, con
/// su etiqueta y su número en newton junto a la punta.
///
/// Una fuerza nula no se dibuja. Dibujarla como un punto la haría parecer
/// una fuerza chiquita, y no lo es: no existe.
pub fn fuerzas(out: &mut Prims, fs: &[Fuerza], est: &Estilo) {
    let k = escala(fs, est);
    let max = maxima(fs);
    for f in fs {
        let m = f.magnitud();
        if m < EPS {
            continue;
        }
        let st = est.banda(m, max);
        let punta = (f.en.0 + f.vec.0 * k, f.en.1 + f.vec.1 * k);
        out.arrow(f.en.0, f.en.1, punta.0, punta.1, st);
        let (ux, uy) = f.direccion();
        let etq = (punta.0 + ux * est.margen, punta.1 + uy * est.margen);
        if let Some(idx) = f.etiqueta {
            out.label(etq.0, etq.1, idx, st);
        }
        out.numero(etq.0, etq.1 - est.margen, m, 0, st);
    }
}

/// Dibuja el brazo de momento de una fuerza alrededor de un eje: la
/// perpendicular punteada del eje a la línea de acción.
///
/// No dibuja nada si el eje ya está sobre la línea. Ahí el brazo es cero
/// y el ejercicio no le pide nada a esa articulación, que es información
/// y no un descuido.
pub fn brazo(out: &mut Prims, eje: (f64, f64), f: &Fuerza, est: &Estilo, minimo: f64) {
    let (d, pie) = palanca(eje, f);
    if d < minimo {
        return;
    }
    out.polyline([eje, pie], est.palanca);
    let medio = ((eje.0 + pie.0) / 2.0, (eje.1 + pie.1) / 2.0);
    out.numero(medio.0, medio.1 + est.margen, d, 2, est.palanca);
}

/// La resultante de un conjunto, como una sola fuerza aplicada en `en`.
#[must_use]
pub fn resultante(fs: &[Fuerza], en: (f64, f64), etiqueta: Option<usize>) -> Fuerza {
    let vec = fs
        .iter()
        .fold((0.0, 0.0), |a, f| (a.0 + f.vec.0, a.1 + f.vec.1));
    Fuerza { en, vec, etiqueta }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EST: Estilo = Estilo {
        largo_max: 0.30,
        calor: &[0, 1, 2, 3],
        palanca: 0,
        margen: 0.02,
    };

    fn peso() -> Fuerza {
        Fuerza::nueva((0.0, 0.0), (0.0, -200.0), 0)
    }

    /// La flecha más grande mide `largo_max`, y las demás salen a escala.
    /// Sin esto el dibujo miente aunque los números estén bien.
    #[test]
    fn la_longitud_significa_la_magnitud() {
        let fs = [peso(), Fuerza::nueva((0.0, 0.0), (50.0, 0.0), 1)];
        let k = escala(&fs, &EST);
        assert!((200.0 * k - EST.largo_max).abs() < 1e-12);
        assert!((50.0 * k - EST.largo_max / 4.0).abs() < 1e-12);
    }

    /// Dos fuerzas casi iguales salen del mismo color aunque una sea la
    /// mayor: la banda es por magnitud, no por orden en una lista.
    #[test]
    fn el_color_es_por_magnitud_no_por_orden() {
        assert_eq!(EST.banda(199.0, 200.0), EST.banda(200.0, 200.0));
    }

    /// El brazo es la perpendicular a la LÍNEA de acción, no la distancia
    /// al punto de aplicación. Un peso aplicado a 3 m de lado, colgando,
    /// tiene brazo 3: aritmética de mano.
    #[test]
    fn el_brazo_va_a_la_linea_no_al_punto() {
        let f = Fuerza::nueva((3.0, 5.0), (0.0, -100.0), 0);
        let (d, pie) = palanca((0.0, 0.0), &f);
        assert!((d - 3.0).abs() < 1e-12, "dio {d}");
        assert!(pie.0.abs() - 3.0 < 1e-12 && pie.1.abs() < 1e-12);
    }

    /// Y se muere cuando la línea pasa por el eje, aunque el punto de
    /// aplicación esté lejísimos.
    #[test]
    fn el_brazo_se_muere_en_la_vertical() {
        let f = Fuerza::nueva((0.0, 9.0), (0.0, -100.0), 0);
        assert!(palanca((0.0, 0.0), &f).0 < 1e-12);
    }

    /// Una fuerza nula no se dibuja. Un punto la haría ver chiquita, y no
    /// es chiquita: no existe.
    #[test]
    fn la_fuerza_nula_no_se_dibuja() {
        let mut buf = [0.0; 64];
        let mut p = Prims::new(&mut buf);
        fuerzas(&mut p, &[Fuerza::nueva((0.0, 0.0), (0.0, 0.0), 0)], &EST);
        assert_eq!(p.len(), 0);
    }

    /// Y una fuerza normal sí, con su flecha, su etiqueta y su número.
    #[test]
    fn una_fuerza_deja_flecha_etiqueta_y_numero() {
        let mut buf = [0.0; 64];
        let mut p = Prims::new(&mut buf);
        fuerzas(&mut p, &[peso()], &EST);
        assert_eq!(buf[0], 3.0, "primero la flecha");
        assert_eq!(buf[6], 4.0, "luego la etiqueta");
        assert_eq!(buf[11], 5.0, "luego el número");
        // [5, x, y, valor, decimales, estilo]: el valor va en la cuarta
        // ranura del registro, no en la tercera.
        assert!(
            (buf[14] - 200.0).abs() < 1e-12,
            "y el número son los newton"
        );
    }

    /// La resultante de dos fuerzas opuestas iguales es cero, y por lo
    /// tanto no se dibuja: el cuerpo está en equilibrio.
    #[test]
    fn el_equilibrio_no_deja_resultante() {
        let fs = [peso(), Fuerza::nueva((0.0, 0.0), (0.0, 200.0), 1)];
        let r = resultante(&fs, (0.0, 0.0), None);
        assert!(r.magnitud() < 1e-12);
    }
}
