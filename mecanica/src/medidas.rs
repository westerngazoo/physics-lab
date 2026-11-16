//! Las medidas de una persona para la sentadilla: qué se midió, cómo y
//! cuándo, y qué se estimó.
//!
//! El protocolo para tomarlas en casa, con una cinta y una foto, está en
//! `docs/protocolo-medidas-sentadilla.md`; la plantilla para llenar, en
//! `medidas/plantilla-sentadilla.toml`.
//!
//! # Lo que se mide son alturas, no huesos
//!
//! De pie y descalzo, la cinta va del piso a cada referencia: el acromion
//! (hombro), el trocánter mayor (cadera), la interlínea lateral de la
//! rodilla y el maléolo lateral (tobillo). Cada segmento es la resta de
//! dos alturas: es la misma construcción que la figura 4.1 de Winter
//! (*Biomechanics and Motor Control of Human Movement*, 4.ª ed., 2009),
//! de donde salen las proporciones de [`crate::maquina_humana::largo`]:
//!
//! | referencia | fracción de la estatura |
//! |---|---|
//! | hombro (acromion) | 0.818 |
//! | trocánter mayor | 0.530 |
//! | interlínea de la rodilla | 0.285 |
//! | maléolo lateral | 0.039 |
//!
//! fémur = 0.530 − 0.285 = 0.245, tibia = 0.285 − 0.039 = 0.246 y tronco
//! (trocánter → hombro) = 0.818 − 0.530 = 0.288.
//!
//! # Medido o estimado, nunca a medias
//!
//! Cada campo es `medido` (con valor, fecha y cómo) o `estimado` (sin
//! valor: lo pone la proporción de Winter a partir de la estatura, o la
//! convención del modelo). Un valor "más o menos" no existe: o se midió o
//! se estima, y el resultado dice cuál fue cuál. Si los dos extremos de un
//! segmento son estimados, el segmento es exactamente el de
//! [`crate::maquina_humana::Persona`], así que un archivo sin medidas
//! reproduce el modelo de siempre.
//!
//! # Privacidad
//!
//! La persona es un seudónimo (`P-` y de 4 a 12 letras mayúsculas o
//! cifras, con al menos una cifra). El archivo no tiene lugar para un
//! nombre, una cara ni una foto: una clave que no conoce es un error.

pub mod informe;
pub mod toml;

use std::fmt;

use crate::maquina_humana::largo;
use crate::sentadilla::Sentadilla;
use toml::{Documento, Entrada, Tabla, Valor};

/// La versión del formato que este código lee.
pub const FORMATO: &str = "medidas-sentadilla/1";

/// Largo del pie (talón → punta del dedo más largo) entre la estatura.
/// Winter, fig. 4.1.
pub const PIE: f64 = 0.152;

/// El ángulo de la tibia cuando no se midió, en grados desde la vertical.
///
/// Es el del modelo publicado (`muneco.py`, el origen de
/// [`crate::sentadilla`]). **Es una convención sin fuente**, y el
/// resultado lo dice cada vez que se usa.
pub const ANGULO_TIBIA_CONVENCION: f64 = 22.0;

/// Un campo del archivo de medidas.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Campo {
    /// Estatura, descalzo.
    Estatura,
    /// Masa corporal.
    Masa,
    /// Piso → acromion.
    AlturaHombro,
    /// Piso → trocánter mayor.
    AlturaTrocanter,
    /// Piso → interlínea articular lateral de la rodilla.
    AlturaRodilla,
    /// Piso → maléolo lateral.
    AlturaMaleolo,
    /// Talón → punta del dedo más largo.
    LargoPie,
    /// Piso → donde descansa la barra en la espalda, de pie.
    AlturaBarra,
    /// Inclinación de la tibia desde la vertical, en el fondo.
    AnguloTibia,
}

/// La unidad en que se escribe un campo. Va en el nombre de la clave
/// (`valor_cm`, `valor_kg`, `valor_grados`) para que un metro escrito
/// donde va un centímetro sea un error y no un fémur de 0.43 cm.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unidad {
    /// Centímetros.
    Cm,
    /// Kilogramos.
    Kg,
    /// Grados sexagesimales.
    Grados,
}

impl Unidad {
    /// La clave del valor en el archivo.
    #[must_use]
    pub fn clave(self) -> &'static str {
        match self {
            Unidad::Cm => "valor_cm",
            Unidad::Kg => "valor_kg",
            Unidad::Grados => "valor_grados",
        }
    }

    /// El símbolo, para mostrar.
    #[must_use]
    pub fn simbolo(self) -> &'static str {
        match self {
            Unidad::Cm => "cm",
            Unidad::Kg => "kg",
            Unidad::Grados => "°",
        }
    }
}

impl Campo {
    /// Todos, en el orden del archivo.
    pub const TODOS: [Campo; 9] = [
        Campo::Estatura,
        Campo::Masa,
        Campo::AlturaHombro,
        Campo::AlturaTrocanter,
        Campo::AlturaRodilla,
        Campo::AlturaMaleolo,
        Campo::LargoPie,
        Campo::AlturaBarra,
        Campo::AnguloTibia,
    ];

