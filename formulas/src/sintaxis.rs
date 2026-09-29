//! La sintaxis: de las fichas a un programa en notación polaca inversa.
//!
//! Descenso recursivo, de menor a mayor precedencia:
//!
//! ```text
//! suma     := producto (('+' | '−') producto)*
//! producto := unario (('*' | '/') unario | (nombre | '(') potencia)*
//! unario   := ('−' | '+') unario | potencia
//! potencia := átomo ('²' | '³')* ('^' unario)?        ← asociativa a la derecha
//! átomo    := número | nombre | función '(' suma ')' | '(' suma ')'
//! ```
//!
//! Tres decisiones que un lector va a notar, escritas aquí para que no
//! vivan sólo en el código:
//!
//! - **`−t²` es `−(t²)`**, como en el pizarrón (y no `(−t)²`).
//! - **La multiplicación implícita pesa lo mismo que la explícita**:
//!   `1/2t` es `(1/2)·t`, igual que en Python. Si se quiere `1/(2t)`, se
//!   escriben los paréntesis. Un número pegado DESPUÉS de algo (`t2`) no
//!   se multiplica: se rechaza, porque casi siempre es un `t^2` o un
//!   `t·2` a medio escribir.
//! - **Las funciones llevan paréntesis**: `sin(t)`, no `sin t`. Ahorra
//!   una ambigüedad (`sin 2t`) y cuesta dos caracteres.

use crate::lexico::{Ficha, Tok};
use crate::{Codigo, Error, Funcion, Op};

/// Anidamiento máximo. Protege la pila NATIVA de wasm (un desborde ahí es
/// una trampa que mata la página) y acota la pila del evaluador.
pub(crate) const PROFUNDIDAD_MAX: usize = 40;

pub(crate) struct Sintaxis<'f, 'v> {
    fichas: &'f [Ficha<'f>],
    i: usize,
    pub ops: Vec<Op>,
    vars: &'v [&'v str],
    profundidad: usize,
}

