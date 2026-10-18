//! El resultado de unas medidas: la postura resuelta, los brazos de
//! momento y los momentos de la barra en cadera y rodilla, cuánto pesa
//! cada medida en ellos, y su forma en JSON para la app.
//!
//! # Los momentos, con el signo de la app
//!
//! El momento se reporta como el que **resiste la extensión**: positivo
//! quiere decir que los extensores trabajan. En la cadera es
//! `m g (x_barra − x_cadera)` y en la rodilla `m g (x_rodilla − x_barra)`:
//! el mismo número que [`Sentadilla::tau_hip`](crate::sentadilla::Sentadilla::tau_hip)
//! y `tau_knee` cuando la barra queda delante de la cadera y detrás de la
//! rodilla, que es el caso normal, pero con signo cuando no. Es la regla de
//! la especificación del modelo de la app (un solo lugar decide el signo).
//!
//! # Cuánto decide cada medida
//!
//! Cada campo se mueve medio centímetro (o medio grado) hacia cada lado y
//! se vuelve a resolver: eso da cuánto cambia cada resultado por unidad. Por
//! el margen de ese campo (abajo) da su **aporte**, y la suma de aportes es
//! una cota de la incertidumbre, sin suponer ninguna distribución. Los
//! márgenes son **convenciones**, no errores medidos:
//!
//! | campo | medido | estimado |
//! |---|---|---|
//! | estatura | ±0.5 cm | — |
//! | alturas y pie | ±1 cm | ±2.5 cm |
//! | barra | ±1 cm | ±5 cm (en el hombro por convención) |
//! | ángulo de la tibia | ±3° | ±10° (22° por convención) |

use std::fmt::{self, Write as _};

use super::{sentadilla_de, Ajuste, Campo, Dato, Entradas, Medidas, Problema, Revisadas};
use crate::sentadilla::Postura;
use crate::G;

/// La versión de la salida.
pub const FORMATO: &str = "mecanica-sentadilla/1";

/// Medio paso de la diferencia central, en la unidad de cada campo.
const PASO: f64 = 0.5;

/// Por qué unas medidas que pasaron la revisión no dan una postura.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Rechazo {
    /// La carga no es un número positivo.
    CargaNoValida {
        /// Lo que se pidió.
        kg: f64,
    },
    /// El tronco no alcanza a poner la barra sobre el medio pie: la cadera
    /// queda más lejos de esa línea que el largo del tronco.
    NoCierra {
        /// Lo que tendría que valer el seno del ángulo del tronco.
        seno: f64,
    },
    /// El tronco tendría que inclinarse hacia atrás para cerrar.
    TroncoHaciaAtras {
        /// El ángulo, negativo.
        grados: f64,
    },
}

impl fmt::Display for Rechazo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Rechazo::CargaNoValida { kg } => {
                write!(f, "la carga debe ser un número positivo de kg, no {kg}")
            }
            Rechazo::NoCierra { seno } => write!(
                f,
                "la postura no cierra: el tronco tendría que inclinarse con seno {seno:.2}, \
                 y no pasa de 0.99. Revisa fémur, tibia y tronco"
            ),
            Rechazo::TroncoHaciaAtras { grados } => write!(
                f,
                "la postura no cierra: el tronco tendría que ir {:.1}° hacia atrás. \
                 Revisa la tibia y su ángulo",
                -grados
            ),
        }
    }
}

/// Un supuesto del resultado. Va en cada salida, no en una nota al pie.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Supuesto {
    /// Dos dimensiones, plano sagital.
    PlanoSagital,
    /// Sin aceleraciones.
    CuasiEstatico,
    /// Los segmentos no se doblan.
    SegmentosRigidos,
    /// Una sola postura: el fondo.
    UnaPosturaElFondo,
    /// La postura se resuelve, no se mide.
    PosturaResuelta,
    /// Muslo paralelo al piso.
    MusloParalelo,
    /// La barra sobre el medio pie.
    BarraSobreMedioPie,
    /// El tobillo es un punto fijo del modelo.
    TobilloFijo,
    /// Solo el peso de la barra; el del cuerpo no entra.
    SoloLaBarra,
    /// La barra a la altura del hombro, porque no se midió.
    BarraEnElHombro,
    /// Algún largo que usa el modelo salió de la tabla de Winter.
    LargosDeTabla,
    /// El ángulo de la tibia es la convención de 22°.
    AnguloTibiaConvencion,
}

