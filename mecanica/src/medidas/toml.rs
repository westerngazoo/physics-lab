//! Un subconjunto de TOML: lo justo para un archivo de medidas.
//!
//! Tablas `[nombre]`, claves `clave = valor` (texto entre comillas, número
//! o booleano) y comentarios `#`. Nada más: ni arreglos, ni tablas en
//! línea, ni claves con punto, ni guiones bajos en los números. Lo que no
//! entiende lo rechaza con su número de línea, porque un archivo que se
//! lee a medias no debe dar números.
//!
//! Todo archivo que este lector acepta es TOML válido, así que la app y
//! cualquier otra herramienta lo leen con un lector completo
//! (`checks/medidas.py` lo comprueba con `tomllib`). Está escrito a mano
//! para que el crate siga sin dependencias.

use std::collections::BTreeMap;

/// Un valor del archivo.
#[derive(Clone, Debug, PartialEq)]
pub enum Valor {
    /// Texto entre comillas dobles.
    Texto(String),
    /// Un número, entero o decimal.
    Numero(f64),
    /// `true` o `false`.
    Booleano(bool),
}

/// Un valor y la línea donde está, para que los errores digan dónde.
#[derive(Clone, Debug, PartialEq)]
pub struct Entrada {
    /// El valor leído.
    pub valor: Valor,
    /// Línea del archivo, desde 1.
    pub linea: usize,
}

/// Las claves de una tabla, en orden alfabético.
pub type Tabla = BTreeMap<String, Entrada>;

/// Un archivo leído: las claves de arriba y cada tabla con su línea.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Documento {
    /// Claves antes de la primera tabla.
    pub raiz: Tabla,
    /// Cada `[tabla]`, con la línea de su encabezado.
    pub tablas: BTreeMap<String, (usize, Tabla)>,
}

/// Por qué no se pudo leer una línea.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ErrorDeLectura {
    /// Línea del archivo, desde 1.
    pub linea: usize,
    /// Qué no se entendió.
    pub motivo: String,
}

/// Lee el texto completo de un archivo.
///
/// # Errors
/// El primer renglón que no entiende, con su número de línea.
pub fn leer(texto: &str) -> Result<Documento, ErrorDeLectura> {
    let mut doc = Documento::default();
    let mut actual: Option<String> = None;
    for (i, crudo) in texto.lines().enumerate() {
        let linea = i + 1;
        let falla = |motivo: &str| ErrorDeLectura {
            linea,
            motivo: motivo.to_string(),
        };
        let renglon = sin_comentario(crudo).trim();
        if renglon.is_empty() {
            continue;
        }
        if let Some(resto) = renglon.strip_prefix('[') {
            let nombre = resto
                .strip_suffix(']')
                .ok_or_else(|| falla("al encabezado le falta el ']'"))?
                .trim();
            if nombre.starts_with('[') || !es_clave(nombre) {
                return Err(falla(
                    "encabezado no válido: usa [nombre] con letras, números y _",
                ));
            }
            if doc.tablas.contains_key(nombre) {
                return Err(falla(&format!("la tabla [{nombre}] aparece dos veces")));
            }
            doc.tablas.insert(nombre.to_string(), (linea, Tabla::new()));
            actual = Some(nombre.to_string());
            continue;
        }
        let (clave, valor) = renglon
            .split_once('=')
            .ok_or_else(|| falla("se esperaba `clave = valor`"))?;
        let clave = clave.trim();
        if !es_clave(clave) {
            return Err(falla(&format!(
                "clave no válida «{clave}»: usa letras, números y _"
            )));
        }
        let valor = leer_valor(valor.trim()).map_err(|m| falla(&m))?;
        let tabla = match &actual {
            Some(nombre) => &mut doc.tablas.get_mut(nombre).expect("se insertó al abrirla").1,
            None => &mut doc.raiz,
        };
        if tabla.contains_key(clave) {
            return Err(falla(&format!("la clave «{clave}» aparece dos veces")));
        }
        tabla.insert(clave.to_string(), Entrada { valor, linea });
    }
    Ok(doc)
}

/// El renglón hasta el primer `#` que no esté dentro de comillas.
fn sin_comentario(renglon: &str) -> &str {
    let mut en_texto = false;
    let mut escapado = false;
    for (i, c) in renglon.char_indices() {
        match c {
            _ if escapado => escapado = false,
            '\\' if en_texto => escapado = true,
            '"' => en_texto = !en_texto,
            '#' if !en_texto => return &renglon[..i],
            _ => {}
        }
    }
    renglon
}

