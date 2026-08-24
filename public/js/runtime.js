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
 *   [2, n, x0,y0, ... , style]      polyline (n points)
 *   [3, x1, y1, x2, y2, style]      arrow (runtime draws the head)
 *   [9, view]                       switch target view for what follows
 * Styles index lesson.json's palette. A lesson has one view (#scene,
 * lesson.world) or several (lesson.views: [{world, viewBox, uniform?}],
 * painted into #scene0..#sceneN). Geometry views must scale both axes
 * alike -- the skew guard throws, as always; a view may declare
 * "uniform": false ONLY where axes carry different quantities (a phase
 * portrait, a graph), which is a statement, not an escape hatch.
 */
"use strict";

/* A lesson that cannot start must SAY SO on the page. A dead stage with
 * live-looking prose is the one failure mode this lab cannot afford. */
function fatal(title, detail) {
  const d = document.createElement("div");
  d.setAttribute("role", "alert");
  d.style.cssText = "margin:16px auto;max-width:1060px;border:1px solid #e0322a;" +
    "background:rgba(224,50,42,.12);color:#ff4a40;padding:14px 18px;" +
    "font:500 13px/1.6 'JetBrains Mono',monospace;white-space:pre-wrap";
  d.textContent = "This lesson could not start.\n" + title +
    (detail ? "\n\n" + detail : "");
  document.body.prepend(d);
}

(async function () {
  const NS = "http://www.w3.org/2000/svg";

  if (location.protocol === "file:") {
    fatal("Opened as a file:// URL, where the browser blocks loading the " +
          "lesson's data and WebAssembly.",
          "Serve the site instead (any static server works):\n" +
          "    cd physics-lab/public && python3 -m http.server 8000\n" +
          "then open http://localhost:8000/");
    return;
  }

  const base = document.querySelector("script[data-lesson]").dataset.lesson;

  let lesson, wasm;
  try {
    lesson = await (await fetch(base + "/lesson.json")).json();
    const bytes = await (await fetch(base + "/lesson.wasm")).arrayBuffer();
    const { instance } = await WebAssembly.instantiate(bytes, {});
    wasm = instance.exports;
  } catch (e) {
    fatal("Loading the lesson failed: " + e,
          "If this is a Content-Security-Policy error, script-src needs " +
          "'wasm-unsafe-eval' (it is set in this site's _headers; a proxy " +
          "or different host may be overriding it).");
    return;
  }
  try {

  // ---- world <-> screen, per view (uniform unless declared otherwise) ---
  function mapping(world, vb, uniform, label) {
    const mx = vb.w / (world.x1 - world.x0), my = vb.h / (world.y1 - world.y0);
    if (uniform !== false && Math.abs(mx - my) > 1e-9 * Math.max(mx, my)) {
      throw new Error("runtime: non-uniform world scale in " + label +
        " (" + mx + " vs " + my + ")");
    }
    return {
      sx: x => (x - world.x0) * mx,
      sy: y => vb.h - (y - world.y0) * my,
      wx: p => world.x0 + p / mx,
    };
  }
  const views = lesson.views
    ? lesson.views.map((v, i) => ({
        map: mapping(v.world, v.viewBox, v.uniform, "view " + i),
        scene: document.getElementById("scene" + i),
      }))
    : [{
        map: mapping(lesson.world, lesson.viewBox, true, "the view"),
        scene: document.getElementById("scene"),
      }];

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
  const styles = lesson.styles;

  function paint(buf, n) {
    for (const v of views) v.scene.textContent = "";
    let view = views[0];
    let i = 0;
    while (i < n) {
      const tag = buf[i];
      const { sx, sy } = view.map;
      if (tag === 9) {
        view = views[buf[i + 1] | 0];
        i += 2;
      } else if (tag === 0) {
        const [x, y, st] = [buf[i + 1], buf[i + 2], buf[i + 3]];
        view.scene.appendChild(el("circle", { cx: sx(x), cy: sy(y), r: 4.5,
          fill: css(st) }));
        i += 4;
      } else if (tag === 2) {
        const cnt = buf[i + 1] | 0;
        const st = buf[i + 2 + 2 * cnt];
        const s = styles[st | 0];
        let pts = "";
        for (let j = 0; j < cnt; j++) {
          pts += (j ? " " : "") + sx(buf[i + 2 + 2 * j]).toFixed(2) + "," +
                 sy(buf[i + 3 + 2 * j]).toFixed(2);
        }
        view.scene.appendChild(el("polyline", { points: pts, fill: "none",
          stroke: css(st), "stroke-width": s.width ?? 2,
          "stroke-dasharray": s.dash ? "6 5" : "none",
          "stroke-linejoin": "round", "stroke-linecap": "round" }));
        i += 3 + 2 * cnt;
      } else if (tag === 1 || tag === 3) {
        const [x1, y1, x2, y2, st] = [buf[i+1], buf[i+2], buf[i+3], buf[i+4], buf[i+5]];
        const s = styles[st | 0];
        view.scene.appendChild(el("line", { x1: sx(x1), y1: sy(y1), x2: sx(x2), y2: sy(y2),
          stroke: css(st), "stroke-width": s.width ?? 2,
          "stroke-dasharray": s.dash ? "6 5" : "none", "stroke-linecap": "round" }));
        if (tag === 3) {
          const ang = Math.atan2(sy(y2) - sy(y1), sx(x2) - sx(x1));
          const L = 11, Wd = 4.5;
          const hx = sx(x2), hy = sy(y2);
          view.scene.appendChild(el("path", {
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
  } catch (e) {
    fatal("The lesson loaded but failed while starting: " + e,
          e && e.stack ? String(e.stack).split("\n").slice(0, 3).join("\n") : "");
  }
})();