impl Supuesto {
    /// El código en la salida JSON. Donde existe, es el nombre que usa la
    /// especificación del modelo de la app.
    #[must_use]
    pub fn codigo(self) -> &'static str {
        match self {
            Supuesto::PlanoSagital => "sagittal_plane",
            Supuesto::CuasiEstatico => "quasi_static",
            Supuesto::SegmentosRigidos => "rigid_segments",
            Supuesto::UnaPosturaElFondo => "single_posture_bottom",
            Supuesto::PosturaResuelta => "solved_baseline",
            Supuesto::MusloParalelo => "thigh_parallel",
            Supuesto::BarraSobreMedioPie => "bar_over_midfoot",
            Supuesto::TobilloFijo => "fixed_ankle",
            Supuesto::SoloLaBarra => "bar_only",
            Supuesto::BarraEnElHombro => "bar_at_shoulder",
            Supuesto::LargosDeTabla => "table_derived_lengths",
            Supuesto::AnguloTibiaConvencion => "conventional_shank_angle",
        }
    }

    /// En palabras.
    #[must_use]
    pub fn texto(self) -> &'static str {
        match self {
            Supuesto::PlanoSagital => "plano sagital (2D): de lado, sin ancho de pies ni giro",
            Supuesto::CuasiEstatico => "cuasi-estático: sin aceleraciones",
            Supuesto::SegmentosRigidos => "segmentos rígidos",
            Supuesto::UnaPosturaElFondo => "una sola postura, el fondo; ni la bajada ni la subida",
            Supuesto::PosturaResuelta => {
                "la postura se resuelve con las medidas; no se mide en video"
            }
            Supuesto::MusloParalelo => "muslo paralelo al piso",
            Supuesto::BarraSobreMedioPie => "la barra sobre el medio pie",
            Supuesto::TobilloFijo => {
                "el tobillo es un punto fijo del modelo, a 4 cm detrás del medio pie: \
                 el largo del pie no entra"
            }
            Supuesto::SoloLaBarra => {
                "solo la barra: el peso del cuerpo (tronco, cabeza, brazos) no entra, \
                 y con él los momentos son más altos"
            }
            Supuesto::BarraEnElHombro => "la barra a la altura del hombro (no se midió)",
            Supuesto::LargosDeTabla => "algún largo es estimado (proporciones de Winter)",
            Supuesto::AnguloTibiaConvencion => {
                "tibia a 22° de la vertical: convención del modelo, sin fuente"
            }
        }
    }
}

/// Cuánto mueve un campo el resultado.
#[derive(Clone, Debug, PartialEq)]
pub struct Huella {
    /// El campo.
    pub campo: Campo,
    /// Cambio por unidad del campo: brazo de cadera (m), brazo de rodilla
    /// (m) y ángulo del tronco (grados).
    pub por_unidad: [f64; 3],
    /// Su margen, en la unidad del campo (convención; ver el módulo).
    pub margen: f64,
}

impl Huella {
    /// `true` si mueve algún resultado.
    #[must_use]
    pub fn usado(&self) -> bool {
        self.por_unidad.iter().any(|x| x.abs() > 1e-9)
    }

    /// Lo que su margen mueve cada resultado.
    #[must_use]
    pub fn aporte(&self) -> [f64; 3] {
        self.por_unidad.map(|x| (x * self.margen).abs())
    }
}

/// Unas medidas resueltas.
#[derive(Clone, Debug, PartialEq)]
pub struct Informe {
    /// Las medidas de entrada.
    pub medidas: Medidas,
    /// La carga, si se dio. Sin carga hay postura y brazos, pero no momentos:
    /// el modelo no supone un peso.
    pub carga_kg: Option<f64>,
    /// Lo que entró al modelo.
    pub entradas: Entradas,
    /// La postura que resolvió el modelo.
    pub postura: Postura,
    /// El hombro, sobre la línea del tronco.
    pub hombro: (f64, f64),
    /// Brazos de momento que resisten la extensión: cadera, rodilla (m).
    pub brazos_m: [f64; 2],
    /// Cada campo, en el orden del archivo.
    pub huellas: Vec<Huella>,
    /// Los avisos de la revisión.
    pub avisos: Vec<Problema>,
    /// Los supuestos de este resultado.
    pub supuestos: Vec<Supuesto>,
}

