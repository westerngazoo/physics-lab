/* runtime.js -- THE one JS file (RFC-001 §4.0/§4.2), written once.
 *
 * Everything a lesson page needs from the browser: load the lesson's
 * wasm, generate controls and readouts from lesson.json, run the clock,
 * and paint the primitive buffer the Rust side fills. No lesson ever
 * ships its own JS; physics lives in Rust on garust, on the other side
 * of a flat f64 boundary.
 *
 * Prim records (motoreel's vocabulary as a convention):
 *   [0, x, y, style]                point
 *   [1, x1, y1, x2, y2, style]      segment
 *   [3, x1, y1, x2, y2, style]      arrow (runtime draws the head)
 * Styles index lesson.json's palette. Coordinates are world units; the
 * world box comes from lesson.json and must scale both axes alike -- a
 * skewed mapping throws at load, as always.
 */
"use strict";

(async function () {
  const NS = "http://www.w3.org/2000/svg";
  const TAU = 2 * Math.PI;

  const base = document.querySelector("script[data-lesson]").dataset.lesson;

  const lesson = await (await fetch(base + "/lesson.json")).json();
  const bytes = await (await fetch(base + "/lesson.wasm")).arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, {});
  const wasm = instance.exports;

  // ---- world <-> screen (uniform, or refuse to run) ----------------------
  const W = lesson.world, VB = lesson.viewBox;
  const px = VB.w / (W.x1 - W.x0), py = VB.h / (W.y1 - W.y0);
  if (Math.abs(px - py) > 1e-9 * Math.max(px, py)) {
    throw new Error("runtime: non-uniform world scale (" + px + " vs " + py + ")");
  }
  const sx = x => (x - W.x0) * px;
  const sy = y => VB.h - (y - W.y0) * py;

  // ---- controls generated from lesson.json -------------------------------
  const state = {};
  const updaters = [];
  const ctlBox = document.getElementById("controls");
  for (const [key, p] of Object.entries(lesson.params)) {
    state[key] = p.value;
    const wrap = document.createElement("div");
    const lab = document.createElement("label");
    lab.className = "k";
    lab.htmlFor = "p-" + key;
    lab.innerHTML = p.label + ' <span class="val" id="p-' + key + 'lab"></span>';
    const input = document.createElement("input");
    input.type = "range";
    input.id = "p-" + key;
    input.min = p.min; input.max = p.max; input.step = p.step; input.value = p.value;
    const out = lab.querySelector("span");
    const upd = () => {
      state[key] = +input.value;
      out.textContent = (+input.value).toFixed(p.digits ?? 2) + (p.unit ? " " + p.unit : "");
      draw();
    };
    input.addEventListener("input", () => { if (sweep) stopSweep(); upd(); });
    wrap.append(lab, input);
    ctlBox.appendChild(wrap);
    updaters.push(upd);
  }

  // ---- readouts generated from lesson.json -------------------------------
  const roBox = document.getElementById("readouts");
  const roEls = [];
  for (const r of lesson.readouts) {
    const d = document.createElement("div");
    d.className = "num" + (r.hero ? " hero" : "");
    d.innerHTML = "<dt>" + r.label + "</dt><dd>—</dd>";
    roBox.appendChild(d);
    roEls.push({ slot: r.slot, fmt: r.fmt, el: d.querySelector("dd") });
  }
  const FMT = {
    turns3: v => v.toFixed(3) + " τ",
    fix3: v => v.toFixed(3),
    sci: v => (v === 0 ? "0" : v.toExponential(1)),
  };

  // ---- painting ----------------------------------------------------------
  const scene = document.getElementById("scene");
  const styles = lesson.styles;

  function paint(buf, n) {
    scene.textContent = "";
    let i = 0;
    while (i < n) {
      const tag = buf[i];
      if (tag === 0) {
        const [x, y, st] = [buf[i + 1], buf[i + 2], buf[i + 3]];
        scene.appendChild(el("circle", { cx: sx(x), cy: sy(y), r: 4,
          fill: css(st) }));
        i += 4;
      } else if (tag === 1 || tag === 3) {
        const [x1, y1, x2, y2, st] = [buf[i+1], buf[i+2], buf[i+3], buf[i+4], buf[i+5]];
        const s = styles[st | 0];
        scene.appendChild(el("line", { x1: sx(x1), y1: sy(y1), x2: sx(x2), y2: sy(y2),
          stroke: css(st), "stroke-width": s.width ?? 2,
          "stroke-dasharray": s.dash ? "6 5" : "none", "stroke-linecap": "round" }));
        if (tag === 3) {
          const ang = Math.atan2(sy(y2) - sy(y1), sx(x2) - sx(x1));
          const L = 11, Wd = 4.5;
          const hx = sx(x2), hy = sy(y2);
          scene.appendChild(el("path", {
            d: "M " + hx + " " + hy +
               " L " + (hx - L * Math.cos(ang) + Wd * Math.sin(ang)) + " " +
                       (hy - L * Math.sin(ang) - Wd * Math.cos(ang)) +
               " L " + (hx - L * Math.cos(ang) - Wd * Math.sin(ang)) + " " +
                       (hy - L * Math.sin(ang) + Wd * Math.cos(ang)) + " Z",
            fill: css(st) }));
        }
        i += 6;
      } else {
        throw new Error("runtime: unknown prim tag " + tag);
      }
    }
  }
  function css(st) { return "var(" + styles[st | 0].var + ")"; }
  function el(name, attrs) {
    const e = document.createElementNS(NS, name);
    for (const k in attrs) e.setAttribute(k, attrs[k]);
    return e;
  }

  // ---- the frame: args in declared order, scaled ------------------------
  function draw() {
    const args = Object.entries(lesson.params)
      .map(([k, p]) => state[k] * (p.scale ?? 1));
    const n = wasm.state_at(...args);
    const prims = new Float64Array(wasm.memory.buffer, wasm.prims_ptr(), n);
    paint(prims, n);
    const rd = new Float64Array(wasm.memory.buffer, wasm.readouts_ptr(), 6);
    for (const r of roEls) r.el.textContent = (FMT[r.fmt] || FMT.fix3)(rd[r.slot]);
  }

  // ---- sweep clock (optional, from lesson.json) --------------------------
  let sweep = false, rate = 1, last = null;
  const sw = lesson.sweep;
  const btn = document.getElementById("play");
  function stopSweep() { sweep = false; if (btn) btn.textContent = sw.label; }
  if (sw && btn) {
    btn.addEventListener("click", () => {
      sweep = !sweep;
      btn.textContent = sweep ? "Pause" : sw.label;
    });
    document.querySelectorAll(".speed button").forEach(b =>
      b.addEventListener("click", () => {
        rate = +b.dataset.rate;
        document.querySelectorAll(".speed button").forEach(o =>
          o.setAttribute("aria-pressed", String(o === b)));
      }));
    (function tick(ts) {
      if (last === null) last = ts;
      const dt = Math.min(60, ts - last) / 1000; last = ts;
      if (sweep) {
        const p = lesson.params[sw.param];
        let v = state[sw.param] + sw.rate * dt * rate;
        const span = p.max - p.min;
        v = p.min + ((v - p.min) % span + span) % span;
        state[sw.param] = v;
        const input = document.getElementById("p-" + sw.param);
        input.value = v;
        document.getElementById("p-" + sw.param + "lab").textContent =
          v.toFixed(p.digits ?? 2) + (p.unit ? " " + p.unit : "");
        draw();
      }
      requestAnimationFrame(tick);
    })(performance.now());
  }

  for (const f of updaters) f();   // set labels; each ends in draw()
  draw();
})();