    /// El nombre de su tabla en el archivo.
    #[must_use]
    pub fn nombre(self) -> &'static str {
        match self {
            Campo::Estatura => "estatura",
            Campo::Masa => "masa",
            Campo::AlturaHombro => "altura_hombro",
            Campo::AlturaTrocanter => "altura_trocanter",
            Campo::AlturaRodilla => "altura_rodilla",
            Campo::AlturaMaleolo => "altura_maleolo",
            Campo::LargoPie => "largo_pie",
            Campo::AlturaBarra => "altura_barra",
            Campo::AnguloTibia => "angulo_tibia",
        }
    }

    /// En qué unidad se escribe.
    #[must_use]
    pub fn unidad(self) -> Unidad {
        match self {
            Campo::Masa => Unidad::Kg,
            Campo::AnguloTibia => Unidad::Grados,
            _ => Unidad::Cm,
        }
    }

    /// La proporción de Winter (fig. 4.1) que lo estima a partir de la
    /// estatura, si la hay.
    #[must_use]
    pub fn fraccion_de_estatura(self) -> Option<f64> {
        let maleolo = largo::TOBILLO;
        let rodilla = maleolo + largo::TIBIA;
        let trocanter = rodilla + largo::FEMUR;
        match self {
            Campo::AlturaMaleolo => Some(maleolo),
            Campo::AlturaRodilla => Some(rodilla),
            Campo::AlturaTrocanter => Some(trocanter),
            Campo::AlturaHombro => Some(trocanter + largo::TORSO),
            Campo::LargoPie => Some(PIE),
            _ => None,
        }
    }

    fn indice(self) -> usize {
        Campo::TODOS
            .iter()
            .position(|&c| c == self)
            .expect("TODOS tiene cada campo")
    }

    fn por_nombre(nombre: &str) -> Option<Campo> {
        Campo::TODOS.into_iter().find(|c| c.nombre() == nombre)
    }
}

/// Un valor medido, con su procedencia.
#[derive(Clone, Debug, PartialEq)]
pub struct Medicion {
    /// En la unidad del campo ([`Campo::unidad`]).
    pub valor: f64,
    /// Cuándo, `AAAA-MM-DD`.
    pub fecha: String,
    /// Cómo: instrumento, postura, quién ayudó (sin nombres).
    pub como: String,
}

/// Lo que dice el archivo de un campo.
#[derive(Clone, Debug, PartialEq)]
pub enum Dato {
    /// Se midió.
    Medido(Medicion),
    /// No se midió: lo pone la proporción de Winter o la convención del
    /// modelo. La nota es opcional (por qué no se midió).
    Estimado {
        /// Nota libre del archivo, si la hay.
        nota: Option<String>,
    },
}

impl Dato {
    /// `true` si se midió.
    #[must_use]
    pub fn es_medido(&self) -> bool {
        matches!(self, Dato::Medido(_))
    }
}

/// Un archivo de medidas que pasó la revisión.
#[derive(Clone, Debug, PartialEq)]
pub struct Medidas {
    /// Seudónimo.
    pub persona: String,
    /// Fecha de la sesión de medidas.
    pub fecha: String,
    datos: [Dato; 9],
}

/// Lo que entra al modelo, en metros y radianes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Entradas {
    /// Cadera → rodilla.
    pub femur_m: f64,
    /// Tobillo → rodilla.
    pub tibia_m: f64,
    /// Cadera → barra, sobre la línea del tronco. Es lo que el modelo
    /// llama `torso_m`: pone la barra al final del tronco.
    pub torso_m: f64,
    /// Cadera → hombro, para dibujar el hombro en la misma línea.
    pub hombro_m: f64,
    /// Inclinación de la tibia desde la vertical.
    pub tibia_rad: f64,
}

/// Un cambio pequeño en un campo, para medir cuánto mueve el resultado.
pub(crate) type Ajuste = Option<(Campo, f64)>;

impl Medidas {
    /// Lo que dice el archivo de ese campo.
    #[must_use]
    pub fn dato(&self, c: Campo) -> &Dato {
        &self.datos[c.indice()]
    }

    /// Estatura, cm (siempre medida).
    #[must_use]
    pub fn estatura_cm(&self) -> f64 {
        self.valor(Campo::Estatura)
    }

    /// Masa, kg (siempre medida).
    #[must_use]
    pub fn masa_kg(&self) -> f64 {
        self.valor(Campo::Masa)
    }

    /// El valor que se usa, en la unidad del campo: el medido, o el
    /// estimado si no se midió.
    #[must_use]
    pub fn valor(&self, c: Campo) -> f64 {
        self.valor_con(c, None)
    }

    pub(crate) fn valor_con(&self, c: Campo, aj: Ajuste) -> f64 {
        let base = match (self.dato(c), c) {
            (Dato::Medido(m), _) => m.valor,
            (Dato::Estimado { .. }, Campo::AlturaBarra) => self.valor_con(Campo::AlturaHombro, aj),
            (Dato::Estimado { .. }, Campo::AnguloTibia) => ANGULO_TIBIA_CONVENCION,
            (Dato::Estimado { .. }, otro) => {
                otro.fraccion_de_estatura()
                    .expect("solo los campos con proporción pueden estimarse")
                    * self.valor_con(Campo::Estatura, aj)
            }
        };
        match aj {
            Some((campo, delta)) if campo == c => base + delta,
            _ => base,
        }
    }

    /// Los campos de los que depende el valor de `c`.
    fn depende_de(&self, c: Campo) -> Vec<Campo> {
        match (c, self.dato(c)) {
            (Campo::AlturaBarra, Dato::Estimado { .. }) => vec![c, Campo::AlturaHombro],
            _ => vec![c],
        }
    }