/// Resuelve la sentadilla de unas medidas revisadas.
///
/// # Errors
/// [`Rechazo`] si la carga no vale o si la postura no cierra. El modelo
/// por sí solo recorta el seno a ±0.99 y devolvería una postura que no
/// pone la barra donde dice; aquí eso es un rechazo, no un número.
pub fn resolver(r: &Revisadas, carga_kg: Option<f64>) -> Result<Informe, Rechazo> {
    if let Some(kg) = carga_kg {
        if !(kg.is_finite() && kg > 0.0) {
            return Err(Rechazo::CargaNoValida { kg });
        }
    }
    let m = &r.medidas;
    let entradas = m.entradas();
    let postura = sentadilla_de(&entradas, carga_kg.unwrap_or(0.0)).postura();
    if postura.bar.0.abs() > 1e-9 {
        return Err(Rechazo::NoCierra {
            seno: -postura.hip.0 / entradas.torso_m,
        });
    }
    if postura.torso_rad < 0.0 {
        return Err(Rechazo::TroncoHaciaAtras {
            grados: postura.torso_rad.to_degrees(),
        });
    }
    let (s, c) = postura.torso_rad.sin_cos();
    let hombro = (
        postura.hip.0 + entradas.hombro_m * s,
        postura.hip.1 + entradas.hombro_m * c,
    );
    let huellas = Campo::TODOS
        .into_iter()
        .map(|campo| {
            let f = |d: f64| resultados(m, Some((campo, d)));
            let (mas, menos) = (f(PASO), f(-PASO));
            Huella {
                campo,
                por_unidad: [0, 1, 2].map(|i| (mas[i] - menos[i]) / (2.0 * PASO)),
                margen: margen(campo, m.dato(campo)),
            }
        })
        .collect();
    let [cadera, rodilla, _] = resultados(m, None);
    Ok(Informe {
        medidas: m.clone(),
        carga_kg,
        entradas,
        postura,
        hombro,
        brazos_m: [cadera, rodilla],
        huellas,
        avisos: r.avisos.clone(),
        supuestos: supuestos(m),
    })
}

/// Brazo de cadera, brazo de rodilla (m, resistiendo extensión) y ángulo
/// del tronco (grados), con un campo movido.
fn resultados(m: &Medidas, aj: Ajuste) -> [f64; 3] {
    let p = sentadilla_de(&m.entradas_con(aj), 0.0).postura();
    [
        p.bar.0 - p.hip.0,
        p.knee.0 - p.bar.0,
        p.torso_rad.to_degrees(),
    ]
}

/// El margen de un campo, en su unidad. Convenciones (ver el módulo).
fn margen(c: Campo, d: &Dato) -> f64 {
    match (c, d.es_medido()) {
        (Campo::Masa, _) => 0.0,
        (Campo::Estatura, _) => 0.5,
        (Campo::AnguloTibia, true) => 3.0,
        (Campo::AnguloTibia, false) => 10.0,
        (Campo::AlturaBarra, false) => 5.0,
        (_, true) => 1.0,
        (_, false) => 2.5,
    }
}

fn supuestos(m: &Medidas) -> Vec<Supuesto> {
    use Supuesto::*;
    let mut v = vec![
        PlanoSagital,
        CuasiEstatico,
        SegmentosRigidos,
        UnaPosturaElFondo,
        PosturaResuelta,
        MusloParalelo,
        BarraSobreMedioPie,
        TobilloFijo,
        SoloLaBarra,
    ];
    if !m.dato(Campo::AlturaBarra).es_medido() {
        v.push(BarraEnElHombro);
    }
    let mut largos = m.depende_de(Campo::AlturaBarra);
    largos.extend([
        Campo::AlturaTrocanter,
        Campo::AlturaRodilla,
        Campo::AlturaMaleolo,
    ]);
    if largos.iter().any(|&c| !m.dato(c).es_medido()) {
        v.push(LargosDeTabla);
    }
    if !m.dato(Campo::AnguloTibia).es_medido() {
        v.push(AnguloTibiaConvencion);
    }
    v
}

impl Informe {
    /// Momentos de la barra que resisten la extensión, cadera y rodilla
    /// (N·m). `None` sin carga.
    #[must_use]
    pub fn momentos_nm(&self) -> Option<[f64; 2]> {
        self.carga_kg.map(|kg| self.brazos_m.map(|d| kg * G * d))
    }

    /// Ángulo del tronco desde la vertical, grados.
    #[must_use]
    pub fn tronco_grados(&self) -> f64 {
        self.postura.torso_rad.to_degrees()
    }

    /// La cota de incertidumbre de cada resultado: brazo de cadera (m),
    /// brazo de rodilla (m) y tronco (grados). Suma de aportes.
    #[must_use]
    pub fn incertidumbre(&self) -> [f64; 3] {
        self.huellas.iter().fold([0.0; 3], |acc, h| {
            let a = h.aporte();
            [acc[0] + a[0], acc[1] + a[1], acc[2] + a[2]]
        })
    }

