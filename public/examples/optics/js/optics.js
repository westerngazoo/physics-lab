/* Optics bench -- four thin optical elements over ONE solver.
 *
 * This is the data-driven part of the lab, and it is data-driven for a
 * reason: converging lens, diverging lens, concave mirror and convex
 * mirror all obey the same equation, 1/f = 1/do + 1/di with m = -di/do.
 * What differs is data (the sign of f, which side images land on) and
 * drawing (how the three principal rays are constructed). The deep
 * category is NOT lens vs mirror -- it is converging vs diverging, i.e.
 * the sign of f. The traits panel is generated from this table so the
 * page cannot drift from its own model.
 *
 * Every ray construction here was verified numerically before drawing:
 * all three rays (or their back-extensions) pass through the computed
 * image point for every element and object distance tested.
 */
"use strict";

// ---- physics + presentation config (data) ------------------------------
const CONFIG = {
  objectHeight: 1.0,        // world units
  noImageBand: 0.055,       // |do - f| below this: rays emerge parallel
  world: { x0: -9, x1: 9, y0: -3.6, y1: 3.6 },  // 18 x 7.2 == 1000 x 400
  viewBox: { w: 1000, h: 400 },
};

// The four elements, as data over one solver. sign = sign of f.
const ELEMENTS = {
  "lens-conv":   { kind: "lens",   sign: +1, label: "Converging lens" },
  "lens-div":    { kind: "lens",   sign: -1, label: "Diverging lens" },
  "mirror-conc": { kind: "mirror", sign: +1, label: "Concave mirror" },
  "mirror-conv": { kind: "mirror", sign: -1, label: "Convex mirror" },
};

// Kind-owned traits: the only things lens vs mirror decides.
const KIND = {
  lens: {
    realSide: "far side \u00B7 catchable on a screen",
    virtSide: "same side \u00B7 no screen catches it",
    farMark: "2F\u2032", nearMark: "2F",
    rays: [
      "parallel in \u2192 through F\u2032",
      "straight through the centre",
      "through F \u2192 parallel out",
    ],
  },
  mirror: {
    realSide: "object's side \u00B7 catchable on a screen",
    virtSide: "behind the mirror \u00B7 no screen catches it",
    farMark: "C", nearMark: "C",
    rays: [
      "parallel in \u2192 through F",
      "to the vertex, reflected symmetric",
      "through F \u2192 parallel out",
    ],
  },
};

const { sx, sy, wx } = Lab.world(CONFIG.world, CONFIG.viewBox);
const el = Lab.el;
const W = CONFIG.world, VB = CONFIG.viewBox;
const H = CONFIG.objectHeight;
const NO_IMAGE = CONFIG.noImageBand;

const state = { el: "lens-conv", do: 6, f: 2, playing: false, dir: -1 };
const scene = document.getElementById("scene");
const svg = document.getElementById("svg");

// ---- the one solver ----------------------------------------------------
function solve(dobj, f) {
  const den = dobj - f;
  if (Math.abs(den) < NO_IMAGE) return { none: true };
  const di = dobj * f / den;
  return { none: false, di, m: -di / dobj, real: di > 0 };
}

/** Where on the bench the image sits: transmission vs reflection. */
function imageX(kind, di) {
  return kind === "lens" ? di : -di;
}

// ---- principal rays, per kind (verified constructions) -----------------
function lensRays(dobj, f, s) {
  const tip = [-dobj, H], R = W.x1, out = [];
  out.push({ c: "r1", inc: [tip, [0, H]],
             refr: [[0, H], [R, H + (R / f) * -H]], exit: [0, H] });
  out.push({ c: "r2", inc: [tip, [0, 0]],
             refr: [[0, 0], [R, -H * R / dobj]], exit: [0, 0] });
  if (!s.none) {
    const y3 = -H * f / (dobj - f);
    if (Math.abs(y3) < 3.45) {
      out.push({ c: "r3", inc: [tip, [0, y3]], refr: [[0, y3], [R, y3]], exit: [0, y3] });
    }
  }
  return out;
}

