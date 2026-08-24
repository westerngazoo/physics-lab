const NS = "http://www.w3.org/2000/svg";
const scene = document.getElementById("scene");
const svg = document.getElementById("svg");

// ---- world <-> screen -------------------------------------------------
const W = { x0: -9, x1: 9, y0: -3.6, y1: 3.6 };   // 18 x 7.2 == 1000 x 400, so x and y scale alike
const VB = { w: 1000, h: 400 };
const sx = x => (x - W.x0) / (W.x1 - W.x0) * VB.w;
const sy = y => VB.h - (y - W.y0) / (W.y1 - W.y0) * VB.h;
const wx = px => W.x0 + px / VB.w * (W.x1 - W.x0);

const H = 1.0;              // object height, world units
const NO_IMAGE = 0.055;     // |do - f| below this: rays emerge parallel
const state = { do: 6, f: 2, converging: true, playing: false, dir: -1 };

// ---- optics ------------------------------------------------------------
function solve(dobj, f) {
  const den = dobj - f;
  if (Math.abs(den) < NO_IMAGE) return { none: true };
  const di = dobj * f / den;
  return { none: false, di, m: -di / dobj, real: di > 0 };
}

/** The three principal rays, in world coordinates. */
function rays(dobj, f, s) {
  const tip = [-dobj, H];
  const R = W.x1;                       // right edge
  const out = [];

  // 1 \u2014 parallel in, refracted along the line through F' at (f, 0)
  const d1 = f > 0 ? [f, -H] : [-f, H];
  out.push({ c: "r1", inc: [tip, [0, H]], refr: [[0, H], [R, H + (R / d1[0]) * d1[1]]], exit: [0, H] });

  // 2 \u2014 straight through the optical centre, undeviated
  out.push({ c: "r2", inc: [tip, [0, 0]], refr: [[0, 0], [R, -H * R / dobj]], exit: [0, 0] });

  // 3 \u2014 through F, emerging parallel to the axis.
  // Degenerate exactly at do = f (the incident ray never reaches the lens).
  if (!s.none) {
    const y3 = -H * f / (dobj - f);
    if (Math.abs(y3) < 3.45) {
      out.push({ c: "r3", inc: [tip, [0, y3]], refr: [[0, y3], [R, y3]], exit: [0, y3] });
    }
  }
  return out;
}

// ---- drawing helpers ---------------------------------------------------
function el(name, attrs) {
  const e = document.createElementNS(NS, name);
  for (const k in attrs) e.setAttribute(k, attrs[k]);
  return e;
}
function line(a, b, cls, dashed, width) {
  return el("line", {
    x1: sx(a[0]), y1: sy(a[1]), x2: sx(b[0]), y2: sy(b[1]),
    stroke: `var(--ob-${cls})`, "stroke-width": width || 1.8,
    "stroke-dasharray": dashed ? "6 5" : "none",
    "stroke-linecap": "round", opacity: dashed ? 0.72 : 1
  });
}
function label(x, y, text, cls, anchor, size) {
  const t = el("text", {
    x: sx(x), y: sy(y), fill: `var(--ob-${cls})`, "text-anchor": anchor || "middle",
    "font-family": "IBM Plex Mono, monospace", "font-size": size || 12.5, "font-weight": 500
  });
  t.textContent = text;
  return t;
}
function arrow(x, height, cls, dashed) {
  const g = el("g", {});
  const up = height >= 0;
  g.appendChild(line([x, 0], [x, height], cls, dashed, 3));
  const hw = 0.17, hl = 0.34 * (up ? 1 : -1);
  g.appendChild(el("path", {
    d: `M ${sx(x)} ${sy(height)} L ${sx(x - hw)} ${sy(height - hl)} L ${sx(x + hw)} ${sy(height - hl)} Z`,
    fill: `var(--ob-${cls})`, opacity: dashed ? 0.72 : 1
  }));
  return g;
}