    /// Los campos que mueven el resultado, del que más aporta a los
    /// momentos al que menos (el tronco desempata).
    #[must_use]
    pub fn decisivas(&self) -> Vec<&Huella> {
        let mut v: Vec<&Huella> = self.huellas.iter().filter(|h| h.usado()).collect();
        let peso = |h: &Huella| {
            let a = h.aporte();
            (a[0] + a[1], a[2])
        };
        v.sort_by(|a, b| {
            peso(b)
                .partial_cmp(&peso(a))
                .expect("los aportes son finitos")
        });
        v
    }

    /// Lo que la app necesita como entrada: estatura, masa, carga y los
    /// largos **medidos** (los estimados no se mandan: la app los estima
    /// con su propia tabla).
    fn entrada_app(&self) -> J {
        let m = &self.medidas;
        let medido = |cs: &[Campo]| cs.iter().all(|&c| m.dato(c).es_medido());
        let largo = |arriba: Campo, abajo: Campo| {
            medido(&[arriba, abajo]).then(|| (m.valor(arriba) - m.valor(abajo)) / 100.0)
        };
        use Campo::*;
        J::Obj(vec![
            ("height_cm", J::Num(m.estatura_cm())),
            ("weight_kg", J::Num(m.masa_kg())),
            ("load_kg", opcion(self.carga_kg)),
            (
                "measured_lengths",
                J::Obj(vec![
                    ("thigh_m", opcion(largo(AlturaTrocanter, AlturaRodilla))),
                    ("shank_m", opcion(largo(AlturaRodilla, AlturaMaleolo))),
                    ("trunk_m", opcion(largo(AlturaHombro, AlturaTrocanter))),
                    (
                        "foot_m",
                        opcion(medido(&[LargoPie]).then(|| m.valor(LargoPie) / 100.0)),
                    ),
                    (
                        "ankle_height_m",
                        opcion(medido(&[AlturaMaleolo]).then(|| m.valor(AlturaMaleolo) / 100.0)),
                    ),
                ]),
            ),
            (
                "bar_on_trunk",
                opcion(
                    medido(&[AlturaBarra, AlturaHombro, AlturaTrocanter]).then(|| {
                        (m.valor(AlturaBarra) - m.valor(AlturaTrocanter))
                            / (m.valor(AlturaHombro) - m.valor(AlturaTrocanter))
                    }),
                ),
            ),
            (
                "shank_angle_deg",
                opcion(medido(&[AnguloTibia]).then(|| m.valor(AnguloTibia))),
            ),
        ])
    }