function mirrorRays(dobj, f, s) {
  const tip = [-dobj, H], Lx = W.x0, out = [];
  // parallel in; reflected ray lies on the line through (0,H) and (-f, 0)
  out.push({ c: "r1", inc: [tip, [0, H]],
             refr: [[0, H], [Lx, H + Lx * H / f]], exit: [0, H] });
  // vertex ray: reflected symmetric about the axis
  out.push({ c: "r2", inc: [tip, [0, 0]],
             refr: [[0, 0], [Lx, H * Lx / dobj]], exit: [0, 0] });
  if (!s.none) {
    const y3 = -H * f / (dobj - f);
    if (Math.abs(y3) < 3.45) {
      out.push({ c: "r3", inc: [tip, [0, y3]], refr: [[0, y3], [Lx, y3]], exit: [0, y3] });
    }
  }
  return out;
}

// ---- drawing helpers ---------------------------------------------------
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
    "font-family": "JetBrains Mono, monospace", "font-size": size || 12.5, "font-weight": 500
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

/** The element glyph. Lens curvature and mirror radius both follow |f|. */
function glyph(kind, sign, af) {
  const g = el("g", {});
  if (kind === "lens") {
    const lh = 2.5;
    const curve = 0.85 / af;
    const edge = 0.06 + 0.30 / af;
    const waist = 0.035;
    const d = sign > 0
      ? `M ${sx(0)} ${sy(lh)} Q ${sx(curve)} ${sy(0)} ${sx(0)} ${sy(-lh)} `
        + `Q ${sx(-curve)} ${sy(0)} ${sx(0)} ${sy(lh)} Z`
      : `M ${sx(-edge)} ${sy(lh)} Q ${sx(edge - 2 * waist)} ${sy(0)} ${sx(-edge)} ${sy(-lh)} `
        + `L ${sx(edge)} ${sy(-lh)} Q ${sx(2 * waist - edge)} ${sy(0)} ${sx(edge)} ${sy(lh)} Z`;
    g.appendChild(el("path", {
      d, fill: "var(--ob-glass-fill)", stroke: "var(--ob-glass)",
      "stroke-width": 2, "stroke-linejoin": "round"
    }));
  } else {
    // Mirror surface: an arc of the circle with R = 2|f| (its actual
    // center of curvature), clamped so shallow mirrors still show.
    const R = 2 * af;
    const lh = Math.min(2.5, 0.82 * R);
    const sag = R - Math.sqrt(R * R - lh * lh);
    const xe = (sign > 0 ? -1 : 1) * sag;      // concave bows toward the object
    g.appendChild(el("path", {
      d: `M ${sx(xe)} ${sy(lh)} Q ${sx(-xe)} ${sy(0)} ${sx(xe)} ${sy(-lh)}`,
      fill: "none", stroke: "var(--ob-glass)", "stroke-width": 2.5, "stroke-linecap": "round"
    }));
    // hatching on the back: this is silvered, not glass
    for (let i = -2; i <= 2; i++) {
      const y = (lh * 0.82) * i / 2;
      const xb = xe * Math.pow(y / lh, 2);     // point on the quadratic
      g.appendChild(line([xb + 0.08, y], [xb + 0.42, y - 0.26], "glass", false, 1));
    }
  }
  return g;
}

