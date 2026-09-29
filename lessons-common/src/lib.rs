//! lessons-common — the framework's Rust half.
//!
//! A lesson is ONE function:
//!
//! ```ignore
//! fn draw(p: &[f64], out: &mut Prims, read: &mut Readouts) { ... }
//! lessons_common::lesson!(draw);
//! ```
//!
//! The `lesson!` macro generates the entire wasm boundary — the params
//! buffer the runtime writes into, the primitive and readout buffers it
//! reads back, and the four C-ABI exports — identically for every
//! lesson. Uniform ABI, zero per-lesson plumbing, and `cargo test`
//! exercises the same `draw` the browser calls.
//!
//! A lesson that reads what the student TYPES (a formula, Desmos-style)
//! declares itself with `lesson!(draw, texto)` and also receives the
//! text, plus a [`Diagnosticos`] writer to report where it stopped
//! making sense:
//!
//! ```ignore
//! fn draw(p: &[f64], texto: &str, out: &mut Prims, read: &mut Readouts, diag: &mut Diagnosticos) { ... }
//! lessons_common::lesson!(draw, texto);
//! ```
//!
//! The text is parsed by the `formulas` crate, never executed.
//!
//! Prim records (motoreel's vocabulary as a convention):
//!   [0, x, y, style]                 point
//!   [1, x1, y1, x2, y2, style]       segment
//!   [2, n, x0,y0, ..., style]        polyline
//!   [3, x1, y1, x2, y2, style]       arrow
//!   [4, x, y, idx, style]            label: `lesson.labels[idx]`
//!   [5, x, y, value, decimals, style] a number, formatted by the page
//!   [9, view]                        switch target view
//!
//! Text goes in as an INDEX, never as characters. The buffer is f64s and
//! it stays f64s: the strings live in `lesson.json`, where they are UTF-8
//! and the page renders them with a real font. That is not a workaround,
//! it is the only version that can write `ángulo`, `tensión` or `30°` —
//! a rasteriser with a bitmap table cannot, and the one in the video
//! pipeline still cannot.

/// Primitive-buffer capacity, in f64s.
pub const PRIM_CAP: usize = 8192;
/// Number of readout slots.
pub const READ_SLOTS: usize = 8;
/// Maximum parameters a lesson can declare.
pub const PARAM_CAP: usize = 16;

// ---- el texto: la única entrada que no es un número ----------------------
//
// Una lección declarada con `lesson!(draw, texto)` recibe, además de los
// parámetros, lo que el lector escribió en las cajas de fórmula de la
// página: UTF-8, un renglón por caja, separados por '\n'. Es la misma
// regla que para los rótulos, al revés: el texto ENTRA a la lección como
// bytes que ella analiza (con `formulas`), y lo que SALE hacia la página
// siguen siendo sólo f64 — códigos y columnas, nunca cadenas.

/// Capacidad del buffer de texto, en bytes UTF-8.
pub const TEXT_CAP: usize = 2048;
/// Renglones de diagnóstico: uno por caja de fórmula.
pub const DIAG_SLOTS: usize = 8;
/// f64 por renglón de diagnóstico: `[código, columna, máscara]`.
pub const DIAG_ANCHO: usize = 3;

/// El prefijo UTF-8 válido de `bytes`.
///
/// La página escribe UTF-8 bien formado, pero la lección no se fía: si
/// el buffer llegara cortado a media letra (una `τ` partida en su primer
/// byte), se lee hasta la última letra entera en vez de tirar todo.
#[must_use]
pub fn texto_valido(bytes: &[u8]) -> &str {
    match core::str::from_utf8(bytes) {
        Ok(s) => s,
        Err(e) => core::str::from_utf8(&bytes[..e.valid_up_to()]).unwrap_or_default(),
    }
}

/// Lo que la lección le dice a la página sobre cada caja de fórmula.
///
/// Por renglón, `[código, columna, máscara]`: código 0 es "se leyó bien";
/// cualquier otro es un `formulas::Codigo`, con la columna (en
/// caracteres, desde 1) donde mirar. La máscara dice qué PARÁMETROS (bit
/// k = el k-ésimo del manifiesto) nombra la fórmula — con eso la página
/// muestra el deslizador de `a` sólo cuando alguien escribió una `a`.
pub struct Diagnosticos<'a> {
    buf: &'a mut [f64],
}