    /// Un segmento: la resta de dos alturas, en metros.
    ///
    /// Si nada de lo que lo define se midió (ni se está moviendo), es
    /// directamente la proporción de Winter por la estatura: el mismo
    /// número, bit por bit, que da [`crate::maquina_humana::Persona`].
    fn segmento_m(&self, arriba: Campo, abajo: Campo, fraccion: f64, aj: Ajuste) -> f64 {
        let mut deps = self.depende_de(arriba);
        deps.extend(self.depende_de(abajo));
        let todo_estimado = deps.iter().all(|&c| !self.dato(c).es_medido());
        let se_mueve = aj.is_some_and(|(c, _)| deps.contains(&c));
        if todo_estimado && !se_mueve {
            fraccion * (self.valor_con(Campo::Estatura, aj) / 100.0)
        } else {
            (self.valor_con(arriba, aj) - self.valor_con(abajo, aj)) / 100.0
        }
    }

    /// Lo que entra al modelo.
    #[must_use]
    pub fn entradas(&self) -> Entradas {
        self.entradas_con(None)
    }

    pub(crate) fn entradas_con(&self, aj: Ajuste) -> Entradas {
        use Campo::{AlturaBarra, AlturaHombro, AlturaMaleolo, AlturaRodilla, AlturaTrocanter};
        Entradas {
            femur_m: self.segmento_m(AlturaTrocanter, AlturaRodilla, largo::FEMUR, aj),
            tibia_m: self.segmento_m(AlturaRodilla, AlturaMaleolo, largo::TIBIA, aj),
            torso_m: self.segmento_m(AlturaBarra, AlturaTrocanter, largo::TORSO, aj),
            hombro_m: self.segmento_m(AlturaHombro, AlturaTrocanter, largo::TORSO, aj),
            tibia_rad: self.valor_con(Campo::AnguloTibia, aj).to_radians(),
        }
    }

    /// La sentadilla de esta persona con esa carga, barra sobre el medio pie.
    #[must_use]
    pub fn sentadilla(&self, carga_kg: f64) -> Sentadilla {
        sentadilla_de(&self.entradas(), carga_kg)
    }
}

pub(crate) fn sentadilla_de(e: &Entradas, carga_kg: f64) -> Sentadilla {
    Sentadilla {
        femur_m: e.femur_m,
        tibia_m: e.tibia_m,
        torso_m: e.torso_m,
        load_kg: carga_kg,
        bar_offset_m: 0.0,
        shank_rad: e.tibia_rad,
    }
}

/// Un error o un aviso de la revisión: dónde y por qué.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problema {
    /// La tabla o la clave (`altura_rodilla`, `persona`, `archivo`).
    pub donde: String,
    /// Línea del archivo, si aplica.
    pub linea: Option<usize>,
    /// Qué pasa, en español y con qué hacer.
    pub motivo: String,
}

impl fmt::Display for Problema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.linea {
            Some(l) => write!(f, "línea {l}, {}: {}", self.donde, self.motivo),
            None => write!(f, "{}: {}", self.donde, self.motivo),
        }
    }
}

fn problema(donde: &str, linea: Option<usize>, motivo: impl Into<String>) -> Problema {
    Problema {
        donde: donde.to_string(),
        linea,
        motivo: motivo.into(),
    }
}

/// Un archivo que pasó: las medidas y los avisos (lo raro que no impide
/// calcular, pero que conviene volver a medir).
#[derive(Clone, Debug, PartialEq)]
pub struct Revisadas {
    /// Las medidas.
    pub medidas: Medidas,
    /// Lo que está fuera de lo típico, sin ser imposible.
    pub avisos: Vec<Problema>,
}

/// Lee y revisa un archivo de medidas.
///
/// Junta **todos** los errores en una pasada, para que se corrijan de una
/// vez y no uno por corrida.
///
/// # Errors
/// Cada problema que impide calcular: lectura, claves, procedencia,
/// rangos imposibles y referencias en desorden.
pub fn revisar(texto: &str) -> Result<Revisadas, Vec<Problema>> {
    let doc = toml::leer(texto).map_err(|e| vec![problema("archivo", Some(e.linea), e.motivo)])?;
    let mut errores = Vec::new();
    let (persona, fecha) = revisar_raiz(&doc, &mut errores);
    for (nombre, (linea, _)) in &doc.tablas {
        if Campo::por_nombre(nombre).is_none() {
            errores.push(problema(
                nombre,
                Some(*linea),
                "tabla desconocida: el archivo solo lleva las tablas de la plantilla \
                 (y nunca nombres, caras ni fotos)",
            ));
        }
    }
    let datos: Vec<Option<Dato>> = Campo::TODOS
        .into_iter()
        .map(|c| revisar_campo(c, doc.tablas.get(c.nombre()), &mut errores))
        .collect();
    let (Some(persona), Some(fecha)) = (persona, fecha) else {
        return Err(errores);
    };
    let Ok(datos) = <[Dato; 9]>::try_from(datos.into_iter().flatten().collect::<Vec<_>>()) else {
        return Err(errores);
    };
    if !errores.is_empty() {
        return Err(errores);
    }
    let medidas = Medidas {
        persona,
        fecha,
        datos,
    };
    let mut avisos = Vec::new();
    revisar_rangos(&medidas, &doc, &mut errores, &mut avisos);
    if errores.is_empty() {
        Ok(Revisadas { medidas, avisos })
    } else {
        Err(errores)
    }
}

fn texto<'a>(t: &'a Tabla, clave: &str) -> Option<(&'a str, usize)> {
    match t.get(clave) {
        Some(Entrada {
            valor: Valor::Texto(s),
            linea,
        }) => Some((s.as_str(), *linea)),
        _ => None,
    }
}

