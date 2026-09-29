//! formulas — lo que el lector escribe, convertido en algo que se puede
//! evaluar, derivar y dibujar.
//!
//! Hasta aquí una lección sólo recibía números: la posición de cada
//! deslizador. Este crate es lo que le permite recibir **matemática**:
//! el lector escribe `x(t) = 3t² − 0.2t³` y la lección mueve un coche con
//! eso, como en Desmos, pero con la física del lab detrás.
//!
//! Tres reglas, porque ésta es la única pieza del lab que recibe entrada
//! arbitraria de un desconocido:
//!
//! 1. **Se analiza, nunca se ejecuta.** El texto se convierte en un
//!    programa de una docena de operaciones aritméticas; no hay `eval`,
//!    no hay JavaScript, no hay nombres que el crate no conozca.
//! 2. **Nada entra sin ubicarse.** Todo error dice QUÉ pasó y EN QUÉ
//!    COLUMNA ([`Error`]), porque "fórmula inválida" no le enseña nada a
//!    nadie.
//! 3. **Ninguna entrada puede tumbar la página.** Un pánico en wasm es
//!    una trampa, y una trampa es una lección muerta. El anidamiento está
//!    acotado, la pila del evaluador también, y una [`Formula`] sólo
//!    existe si ya se comprobó que no puede desbordarla.
//!
//! Las derivadas salen EXACTAS, por diferenciación automática: ver
//! [`Jet`]. La secante, que es la definición, se queda para la lección,
//! como aproximación con su error en pantalla.

mod jet;
mod lexico;
mod sintaxis;

pub use jet::Jet;

/// Tamaño de la pila del evaluador, en jets. Una [`Formula`] que la
/// necesitara más profunda no se compila.
pub const PILA_MAX: usize = 64;

/// Qué salió mal. Los números son parte del ABI: viajan por el buffer de
/// diagnósticos hasta la página, que tiene la tabla de mensajes (y una
/// prueba que la compara con [`Codigo::mensaje`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Codigo {
    /// No hay nada escrito.
    Vacia = 1,
    /// Un carácter que no es parte del lenguaje (`#`, `[`, `@`…).
    CaracterInesperado = 2,
    /// Un nombre que no es función, constante ni variable permitida.
    NombreDesconocido = 3,
    /// Un paréntesis que se abrió y nunca se cerró.
    FaltaParentesis = 4,
    /// Un paréntesis que cierra algo que nadie abrió.
    ParentesisDeMas = 5,
    /// Se esperaba un valor (`2+`, `*3`, `sin()`).
    FaltaOperando = 6,
    /// `sin t`: las funciones llevan paréntesis.
    FuncionSinParentesis = 7,
    /// `1.2.3`, `.`, o un número que no cabe en un f64.
    NumeroMalFormado = 8,
    /// Un número pegado después de algo (`t2`).
    NumeroSuelto = 9,
    /// Más anidamiento del que la pila admite.
    DemasiadoProfunda = 10,
    /// `0,5`: el separador decimal es el punto.
    Coma = 11,
}

impl Codigo {
    /// Todos, en orden: la página y las pruebas iteran sobre esto.
    pub const TODOS: [Codigo; 11] = [
        Codigo::Vacia,
        Codigo::CaracterInesperado,
        Codigo::NombreDesconocido,
        Codigo::FaltaParentesis,
        Codigo::ParentesisDeMas,
        Codigo::FaltaOperando,
        Codigo::FuncionSinParentesis,
        Codigo::NumeroMalFormado,
        Codigo::NumeroSuelto,
        Codigo::DemasiadoProfunda,
        Codigo::Coma,
    ];