impl<'a> Diagnosticos<'a> {
    /// Toma el buffer y lo deja en cero: cada cuadro empieza sin errores.
    pub fn new(buf: &'a mut [f64]) -> Self {
        buf.fill(0.0);
        Diagnosticos { buf }
    }
    /// El renglón `linea` no se pudo leer.
    pub fn error(&mut self, linea: usize, codigo: u8, columna: usize) {
        self.pon(linea, [f64::from(codigo), columna as f64, 0.0]);
    }
    /// El renglón `linea` se leyó bien y nombra los parámetros `mascara`.
    pub fn ok(&mut self, linea: usize, mascara: u64) {
        self.pon(linea, [0.0, 0.0, mascara as f64]);
    }
    fn pon(&mut self, linea: usize, rec: [f64; DIAG_ANCHO]) {
        let i = linea * DIAG_ANCHO;
        if let Some(dst) = self.buf.get_mut(i..i + DIAG_ANCHO) {
            dst.copy_from_slice(&rec);
        }
    }
}

pub mod dcl;

/// Recorre el buffer plano y devuelve cada registro como `(tag, campos)`.
///
/// Existe para que las pruebas puedan preguntar "¿cuántas flechas hay?"
/// sin contar f64 sueltos. Contar `3.0` a pelo en el buffer cuenta
/// también las coordenadas que valen tres, y esa prueba pasa o falla por
/// accidente — cosa que se descubrió escribiéndola.
///
/// # Panics
/// Si encuentra una etiqueta que no conoce: un buffer mal formado es un
/// error del que lo escribió, no algo que valga la pena tolerar.
#[must_use]
pub fn recorre(buf: &[f64], n: usize) -> Vec<(usize, &[f64])> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < n {
        let tag = buf[i] as usize;
        let largo = match tag {
            0 => 4,
            1 | 3 | 5 => 6,
            2 => 3 + 2 * (buf[i + 1] as usize),
            4 => 5,
            9 => 2,
            otro => panic!("registro desconocido: {otro}"),
        };
        out.push((tag, &buf[i..i + largo]));
        i += largo;
    }
    out
}

/// Cuántos registros de esa etiqueta trae el buffer.
#[must_use]
pub fn cuenta(buf: &[f64], n: usize, tag: usize) -> usize {
    recorre(buf, n).iter().filter(|(t, _)| *t == tag).count()
}

/// Typed writer over the flat primitive buffer.
pub struct Prims<'a> {
    buf: &'a mut [f64],
    at: usize,
}

impl<'a> Prims<'a> {
    pub fn new(buf: &'a mut [f64]) -> Self {
        Prims { buf, at: 0 }
    }
    pub fn len(&self) -> usize {
        self.at
    }
    pub fn is_empty(&self) -> bool {
        self.at == 0
    }
    fn push(&mut self, rec: &[f64]) {
        self.buf[self.at..self.at + rec.len()].copy_from_slice(rec);
        self.at += rec.len();
    }
    /// Route subsequent primitives to view `v` (index into lesson.json views).
    pub fn view(&mut self, v: usize) {
        self.push(&[9.0, v as f64]);
    }
    pub fn point(&mut self, x: f64, y: f64, style: usize) {
        self.push(&[0.0, x, y, style as f64]);
    }
    pub fn segment(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, style: usize) {
        self.push(&[1.0, x1, y1, x2, y2, style as f64]);
    }
    pub fn arrow(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, style: usize) {
        self.push(&[3.0, x1, y1, x2, y2, style as f64]);
    }
    /// A label from the page's `labels` array, pinned at `(x, y)`.
    ///
    /// The lesson says WHICH label and WHERE; the page says what it
    /// reads. A force arrow without a name is not a free-body diagram.
    pub fn label(&mut self, x: f64, y: f64, idx: usize, style: usize) {
        self.push(&[4.0, x, y, idx as f64, style as f64]);
    }
    /// A live number at `(x, y)`, with `decimals` places.
    ///
    /// Separate from [`Prims::label`] because a magnitude changes with
    /// the parameters and a name does not. Keeping them apart is what
    /// lets the page hold every string while the physics still writes
    /// its own numbers on screen.
    pub fn numero(&mut self, x: f64, y: f64, valor: f64, decimals: usize, style: usize) {
        self.push(&[5.0, x, y, valor, decimals as f64, style as f64]);
    }
    pub fn polyline<I: IntoIterator<Item = (f64, f64)>>(&mut self, pts: I, style: usize) {
        let start = self.at;
        self.push(&[2.0, 0.0]);
        let mut n = 0usize;
        for (x, y) in pts {
            self.push(&[x, y]);
            n += 1;
        }
        self.push(&[style as f64]);
        self.buf[start + 1] = n as f64;
    }
    /// Sample a function over `[t0, t1]` into a polyline — the workhorse
    /// for trajectories, graphs and level sets.
    pub fn curve(
        &mut self,
        t0: f64,
        t1: f64,
        n: usize,
        style: usize,
        f: impl Fn(f64) -> (f64, f64),
    ) {
        self.polyline(
            (0..=n).map(|i| f(t0 + (t1 - t0) * i as f64 / n as f64)),
            style,
        );
    }
}