impl<'f, 'v> Sintaxis<'f, 'v> {
    pub fn new(fichas: &'f [Ficha<'f>], vars: &'v [&'v str]) -> Self {
        Sintaxis {
            fichas,
            i: 0,
            ops: Vec::new(),
            vars,
            profundidad: 0,
        }
    }

    /// Todo el texto debe ser UNA expresión: lo que sobre es un error.
    pub fn completa(&mut self) -> Result<(), Error> {
        self.suma()?;
        let f = self.mira();
        match f.tok {
            Tok::Fin => Ok(()),
            Tok::Cierra => Err(Error::en(Codigo::ParentesisDeMas, f.col)),
            _ => Err(Error::en(Codigo::CaracterInesperado, f.col)),
        }
    }

    fn mira(&self) -> Ficha<'f> {
        // `fichas` siempre termina en `Fin` y nunca se avanza más allá.
        self.fichas[self.i.min(self.fichas.len() - 1)]
    }

    fn avanza(&mut self) -> Ficha<'f> {
        let f = self.mira();
        if !matches!(f.tok, Tok::Fin) {
            self.i += 1;
        }
        f
    }

    fn entra(&mut self, col: usize) -> Result<(), Error> {
        self.profundidad += 1;
        if self.profundidad > PROFUNDIDAD_MAX {
            return Err(Error::en(Codigo::DemasiadoProfunda, col));
        }
        Ok(())
    }

    fn sale(&mut self) {
        self.profundidad -= 1;
    }

    fn suma(&mut self) -> Result<(), Error> {
        self.producto()?;
        loop {
            match self.mira().tok {
                Tok::Mas => {
                    self.avanza();
                    self.producto()?;
                    self.ops.push(Op::Suma);
                }
                Tok::Menos => {
                    self.avanza();
                    self.producto()?;
                    self.ops.push(Op::Resta);
                }
                _ => return Ok(()),
            }
        }
    }

    fn producto(&mut self) -> Result<(), Error> {
        self.unario()?;
        loop {
            let f = self.mira();
            match f.tok {
                Tok::Por => {
                    self.avanza();
                    self.unario()?;
                    self.ops.push(Op::Producto);
                }
                Tok::Entre => {
                    self.avanza();
                    self.unario()?;
                    self.ops.push(Op::Cociente);
                }
                // Multiplicación implícita: `2t`, `3sin(t)`, `(t+1)(t−1)`.
                Tok::Nombre(_) | Tok::Abre => {
                    self.potencia()?;
                    self.ops.push(Op::Producto);
                }
                Tok::Num(_) => return Err(Error::en(Codigo::NumeroSuelto, f.col)),
                _ => return Ok(()),
            }
        }
    }

    fn unario(&mut self) -> Result<(), Error> {
        let f = self.mira();
        self.entra(f.col)?;
        let r = match f.tok {
            Tok::Menos => {
                self.avanza();
                self.unario().map(|()| self.ops.push(Op::Negativo))
            }
            Tok::Mas => {
                self.avanza();
                self.unario()
            }
            _ => self.potencia(),
        };
        self.sale();
        r
    }

    fn potencia(&mut self) -> Result<(), Error> {
        let a = self.atomo()?;
        while let Tok::Sup(n) = self.mira().tok {
            self.avanza();
            self.ops.push(Op::Num(f64::from(n)));
            self.ops.push(Op::Potencia);
        }
        if let Tok::Pot = self.mira().tok {
            let f = self.avanza();
            self.entra(f.col)?;
            if a.es_e && matches!(self.ops.last(), Some(Op::Num(_))) {
                // `e^x` se compila como exp(x): exacto, en vez de pasar
                // por ln(2.718281828459045…), que no es exactamente 1.
                self.ops.pop();
                self.unario()?;
                self.ops.push(Op::Funcion(Funcion::Exp));
            } else {
                self.unario()?;
                self.ops.push(Op::Potencia);
            }
            self.sale();
        }
        // Los productos de un nombre partido (`at` = a·t) se cierran
        // DESPUÉS del exponente, para que `at²` sea a·t² y no (a·t)².
        for _ in 0..a.pendientes {
            self.ops.push(Op::Producto);
        }
        Ok(())
    }

    fn atomo(&mut self) -> Result<Atomo, Error> {
        let f = self.avanza();
        match f.tok {
            Tok::Num(x) => {
                self.ops.push(Op::Num(x));
                Ok(Atomo::SIMPLE)
            }
            Tok::Abre => {
                self.entra(f.col)?;
                self.suma()?;
                self.sale();
                self.cierra(f.col)?;
                Ok(Atomo::SIMPLE)
            }
            Tok::Nombre(n) => self.nombre(n, f.col),
            // `sin()`, `()`, `2+)`: lo que falta es un valor.
            _ => Err(Error::en(Codigo::FaltaOperando, f.col)),
        }
    }

    fn cierra(&mut self, col_abre: usize) -> Result<(), Error> {
        match self.mira().tok {
            Tok::Cierra => {
                self.avanza();
                Ok(())
            }
            // Se señala el paréntesis que quedó abierto: es donde el
            // lector tiene que mirar, no el final de la línea.
            _ => Err(Error::en(Codigo::FaltaParentesis, col_abre)),
        }
    }

    fn nombre(&mut self, n: &str, col: usize) -> Result<Atomo, Error> {
        if let Some(func) = Funcion::por_nombre(n) {
            let abre = self.mira();
            if !matches!(abre.tok, Tok::Abre) {
                return Err(Error::en(Codigo::FuncionSinParentesis, col));
            }
            self.avanza();
            self.entra(abre.col)?;
            self.suma()?;
            self.sale();
            self.cierra(abre.col)?;
            self.ops.push(Op::Funcion(func));
            return Ok(Atomo::SIMPLE);
        }
        if let Some(op) = self.simbolo(n) {
            self.ops.push(op);
            return Ok(Atomo {
                es_e: n == "e",
                pendientes: 0,
            });
        }
        // `at` es `a·t` cuando cada letra, sola, es un símbolo conocido
        // (como en Desmos). `sint` no: `s` no es nada, y el error lo dice.
        // Sólo nombres ASCII llegan aquí con más de un carácter (el
        // léxico parte cualquier otra letra en fichas de una), así que
        // cortar byte por byte es cortar letra por letra.
        if !n.is_ascii() || n.len() < 2 {
            return Err(Error::en(Codigo::NombreDesconocido, col));
        }
        let mut ops = Vec::with_capacity(n.len());
        for i in 0..n.len() {
            match self.simbolo(&n[i..=i]) {
                Some(op) => ops.push(op),
                None => return Err(Error::en(Codigo::NombreDesconocido, col)),
            }
        }
        // Todos los símbolos quedan en la pila; los productos que los
        // unen se emiten en `potencia`, después del exponente, que así
        // se pega sólo a la última letra.
        let pendientes = ops.len() - 1;
        self.ops.extend(ops);
        Ok(Atomo {
            es_e: n.ends_with('e'),
            pendientes,
        })
    }

    /// Una constante o una variable de las permitidas.
    fn simbolo(&self, n: &str) -> Option<Op> {
        constante(n).map(Op::Num).or_else(|| {
            self.vars.iter().position(|v| *v == n).map(Op::Var)
        })
    }
}

/// Lo que `potencia` necesita saber del átomo que acaba de leer.
#[derive(Clone, Copy)]
struct Atomo {
    /// Fue la constante `e` (sola o al final de un nombre partido).
    es_e: bool,
    /// Productos que faltan por emitir para un nombre partido.
    pendientes: usize,
}

impl Atomo {
    const SIMPLE: Atomo = Atomo {
        es_e: false,
        pendientes: 0,
    };
}

/// Las constantes con nombre. τ es la del círculo en esta casa; π sigue
/// aceptándose porque el lector lo va a escribir.
fn constante(n: &str) -> Option<f64> {
    use core::f64::consts::{E, PI, TAU};
    match n {
        "tau" | "τ" => Some(TAU),
        "pi" | "π" => Some(PI),
        "e" => Some(E),
        _ => None,
    }
}
