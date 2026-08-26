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
//! Prim records (motoreel's vocabulary as a convention):
//!   [0, x, y, style]                 point
//!   [1, x1, y1, x2, y2, style]       segment
//!   [2, n, x0,y0, ..., style]        polyline
//!   [3, x1, y1, x2, y2, style]       arrow
//!   [9, view]                        switch target view

/// Primitive-buffer capacity, in f64s.
pub const PRIM_CAP: usize = 8192;
/// Number of readout slots.
pub const READ_SLOTS: usize = 8;
/// Maximum parameters a lesson can declare.
pub const PARAM_CAP: usize = 16;

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
    pub fn curve(&mut self, t0: f64, t1: f64, n: usize, style: usize, f: impl Fn(f64) -> (f64, f64)) {
        self.polyline((0..=n).map(|i| f(t0 + (t1 - t0) * i as f64 / n as f64)), style);
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
#[macro_export]
macro_rules! lesson {
    ($draw:path) => {
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
        /// Recompute everything from the current params. Pure over them.
        ///
        /// # Safety
        /// Single-threaded wasm; the statics have exactly this writer.
        #[no_mangle]
        pub extern "C" fn state_at(n_params: usize) -> usize {
            unsafe {
                let all: &[f64; $crate::PARAM_CAP] =
                    &*core::ptr::addr_of!(LESSON_PARAMS);
                let p = &all[..n_params];
                let mut prims =
                    $crate::Prims::new(&mut *core::ptr::addr_of_mut!(LESSON_PRIMS));
                let mut read =
                    $crate::Readouts::new(&mut *core::ptr::addr_of_mut!(LESSON_READ));
                $draw(p, &mut prims, &mut read);
                prims.len()
            }
        }
    };
}
