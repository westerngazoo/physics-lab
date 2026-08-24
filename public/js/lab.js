/* lab.js -- the shared shell every example page stands on.
 *
 * Deliberately small. The physics models are NOT here and never will be:
 * each example's model is bespoke closed-form math, and flattening that
 * into generic "engine data" is exactly how demos drift from being right
 * to merely looking right. What IS shared is the shell around a model --
 * the world-to-screen mapping, SVG construction, control binding -- and
 * the discipline the early bugs taught, encoded as running checks:
 *
 *   - Lab.world() THROWS if the world box and viewBox imply different
 *     scales per axis. A 6% axis mismatch once skewed every ray angle in
 *     the optics diagram invisibly; now it is a loud failure at load.
 *
 * Division of labor on every page:
 *   CONFIG (data)  -- constants, ranges, presets, world box. Top of file.
 *   model  (code)  -- pure closed-form functions of (config, t).
 *   render (code)  -- rebuilds the SVG from scratch through Lab.world.
 *   Lab    (shell) -- this file.
 */
"use strict";

const Lab = (() => {
  const NS = "http://www.w3.org/2000/svg";

  /** Uniform world-to-screen mapping. Throws on a skewed aspect. */
  function world(W, VB) {
    const px = VB.w / (W.x1 - W.x0);
    const py = VB.h / (W.y1 - W.y0);
    if (Math.abs(px - py) > 1e-9 * Math.max(px, py)) {
      throw new Error(
        "Lab.world: non-uniform scale (" + px.toFixed(4) + " px/unit in x vs " +
        py.toFixed(4) + " in y). Fix the world box or the viewBox -- a skewed " +
        "mapping silently distorts every angle drawn through it."
      );
    }
    return {
      sx: x => (x - W.x0) * px,
      sy: y => VB.h - (y - W.y0) * py,
      wx: p => W.x0 + p / px,
    };
  }

  /** Namespaced SVG element with attributes. */
  function el(name, attrs) {
    const e = document.createElementNS(NS, name);
    for (const k in attrs) e.setAttribute(k, attrs[k]);
    return e;
  }

  /** Bind a range input (id) to obj[key], echoing into #<id>lab. */
  function bind(id, obj, key, fmt, onChange) {
    const input = document.getElementById(id);
    const out = document.getElementById(id + "lab");
    const upd = () => {
      obj[key] = +input.value;
      out.textContent = fmt(obj[key]);
      if (onChange) onChange();
    };
    input.addEventListener("input", upd);
    upd();
  }

  return { world, el, bind };
})();