    /// La salida en JSON, para la app y para los reels.
    #[must_use]
    pub fn json(&self) -> String {
        let p = &self.postura;
        let punto = |(x, y): (f64, f64)| J::Obj(vec![("x", J::Num(x)), ("y", J::Num(y))]);
        let inc = self.incertidumbre();
        let momentos = self.momentos_nm();
        let carga_g = self.carga_kg.map(|kg| kg * G);
        let articulacion = |i: usize, joint: &'static str, resisting: &'static str| {
            J::Obj(vec![
                ("joint", J::Str(joint.into())),
                ("resisting", J::Str(resisting.into())),
                ("moment_arm_m", J::Num(self.brazos_m[i])),
                ("moment_arm_uncertainty_m", J::Num(inc[i])),
                ("moment_nm", opcion(momentos.map(|t| t[i]))),
                ("uncertainty_nm", opcion(carga_g.map(|w| w * inc[i]))),
                ("component", J::Str("bar_only".into())),
            ])
        };
        let terna = |v: [f64; 3]| {
            J::Obj(vec![
                ("hip_arm_m", J::Num(v[0])),
                ("knee_arm_m", J::Num(v[1])),
                ("trunk_deg", J::Num(v[2])),
            ])
        };
        let medidas = self
            .huellas
            .iter()
            .map(|h| {
                let (fuente, fecha, como, nota) = match self.medidas.dato(h.campo) {
                    Dato::Medido(m) => (
                        "measured",
                        J::Str(m.fecha.clone()),
                        J::Str(m.como.clone()),
                        J::Null,
                    ),
                    Dato::Estimado { nota } => (
                        "estimated",
                        J::Null,
                        J::Str(como_se_estima(h.campo)),
                        nota.clone().map_or(J::Null, J::Str),
                    ),
                };
                J::Obj(vec![
                    ("field", J::Str(h.campo.nombre().into())),
                    ("unit", J::Str(h.campo.unidad().simbolo().into())),
                    ("value", J::Num(self.medidas.valor(h.campo))),
                    ("source", J::Str(fuente.into())),
                    ("on", fecha),
                    ("how", como),
                    ("note", nota),
                    ("used", J::Bool(h.usado())),
                    ("per_unit", terna(h.por_unidad)),
                    ("spread", J::Num(h.margen)),
                    ("contribution", terna(h.aporte())),
                ])
            })
            .collect();
        let raiz = J::Obj(vec![
            ("format", J::Str(FORMATO.into())),
            ("measurements_format", J::Str(super::FORMATO.into())),
            ("person", J::Str(self.medidas.persona.clone())),
            ("measured_on", J::Str(self.medidas.fecha.clone())),
            ("app_input", self.entrada_app()),
            (
                "model",
                J::Obj(vec![
                    ("name", J::Str("sentadilla".into())),
                    ("position", J::Str("bottom".into())),
                    (
                        "load",
                        match self.carga_kg {
                            Some(kg) => {
                                J::Obj(vec![("basis", J::Str("bar".into())), ("kg", J::Num(kg))])
                            }
                            None => J::Obj(vec![("basis", J::Str("none".into()))]),
                        },
                    ),
                    (
                        "posture",
                        J::Obj(vec![
                            ("ankle", punto(p.ankle)),
                            ("knee", punto(p.knee)),
                            ("hip", punto(p.hip)),
                            ("shoulder", punto(self.hombro)),
                            (
                                "load",
                                match self.carga_kg {
                                    Some(kg) => {
                                        J::Obj(vec![("kg", J::Num(kg)), ("at", punto(p.bar))])
                                    }
                                    None => J::Null,
                                },
                            ),
                        ]),
                    ),
                    ("bar_point", punto(p.bar)),
                    ("trunk_angle_deg", J::Num(self.tronco_grados())),
                    ("trunk_angle_uncertainty_deg", J::Num(inc[2])),
                    (
                        "shank_angle_deg",
                        J::Num(self.entradas.tibia_rad.to_degrees()),
                    ),
                    (
                        "joints",
                        J::Arr(vec![
                            articulacion(0, "hip", "hip_extensors"),
                            articulacion(1, "knee", "knee_extensors"),
                        ]),
                    ),
                    (
                        "assumptions",
                        J::Arr(
                            self.supuestos
                                .iter()
                                .map(|s| J::Str(s.codigo().into()))
                                .collect(),
                        ),
                    ),
                ]),
            ),
            ("measurements", J::Arr(medidas)),
            (
                "drivers",
                J::Arr(
                    self.decisivas()
                        .iter()
                        .map(|h| J::Str(h.campo.nombre().into()))
                        .collect(),
                ),
            ),
            (
                "warnings",
                J::Arr(self.avisos.iter().map(problema_json).collect()),
            ),
        ]);
        let mut s = String::new();
        raiz.escribir(&mut s, 0);
        s.push('\n');
        s
    }

    /// La salida en palabras, para leerla en la terminal.
    #[must_use]
    pub fn texto(&self) -> String {
        let mut s = String::new();
        let p = &self.postura;
        let inc = self.incertidumbre();
        let _ = writeln!(
            s,
            "Sentadilla en el fondo (muslo paralelo): {}, medidas del {}",
            self.medidas.persona, self.medidas.fecha
        );
        let _ = match self.carga_kg {
            Some(kg) => writeln!(
                s,
                "Carga: {kg} kg en la barra (solo la barra; el peso del cuerpo no entra)"
            ),
            None => writeln!(
                s,
                "Sin carga: hay postura y brazos de momento, no momentos (no se supone un peso)"
            ),
        };
        let pt = |(x, y): (f64, f64)| format!("({x:.3}, {y:.3})");
        let _ = writeln!(
            s,
            "\nPostura (m; x hacia la punta del pie, origen en el medio pie):"
        );
        let _ = writeln!(
            s,
            "  tobillo {}  rodilla {}  cadera {}  hombro {}  barra {}",
            pt(p.ankle),
            pt(p.knee),
            pt(p.hip),
            pt(self.hombro),
            pt(p.bar)
        );
        let _ = writeln!(
            s,
            "  tronco {:.1}° ± {:.1}° desde la vertical; tibia {:.1}°",
            self.tronco_grados(),
            inc[2],
            self.entradas.tibia_rad.to_degrees()
        );
        let _ = writeln!(
            s,
            "\nBrazos y momentos de la barra (positivo: trabajan los extensores):"
        );
        for (i, nombre) in ["cadera ", "rodilla"].iter().enumerate() {
            let momento = match self.momentos_nm() {
                Some(t) => format!(
                    "{:.0} N·m ± {:.0}",
                    t[i],
                    self.carga_kg.unwrap_or(0.0) * G * inc[i]
                ),
                None => "sin carga".to_string(),
            };
            let _ = writeln!(
                s,
                "  {nombre}  brazo {:.3} m ± {:.3}   {momento}",
                self.brazos_m[i], inc[i]
            );
        }
        let _ = writeln!(
            s,
            "\nQué decide el resultado (lo que mueve su margen; de más a menos):"
        );
        for h in self.decisivas() {
            let a = h.aporte();
            let fuente = if self.medidas.dato(h.campo).es_medido() {
                "medido  "
            } else {
                "estimado"
            };
            let _ = writeln!(
                s,
                "  {:<17} {fuente} ±{} {}: cadera ±{:.3} m, rodilla ±{:.3} m, tronco ±{:.1}°",
                h.campo.nombre(),
                h.margen,
                h.campo.unidad().simbolo(),
                a[0],
                a[1],
                a[2]
            );
        }
        let sin_uso: Vec<&str> = self
            .huellas
            .iter()
            .filter(|h| !h.usado())
            .map(|h| h.campo.nombre())
            .collect();
        if !sin_uso.is_empty() {
            let _ = writeln!(
                s,
                "  No las usa este modelo (sí la app): {}",
                sin_uso.join(", ")
            );
        }
        let _ = writeln!(s, "\nSupuestos:");
        for sup in &self.supuestos {
            let _ = writeln!(s, "  - {}", sup.texto());
        }
        if !self.avisos.is_empty() {
            let _ = writeln!(s, "\nAvisos (se calculó igual; conviene volver a medir):");
            for a in &self.avisos {
                let _ = writeln!(s, "  - {a}");
            }
        }
        s
    }
}