/// Typed writer over the readout slots.
pub struct Readouts<'a> {
    buf: &'a mut [f64],
}

impl<'a> Readouts<'a> {
    pub fn new(buf: &'a mut [f64]) -> Self {
        Readouts { buf }
    }
    pub fn set(&mut self, slot: usize, v: f64) {
        self.buf[slot] = v;
    }
}

/// Generate the wasm boundary for a lesson `draw` function.
///
/// Two shapes, one ABI:
///
/// - `lesson!(draw)` — `fn draw(p: &[f64], out: &mut Prims, read: &mut Readouts)`.
///   Every lesson before formulas; its expansion is unchanged.
/// - `lesson!(draw, texto)` — the same exports PLUS the text surface
///   (`text_ptr`, `text_cap`, `set_text_len`, `diag_ptr`), for a lesson
///   that reads what the student types:
///   `fn draw(p: &[f64], texto: &str, out: &mut Prims, read: &mut Readouts, diag: &mut Diagnosticos)`.
///
/// `state_at(n_params)` keeps its signature in both, so a host that only
/// knows the closed-form ABI (guion's `WasmLesson`) still drives either.
#[macro_export]
macro_rules! lesson {
    ($draw:path) => {
        $crate::__lesson_buffers!();

        /// Recompute everything from the current params. Pure over them.
        ///
        /// # Safety
        /// Single-threaded wasm; the statics have exactly this writer.
        #[no_mangle]
        pub extern "C" fn state_at(n_params: usize) -> usize {
            unsafe {
                let all: &[f64; $crate::PARAM_CAP] = &*core::ptr::addr_of!(LESSON_PARAMS);
                let p = &all[..n_params];
                let mut prims = $crate::Prims::new(&mut *core::ptr::addr_of_mut!(LESSON_PRIMS));
                let mut read = $crate::Readouts::new(&mut *core::ptr::addr_of_mut!(LESSON_READ));
                $draw(p, &mut prims, &mut read);
                prims.len()
            }
        }
    };
    ($draw:path, texto) => {
        $crate::__lesson_buffers!();

        static mut LESSON_TEXT: [u8; $crate::TEXT_CAP] = [0; $crate::TEXT_CAP];
        static mut LESSON_TEXT_LEN: usize = 0;
        static mut LESSON_DIAG: [f64; $crate::DIAG_SLOTS * $crate::DIAG_ANCHO] =
            [0.0; $crate::DIAG_SLOTS * $crate::DIAG_ANCHO];

        /// Where the runtime writes the student's text, UTF-8.
        #[no_mangle]
        pub extern "C" fn text_ptr() -> *mut u8 {
            core::ptr::addr_of_mut!(LESSON_TEXT) as *mut u8
        }
        /// How many bytes fit — the page asks instead of hard-coding it.
        #[no_mangle]
        pub extern "C" fn text_cap() -> usize {
            $crate::TEXT_CAP
        }
        /// How many of them the runtime wrote. Clamped: a lying host
        /// cannot make the lesson read past its buffer.
        ///
        /// # Safety
        /// Single-threaded wasm; the static has exactly this writer.
        #[no_mangle]
        pub extern "C" fn set_text_len(n: usize) {
            unsafe {
                *core::ptr::addr_of_mut!(LESSON_TEXT_LEN) = n.min($crate::TEXT_CAP);
            }
        }
        /// Per-line diagnostics, `[code, column, mask]` × `DIAG_SLOTS`.
        #[no_mangle]
        pub extern "C" fn diag_ptr() -> *const f64 {
            core::ptr::addr_of!(LESSON_DIAG) as *const f64
        }
        /// Recompute everything from the current params AND text. Pure
        /// over both: the same numbers and the same words give the same
        /// buffers, bit for bit.
        ///
        /// # Safety
        /// Single-threaded wasm; the statics have exactly this writer.
        #[no_mangle]
        pub extern "C" fn state_at(n_params: usize) -> usize {
            unsafe {
                let all: &[f64; $crate::PARAM_CAP] = &*core::ptr::addr_of!(LESSON_PARAMS);
                let p = &all[..n_params.min($crate::PARAM_CAP)];
                let bytes: &[u8; $crate::TEXT_CAP] = &*core::ptr::addr_of!(LESSON_TEXT);
                let len = (*core::ptr::addr_of!(LESSON_TEXT_LEN)).min($crate::TEXT_CAP);
                let texto = $crate::texto_valido(&bytes[..len]);
                let mut prims = $crate::Prims::new(&mut *core::ptr::addr_of_mut!(LESSON_PRIMS));
                let mut read = $crate::Readouts::new(&mut *core::ptr::addr_of_mut!(LESSON_READ));
                let mut diag =
                    $crate::Diagnosticos::new(&mut *core::ptr::addr_of_mut!(LESSON_DIAG));
                $draw(p, texto, &mut prims, &mut read, &mut diag);
                prims.len()
            }
        }
    };
}

