//! `mecanica`: la línea de comandos. Es el único lugar del crate que lee
//! archivos y escribe en la terminal; la física y la revisión son la
//! biblioteca, puras.
//!
//! ```text
//! mecanica sentadilla --medidas archivo.toml [--carga kg] [--json]
//! ```
//!
//! Salida: 0 si resolvió, 1 si las medidas tienen errores o la postura no
//! cierra, 2 si la orden está mal escrita.

use std::process::ExitCode;

use mecanica::medidas::{informe, revisar, Problema};

const USO: &str = "uso: mecanica sentadilla --medidas archivo.toml [--carga kg] [--json]

  --medidas  el archivo de medidas (plantilla: medidas/plantilla-sentadilla.toml)
  --carga    kg en la barra; sin ella hay postura y brazos, no momentos
  --json     la salida para la app, en JSON";

struct Orden {
    medidas: String,
    carga_kg: Option<f64>,
    json: bool,
}

fn leer_orden(args: &[String]) -> Result<Orden, String> {
    let mut it = args.iter();
    match it.next().map(String::as_str) {
        Some("sentadilla") => {}
        Some(otra) => {
            return Err(format!(
                "no conozco «{otra}»; por ahora solo hay sentadilla"
            ))
        }
        None => return Err("falta el ejercicio".into()),
    }
    let (mut medidas, mut carga_kg, mut json) = (None, None, false);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--medidas" => medidas = Some(it.next().ok_or("--medidas pide un archivo")?.clone()),
            "--carga" => {
                let v = it.next().ok_or("--carga pide los kg")?;
                carga_kg = Some(
                    v.parse::<f64>()
                        .map_err(|_| format!("--carga: «{v}» no es un número"))?,
                );
            }
            "--json" => json = true,
            otro => return Err(format!("no conozco «{otro}»")),
        }
    }
    Ok(Orden {
        medidas: medidas.ok_or("falta --medidas archivo.toml")?,
        carga_kg,
        json,
    })
}

fn fallar(json: bool, problemas: &[Problema]) -> ExitCode {
    if json {
        print!("{}", informe::errores_json(problemas));
    }
    eprintln!("No se calculó nada:");
    for p in problemas {
        eprintln!("  - {p}");
    }
    ExitCode::from(1)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let orden = match leer_orden(&args) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{e}\n\n{USO}");
            return ExitCode::from(2);
        }
    };
    let texto = match std::fs::read_to_string(&orden.medidas) {
        Ok(t) => t,
        Err(e) => {
            let p = Problema {
                donde: orden.medidas.clone(),
                linea: None,
                motivo: format!("no se pudo leer: {e}"),
            };
            return fallar(orden.json, &[p]);
        }
    };
    let revisadas = match revisar(&texto) {
        Ok(r) => r,
        Err(errores) => return fallar(orden.json, &errores),
    };
    match informe::resolver(&revisadas, orden.carga_kg) {
        Ok(i) if orden.json => print!("{}", i.json()),
        Ok(i) => print!("{}", i.texto()),
        Err(rechazo) => {
            let p = Problema {
                donde: "postura".into(),
                linea: None,
                motivo: rechazo.to_string(),
            };
            return fallar(orden.json, &[p]);
        }
    }
    ExitCode::SUCCESS
}