// ---- render ------------------------------------------------------------
function render() {
  const E = ELEMENTS[state.el];
  const f = E.sign * state.f;
  const dobj = state.do;
  const s = solve(dobj, f);
  scene.textContent = "";

  scene.appendChild(line([W.x0, 0], [W.x1, 0], "axis", false, 1.2));
  scene.appendChild(glyph(E.kind, E.sign, state.f));

  // focal markers. F belongs to every element; the far marker is 2F for a
  // lens and C (the center of curvature, R = 2f) for a mirror.
  const af = state.f;
  const marks = E.kind === "lens"
    ? [[-af, "F"], [af, "F\u2032"], [-2 * af, "2F"], [2 * af, "2F\u2032"]]
    : [[-f, "F"], [-2 * f, "C"]];
  for (const [x, name] of marks) {
    scene.appendChild(el("circle", { cx: sx(x), cy: sy(0), r: 3.2, fill: "var(--ob-faint)" }));
    scene.appendChild(label(x, -0.42, name, "faint", "middle", 12));
  }

  const rs = E.kind === "lens" ? lensRays(dobj, f, s) : mirrorRays(dobj, f, s);
  for (const r of rs) {
    scene.appendChild(line(r.inc[0], r.inc[1], r.c, false));
    scene.appendChild(line(r.refr[0], r.refr[1], r.c, false));
  }

  let xi = 0;
  if (!s.none) {
    xi = imageX(E.kind, s.di);
    if (!s.real) {
      for (const r of rs) scene.appendChild(line(r.exit, [xi, s.m * H], r.c, true));
    }
  }

  scene.appendChild(arrow(-dobj, H, "object", false));
  scene.appendChild(label(-dobj, -0.55, "object", "muted", "middle", 12));

  let offFrame = false;
  if (!s.none) {
    const ih = s.m * H;
    if (Math.abs(xi) > W.x1 - 0.15 || Math.abs(ih) > 3.4) {
      offFrame = true;
    } else {
      scene.appendChild(arrow(xi, ih, "image", !s.real));
      scene.appendChild(label(xi, ih >= 0 ? ih + 0.42 : ih - 0.62,
        s.real ? "real image" : "virtual image", "image", "middle", 12.5));
    }
  }
  if (s.none) {
    scene.appendChild(label(4.6, 2.9, "rays emerge parallel \u2014 no image forms", "muted", "middle", 14));
  } else if (offFrame) {
    scene.appendChild(label(4.6, 2.9, "image forms far off-frame", "muted", "middle", 14));
  }

  readout(s, f, E);
}

// ---- readout, chips, case strip, traits --------------------------------
function chip(text, cls, big) {
  const d = document.createElement("span");
  d.className = "chip" + (cls ? " " + cls : "") + (big ? " big" : "");
  d.textContent = text;
  return d;
}

function readout(s, f, E) {
  const v = document.getElementById("verdict");
  v.textContent = "";
  v.appendChild(chip(E.label, "el"));
  v.appendChild(chip(E.sign > 0 ? "converging family" : "diverging family",
                     E.sign > 0 ? "fam-conv" : "fam-div"));
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
    v.appendChild(chip(s.real ? KIND[E.kind].realSide : KIND[E.kind].virtSide));
    document.getElementById("n-di").textContent = (s.di >= 0 ? "+" : "\u2212") + Math.abs(s.di).toFixed(2);
    document.getElementById("n-m").textContent = (s.m >= 0 ? "+" : "\u2212") + mag.toFixed(2);
  }
  strip(s, E);
  legend(E);
  traits(s, E);
}

function strip(s, E) {
  const box = document.getElementById("zones");
  const title = document.getElementById("strip-title");
  box.textContent = "";
  const lo = 0.4, hi = 8, span = hi - lo;
  const pct = x => (x - lo) / span * 100;
  const af = state.f;
  const far = E.kind === "lens" ? "2F" : "C";

  let zones;
  if (E.sign > 0) {
    title.textContent = "Where you are \u2014 the five standard cases";
    zones = [
      [lo, af, "inside F \u00B7 virtual"],
      [af, 2 * af, "F to " + far + " \u00B7 real, magnified"],
      [2 * af, hi, "beyond " + far + " \u00B7 real, reduced"]
    ];
  } else {
    title.textContent = "Where you are \u2014 a diverging element has only one case";
    zones = [[lo, hi, "always virtual \u00B7 upright \u00B7 reduced"]];
  }

  for (const [a, b, name] of zones) {
    const d = document.createElement("div");
    const isOn = state.do >= a && state.do < b && !s.none;
    d.className = "zone" + (isOn ? " active" : "");
    d.style.left = pct(a) + "%";
    d.style.width = (pct(b) - pct(a)) + "%";
    d.textContent = name;
    box.appendChild(d);
  }
  const head = document.createElement("div");
  head.className = "zhead";
  head.style.left = pct(Math.min(Math.max(state.do, lo), hi)) + "%";
  box.appendChild(head);
}

