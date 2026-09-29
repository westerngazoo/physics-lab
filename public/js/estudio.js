/* estudio.js -- the studio: record a lesson as a TAKE, and play it back.
 *
 * Loaded by runtime.js only when the URL carries ?estudio, and built ONLY
 * on the port (window.physicsLab): it never touches the lesson's wasm or
 * the runtime's internals. That is the design, not a limitation -- the
 * same port serves a headless renderer, guion, or a test, the way one
 * debug port serves any probe.
 *
 * A take records INPUTS, never pixels: {p, x} over time. Every frame is a
 * pure function of the inputs, so a take replays exactly -- at any frame
 * rate, any resolution, any aspect -- the way a MIDI file replays a
 * performance on any instrument.
 *
 * Format physics-lab/toma@1:
 *   { formato, leccion, wasm: "sha256:…", params: [names], expresiones: n,
 *     creada, duracion,
 *     eventos: [ {t: 0, p: [...], x: [...]},     full state at t = 0
 *                {t, p?: [...], x?: [...]},       only what changed
 *                {t, marca: n} ] }                a cue for the narration
 * The state at τ is the last p and the last x with t ≤ τ: a zero-order
 * hold, so the take is a function of time defined everywhere.
 */
export const FORMATO = "physics-lab/toma@1";

const igual = (a, b) => a.length === b.length && a.every((v, i) => v === b[i]);
const ahora = t0 => Math.round(performance.now() - t0) / 1000;

/** Checks a take against the lesson on this page; throws what is wrong. */
export function valida(toma, port) {
  if (!toma || toma.formato !== FORMATO) throw new Error("no es una toma " + FORMATO);
  if (toma.leccion !== port.leccion) {
    throw new Error("la toma es de «" + toma.leccion + "», esta página es «" + port.leccion + "»");
  }
  if (!Array.isArray(toma.params) || !igual(toma.params, port.params)) {
    throw new Error("la toma nombra otros parámetros: " + toma.params);
  }
  const ev = toma.eventos;
  if (!Array.isArray(ev) || !ev.length || ev[0].t !== 0 || !Array.isArray(ev[0].p)) {
    throw new Error("la toma no empieza con un estado completo en t = 0");
  }
  for (let i = 1; i < ev.length; i++) {
    if (!(ev[i].t >= ev[i - 1].t)) throw new Error("tiempos no crecientes en el evento " + i);
  }
  return toma;
}

/** Indexes a (valid) take for O(log n) lookups at any instant. */
export function prepara(toma) {
  const ps = [], xs = [], marcas = [];
  for (const e of toma.eventos) {
    if (e.p) ps.push([e.t, e.p]);
    if (e.x) xs.push([e.t, e.x]);
    if (e.marca !== undefined) marcas.push([e.t, e.marca]);
  }
  return { toma, ps, xs, marcas, duracion: toma.duracion };
}

function ultimo(arr, tau) {
  let lo = 0, hi = arr.length - 1;
  if (hi < 0) return undefined;
  while (lo < hi) {
    const mid = (lo + hi + 1) >> 1;
    if (arr[mid][0] <= tau) lo = mid; else hi = mid - 1;
  }
  return arr[lo][1];
}

/** The inputs at instant τ: zero-order hold over the events. */
export function estadoEn(prep, tau) {
  return { p: ultimo(prep.ps, tau), x: ultimo(prep.xs, tau) };
}

async function huella() {
  const base = document.querySelector("script[data-lesson]").dataset.lesson;
  const bytes = await (await fetch(base + "/lesson.wasm")).arrayBuffer();
  const d = new Uint8Array(await crypto.subtle.digest("SHA-256", bytes));
  return "sha256:" + Array.from(d, b => b.toString(16).padStart(2, "0")).join("");
}