fn revisar_raiz(doc: &Documento, errores: &mut Vec<Problema>) -> (Option<String>, Option<String>) {
    for (clave, e) in &doc.raiz {
        if !["formato", "persona", "fecha"].contains(&clave.as_str()) {
            errores.push(problema(
                clave,
                Some(e.linea),
                "clave desconocida arriba del archivo: solo van formato, persona y fecha",
            ));
        }
    }
    match texto(&doc.raiz, "formato") {
        Some((f, _)) if f == FORMATO => {}
        Some((f, l)) => errores.push(problema(
            "formato",
            Some(l),
            format!("este código lee «{FORMATO}», el archivo dice «{f}»"),
        )),
        None => errores.push(problema(
            "formato",
            None,
            format!("falta formato = \"{FORMATO}\""),
        )),
    }
    let persona = match texto(&doc.raiz, "persona") {
        Some((p, _)) if es_seudonimo(p) => Some(p.to_string()),
        Some((_, l)) => {
            errores.push(problema(
                "persona",
                Some(l),
                "el seudónimo es P- y de 4 a 12 letras mayúsculas o cifras, con al menos \
                 una cifra (por ejemplo P-7K3Q). Nunca un nombre",
            ));
            None
        }
        None => {
            errores.push(problema(
                "persona",
                None,
                "falta persona = \"P-…\" (un seudónimo)",
            ));
            None
        }
    };
    let fecha = match texto(&doc.raiz, "fecha") {
        Some((f, _)) if es_fecha(f) => Some(f.to_string()),
        Some((f, l)) => {
            errores.push(problema(
                "fecha",
                Some(l),
                format!("«{f}» no es una fecha AAAA-MM-DD"),
            ));
            None
        }
        None => {
            errores.push(problema(
                "fecha",
                None,
                "falta la fecha de la sesión, AAAA-MM-DD",
            ));
            None
        }
    };
    (persona, fecha)
}

/// `P-` y de 4 a 12 mayúsculas o cifras, con al menos una cifra.
fn es_seudonimo(p: &str) -> bool {
    p.strip_prefix("P-").is_some_and(|r| {
        (4..=12).contains(&r.len())
            && r.bytes()
                .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
            && r.bytes().any(|b| b.is_ascii_digit())
    })
}

/// `AAAA-MM-DD` con un día que existe.
fn es_fecha(f: &str) -> bool {
    let partes: Vec<&str> = f.split('-').collect();
    let [a, m, d] = partes.as_slice() else {
        return false;
    };
    if a.len() != 4 || m.len() != 2 || d.len() != 2 {
        return false;
    }
    let (Ok(a), Ok(m), Ok(d)) = (a.parse::<u32>(), m.parse::<u32>(), d.parse::<u32>()) else {
        return false;
    };
    let bisiesto = (a % 4 == 0 && a % 100 != 0) || a % 400 == 0;
    let dias = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if bisiesto => 29,
        2 => 28,
        _ => return false,
    };
    (1..=dias).contains(&d)
}

/// Las claves de valor que alguien podría escribir por error, y la pista.
const CLAVES_DE_VALOR: [&str; 8] = [
    "valor",
    "valor_m",
    "valor_mm",
    "valor_in",
    "valor_cm",
    "valor_kg",
    "valor_lb",
    "valor_grados",
];

fn revisar_campo(
    c: Campo,
    tabla: Option<&(usize, Tabla)>,
    errores: &mut Vec<Problema>,
) -> Option<Dato> {
    let nombre = c.nombre();
    let Some((linea_tabla, t)) = tabla else {
        errores.push(problema(
            nombre,
            None,
            format!("falta la tabla [{nombre}]. Si no la mediste, ponla con fuente = \"estimado\""),
        ));
        return None;
    };
    let clave_valor = c.unidad().clave();
    let mut ok = true;
    for (clave, e) in t {
        if ["fuente", "fecha", "como"].contains(&clave.as_str()) || clave == clave_valor {
            continue;
        }
        ok = false;
        let pista = if CLAVES_DE_VALOR.contains(&clave.as_str()) {
            format!(
                "este campo va en {}: escribe {clave_valor} (1.75 m son 175.0 cm)",
                c.unidad().simbolo()
            )
        } else {
            format!("clave desconocida; en [{nombre}] van fuente, {clave_valor}, fecha y como")
        };
        errores.push(problema(nombre, Some(e.linea), pista));
    }
    let fuente = texto(t, "fuente");
    let solo_medido = matches!(c, Campo::Estatura | Campo::Masa);
    match fuente {
        Some(("medido", _)) => {
            let valor = match t.get(clave_valor) {
                Some(Entrada {
                    valor: Valor::Numero(x),
                    ..
                }) => Some(*x),
                Some(e) => {
                    errores.push(problema(
                        nombre,
                        Some(e.linea),
                        format!("{clave_valor} debe ser un número"),
                    ));
                    None
                }
                None => {
                    errores.push(problema(
                        nombre,
                        Some(*linea_tabla),
                        format!("medido, pero falta {clave_valor}"),
                    ));
                    None
                }
            };
            let fecha = match texto(t, "fecha") {
                Some((f, _)) if es_fecha(f) => Some(f.to_string()),
                Some((f, l)) => {
                    errores.push(problema(
                        nombre,
                        Some(l),
                        format!("«{f}» no es una fecha AAAA-MM-DD"),
                    ));
                    None
                }
                None => {
                    errores.push(problema(
                        nombre,
                        Some(*linea_tabla),
                        "medido, pero falta la fecha (AAAA-MM-DD)",
                    ));
                    None
                }
            };
            let como = match texto(t, "como") {
                Some((s, _)) if !s.trim().is_empty() => Some(s.trim().to_string()),
                _ => {
                    errores.push(problema(
                        nombre,
                        Some(*linea_tabla),
                        "medido, pero falta como = \"…\" (instrumento y postura)",
                    ));
                    None
                }
            };
            match (valor, fecha, como, ok) {
                (Some(valor), Some(fecha), Some(como), true) => {
                    Some(Dato::Medido(Medicion { valor, fecha, como }))
                }
                _ => None,
            }
        }
        Some(("estimado", l)) if solo_medido => {
            errores.push(problema(
                nombre,
                Some(l),
                "este campo no se puede estimar: todo lo demás se estima a partir de él. Mídelo",
            ));
            None
        }
        Some(("estimado", _)) => {
            if let Some(e) = t.get(clave_valor) {
                errores.push(problema(
                    nombre,
                    Some(e.linea),
                    format!(
                        "estimado no lleva {clave_valor}: si tienes el número, es medido \
                         (con fecha y como); si no, borra el valor"
                    ),
                ));
                return None;
            }
            ok.then(|| Dato::Estimado {
                nota: texto(t, "como").map(|(s, _)| s.to_string()),
            })
        }
        Some((otra, l)) => {
            errores.push(problema(
                nombre,
                Some(l),
                format!("fuente «{otra}» no existe: es \"medido\" o \"estimado\""),
            ));
            None
        }
        None => {
            errores.push(problema(
                nombre,
                Some(*linea_tabla),
                "falta fuente = \"medido\" o \"estimado\"",
            ));
            None
        }
    }
}

