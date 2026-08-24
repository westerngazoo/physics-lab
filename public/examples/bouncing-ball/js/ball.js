/* Bouncing ball -- event-driven, not integrated.
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

// ---- physics + presentation config (data) ------------------------------
// Everything tunable lives here; the model below is pure code over it.
const CONFIG = {
  h0: 4.0,                    // drop height, m
  e: 0.75,                    // restitution: speed fraction kept per impact
  g: 9.81,                    // gravity, m/s^2 (slider spans Moon..Jupiter)
  gPresets: { moon: 1.62, earth: 9.81, jupiter: 24.79 },
  segments: 14,               // bounces in the flight table (apex 15 < 1 mm)
  apexCutoff: 0.045,          // hide apex markers below this height, m
  ballRadius: 0.075,          // drawn ball radius, world units
  timeSpan: 10.4,             // world-x units the full path is scaled into
  world: { x0: -0.6, x1: 11.4, y0: -0.6, y1: 5.4 },
  viewBox: { w: 1000, h: 500 },
};

const state = { h0: CONFIG.h0, e: CONFIG.e, g: CONFIG.g, t: 0, playing: true, rate: 1 };

// Shared shell: uniform mapping (throws on a skewed aspect) + SVG helper.
const { sx, sy } = Lab.world(CONFIG.world, CONFIG.viewBox);
const el = Lab.el;
const W = CONFIG.world, VB = CONFIG.viewBox;

/** Flight segments: segment n starts at t_n with upward speed v_n.
 * Gravity sets every time and speed here -- but notice it cancels out of
 * the apex heights entirely: h_n = e^(2n)*h0, no g in sight. */
function flights(h0, e, g, count) {
  const v0 = Math.sqrt(2 * g * h0);          // speed arriving at the floor
  const segs = [];
  let t = Math.sqrt(2 * h0 / g);             // the initial fall ends here
  segs.push({ t0: 0, dur: t, vUp: 0, y0: h0, drop: true });
  for (let n = 1; n <= count; n++) {
    const v = v0 * Math.pow(e, n);           // speed leaving the nth bounce
    const dur = 2 * v / g;
    segs.push({ t0: t, dur, vUp: v, y0: 0, drop: false, n });
    t += dur;
  }
  return { segs, tRest: Math.sqrt(2 * h0 / g) * (1 + e) / (1 - e) };
}

/** Height at time t, exactly -- no accumulated state. */
function heightAt(t, segs, g) {
  for (const s of segs) {
    if (t >= s.t0 && t < s.t0 + s.dur) {
      const dt = t - s.t0;
      return s.drop ? s.y0 - 0.5 * g * dt * dt
                    : s.vUp * dt - 0.5 * g * dt * dt;
    }
  }
  return 0;
}

const scene = document.getElementById("scene");

function render() {
  const { h0, e, g } = state;
  const { segs, tRest } = flights(h0, e, g, CONFIG.segments);
  // Time axis scaled per configuration: the whole path to rest always fits
  // the frame, whatever e does to the rest time (it blows up as e -> 1).
  const vx = CONFIG.timeSpan / tRest;
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
      const y = heightAt(tt, segs, g);
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
    if (hApex < CONFIG.apexCutoff) continue;
    scene.appendChild(el("line", {
      x1: sx(tApex * vx), y1: sy(0), x2: sx(tApex * vx), y2: sy(hApex),
      stroke: "var(--bb-apex)", "stroke-width": 1, "stroke-dasharray": "2 4", opacity: 0.55
    }));
    scene.appendChild(el("circle", {
      cx: sx(tApex * vx), cy: sy(hApex), r: 2.5, fill: "var(--bb-apex)"
    }));
  }

  // the ball
  const y = heightAt(t, segs, g);
  const ballX = t * vx;
  scene.appendChild(el("circle", {
    cx: sx(ballX), cy: sy(y + CONFIG.ballRadius),
    r: CONFIG.ballRadius / (W.y1 - W.y0) * VB.h,
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

// ---- controls (shared binder; any change restarts the run) --------------
const restart = () => { state.t = 0; render(); };
Lab.bind("hh", state, "h0", v => v.toFixed(1) + " m", restart);
Lab.bind("ee", state, "e", v => v.toFixed(2), restart);
Lab.bind("gg", state, "g", v => v.toFixed(2) + " m/s\u00B2", restart);

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
    const { tRest } = flights(state.h0, state.e, state.g, CONFIG.segments);
    state.t += dt * state.rate;
    if (state.t > tRest + 0.8) state.t = 0;
    render();
  }
  requestAnimationFrame(tick);
}

render();
requestAnimationFrame(tick);