/// Una clave sencilla de TOML, sin comillas ni puntos.
fn es_clave(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn leer_valor(s: &str) -> Result<Valor, String> {
    if let Some(resto) = s.strip_prefix('"') {
        return leer_texto(resto).map(Valor::Texto);
    }
    match s {
        "" => Err("falta el valor después del =".to_string()),
        "true" => Ok(Valor::Booleano(true)),
        "false" => Ok(Valor::Booleano(false)),
        _ if es_numero(s) => s
            .parse()
            .map(Valor::Numero)
            .map_err(|_| format!("número no válido «{s}»")),
        _ => Err(format!(
            "valor no válido «{s}»: escribe un número (175.0), texto entre comillas o true/false"
        )),
    }
}

/// El texto después de la comilla que abre, hasta la que cierra.
fn leer_texto(resto: &str) -> Result<String, String> {
    let mut texto = String::new();
    let mut chars = resto.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                return if chars.as_str().trim().is_empty() {
                    Ok(texto)
                } else {
                    Err("sobra algo después de las comillas".to_string())
                };
            }
            '\\' => match chars.next() {
                Some('"') => texto.push('"'),
                Some('\\') => texto.push('\\'),
                Some('n') => texto.push('\n'),
                Some('t') => texto.push('\t'),
                _ => return Err("escape no válido: usa \\\" o \\\\".to_string()),
            },
            _ => texto.push(c),
        }
    }
    Err("al texto le falta la comilla que cierra".to_string())
}

/// Un número en la forma que TOML acepta y que basta aquí: signo opcional,
/// enteros sin ceros a la izquierda, decimales con cifras a ambos lados del
/// punto y exponente opcional. `inf`, `nan` y `1_000` quedan fuera.
fn es_numero(s: &str) -> bool {
    let s = s.strip_prefix(['+', '-']).unwrap_or(s);
    let (mantisa, exponente) = match s.find(['e', 'E']) {
        Some(i) => (&s[..i], Some(&s[i + 1..])),
        None => (s, None),
    };
    let (entera, decimal) = match mantisa.split_once('.') {
        Some((e, d)) => (e, Some(d)),
        None => (mantisa, None),
    };
    let cifras = |t: &str| !t.is_empty() && t.bytes().all(|b| b.is_ascii_digit());
    let entera_ok = cifras(entera) && (entera == "0" || !entera.starts_with('0'));
    let decimal_ok = decimal.is_none_or(cifras);
    let exponente_ok = exponente.is_none_or(|e| cifras(e.strip_prefix(['+', '-']).unwrap_or(e)));
    entera_ok && decimal_ok && exponente_ok
}

#[cfg(test)]
mod tests {
    use super::*;

    fn numero(doc: &Documento, tabla: &str, clave: &str) -> f64 {
        match doc.tablas[tabla].1[clave].valor {
            Valor::Numero(x) => x,
            ref otro => panic!("se esperaba número, hay {otro:?}"),
        }
    }

    #[test]
    fn lee_tablas_claves_y_comentarios() {
        let doc = leer(
            "formato = \"x/1\"  # comentario\n\n[estatura]\nvalor_cm = 175.5\nfuente = \"medido\"\n",
        )
        .unwrap();
        assert_eq!(doc.raiz["formato"].valor, Valor::Texto("x/1".into()));
        assert_eq!(numero(&doc, "estatura", "valor_cm"), 175.5);
        assert_eq!(doc.tablas["estatura"].0, 3, "la línea del encabezado");
        assert_eq!(doc.tablas["estatura"].1["fuente"].linea, 5);
    }

    #[test]
    fn un_gato_dentro_de_comillas_no_es_comentario() {
        let doc = leer("[a]\ncomo = \"marca #2 en la pared\" # esto sí\n").unwrap();
        assert_eq!(
            doc.tablas["a"].1["como"].valor,
            Valor::Texto("marca #2 en la pared".into())
        );
    }

    #[test]
    fn entiende_los_escapes_basicos() {
        let doc = leer("x = \"dijo \\\"aquí\\\" \\\\ fin\"\n").unwrap();
        assert_eq!(
            doc.raiz["x"].valor,
            Valor::Texto("dijo \"aquí\" \\ fin".into())
        );
    }

    #[test]
    fn enteros_y_negativos_son_numeros() {
        let doc = leer("a = 175\nb = -3.5\nc = 1.5e2\nd = 0.75\n").unwrap();
        assert_eq!(doc.raiz["a"].valor, Valor::Numero(175.0));
        assert_eq!(doc.raiz["b"].valor, Valor::Numero(-3.5));
        assert_eq!(doc.raiz["c"].valor, Valor::Numero(150.0));
        assert_eq!(doc.raiz["d"].valor, Valor::Numero(0.75));
    }

    #[test]
    fn rechaza_lo_que_no_es_toml_o_no_entiende() {
        for (texto, linea) in [
            ("a = 1\na = 2\n", 2),
            ("[t]\n[t]\n", 2),
            ("a = [1, 2]\n", 1),
            ("a = { b = 1 }\n", 1),
            ("a.b = 1\n", 1),
            ("a = 1_000\n", 1),
            ("a = .5\n", 1),
            ("a = 5.\n", 1),
            ("a = 07\n", 1),
            ("a = nan\n", 1),
            ("a = \"sin cerrar\n", 1),
            ("a = \"x\" y\n", 1),
            ("solo texto\n", 1),
            ("\n[[t]]\n", 2),
            ("a =\n", 1),
        ] {
            let e = leer(texto).expect_err(texto);
            assert_eq!(e.linea, linea, "{texto:?}: {}", e.motivo);
        }
    }
}
