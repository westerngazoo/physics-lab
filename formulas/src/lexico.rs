//! El léxico: del texto a fichas, cada una con la columna donde empieza.
//!
//! La columna se cuenta en CARACTERES, no en bytes, y empieza en 1: es la
//! que el lector ve en la caja de texto. `τ` ocupa dos bytes y una
//! columna; si el error de `2τ(` señalara la columna 4 en vez de la 3,
//! la flechita del mensaje apuntaría al lugar equivocado.

use crate::{Codigo, Error};

/// Una ficha del léxico.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Tok<'a> {
    Num(f64),
    Nombre(&'a str),
    Mas,
    Menos,
    Por,
    Entre,
    Pot,
    Abre,
    Cierra,
    /// Un superíndice pegado: `t²` es `t^2`.
    Sup(i32),
    Fin,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Ficha<'a> {
    pub tok: Tok<'a>,
    /// Columna (en caracteres, desde 1) donde empieza.
    pub col: usize,
}

/// Parte `fuente` en fichas. La última siempre es `Fin`, con la columna
/// justo después del último carácter: ahí apunta un "falta algo".
pub(crate) fn fichas(fuente: &str) -> Result<Vec<Ficha<'_>>, Error> {
    let mut out = Vec::new();
    let mut it = fuente.char_indices().peekable();
    let mut col = 0usize;
    while let Some((i, c)) = it.next() {
        col += 1;
        let aqui = col;
        let tok = match c {
            ' ' | '\t' | '\r' | '\u{a0}' => continue,
            '+' => Tok::Mas,
            // El guion, el signo menos de verdad (U+2212) y la raya corta
            // que pegan los procesadores de texto: los tres restan.
            '-' | '\u{2212}' | '\u{2013}' => Tok::Menos,
            '*' | '·' | '×' | '\u{22c5}' | '\u{2219}' => Tok::Por,
            '/' | '÷' => Tok::Entre,
            '^' => Tok::Pot,
            '(' => Tok::Abre,
            ')' => Tok::Cierra,
            '²' => Tok::Sup(2),
            '³' => Tok::Sup(3),
            ',' => return Err(Error::en(Codigo::Coma, aqui)),
            '0'..='9' | '.' => {
                let mut fin = i + c.len_utf8();
                let mut puntos = usize::from(c == '.');
                while let Some(&(j, d)) = it.peek() {
                    if d.is_ascii_digit() || d == '.' {
                        puntos += usize::from(d == '.');
                        fin = j + d.len_utf8();
                        col += 1;
                        it.next();
                    } else {
                        break;
                    }
                }
                let txt = &fuente[i..fin];
                // `str::parse` aceptaría `1.` y `.5` (bien) pero también
                // tragaría cosas que un lector no escribió como número.
                // Un número es dígitos con a lo más un punto, y al menos
                // un dígito.
                let digitos = txt.bytes().any(|b| b.is_ascii_digit());
                match txt.parse::<f64>() {
                    Ok(x) if puntos <= 1 && digitos && x.is_finite() => Tok::Num(x),
                    _ => return Err(Error::en(Codigo::NumeroMalFormado, aqui)),
                }
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let mut fin = i + 1;
                while let Some(&(j, d)) = it.peek() {
                    if d.is_ascii_alphabetic() || d == '_' {
                        fin = j + 1;
                        col += 1;
                        it.next();
                    } else {
                        break;
                    }
                }
                Tok::Nombre(&fuente[i..fin])
            }
            // Una letra que no es ASCII (τ, π, θ…) es un nombre de UN
            // carácter: `2πt` son tres fichas, no una palabra rara.
            c if c.is_alphabetic() => Tok::Nombre(&fuente[i..i + c.len_utf8()]),
            _ => return Err(Error::en(Codigo::CaracterInesperado, aqui)),
        };
        out.push(Ficha { tok, col: aqui });
    }
    out.push(Ficha {
        tok: Tok::Fin,
        col: col + 1,
    });
    Ok(out)
}