/// Una banda alrededor de la proporción de Winter: (media, típico ±, posible ±).
///
/// **Son convenciones, no rangos poblacionales citados.** Lo "típico" es
/// lo que no extraña; lo "posible" es lo que todavía puede ser un cuerpo
/// y no un error de cinta (centímetros por milímetros, otra referencia).
fn banda(c: Campo) -> Option<(f64, f64, f64)> {
    let media = c.fraccion_de_estatura()?;
    let (tipico, posible) = match c {
        Campo::AlturaHombro | Campo::AlturaTrocanter => (0.030, 0.060),
        Campo::AlturaRodilla => (0.025, 0.050),
        _ => (0.012, 0.025),
    };
    Some((media, tipico, posible))
}

/// Rangos absolutos: (típico, posible). Convenciones, como [`banda`].
fn rango(c: Campo) -> Option<((f64, f64), (f64, f64))> {
    match c {
        Campo::Estatura => Some(((145.0, 205.0), (120.0, 230.0))),
        Campo::Masa => Some(((40.0, 180.0), (30.0, 300.0))),
        Campo::AnguloTibia => Some(((10.0, 45.0), (0.0, 60.0))),
        _ => None,
    }
}

/// La barra respecto del hombro, cm: (típico, posible). Barra alta va sobre
/// el trapecio, un poco arriba del acromion; barra baja, unos centímetros
/// abajo. Convención.
const BARRA_SOBRE_HOMBRO: ((f64, f64), (f64, f64)) = ((-12.0, 10.0), (-20.0, 15.0));

/// Fémur entre tibia, medidos como aquí (trocánter → interlínea y
/// interlínea → maléolo): Winter da 0.245/0.246 ≈ 1. (típico, posible).
/// Convención.
const FEMUR_ENTRE_TIBIA: ((f64, f64), (f64, f64)) = ((0.85, 1.20), (0.70, 1.40));

fn linea_de(doc: &Documento, c: Campo) -> Option<usize> {
    let (linea, t) = doc.tablas.get(c.nombre())?;
    Some(t.get(c.unidad().clave()).map_or(*linea, |e| e.linea))
}

fn fuera(x: f64, (lo, hi): (f64, f64)) -> bool {
    !(lo..=hi).contains(&x)
}

