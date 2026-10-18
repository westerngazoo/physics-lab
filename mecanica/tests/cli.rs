//! La línea de comandos de punta a punta: archivo → `mecanica sentadilla`
//! → JSON → de vuelta a números, comparados con la biblioteca y con el
//! archivo de entrada.

use std::path::PathBuf;
use std::process::{Command, Output};

use mecanica::maquina_humana::Persona;
use mecanica::medidas::{informe, revisar};
use mecanica::sentadilla::Sentadilla;

fn raiz(ruta: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(ruta)
}

fn correr(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mecanica"))
        .args(args)
        .output()
        .expect("el binario corre")
}

fn json_de(out: &Output) -> Json {
    let s = String::from_utf8(out.stdout.clone()).expect("UTF-8");
    Lector::new(&s).documento()
}

const EJEMPLO: &str = "medidas/ejemplo-sentadilla.toml";
const PLANTILLA: &str = "medidas/plantilla-sentadilla.toml";

#[test]
fn el_ejemplo_va_y_vuelve_sin_perder_nada() {
    let ruta = raiz(EJEMPLO);
    let out = correr(&[
        "sentadilla",
        "--medidas",
        ruta.to_str().unwrap(),
        "--carga",
        "100",
        "--json",
    ]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let j = json_de(&out);

    // Lo que entró sale igual.
    assert_eq!(j["format"].texto(), informe::FORMATO);
    assert_eq!(j["person"].texto(), "P-EJEMPLO1");
    let app = &j["app_input"];
    assert_eq!(app["height_cm"].num(), 172.0);
    assert_eq!(app["weight_kg"].num(), 78.0);
    assert_eq!(app["load_kg"].num(), 100.0);
    let largos = &app["measured_lengths"];
    assert_eq!(largos["thigh_m"].num(), (91.5 - 49.0) / 100.0);
    assert_eq!(largos["shank_m"].num(), (49.0 - 6.5) / 100.0);
    assert_eq!(largos["trunk_m"].num(), (141.0 - 91.5) / 100.0);
    assert_eq!(largos["ankle_height_m"].num(), 6.5 / 100.0);
    assert!(
        largos["foot_m"].es_nulo(),
        "el pie se estimó: no va a la app"
    );
    assert_eq!(app["bar_on_trunk"].num(), (143.5 - 91.5) / (141.0 - 91.5));
    assert_eq!(app["shank_angle_deg"].num(), 26.0);

    // Lo que salió es, bit por bit, lo que calcula la biblioteca.
    let texto = std::fs::read_to_string(&ruta).unwrap();
    let i = informe::resolver(&revisar(&texto).unwrap(), Some(100.0)).unwrap();
    let modelo = &j["model"];
    let [cadera, rodilla] = i.momentos_nm().unwrap();
    assert_eq!(modelo["joints"][0]["joint"].texto(), "hip");
    assert_eq!(modelo["joints"][0]["moment_nm"].num(), cadera);
    assert_eq!(modelo["joints"][1]["joint"].texto(), "knee");
    assert_eq!(modelo["joints"][1]["moment_nm"].num(), rodilla);
    assert_eq!(modelo["joints"][0]["moment_arm_m"].num(), i.brazos_m[0]);
    // La misma afirmación que comprueba `checks/medidas.py` por otro camino.
    assert_eq!((cadera * 100.0).round() / 100.0, 273.40);
    assert_eq!((rodilla * 100.0).round() / 100.0, 143.53);
    assert_eq!((i.tronco_grados() * 100.0).round() / 100.0, 32.41);
    let p = &modelo["posture"];
    assert_eq!((p["hip"]["x"].num(), p["hip"]["y"].num()), i.postura.hip);
    assert_eq!((p["knee"]["x"].num(), p["knee"]["y"].num()), i.postura.knee);
    assert_eq!(p["load"]["kg"].num(), 100.0);
    assert_eq!(p["load"]["at"]["x"].num(), 0.0, "barra sobre el medio pie");
    assert_eq!(modelo["trunk_angle_deg"].num(), i.tronco_grados());

    // Y dice de dónde salió cada número.
    let medidas = j["measurements"].arreglo();
    assert_eq!(medidas.len(), 9);
    let pie = medidas
        .iter()
        .find(|m| m["field"].texto() == "largo_pie")
        .unwrap();
    assert_eq!(pie["source"].texto(), "estimated");
    assert!(!pie["used"].booleano());
    let rodilla_m = medidas
        .iter()
        .find(|m| m["field"].texto() == "altura_rodilla")
        .unwrap();
    assert_eq!(rodilla_m["source"].texto(), "measured");
    assert_eq!(rodilla_m["on"].texto(), "2026-10-18");
    let decisivas: Vec<&str> = j["drivers"].arreglo().iter().map(Json::texto).collect();
    assert_eq!(decisivas.first(), Some(&"angulo_tibia"), "{decisivas:?}");
    assert!(!decisivas.contains(&"masa"));
    let supuestos: Vec<&str> = modelo["assumptions"]
        .arreglo()
        .iter()
        .map(Json::texto)
        .collect();
    assert!(supuestos.contains(&"bar_only"));
    assert!(
        !supuestos.contains(&"table_derived_lengths"),
        "los largos del modelo se midieron"
    );
}

/// Un archivo sin medidas da, por la línea de comandos, lo mismo que
/// `Sentadilla` con los segmentos de Winter.
#[test]
fn sin_medidas_la_linea_de_comandos_reproduce_sentadilla() {
    let mut texto = String::from(
        "formato = \"medidas-sentadilla/1\"\npersona = \"P-W1NT\"\nfecha = \"2026-10-18\"\n\
         [estatura]\nfuente = \"medido\"\nvalor_cm = 175.0\nfecha = \"2026-10-18\"\ncomo = \"cinta\"\n\
         [masa]\nfuente = \"medido\"\nvalor_kg = 80.0\nfecha = \"2026-10-18\"\ncomo = \"báscula\"\n",
    );
    for t in [
        "altura_hombro",
        "altura_trocanter",
        "altura_rodilla",
        "altura_maleolo",
        "largo_pie",
        "altura_barra",
        "angulo_tibia",
    ] {
        texto.push_str(&format!("[{t}]\nfuente = \"estimado\"\n"));
    }
    let ruta = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("winter-175.toml");
    std::fs::write(&ruta, texto).unwrap();
    let out = correr(&[
        "sentadilla",
        "--medidas",
        ruta.to_str().unwrap(),
        "--carga",
        "100",
        "--json",
    ]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let j = json_de(&out);

    let p = Persona {
        estatura_m: 1.75,
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
    let joints = &j["model"]["joints"];
    assert_eq!(joints[0]["moment_nm"].num(), s.tau_hip());
    assert_eq!(joints[1]["moment_nm"].num(), s.tau_knee());
    assert_eq!(joints[0]["moment_arm_m"].num(), s.postura().d_hip);
    assert_eq!(
        j["model"]["trunk_angle_deg"].num(),
        s.postura().torso_rad.to_degrees()
    );
    let largos = &j["app_input"]["measured_lengths"];
    for k in ["thigh_m", "shank_m", "trunk_m", "foot_m", "ankle_height_m"] {
        assert!(
            largos[k].es_nulo(),
            "{k}: nada se midió, nada se manda como medido"
        );
    }
    let supuestos: Vec<&str> = j["model"]["assumptions"]
        .arreglo()
        .iter()
        .map(Json::texto)
        .collect();
    for s in [
        "table_derived_lengths",
        "bar_at_shoulder",
        "conventional_shank_angle",
    ] {
        assert!(supuestos.contains(&s), "{s}");
    }
}

/// La plantilla está completa salvo lo que solo el dueño puede poner.
#[test]
fn la_plantilla_solo_pide_lo_que_falta_medir() {
    let ruta = raiz(PLANTILLA);
    let out = correr(&["sentadilla", "--medidas", ruta.to_str().unwrap(), "--json"]);
    assert_eq!(out.status.code(), Some(1));
    let j = json_de(&out);
    let mut donde: Vec<&str> = j["errors"]
        .arreglo()
        .iter()
        .map(|e| e["where"].texto())
        .collect();
    donde.dedup();
    assert_eq!(donde, ["persona", "fecha", "estatura", "masa"]);
}

#[test]
fn sin_carga_hay_brazos_y_no_momentos() {
    let ruta = raiz(EJEMPLO);
    let out = correr(&["sentadilla", "--medidas", ruta.to_str().unwrap(), "--json"]);
    assert!(out.status.success());
    let j = json_de(&out);
    assert!(j["app_input"]["load_kg"].es_nulo());
    assert_eq!(j["model"]["load"]["basis"].texto(), "none");
    assert!(j["model"]["posture"]["load"].es_nulo());
    assert!(j["model"]["joints"][0]["moment_nm"].es_nulo());
    assert!(j["model"]["joints"][0]["moment_arm_m"].num() > 0.0);
}

#[test]
fn el_texto_sale_en_palabras() {
    let ruta = raiz(EJEMPLO);
    let out = correr(&[
        "sentadilla",
        "--medidas",
        ruta.to_str().unwrap(),
        "--carga",
        "100",
    ]);
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(s.contains("Supuestos:") && s.contains("N·m"), "{s}");
}

#[test]
fn una_orden_mal_escrita_sale_con_dos() {
    let ruta = raiz(EJEMPLO);
    let r = ruta.to_str().unwrap();
    for args in [
        vec![],
        vec!["peso-muerto", "--medidas", r],
        vec!["sentadilla"],
        vec!["sentadilla", "--medidas", r, "--carga", "cien"],
        vec!["sentadilla", "--medidas", r, "--otra"],
    ] {
        assert_eq!(correr(&args).status.code(), Some(2), "{args:?}");
    }
}

#[test]
fn una_carga_no_positiva_o_un_archivo_que_no_existe_salen_con_uno() {
    let ruta = raiz(EJEMPLO);
    let out = correr(&[
        "sentadilla",
        "--medidas",
        ruta.to_str().unwrap(),
        "--carga",
        "-5",
        "--json",
    ]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(json_de(&out)["errors"][0]["where"].texto(), "postura");
    let out = correr(&["sentadilla", "--medidas", "/no/existe.toml"]);
    assert_eq!(out.status.code(), Some(1));
}

// ---- Un lector de JSON mínimo: la prueba no le cree al escritor. ----

#[derive(Debug, PartialEq)]
enum Json {
    Nulo,
    Bool(bool),
    Num(f64),
    Texto(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    fn texto(&self) -> &str {
        match self {
            Json::Texto(s) => s,
            otro => panic!("se esperaba texto: {otro:?}"),
        }
    }
    fn num(&self) -> f64 {
        match self {
            Json::Num(x) => *x,
            otro => panic!("se esperaba número: {otro:?}"),
        }
    }
    fn booleano(&self) -> bool {
        match self {
            Json::Bool(b) => *b,
            otro => panic!("se esperaba booleano: {otro:?}"),
        }
    }
    fn arreglo(&self) -> &[Json] {
        match self {
            Json::Arr(v) => v,
            otro => panic!("se esperaba arreglo: {otro:?}"),
        }
    }
    fn es_nulo(&self) -> bool {
        matches!(self, Json::Nulo)
    }
}

impl std::ops::Index<&str> for Json {
    type Output = Json;
    fn index(&self, k: &str) -> &Json {
        match self {
            Json::Obj(v) => {
                &v.iter()
                    .find(|(c, _)| c == k)
                    .unwrap_or_else(|| panic!("falta «{k}»"))
                    .1
            }
            otro => panic!("se esperaba objeto para «{k}»: {otro:?}"),
        }
    }
}

impl std::ops::Index<usize> for Json {
    type Output = Json;
    fn index(&self, i: usize) -> &Json {
        &self.arreglo()[i]
    }
}

struct Lector<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> Lector<'a> {
    fn new(s: &'a str) -> Self {
        Lector {
            b: s.as_bytes(),
            i: 0,
        }
    }

    fn documento(mut self) -> Json {
        let v = self.valor();
        self.espacios();
        assert_eq!(self.i, self.b.len(), "sobra texto después del JSON");
        v
    }

    fn espacios(&mut self) {
        while self.i < self.b.len() && self.b[self.i].is_ascii_whitespace() {
            self.i += 1;
        }
    }

    fn come(&mut self, c: u8) {
        self.espacios();
        assert_eq!(self.b[self.i], c, "en {}", self.i);
        self.i += 1;
    }

    fn mira(&mut self) -> u8 {
        self.espacios();
        self.b[self.i]
    }

    fn valor(&mut self) -> Json {
        match self.mira() {
            b'n' => self.literal("null", Json::Nulo),
            b't' => self.literal("true", Json::Bool(true)),
            b'f' => self.literal("false", Json::Bool(false)),
            b'"' => Json::Texto(self.texto()),
            b'[' => {
                self.come(b'[');
                let mut v = Vec::new();
                if self.mira() == b']' {
                    self.come(b']');
                    return Json::Arr(v);
                }
                loop {
                    v.push(self.valor());
                    if self.mira() == b',' {
                        self.come(b',');
                    } else {
                        self.come(b']');
                        return Json::Arr(v);
                    }
                }
            }
            b'{' => {
                self.come(b'{');
                let mut v = Vec::new();
                if self.mira() == b'}' {
                    self.come(b'}');
                    return Json::Obj(v);
                }
                loop {
                    self.espacios();
                    let k = self.texto();
                    self.come(b':');
                    v.push((k, self.valor()));
                    if self.mira() == b',' {
                        self.come(b',');
                    } else {
                        self.come(b'}');
                        return Json::Obj(v);
                    }
                }
            }
            _ => {
                let inicio = self.i;
                while self.i < self.b.len() && b"+-.eE0123456789".contains(&self.b[self.i]) {
                    self.i += 1;
                }
                let s = std::str::from_utf8(&self.b[inicio..self.i]).unwrap();
                Json::Num(
                    s.parse()
                        .unwrap_or_else(|_| panic!("número no válido «{s}»")),
                )
            }
        }
    }

    fn literal(&mut self, palabra: &str, v: Json) -> Json {
        assert!(self.b[self.i..].starts_with(palabra.as_bytes()));
        self.i += palabra.len();
        v
    }

    fn texto(&mut self) -> String {
        self.come(b'"');
        let mut bytes = Vec::new();
        loop {
            let c = self.b[self.i];
            self.i += 1;
            match c {
                b'"' => return String::from_utf8(bytes).unwrap(),
                b'\\' => {
                    let e = self.b[self.i];
                    self.i += 1;
                    match e {
                        b'"' => bytes.push(b'"'),
                        b'\\' => bytes.push(b'\\'),
                        b'n' => bytes.push(b'\n'),
                        b't' => bytes.push(b'\t'),
                        b'u' => {
                            let h = std::str::from_utf8(&self.b[self.i..self.i + 4]).unwrap();
                            let ch = char::from_u32(u32::from_str_radix(h, 16).unwrap()).unwrap();
                            self.i += 4;
                            bytes.extend(ch.to_string().as_bytes());
                        }
                        otro => panic!("escape no válido \\{}", otro as char),
                    }
                }
                _ => bytes.push(c),
            }
        }
    }
}