/// Cómo se estimó un campo que no se midió.
fn como_se_estima(c: Campo) -> String {
    match c {
        Campo::AlturaBarra => "convención del modelo: a la altura del hombro".into(),
        Campo::AnguloTibia => format!(
            "convención del modelo: {}°, sin fuente",
            super::ANGULO_TIBIA_CONVENCION
        ),
        _ => match c.fraccion_de_estatura() {
            Some(f) => format!("Winter, fig. 4.1: {f:.3} × estatura"),
            None => "no se estima".into(),
        },
    }
}

fn problema_json(p: &Problema) -> J {
    J::Obj(vec![
        ("where", J::Str(p.donde.clone())),
        ("line", opcion(p.linea.map(|l| l as f64))),
        ("message", J::Str(p.motivo.clone())),
    ])
}

/// Los errores de revisión en JSON, para que la app los muestre.
#[must_use]
pub fn errores_json(errores: &[Problema]) -> String {
    let mut s = String::new();
    J::Obj(vec![
        ("format", J::Str(FORMATO.into())),
        (
            "errors",
            J::Arr(errores.iter().map(problema_json).collect()),
        ),
    ])
    .escribir(&mut s, 0);
    s.push('\n');
    s
}

fn opcion(x: Option<f64>) -> J {
    x.map_or(J::Null, J::Num)
}

/// Un valor JSON, lo justo para escribir la salida sin dependencias.
enum J {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<J>),
    Obj(Vec<(&'static str, J)>),
}

impl J {
    /// Con sangría de dos espacios. Los números salen con la representación
    /// más corta que vuelve al mismo `f64`: leer y escribir no pierde nada.
    fn escribir(&self, s: &mut String, nivel: usize) {
        let sangria = |s: &mut String, n: usize| s.push_str(&"  ".repeat(n));
        match self {
            J::Null => s.push_str("null"),
            J::Bool(b) => s.push_str(if *b { "true" } else { "false" }),
            J::Num(x) => {
                assert!(x.is_finite(), "JSON no tiene NaN ni infinito");
                let _ = write!(s, "{x}");
            }
            J::Str(t) => escribir_texto(s, t),
            J::Arr(v) if v.is_empty() => s.push_str("[]"),
            J::Arr(v) => {
                s.push_str("[\n");
                for (i, x) in v.iter().enumerate() {
                    sangria(s, nivel + 1);
                    x.escribir(s, nivel + 1);
                    s.push_str(if i + 1 < v.len() { ",\n" } else { "\n" });
                }
                sangria(s, nivel);
                s.push(']');
            }
            J::Obj(v) => {
                s.push_str("{\n");
                for (i, (k, x)) in v.iter().enumerate() {
                    sangria(s, nivel + 1);
                    escribir_texto(s, k);
                    s.push_str(": ");
                    x.escribir(s, nivel + 1);
                    s.push_str(if i + 1 < v.len() { ",\n" } else { "\n" });
                }
                sangria(s, nivel);
                s.push('}');
            }
        }
    }
}

fn escribir_texto(s: &mut String, t: &str) {
    s.push('"');
    for c in t.chars() {
        match c {
            '"' => s.push_str("\\\""),
            '\\' => s.push_str("\\\\"),
            '\n' => s.push_str("\\n"),
            '\t' => s.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(s, "\\u{:04x}", c as u32);
            }
            c => s.push(c),
        }
    }
    s.push('"');
}