function descarga(toma) {
  const blob = new Blob([JSON.stringify(toma)], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = "toma-" + toma.leccion + "-" + toma.creada.replace(/[:.]/g, "-") + ".json";
  document.body.append(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 4000);
}

export function monta(port, bar) {
  const wasm = huella();
  let rec = null;      // { t0, eventos, ultimo, marcas }
  let play = null;     // { prep, t0 }
  let cargada = null;  // the take cuadro() renders from

  const row = document.createElement("div");
  row.className = "estudio";
  row.innerHTML =
    "<label class='k'>Estudio</label>" +
    "<button class='rec' aria-pressed='false'>● Grabar toma</button>" +
    "<button class='marca' disabled>◆ Marca</button>" +
    "<button class='abre'>▶ Reproducir toma…</button>" +
    "<input type='file' accept='application/json,.json' hidden>" +
    "<span class='estado' aria-live='polite'></span>";
  bar.append(row);
  const [bRec, bMarca, bAbre] = row.querySelectorAll("button");
  const archivo = row.querySelector("input");
  const estado = row.querySelector(".estado");
  const dice = s => { estado.textContent = s; };
  dice("graba las entradas, no los píxeles: la toma se re-renderiza a cualquier resolución");

  // Every frame the runtime draws, the recorder keeps what changed.
  port.alCuadro(() => {
    if (!rec) return;
    const s = port.estado();
    const ev = { t: ahora(rec.t0) };
    if (!igual(s.p, rec.ultimo.p)) ev.p = s.p;
    if (!igual(s.x, rec.ultimo.x)) ev.x = s.x;
    if (ev.p || ev.x) {
      rec.eventos.push(ev);
      rec.ultimo = s;
    }
  });

  function grabar() {
    if (play) para();
    const s = port.estado();
    rec = { t0: performance.now(), eventos: [{ t: 0, p: s.p, x: s.x }], ultimo: s, marcas: 0 };
    bRec.textContent = "■ Detener y guardar";
    bRec.setAttribute("aria-pressed", "true");
    bMarca.disabled = false;
    bAbre.disabled = true;
    requestAnimationFrame(function reloj() {
      if (!rec) return;
      dice("● grabando " + ahora(rec.t0).toFixed(1) + " s · " + rec.eventos.length +
           " eventos · " + rec.marcas + " marcas (tecla M)");
      requestAnimationFrame(reloj);
    });
  }
  function marca() {
    if (!rec) return;
    rec.eventos.push({ t: ahora(rec.t0), marca: ++rec.marcas });
  }
  async function detener() {
    if (!rec) return null;
    const duracion = ahora(rec.t0);
    const toma = {
      formato: FORMATO, leccion: port.leccion, wasm: await wasm,
      params: port.params.slice(), expresiones: port.expresiones,
      creada: new Date().toISOString(), duracion, eventos: rec.eventos,
    };
    rec = null;
    bRec.textContent = "● Grabar toma";
    bRec.setAttribute("aria-pressed", "false");
    bMarca.disabled = true;
    bAbre.disabled = false;
    dice("toma de " + duracion.toFixed(1) + " s, " + toma.eventos.length + " eventos");
    return toma;
  }
  function cargar(toma) {
    cargada = prepara(valida(toma, port));
    return cargada.duracion;
  }
  function cuadro(t) {
    if (!cargada) throw new Error("primero cargar(toma)");
    return port.aplica(estadoEn(cargada, t));
  }
  function reproduce(toma) {
    cargar(toma);
    port.detenerReloj();
    play = { prep: cargada, t0: performance.now() };
    bRec.disabled = true;
    requestAnimationFrame(function paso() {
      if (!play) return;
      const tau = (performance.now() - play.t0) / 1000;
      port.aplica(estadoEn(play.prep, Math.min(tau, play.prep.duracion)));
      const m = play.prep.marcas.filter(([t]) => t <= tau).length;
      dice("▶ " + Math.min(tau, play.prep.duracion).toFixed(1) + " / " +
           play.prep.duracion.toFixed(1) + " s" + (m ? " · marca " + m : ""));
      if (tau >= play.prep.duracion) { para(); dice("fin de la toma"); return; }
      requestAnimationFrame(paso);
    });
  }
  function para() {
    play = null;
    bRec.disabled = false;
  }

  bRec.addEventListener("click", async () => {
    if (rec) descarga(await detener()); else grabar();
  });
  bMarca.addEventListener("click", marca);
  bAbre.addEventListener("click", () => archivo.click());
  archivo.addEventListener("change", async () => {
    const f = archivo.files[0];
    archivo.value = "";
    if (!f) return;
    try { reproduce(JSON.parse(await f.text())); }
    catch (e) { dice("no se pudo reproducir: " + e.message); }
  });
  // Touching any control takes the page back from the take.
  bar.addEventListener("pointerdown", e => {
    if (play && !e.target.closest(".estudio")) { para(); dice("reproducción interrumpida"); }
  }, true);
  bar.addEventListener("input", () => { if (play) { para(); dice("reproducción interrumpida"); } }, true);
  document.addEventListener("keydown", e => {
    const escribiendo = e.target instanceof HTMLInputElement && e.target.type === "text";
    if (rec && !escribiendo && (e.key === "m" || e.key === "M")) marca();
  });

  // For programs (a headless renderer, guion, a test): same verbs, no UI.
  window.physicsLabEstudio = Object.freeze({ formato: FORMATO, grabar, marca, detener, cargar, cuadro });
}
