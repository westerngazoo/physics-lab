/* Bouncing ball — event-driven, not integrated.
 *
 * Each flight between bounces is an exact parabola, and each bounce time is
 * solved for in closed form, so apex heights land on h_n = e^(2n)*h0 to the
 * last digit instead of drifting the way a fixed-step integrator does. That
 * matters here because the whole point of the page is a quantitative claim:
 * the ball bounces infinitely many times and still stops at a finite time,
 *
 *     T = sqrt(2*h0/g) * (1 + e) / (1 - e)
 *
 * which is only convincing if the numbers on screen actually agree with it.
 */

const NS = "http://www.w3.org/2000/svg";
const G = 9.81;

const state = { h0: 4.0, e: 0.75, t: 0, playing: true, rate: 1 };

// ---- world <-> screen. 12 x 6 world units into a 1000 x 500 viewBox, so
// ---- both axes scale alike and a parabola is not skewed into something else.
const W = { x0: -0.6, x1: 11.4, y0: -0.6, y1: 5.4 };
const VB = { w: 1000, h: 500 };
const sx = x => (x - W.x0) / (W.x1 - W.x0) * VB.w;
const sy = y => VB.h - (y - W.y0) / (W.y1 - W.y0) * VB.h;

/** Flight segments: segment n starts at t_n with upward speed v_n. */
function flights(h0, e, count) {
  const v0 = Math.sqrt(2 * G * h0);          // speed arriving at the floor
  const segs = [];
  let t = Math.sqrt(2 * h0 / G);             // the initial fall ends here
  segs.push({ t0: 0, dur: t, vUp: 0, y0: h0, drop: true });
  for (let n = 1; n <= count; n++) {
    const v = v0 * Math.pow(e, n);           // speed leaving the nth bounce
    const dur = 2 * v / G;
    segs.push({ t0: t, dur, vUp: v, y0: 0, drop: false, n });
    t += dur;
  }
  return { segs, tRest: Math.sqrt(2 * h0 / G) * (1 + e) / (1 - e) };
}

/** Height at time t, exactly — no accumulated state. */
function heightAt(t, segs) {
  for (const s of segs) {
    if (t >= s.t0 && t < s.t0 + s.dur) {
      const dt = t - s.t0;
      return s.drop ? s.y0 - 0.5 * G * dt * dt
                    : s.vUp * dt - 0.5 * G * dt * dt;
    }
  }
  return 0;
}

function el(name, attrs) {
  const e = document.createElementNS(NS, name);
  for (const k in attrs) e.setAttribute(k, attrs[k]);
  return e;
}

const scene = document.getElementById("scene");

function render() {
  const { h0, e } = state;
  const { segs, tRest } = flights(h0, e, 14);
  // Time axis scaled per configuration: the whole path to rest always fits
  // the frame, whatever e does to the rest time (it blows up as e -> 1).
  const vx = 10.4 / tRest;
  const t = Math.min(state.t, tRest);
  scene.textContent = "";

  // floor
  scene.appendChild(el("line", {
    x1: sx(W.x0), y1: sy(0), x2: sx(W.x1), y2: sy(0),
    stroke: "var(--bb-floor)", "stroke-width": 2
  }));

  // the full path, drawn once as a polyline sampled per segment
  let d = "";
  for (const s of segs) {
    const N = 26;
    for (let i = 0; i <= N; i++) {
      const tt = s.t0 + s.dur * (i / N);
      const y = heightAt(tt, segs);
      d += (d ? " " : "") + sx(tt * vx).toFixed(2) + "," + sy(y).toFixed(2);
    }
  }
  scene.appendChild(el("polyline", {
    points: d, fill: "none", stroke: "var(--bb-path)",
    "stroke-width": 1.2, "stroke-dasharray": "4 4", opacity: 0.5
  }));

  // apex markers with their analytic heights
  for (const s of segs) {
    if (s.drop) continue;
    const tApex = s.t0 + s.dur / 2;
    const hApex = h0 * Math.pow(e, 2 * s.n);
    if (hApex < 0.045) continue;
    scene.appendChild(el("line", {
      x1: sx(tApex * vx), y1: sy(0), x2: sx(tApex * vx), y2: sy(hApex),
      stroke: "var(--bb-apex)", "stroke-width": 1, "stroke-dasharray": "2 4", opacity: 0.55
    }));
    scene.appendChild(el("circle", {
      cx: sx(tApex * vx), cy: sy(hApex), r: 2.5, fill: "var(--bb-apex)"
    }));
  }

  // the ball
  const y = heightAt(t, segs);
  const ballX = t * vx;
  scene.appendChild(el("circle", {
    cx: sx(ballX), cy: sy(y + 0.075), r: 0.075 / (W.y1 - W.y0) * VB.h,
    fill: "var(--bb-ball)"
  }));

  readout(t, y, segs, tRest);
}

function bouncesBy(t, segs) {
  let n = 0;
  for (const s of segs) if (!s.drop && t >= s.t0) n = s.n;
  return n;
}

function readout(t, y, segs, tRest) {
  const n = bouncesBy(t, segs);
  const frac = Math.pow(state.e, 2 * n);
  document.getElementById("r-t").textContent = t.toFixed(2) + " s";
  document.getElementById("r-bounces").textContent = n;
  document.getElementById("r-apex").textContent =
    n === 0 ? state.h0.toFixed(3) + " m" : (state.h0 * frac).toFixed(3) + " m";
  document.getElementById("r-energy").textContent = (frac * 100).toFixed(1) + " %";
  document.getElementById("r-rest").textContent = tRest.toFixed(3) + " s";
  document.getElementById("atrest").hidden = t < tRest - 1e-9;
}

// ---- controls -----------------------------------------------------------
function bind(id, key, fmt) {
  const input = document.getElementById(id);
  const out = document.getElementById(id + "lab");
  const upd = () => {
    state[key] = +input.value;
    out.textContent = fmt(state[key]);
    state.t = 0;
    render();
  };
  input.addEventListener("input", upd);
  upd();
}
bind("hh", "h0", v => v.toFixed(1) + " m");
bind("ee", "e", v => v.toFixed(2));

const play = document.getElementById("play");
play.addEventListener("click", () => {
  state.playing = !state.playing;
  play.textContent = state.playing ? "Pause" : "Play";
});
document.getElementById("replay").addEventListener("click", () => {
  state.t = 0;
  if (!state.playing) render();
});
document.querySelectorAll(".speed button").forEach(b => {
  b.addEventListener("click", () => {
    state.rate = +b.dataset.rate;
    document.querySelectorAll(".speed button")
      .forEach(o => o.setAttribute("aria-pressed", String(o === b)));
  });
});

let last = null;
function tick(ts) {
  if (last === null) last = ts;
  const dt = Math.min(60, ts - last) / 1000;
  last = ts;
  if (state.playing) {
    const { tRest } = flights(state.h0, state.e, 14);
    state.t += dt * state.rate;
    if (state.t > tRest + 0.8) state.t = 0;
    render();
  }
  requestAnimationFrame(tick);
}

render();
requestAnimationFrame(tick);