/// The three buffers and pointer exports every lesson shares, generated
/// ONCE here so the two `lesson!` shapes cannot drift apart.
#[doc(hidden)]
#[macro_export]
macro_rules! __lesson_buffers {
    () => {
        static mut LESSON_PARAMS: [f64; $crate::PARAM_CAP] = [0.0; $crate::PARAM_CAP];
        static mut LESSON_PRIMS: [f64; $crate::PRIM_CAP] = [0.0; $crate::PRIM_CAP];
        static mut LESSON_READ: [f64; $crate::READ_SLOTS] = [0.0; $crate::READ_SLOTS];

        /// Where the runtime writes the parameter values, in manifest order.
        #[no_mangle]
        pub extern "C" fn params_ptr() -> *mut f64 {
            core::ptr::addr_of_mut!(LESSON_PARAMS) as *mut f64
        }
        /// The primitive buffer (`state_at`'s return is its length).
        #[no_mangle]
        pub extern "C" fn prims_ptr() -> *const f64 {
            core::ptr::addr_of!(LESSON_PRIMS) as *const f64
        }
        /// The readout slots.
        #[no_mangle]
        pub extern "C" fn readouts_ptr() -> *const f64 {
            core::ptr::addr_of!(LESSON_READ) as *const f64
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    // El brazo de texto, expandido de verdad y manejado como lo maneja
    // la página: escribir bytes, fijar el largo, llamar a state_at, leer.
    fn eco(p: &[f64], texto: &str, out: &mut Prims, read: &mut Readouts, diag: &mut Diagnosticos) {
        read.set(0, p.len() as f64);
        read.set(1, texto.chars().count() as f64);
        out.point(0.0, 0.0, 0);
        for (i, linea) in texto.split('\n').enumerate() {
            if linea.contains('#') {
                diag.error(i, 2, linea.find('#').unwrap_or(0) + 1);
            } else {
                diag.ok(i, 0b101);
            }
        }
    }
    lesson!(eco, texto);

    fn escribe(s: &[u8]) {
        let cap = text_cap();
        let dst = unsafe { core::slice::from_raw_parts_mut(text_ptr(), cap) };
        dst[..s.len()].copy_from_slice(s);
        set_text_len(s.len());
    }

    #[test]
    fn el_texto_cruza_el_abi_y_los_diagnosticos_regresan() {
        escribe("tτ\nuno#".as_bytes());
        let n = state_at(3);
        assert_eq!(n, 4); // un punto
        let read = unsafe { core::slice::from_raw_parts(readouts_ptr(), READ_SLOTS) };
        assert_eq!(read[0], 3.0);
        assert_eq!(read[1], 7.0); // "tτ\nuno#": 7 caracteres, no 8 bytes
        let d = unsafe { core::slice::from_raw_parts(diag_ptr(), DIAG_SLOTS * DIAG_ANCHO) };
        assert_eq!(&d[0..3], &[0.0, 0.0, 5.0]);
        assert_eq!(&d[3..6], &[2.0, 4.0, 0.0]);
        // Un largo mentiroso se recorta a la capacidad, no lee de más.
        set_text_len(usize::MAX);
        let _ = state_at(0);
    }

    #[test]
    fn texto_cortado_a_media_letra() {
        let b = "tτ".as_bytes(); // [0x74, 0xCF, 0x84]
        assert_eq!(texto_valido(&b[..2]), "t");
        assert_eq!(texto_valido(b), "tτ");
    }
}