    /// El mensaje, tal como lo pinta la página.
    #[must_use]
    pub fn mensaje(self) -> &'static str {
        match self {
            Codigo::Vacia => "escribe una fórmula",
            Codigo::CaracterInesperado => "ese carácter no es parte de una fórmula",
            Codigo::NombreDesconocido => "no conozco ese nombre",
            Codigo::FaltaParentesis => "este paréntesis nunca se cierra",
            Codigo::ParentesisDeMas => "este paréntesis cierra algo que nadie abrió",
            Codigo::FaltaOperando => "aquí falta un valor",
            Codigo::FuncionSinParentesis => "las funciones llevan paréntesis: sin(t)",
            Codigo::NumeroMalFormado => "ese número está mal escrito",
            Codigo::NumeroSuelto => "número pegado: ¿quisiste ^ o ·?",
            Codigo::DemasiadoProfunda => "demasiados paréntesis anidados",
            Codigo::Coma => "el decimal va con punto: 0.5",
        }
    }
}

/// Un error, con la columna (en caracteres, desde 1) donde mirar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Error {
    pub codigo: Codigo,
    pub columna: usize,
}

impl Error {
    pub(crate) fn en(codigo: Codigo, columna: usize) -> Self {
        Error { codigo, columna }
    }
}

/// Las funciones del lenguaje. Todas de una variable, todas con derivada
/// conocida en forma cerrada (ver [`Jet`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Funcion {
    Sin,
    Cos,
    Tan,
    Exp,
    Ln,
    Log,
    Sqrt,
    Abs,
    Sinh,
    Cosh,
    Tanh,
    Atan,
    Asin,
    Acos,
}

impl Funcion {
    /// `sen` también: así se escribe en español.
    fn por_nombre(n: &str) -> Option<Funcion> {
        Some(match n {
            "sin" | "sen" => Funcion::Sin,
            "cos" => Funcion::Cos,
            "tan" | "tg" => Funcion::Tan,
            "exp" => Funcion::Exp,
            "ln" => Funcion::Ln,
            "log" => Funcion::Log,
            "sqrt" | "raiz" => Funcion::Sqrt,
            "abs" => Funcion::Abs,
            "sinh" | "senh" => Funcion::Sinh,
            "cosh" => Funcion::Cosh,
            "tanh" => Funcion::Tanh,
            "atan" | "arctan" => Funcion::Atan,
            "asin" | "arcsin" | "arcsen" => Funcion::Asin,
            "acos" | "arccos" => Funcion::Acos,
            _ => return None,
        })
    }

    fn aplica(self, x: Jet) -> Jet {
        match self {
            Funcion::Sin => x.sin(),
            Funcion::Cos => x.cos(),
            Funcion::Tan => x.tan(),
            Funcion::Exp => x.exp(),
            Funcion::Ln => x.ln(),
            Funcion::Log => x.log10(),
            Funcion::Sqrt => x.sqrt(),
            Funcion::Abs => x.abs(),
            Funcion::Sinh => x.sinh(),
            Funcion::Cosh => x.cosh(),
            Funcion::Tanh => x.tanh(),
            Funcion::Atan => x.atan(),
            Funcion::Asin => x.asin(),
            Funcion::Acos => x.acos(),
        }
    }
}

/// Una instrucción del programa en notación polaca inversa.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Op {
    Num(f64),
    /// El índice en la lista de variables con que se compiló.
    Var(usize),
    Negativo,
    Suma,
    Resta,
    Producto,
    Cociente,
    Potencia,
    Funcion(Funcion),
}

impl Op {
    /// Cuántos valores toma de la pila. Todas dejan exactamente uno.
    fn aridad(self) -> usize {
        match self {
            Op::Num(_) | Op::Var(_) => 0,
            Op::Negativo | Op::Funcion(_) => 1,
            Op::Suma | Op::Resta | Op::Producto | Op::Cociente | Op::Potencia => 2,
        }
    }
}