#[cfg(test)]
mod tests {
    use super::super::tests::{con, medido, todo_estimado};
    use super::super::{revisar, Campo};
    use super::*;
    use crate::maquina_humana::Persona;
    use crate::sentadilla::Sentadilla;

    fn informe(texto: &str, carga: Option<f64>) -> Informe {
        resolver(&revisar(texto).unwrap(), carga).unwrap()
    }

    /// Las medidas de MECÁNICA 01 (tibia 43, tronco 53 cm, tibia a 22°)
    /// escritas como alturas de una persona de 1.77 m: maléolo 7, rodilla
    /// 50, trocánter 50 + fémur, hombro 53 más arriba.
    fn mecanica_01(femur_cm: f64) -> String {
        let t = con(
            &todo_estimado(177.0),
            Campo::AlturaMaleolo,
            &medido("valor_cm", 7.0),
        );
        let t = con(&t, Campo::AlturaRodilla, &medido("valor_cm", 50.0));
        let t = con(
            &t,
            Campo::AlturaTrocanter,
            &medido("valor_cm", 50.0 + femur_cm),
        );
        let t = con(
            &t,
            Campo::AlturaHombro,
            &medido("valor_cm", 50.0 + femur_cm + 53.0),
        );
        con(&t, Campo::AnguloTibia, &medido("valor_grados", 22.0))
    }

    /// Con todo estimado, la postura es la de `Sentadilla` con los
    /// segmentos de Winter y la tibia a 22°, exactamente.
    #[test]
    fn con_todo_estimado_reproduce_sentadilla_con_winter() {
        for h in [160.0, 175.0, 190.0] {
            let i = informe(&todo_estimado(h), Some(100.0));
            let p = Persona {
                estatura_m: h / 100.0,
                masa_kg: 80.0,
            };
            let s = Sentadilla {
                femur_m: p.femur(),
                tibia_m: p.tibia(),
                torso_m: p.torso(),
                load_kg: 100.0,
                bar_offset_m: 0.0,
                shank_rad: 22.0_f64.to_radians(),
            };
            assert_eq!(i.postura, s.postura());
            assert_eq!(i.momentos_nm().unwrap(), [s.tau_hip(), s.tau_knee()]);
            assert_eq!(i.hombro, s.postura().bar, "barra estimada: en el hombro");
        }
    }

    /// Las medidas de la publicación dan sus números: 28 y 38 cm de
    /// brazo, 274 y 372 N·m en la cadera, 119 N·m en la rodilla.
    #[test]
    fn las_medidas_de_mecanica_01_dan_sus_numeros() {
        let corto = informe(&mecanica_01(40.0), Some(100.0));
        let largo = informe(&mecanica_01(50.0), Some(100.0));
        assert_eq!((corto.brazos_m[0] * 100.0).round(), 28.0);
        assert_eq!((largo.brazos_m[0] * 100.0).round(), 38.0);
        assert_eq!(corto.momentos_nm().unwrap()[0].round(), 274.0);
        assert_eq!(largo.momentos_nm().unwrap()[0].round(), 372.0);
        assert_eq!(corto.momentos_nm().unwrap()[1].round(), 119.0);
        assert_eq!(largo.momentos_nm().unwrap()[1].round(), 119.0);
    }

    #[test]
    fn sin_carga_hay_brazos_y_no_momentos() {
        let i = informe(&mecanica_01(40.0), None);
        assert!(i.momentos_nm().is_none());
        assert_eq!((i.brazos_m[0] * 100.0).round(), 28.0);
        let j = i.json();
        assert!(j.contains("\"moment_nm\": null"), "{j}");
        assert!(j.contains("\"basis\": \"none\""));
    }

    /// El análogo de la afirmación C2: medir mal el trocánter (el fémur)
    /// no mueve el brazo de la rodilla.
    #[test]
    fn el_trocanter_no_mueve_la_rodilla() {
        let i = informe(&mecanica_01(40.0), Some(100.0));
        let h = &i.huellas[Campo::AlturaTrocanter.indice()];
        assert!(
            (h.por_unidad[0] - 0.01).abs() < 1e-9,
            "1 cm de fémur, 1 cm de brazo de cadera"
        );
        assert!(h.por_unidad[1].abs() < 1e-12, "{:?}", h.por_unidad);
    }