function legend(E) {
  const rows = KIND[E.kind].rays;
  ["l-r1", "l-r2", "l-r3"].forEach((id, i) => {
    document.getElementById(id).textContent = rows[i];
  });
}

/** The abstraction, generated from the data so it cannot drift from it. */
function traits(s, E) {
  const box = document.getElementById("traits");
  box.textContent = "";
  const fams = [
    { sign: +1, name: "Converging family", cases: [
      ["beyond " + (E.kind === "lens" ? "2F" : "C"), "real \u00B7 inverted \u00B7 reduced"],
      ["at " + (E.kind === "lens" ? "2F" : "C"), "real \u00B7 inverted \u00B7 same size"],
      ["between F and " + (E.kind === "lens" ? "2F" : "C"), "real \u00B7 inverted \u00B7 magnified"],
      ["at F", "no image \u2014 rays parallel"],
      ["inside F", "virtual \u00B7 upright \u00B7 magnified"],
    ]},
    { sign: -1, name: "Diverging family", cases: [
      ["anywhere", "virtual \u00B7 upright \u00B7 reduced"],
    ]},
  ];
  for (const fam of fams) {
    const members = Object.values(ELEMENTS).filter(e => e.sign === fam.sign)
      .map(e => e.label.toLowerCase()).join(" \u00B7 ");
    const h = document.createElement("div");
    h.className = "tr-head" + (E.sign === fam.sign ? " on" : "");
    h.textContent = fam.name + "  (" + members + ")";
    box.appendChild(h);
    for (const [where, what] of fam.cases) {
      const row = document.createElement("div");
      const active = E.sign === fam.sign && caseActive(fam.sign, where, s);
      row.className = "tr-row" + (active ? " on" : "");
      const a = document.createElement("span"); a.textContent = where;
      const b = document.createElement("span"); b.textContent = what;
      row.append(a, b);
      box.appendChild(row);
    }
  }
}
function caseActive(sign, where, s) {
  if (sign < 0) return true;
  const af = state.f, d = state.do;
  if (s.none) return where.startsWith("at F");
  if (where.startsWith("beyond")) return d >= 2 * af;
  if (where.startsWith("at 2") || where === "at C") return false; // knife-edge, strip shows it
  if (where.startsWith("between")) return d > af && d < 2 * af;
  if (where === "inside F") return d < af;
  return false;
}

// ---- interaction -------------------------------------------------------
const dist = document.getElementById("dist"), foc = document.getElementById("foc");
const dolab = document.getElementById("dolab"), flab = document.getElementById("flab");

function sync() {
  dist.value = state.do; dolab.textContent = state.do.toFixed(2);
  foc.value = state.f;
  flab.textContent = (ELEMENTS[state.el].sign * state.f).toFixed(2);
  render();
}
dist.addEventListener("input", () => { state.do = +dist.value; stop(); sync(); });
foc.addEventListener("input", () => { state.f = +foc.value; sync(); });

document.querySelectorAll("[data-el]").forEach(b => {
  b.addEventListener("click", () => {
    state.el = b.dataset.el;
    document.querySelectorAll("[data-el]").forEach(o =>
      o.setAttribute("aria-pressed", String(o === b)));
    sync();
  });
});

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