/// Una fórmula compilada, lista para evaluarse miles de veces por cuadro.
///
/// Es un **invariante encapsulado**, como `Motor` en garust: los campos
/// son privados porque su significado es una promesa — este programa
/// nunca vacía la pila de más, nunca pasa de [`PILA_MAX`], y termina con
/// exactamente un valor. La única manera de obtener una es [`compila`],
/// que lo verifica; por eso evaluarla no puede fallar.
#[derive(Clone, Debug, PartialEq)]
pub struct Formula {
    ops: Vec<Op>,
}

/// Compila `fuente` sobre las variables `vars` (por ejemplo
/// `["t", "a", "b", "c"]`). Los índices de `vars` son los que después se
/// usan para darles valor en [`Formula::jet`].
///
/// # Errors
/// Cualquier texto que no sea una fórmula completa, con su [`Codigo`] y
/// la columna donde mirar.
pub fn compila(fuente: &str, vars: &[&str]) -> Result<Formula, Error> {
    if fuente.trim().is_empty() {
        return Err(Error::en(Codigo::Vacia, 1));
    }
    let fichas = lexico::fichas(fuente)?;
    let mut s = sintaxis::Sintaxis::new(&fichas, vars);
    s.completa()?;
    let ops = s.ops;
    // La promesa de `Formula`, comprobada en vez de supuesta. El análisis
    // sintáctico ya la cumple; esto es la segunda llave, porque de ella
    // depende que `jet` pueda indexar su pila sin preguntar.
    let mut alto = 0usize;
    for op in &ops {
        let toma = op.aridad();
        if alto < toma {
            return Err(Error::en(Codigo::FaltaOperando, 1));
        }
        alto = alto - toma + 1;
        if alto > PILA_MAX {
            return Err(Error::en(Codigo::DemasiadoProfunda, 1));
        }
    }
    if alto != 1 {
        return Err(Error::en(Codigo::FaltaOperando, 1));
    }
    Ok(Formula { ops })
}

impl Formula {
    /// El valor y sus dos derivadas respecto a la variable `respecto`,
    /// con las variables valiendo `vals` (en el orden de `vars`).
    ///
    /// Una variable sin valor en `vals` vale `NaN`: la curva sale con un
    /// hueco en vez de inventarse un cero.
    #[must_use]
    pub fn jet(&self, vals: &[f64], respecto: usize) -> Jet {
        let mut pila = [Jet::constante(0.0); PILA_MAX];
        let mut n = 0usize;
        for op in &self.ops {
            match *op {
                Op::Num(c) => {
                    pila[n] = Jet::constante(c);
                    n += 1;
                }
                Op::Var(k) => {
                    let x = vals.get(k).copied().unwrap_or(f64::NAN);
                    pila[n] = if k == respecto {
                        Jet::variable(x)
                    } else {
                        Jet::constante(x)
                    };
                    n += 1;
                }
                Op::Negativo => pila[n - 1] = -pila[n - 1],
                Op::Funcion(f) => pila[n - 1] = f.aplica(pila[n - 1]),
                binaria => {
                    n -= 1;
                    let (a, b) = (pila[n - 1], pila[n]);
                    pila[n - 1] = match binaria {
                        Op::Suma => a + b,
                        Op::Resta => a - b,
                        Op::Producto => a * b,
                        Op::Cociente => a / b,
                        _ => a.potencia(b),
                    };
                }
            }
        }
        pila[0]
    }

    /// Sólo el valor.
    #[must_use]
    pub fn valor(&self, vals: &[f64]) -> f64 {
        self.jet(vals, usize::MAX).v
    }

    /// Qué variables nombra, como máscara de bits sobre sus índices en
    /// `vars`. Es lo que deja a la página mostrar el deslizador de `a`
    /// sólo cuando alguien escribió una `a`.
    #[must_use]
    pub fn mascara(&self) -> u64 {
        self.ops.iter().fold(0, |m, op| match *op {
            Op::Var(k) if k < 64 => m | (1 << k),
            _ => m,
        })
    }
}

#[cfg(test)]
mod tests;