    /// Lo que no usa el modelo de la barra se dice, no se esconde.
    #[test]
    fn masa_y_pie_no_los_usa_este_modelo() {
        let i = informe(&mecanica_01(40.0), Some(100.0));
        for c in [Campo::Masa, Campo::LargoPie] {
            assert!(!i.huellas[c.indice()].usado(), "{c:?}");
        }
        // Con todo medido, la estatura tampoco: nada se estima con ella.
        let t = con(
            &mecanica_01(40.0),
            Campo::AlturaBarra,
            &medido("valor_cm", 145.0),
        );
        let i = informe(&t, Some(100.0));
        assert!(!i.huellas[Campo::Estatura.indice()].usado());
        assert!(!i.supuestos.contains(&Supuesto::LargosDeTabla));
        assert!(!i.supuestos.contains(&Supuesto::BarraEnElHombro));
    }

    #[test]
    fn con_todo_estimado_la_estatura_decide() {
        let i = informe(&todo_estimado(175.0), Some(100.0));
        assert_eq!(
            i.decisivas()[0].campo,
            Campo::AnguloTibia,
            "±10° pesa más que ±2.5 cm"
        );
        assert!(i.huellas[Campo::Estatura.indice()].usado());
        for s in [
            Supuesto::LargosDeTabla,
            Supuesto::BarraEnElHombro,
            Supuesto::AnguloTibiaConvencion,
        ] {
            assert!(i.supuestos.contains(&s), "{s:?}");
        }
    }

    /// Medir baja la incertidumbre: lo mismo, medido, da una cota menor.
    #[test]
    fn medir_angosta_la_cota() {
        let estimado = informe(&todo_estimado(177.0), Some(100.0)).incertidumbre();
        let medido = informe(&mecanica_01(40.0), Some(100.0)).incertidumbre();
        assert!(
            medido[0] < estimado[0] && medido[1] < estimado[1],
            "{medido:?} vs {estimado:?}"
        );
    }

    #[test]
    fn una_postura_que_no_cierra_se_rechaza() {
        // Tronco de 20 cm (barra medida 20 cm arriba del trocánter): la cadera
        // queda a ~30 cm detrás del medio pie y el tronco no alcanza.
        let t = con(
            &mecanica_01(40.0),
            Campo::AlturaBarra,
            &medido("valor_cm", 110.0),
        );
        let r = revisar(&t);
        // La barra tan baja ya es un error de revisión; el rechazo del
        // modelo se prueba con el resolvedor directo sobre unas medidas
        // que sí pasan pero cuyo tronco no alcanza.
        assert!(r.is_err());
        let mut rev = revisar(&mecanica_01(40.0)).unwrap();
        rev.medidas.datos[Campo::AlturaBarra.indice()] = Dato::Medido(super::super::Medicion {
            valor: 110.0,
            fecha: "2026-10-18".into(),
            como: "prueba".into(),
        });
        assert!(matches!(
            resolver(&rev, Some(100.0)),
            Err(Rechazo::NoCierra { .. })
        ));
    }

    #[test]
    fn la_carga_debe_ser_positiva() {
        let rev = revisar(&todo_estimado(175.0)).unwrap();
        for kg in [0.0, -20.0, f64::NAN, f64::INFINITY] {
            assert!(
                matches!(resolver(&rev, Some(kg)), Err(Rechazo::CargaNoValida { .. })),
                "{kg}"
            );
        }
    }

    #[test]
    fn la_entrada_de_la_app_solo_lleva_lo_medido() {
        let j = informe(&mecanica_01(40.0), Some(100.0)).json();
        assert!(j.contains("\"thigh_m\": 0.4"), "{j}");
        assert!(j.contains("\"foot_m\": null"));
        assert!(j.contains("\"bar_on_trunk\": null"));
        assert!(j.contains("\"shank_angle_deg\": 22"));
    }

    #[test]
    fn el_texto_dice_los_supuestos_y_lo_que_no_se_usa() {
        let t = informe(&todo_estimado(175.0), Some(100.0)).texto();
        assert!(t.contains("solo la barra"), "{t}");
        assert!(
            t.contains("No las usa este modelo (sí la app): masa, largo_pie"),
            "{t}"
        );
    }

    #[test]
    fn el_json_escapa_el_texto() {
        let mut s = String::new();
        escribir_texto(&mut s, "a \"b\" \\ c\n\u{1}");
        assert_eq!(s, "\"a \\\"b\\\" \\\\ c\\n\\u0001\"");
    }
}