// ---- render ------------------------------------------------------------
function render() {
  const f = state.converging ? state.f : -state.f;
  const dobj = state.do;
  const s = solve(dobj, f);
  scene.textContent = "";

  // optical axis
  scene.appendChild(line([W.x0, 0], [W.x1, 0], "axis", false, 1.2));

  // Lens outline follows the lensmaker's relation 1/f = (n-1)(1/R1 - 1/R2):
  // curvature goes as 1/f, so a shorter focal length draws a fatter, more
  // strongly curved lens. The outline is indicative -- the thin-lens model
  // still refracts at the plane, which is why the rays bend there.
  const lh = 2.5;
  const curve = 0.85 / Math.abs(f);                 // sagitta of each face
  const edge = 0.06 + 0.30 / Math.abs(f);           // biconcave rim thickness
  const waist = 0.035;                              // keep the rim from pinching shut
  const lensPath = state.converging
    ? `M ${sx(0)} ${sy(lh)} Q ${sx(curve)} ${sy(0)} ${sx(0)} ${sy(-lh)} `
      + `Q ${sx(-curve)} ${sy(0)} ${sx(0)} ${sy(lh)} Z`
    : `M ${sx(-edge)} ${sy(lh)} Q ${sx(edge - 2 * waist)} ${sy(0)} ${sx(-edge)} ${sy(-lh)} `
      + `L ${sx(edge)} ${sy(-lh)} Q ${sx(2 * waist - edge)} ${sy(0)} ${sx(edge)} ${sy(lh)} Z`;
  scene.appendChild(el("path", {
    d: lensPath, fill: "var(--ob-glass-fill)", stroke: "var(--ob-glass)",
    "stroke-width": 2, "stroke-linejoin": "round"
  }));

  // focal markers
  const af = Math.abs(f);
  for (const [x, name] of [[-af, "F"], [af, "F\u2032"], [-2 * af, "2F"], [2 * af, "2F\u2032"]]) {
    scene.appendChild(el("circle", { cx: sx(x), cy: sy(0), r: 3.2, fill: "var(--ob-faint)" }));
    scene.appendChild(label(x, -0.42, name, "faint", "middle", 12));
  }

  // rays
  const rs = rays(dobj, f, s);
  for (const r of rs) {
    scene.appendChild(line(r.inc[0], r.inc[1], r.c, false));
    scene.appendChild(line(r.refr[0], r.refr[1], r.c, false));
  }

  // virtual image: dashed back-extensions from each exit point through it
  if (!s.none && !s.real) {
    for (const r of rs) scene.appendChild(line(r.exit, [s.di, s.m * H], r.c, true));
  }

  // object
  scene.appendChild(arrow(-dobj, H, "object", false));
  scene.appendChild(label(-dobj, -0.55, "object", "muted", "middle", 12));

  // image
  let offFrame = false;
  if (!s.none) {
    const ih = s.m * H;
    if (Math.abs(s.di) > W.x1 - 0.15 || Math.abs(ih) > 3.4) {
      offFrame = true;
    } else {
      scene.appendChild(arrow(s.di, ih, "image", !s.real));
      scene.appendChild(label(s.di, ih >= 0 ? ih + 0.42 : ih - 0.62,
        s.real ? "real image" : "virtual image", "image", "middle", 12.5));
    }
  }
  if (s.none) {
    scene.appendChild(label(4.6, 2.5, "rays emerge parallel \u2014 no image forms", "muted", "middle", 14));
  } else if (offFrame) {
    scene.appendChild(label(4.6, 2.5, "image forms far off-frame", "muted", "middle", 14));
  }

  readout(s, f);
}

// ---- readout, chips, case strip ---------------------------------------
function chip(text, cls, big) {
  const d = document.createElement("span");
  d.className = "chip" + (cls ? " " + cls : "") + (big ? " big" : "");
  d.textContent = text;
  return d;
}
function readout(s, f) {
  const v = document.getElementById("verdict");
  v.textContent = "";
  document.getElementById("n-do").textContent = state.do.toFixed(2);
  document.getElementById("n-f").textContent = f.toFixed(2);

  if (s.none) {
    v.appendChild(chip("no image", "none", true));
    v.appendChild(chip("object sits at F", "none"));
    document.getElementById("n-di").textContent = "\u221E";
    document.getElementById("n-m").textContent = "\u221E";
  } else {
    const mag = Math.abs(s.m);
    v.appendChild(chip(s.real ? "real" : "virtual", s.real ? "real" : "virtual", true));
    v.appendChild(chip(s.m > 0 ? "upright" : "inverted"));
    v.appendChild(chip(Math.abs(mag - 1) < 0.02 ? "same size" : mag > 1 ? "magnified" : "reduced"));
    v.appendChild(chip(s.real ? "far side \u00B7 catchable on a screen" : "same side \u00B7 no screen catches it"));
    document.getElementById("n-di").textContent = (s.di >= 0 ? "+" : "\u2212") + Math.abs(s.di).toFixed(2);
    document.getElementById("n-m").textContent = (s.m >= 0 ? "+" : "\u2212") + mag.toFixed(2);
  }
  strip(s, f);
}