fn revisar_rangos(
    m: &Medidas,
    doc: &Documento,
    errores: &mut Vec<Problema>,
    avisos: &mut Vec<Problema>,
) {
    let h = m.estatura_cm();
    let mut juzga = |c: Campo, posible: bool, tipico: bool, que: String| {
        let p = problema(c.nombre(), linea_de(doc, c), que);
        if !posible {
            errores.push(p);
        } else if !tipico {
            avisos.push(p);
        }
    };
    for c in Campo::TODOS {
        let Dato::Medido(med) = m.dato(c) else {
            continue;
        };
        let x = med.valor;
        let u = c.unidad().simbolo();
        if let Some((tipico, posible)) = rango(c) {
            juzga(
                c,
                !fuera(x, posible),
                !fuera(x, tipico),
                format!(
                    "{x} {u} está fuera de {}–{} {u}{}",
                    if fuera(x, posible) {
                        posible.0
                    } else {
                        tipico.0
                    },
                    if fuera(x, posible) {
                        posible.1
                    } else {
                        tipico.1
                    },
                    if fuera(x, posible) {
                        ": revisa la unidad y vuelve a medir"
                    } else {
                        " (lo típico): vuelve a medir para confirmar"
                    },
                ),
            );
        }
        if let Some((media, tipico, posible)) = banda(c) {
            let f = x / h;
            let d = (f - media).abs();
            juzga(
                c,
                d <= posible,
                d <= tipico,
                format!(
                    "{x} cm es {:.3} de la estatura; Winter da {media:.3} (±{:.3} es lo típico, \
                     ±{:.3} lo posible). Revisa la referencia y la cinta",
                    f, tipico, posible
                ),
            );
        }
    }
    if let Dato::Medido(b) = m.dato(Campo::AlturaBarra) {
        let d = b.valor - m.valor(Campo::AlturaHombro);
        let (tipico, posible) = BARRA_SOBRE_HOMBRO;
        juzga(
            Campo::AlturaBarra,
            !fuera(d, posible),
            !fuera(d, tipico),
            format!(
                "la barra queda {d:+.1} cm respecto del hombro; lo típico es de {:+} a {:+} cm",
                tipico.0, tipico.1
            ),
        );
    }
    let orden = [
        Campo::Estatura,
        Campo::AlturaHombro,
        Campo::AlturaTrocanter,
        Campo::AlturaRodilla,
        Campo::AlturaMaleolo,
    ];
    for par in orden.windows(2) {
        let (arriba, abajo) = (par[0], par[1]);
        if m.valor(arriba) <= m.valor(abajo) {
            errores.push(problema(
                abajo.nombre(),
                linea_de(doc, abajo),
                format!(
                    "{} ({:.1} cm) no queda abajo de {} ({:.1} cm): las referencias van en orden \
                     de la cabeza al piso",
                    abajo.nombre(),
                    m.valor(abajo),
                    arriba.nombre(),
                    m.valor(arriba)
                ),
            ));
        }
    }
    if m.valor(Campo::AlturaMaleolo) <= 0.0 {
        errores.push(problema(
            Campo::AlturaMaleolo.nombre(),
            linea_de(doc, Campo::AlturaMaleolo),
            "el maléolo tiene que quedar arriba del piso",
        ));
    }
    if m.valor(Campo::AlturaBarra) <= m.valor(Campo::AlturaTrocanter) {
        errores.push(problema(
            Campo::AlturaBarra.nombre(),
            linea_de(doc, Campo::AlturaBarra),
            "la barra no queda arriba del trocánter",
        ));
    }
    // La razón solo se juzga si algo de ella se midió (la de Winter es
    // 1.00 por construcción) y si los dos largos son positivos (si no, el
    // orden ya es el error).
    let involucra = [
        Campo::AlturaTrocanter,
        Campo::AlturaRodilla,
        Campo::AlturaMaleolo,
    ];
    let e = m.entradas();
    if involucra.iter().any(|&c| m.dato(c).es_medido()) && e.femur_m > 0.0 && e.tibia_m > 0.0 {
        let r = e.femur_m / e.tibia_m;
        let (tipico, posible) = FEMUR_ENTRE_TIBIA;
        let p = problema(
            "femur/tibia",
            None,
            format!(
                "fémur {:.1} cm entre tibia {:.1} cm da {r:.2}; lo típico va de {} a {} \
                 (Winter: 1.00). Revisa trocánter, rodilla y maléolo",
                e.femur_m * 100.0,
                e.tibia_m * 100.0,
                tipico.0,
                tipico.1
            ),
        );
        if fuera(r, posible) {
            errores.push(p);
        } else if fuera(r, tipico) {
            avisos.push(p);
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::maquina_humana::Persona;

    /// Un archivo con todo estimado salvo estatura y masa.
    pub(crate) fn todo_estimado(estatura_cm: f64) -> String {
        let mut s = format!(
            "formato = \"{FORMATO}\"\npersona = \"P-T35T\"\nfecha = \"2026-10-18\"\n\n\
             [estatura]\nfuente = \"medido\"\nvalor_cm = {estatura_cm:?}\nfecha = \"2026-10-18\"\n\
             como = \"cinta\"\n\n[masa]\nfuente = \"medido\"\nvalor_kg = 80.0\n\
             fecha = \"2026-10-18\"\ncomo = \"báscula\"\n"
        );
        for c in &Campo::TODOS[2..] {
            s.push_str(&format!("\n[{}]\nfuente = \"estimado\"\n", c.nombre()));
        }
        s
    }

    /// Cambia una tabla entera del archivo por `nuevo`.
    pub(crate) fn con(texto: &str, campo: Campo, nuevo: &str) -> String {
        let encabezado = format!("[{}]", campo.nombre());
        let mut fuera_de_tabla = true;
        let mut s = String::new();
        for l in texto.lines() {
            if l.trim_start().starts_with('[') {
                fuera_de_tabla = l.trim() != encabezado;
                if !fuera_de_tabla {
                    s.push_str(&format!("{encabezado}\n{nuevo}\n"));
                    continue;
                }
            }
            if fuera_de_tabla {
                s.push_str(l);
                s.push('\n');
            }
        }
        s
    }

    pub(crate) fn medido(clave: &str, valor: f64) -> String {
        format!(
            "fuente = \"medido\"\n{clave} = {valor:?}\nfecha = \"2026-10-18\"\ncomo = \"cinta\""
        )
    }

    fn errores(texto: &str) -> Vec<Problema> {
        revisar(texto).expect_err("debía fallar")
    }

    fn donde(ps: &[Problema]) -> Vec<&str> {
        ps.iter().map(|p| p.donde.as_str()).collect()
    }

    #[test]
    fn todo_estimado_pasa_sin_avisos() {
        let r = revisar(&todo_estimado(175.0)).unwrap();
        assert!(r.avisos.is_empty(), "{:?}", r.avisos);
        assert_eq!(r.medidas.persona, "P-T35T");
    }

    /// Sin medidas, los segmentos son los de `Persona`, bit por bit.
    #[test]
    fn todo_estimado_da_los_segmentos_de_winter_exactos() {
        for h in [158.0, 168.0, 175.0, 183.5, 196.0] {
            let m = revisar(&todo_estimado(h)).unwrap().medidas;
            let p = Persona {
                estatura_m: h / 100.0,
                masa_kg: 80.0,
            };
            let e = m.entradas();
            assert_eq!(e.femur_m, p.femur());
            assert_eq!(e.tibia_m, p.tibia());
            assert_eq!(e.torso_m, p.torso());
            assert_eq!(e.hombro_m, p.torso(), "barra estimada: en el hombro");
            assert_eq!(e.tibia_rad, ANGULO_TIBIA_CONVENCION.to_radians());
        }
    }

    #[test]
    fn un_segmento_medido_es_la_resta_de_sus_alturas() {
        let t = con(
            &todo_estimado(175.0),
            Campo::AlturaTrocanter,
            &medido("valor_cm", 92.0),
        );
        let t = con(&t, Campo::AlturaRodilla, &medido("valor_cm", 50.0));
        let e = revisar(&t).unwrap().medidas.entradas();
        assert!((e.femur_m - 0.42).abs() < 1e-12);
        let maleolo = largo::TOBILLO * 175.0;
        assert!(
            (e.tibia_m - (50.0 - maleolo) / 100.0).abs() < 1e-12,
            "mezcla: medido menos estimado"
        );
    }

    #[test]
    fn la_barra_medida_alarga_el_tronco_del_modelo_y_no_el_hombro() {
        let t = con(
            &todo_estimado(175.0),
            Campo::AlturaBarra,
            &medido("valor_cm", 146.0),
        );
        let m = revisar(&t).unwrap().medidas;
        let e = m.entradas();
        let troc = m.valor(Campo::AlturaTrocanter);
        assert!((e.torso_m - (146.0 - troc) / 100.0).abs() < 1e-12);
        assert_eq!(e.hombro_m, 0.288 * 1.75);
    }

    #[test]
    fn falta_una_tabla() {
        let t = todo_estimado(175.0).replace("[altura_rodilla]\nfuente = \"estimado\"\n", "");
        assert_eq!(donde(&errores(&t)), ["altura_rodilla"]);
    }

    #[test]
    fn una_clave_desconocida_es_error_y_un_nombre_no_cabe() {
        let t = todo_estimado(175.0).replace(
            "fecha = \"2026-10-18\"\n\n[estatura]",
            "fecha = \"2026-10-18\"\nnombre = \"Ana\"\n\n[estatura]",
        );
        assert_eq!(donde(&errores(&t)), ["nombre"]);
        let t = format!("{}\n[foto]\narchivo = \"cara.jpg\"\n", todo_estimado(175.0));
        assert_eq!(donde(&errores(&t)), ["foto"]);
    }

    #[test]
    fn metros_donde_van_centimetros_da_la_pista() {
        let t = con(
            &todo_estimado(175.0),
            Campo::AlturaRodilla,
            &medido("valor_m", 0.5),
        );
        let e = errores(&t);
        assert_eq!(donde(&e), ["altura_rodilla", "altura_rodilla"], "{e:?}");
        assert!(e[0].motivo.contains("valor_cm"), "{}", e[0].motivo);
    }

    #[test]
    fn medido_pide_valor_fecha_y_como() {
        let t = con(
            &todo_estimado(175.0),
            Campo::AlturaRodilla,
            "fuente = \"medido\"",
        );
        let e = errores(&t);
        assert_eq!(e.len(), 3, "{e:?}");
        assert!(donde(&e).iter().all(|d| *d == "altura_rodilla"));
    }

    #[test]
    fn estimado_no_lleva_valor() {
        let t = con(
            &todo_estimado(175.0),
            Campo::AlturaRodilla,
            "fuente = \"estimado\"\nvalor_cm = 49.0",
        );
        assert_eq!(donde(&errores(&t)), ["altura_rodilla"]);
    }

    #[test]
    fn estimado_puede_llevar_una_nota() {
        let t = con(
            &todo_estimado(175.0),
            Campo::AlturaRodilla,
            "fuente = \"estimado\"\ncomo = \"no encontré la interlínea\"",
        );
        let m = revisar(&t).unwrap().medidas;
        assert_eq!(
            m.dato(Campo::AlturaRodilla),
            &Dato::Estimado {
                nota: Some("no encontré la interlínea".into())
            }
        );
    }

    #[test]
    fn estatura_y_masa_no_se_estiman() {
        let t = con(&todo_estimado(175.0), Campo::Masa, "fuente = \"estimado\"");
        assert_eq!(donde(&errores(&t)), ["masa"]);
    }

    #[test]
    fn fuente_desconocida() {
        let t = con(
            &todo_estimado(175.0),
            Campo::AlturaRodilla,
            "fuente = \"masomenos\"",
        );
        assert_eq!(donde(&errores(&t)), ["altura_rodilla"]);
    }

    #[test]
    fn el_seudonimo_no_puede_ser_un_nombre() {
        for p in [
            "Gustavo",
            "P-GUSTAVO",
            "P-12",
            "P-abc1",
            "P-1234567890123",
            "P-7K 3Q",
        ] {
            let t = todo_estimado(175.0).replace("P-T35T", p);
            assert_eq!(donde(&errores(&t)), ["persona"], "{p}");
        }
        for p in ["P-7K3Q", "P-0001", "P-EJEMPLO1"] {
            assert!(es_seudonimo(p), "{p}");
        }
    }

    #[test]
    fn fechas() {
        for f in ["2026-10-18", "2024-02-29", "2000-02-29"] {
            assert!(es_fecha(f), "{f}");
        }
        for f in [
            "2026-13-01",
            "2026-02-29",
            "1900-02-29",
            "2026-4-01",
            "18/10/2026",
            "AAAA-MM-DD",
            "2026-10-32",
        ] {
            assert!(!es_fecha(f), "{f}");
        }
    }

    #[test]
    fn otro_formato_se_rechaza() {
        let t = todo_estimado(175.0).replace(FORMATO, "medidas-sentadilla/2");
        assert_eq!(donde(&errores(&t)), ["formato"]);
    }

    #[test]
    fn junta_todos_los_errores_de_una_vez() {
        let t = con(&todo_estimado(175.0), Campo::Masa, "fuente = \"estimado\"");
        let t = con(&t, Campo::AlturaRodilla, "fuente = \"medido\"");
        let t = t.replace("P-T35T", "Ana");
        assert_eq!(errores(&t).len(), 5, "persona + masa + rodilla×3");
    }

    #[test]
    fn estatura_imposible_es_error_y_rara_es_aviso() {
        let t = con(
            &todo_estimado(175.0),
            Campo::Estatura,
            &medido("valor_cm", 1.75),
        );
        assert_eq!(donde(&errores(&t)), ["estatura"], "metros escritos como cm");
        let r = revisar(&todo_estimado(210.0)).unwrap();
        assert_eq!(donde(&r.avisos), ["estatura"]);
    }

    #[test]
    fn una_referencia_lejos_de_winter() {
        // Rodilla a 0.285·175 = 49.9 cm. A 55 cm (0.314) es rara; a 62 (0.354), imposible.
        let t = con(
            &todo_estimado(175.0),
            Campo::AlturaRodilla,
            &medido("valor_cm", 55.0),
        );
        let r = revisar(&t).unwrap();
        assert!(
            donde(&r.avisos).contains(&"altura_rodilla"),
            "{:?}",
            r.avisos
        );
        let t = con(
            &todo_estimado(175.0),
            Campo::AlturaRodilla,
            &medido("valor_cm", 62.0),
        );
        assert!(donde(&errores(&t)).contains(&"altura_rodilla"));
    }

    #[test]
    fn referencias_en_desorden() {
        // Trocánter medido abajo de la rodilla medida.
        let t = con(
            &todo_estimado(175.0),
            Campo::AlturaTrocanter,
            &medido("valor_cm", 88.0),
        );
        let t = con(&t, Campo::AlturaRodilla, &medido("valor_cm", 90.0));
        let e = errores(&t);
        assert!(e.iter().any(|p| p.motivo.contains("en orden")), "{e:?}");
    }

    #[test]
    fn femur_entre_tibia_fuera_de_lo_tipico_y_de_lo_posible() {
        // Estatura 175: maléolo estimado 6.825 cm. Rodilla 48 → tibia 41.2.
        // Trocánter 95 → fémur 47 → 1.14: típico. 99 → 51 → 1.24: aviso.
        let base = con(
            &todo_estimado(175.0),
            Campo::AlturaRodilla,
            &medido("valor_cm", 48.0),
        );
        let r = revisar(&con(
            &base,
            Campo::AlturaTrocanter,
            &medido("valor_cm", 95.0),
        ))
        .unwrap();
        assert!(!donde(&r.avisos).contains(&"femur/tibia"), "{:?}", r.avisos);
        let r = revisar(&con(
            &base,
            Campo::AlturaTrocanter,
            &medido("valor_cm", 99.0),
        ))
        .unwrap();
        assert!(donde(&r.avisos).contains(&"femur/tibia"), "{:?}", r.avisos);
        // Rodilla 40, maléolo 7 → tibia 33; trocánter 87 → fémur 47 → 1.42: imposible.
        let t = con(
            &todo_estimado(160.0),
            Campo::AlturaRodilla,
            &medido("valor_cm", 40.0),
        );
        let t = con(&t, Campo::AlturaMaleolo, &medido("valor_cm", 7.0));
        let t = con(&t, Campo::AlturaTrocanter, &medido("valor_cm", 87.0));
        assert!(donde(&errores(&t)).contains(&"femur/tibia"));
    }

    #[test]
    fn la_barra_lejos_del_hombro() {
        let hombro = 0.818 * 175.0;
        let t = con(
            &todo_estimado(175.0),
            Campo::AlturaBarra,
            &medido("valor_cm", hombro + 12.0),
        );
        assert_eq!(donde(&revisar(&t).unwrap().avisos), ["altura_barra"]);
        let t = con(
            &todo_estimado(175.0),
            Campo::AlturaBarra,
            &medido("valor_cm", hombro - 30.0),
        );
        assert_eq!(donde(&errores(&t)), ["altura_barra"]);
    }

    #[test]
    fn angulo_de_tibia() {
        let t = con(
            &todo_estimado(175.0),
            Campo::AnguloTibia,
            &medido("valor_grados", 50.0),
        );
        assert_eq!(donde(&revisar(&t).unwrap().avisos), ["angulo_tibia"]);
        let t = con(
            &todo_estimado(175.0),
            Campo::AnguloTibia,
            &medido("valor_grados", 75.0),
        );
        assert_eq!(donde(&errores(&t)), ["angulo_tibia"]);
    }

    #[test]
    fn un_error_de_lectura_dice_la_linea() {
        let e = errores("formato = \"medidas-sentadilla/1\"\npersona = P-1234\n");
        assert_eq!(e[0].linea, Some(2));
        assert_eq!(e[0].donde, "archivo");
    }
}