function strip(s, f) {
  const box = document.getElementById("zones");
  const title = document.getElementById("strip-title");
  box.textContent = "";
  const lo = 0.4, hi = 8, span = hi - lo;
  const pct = x => (x - lo) / span * 100;
  const af = Math.abs(f);

  let zones;
  if (state.converging) {
    title.textContent = "Where you are \u2014 the five standard cases";
    zones = [
      [lo, af, "inside F \u00B7 virtual"],
      [af, 2 * af, "F to 2F \u00B7 real, magnified"],
      [2 * af, hi, "beyond 2F \u00B7 real, reduced"]
    ];
  } else {
    title.textContent = "Where you are \u2014 a diverging lens has only one case";
    zones = [[lo, hi, "always virtual \u00B7 upright \u00B7 reduced"]];
  }

  const active = state.do;
  for (const [a, b, name] of zones) {
    const d = document.createElement("div");
    const isOn = active >= a && active < b && !s.none;
    d.className = "zone" + (isOn ? " active" : "");
    d.style.left = pct(a) + "%";
    d.style.width = (pct(b) - pct(a)) + "%";
    d.textContent = name;
    box.appendChild(d);
  }
  if (state.converging) {
    for (const [x, t] of [[af, "F"], [2 * af, "2F"]]) {
      const m = document.createElement("div");
      m.className = "zone";
      m.style.left = pct(x) + "%"; m.style.width = "0";
      m.style.borderRight = "1px dashed var(--ob-faint)";
      m.title = t;
      box.appendChild(m);
    }
  }
  const head = document.createElement("div");
  head.className = "zhead";
  head.style.left = pct(Math.min(Math.max(active, lo), hi)) + "%";
  box.appendChild(head);
}

// ---- interaction -------------------------------------------------------
const dist = document.getElementById("dist"), foc = document.getElementById("foc");
const dolab = document.getElementById("dolab"), flab = document.getElementById("flab");

function sync() {
  dist.value = state.do; dolab.textContent = state.do.toFixed(2);
  foc.value = state.f; flab.textContent = (state.converging ? state.f : -state.f).toFixed(2);
  render();
}
dist.addEventListener("input", () => { state.do = +dist.value; stop(); sync(); });
foc.addEventListener("input", () => { state.f = +foc.value; sync(); });

const convB = document.getElementById("conv"), divB = document.getElementById("div");
function setLens(c) {
  state.converging = c;
  convB.setAttribute("aria-pressed", String(c));
  divB.setAttribute("aria-pressed", String(!c));
  sync();
}
convB.addEventListener("click", () => setLens(true));
divB.addEventListener("click", () => setLens(false));

// drag the object arrow directly on the bench
let dragging = false;
function pointerToDo(ev) {
  const r = svg.getBoundingClientRect();
  const x = wx((ev.clientX - r.left) / r.width * VB.w);
  return Math.min(8, Math.max(0.4, -x));
}
svg.addEventListener("pointerdown", ev => {
  const target = pointerToDo(ev);
  if (Math.abs(target - state.do) < 0.9) {
    dragging = true; stop(); svg.setPointerCapture(ev.pointerId);
    state.do = target; sync();
  }
});
svg.addEventListener("pointermove", ev => { if (dragging) { state.do = pointerToDo(ev); sync(); } });
svg.addEventListener("pointerup", ev => { dragging = false; svg.releasePointerCapture(ev.pointerId); });

// sweep
const play = document.getElementById("play");
let raf = null, last = null;
function stop() { state.playing = false; play.textContent = "Play sweep"; if (raf) cancelAnimationFrame(raf); raf = null; last = null; }
function tick(ts) {
  if (!state.playing) return;
  if (last === null) last = ts;
  const dt = Math.min(50, ts - last) / 1000; last = ts;
  state.do += state.dir * dt * 1.35;
  if (state.do <= 0.45) { state.do = 0.45; state.dir = 1; }
  if (state.do >= 7.8) { state.do = 7.8; state.dir = -1; }
  sync();
  raf = requestAnimationFrame(tick);
}
play.addEventListener("click", () => {
  if (state.playing) { stop(); }
  else { state.playing = true; play.textContent = "Pause sweep"; raf = requestAnimationFrame(tick); }
});

sync();
