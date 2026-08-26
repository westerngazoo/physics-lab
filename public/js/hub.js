/* hub.js -- renders the lesson cards from lessons/index.json (generated
 * by tools/gen-index.py from the manifests). Adding a lesson never means
 * editing the hub. */
"use strict";
(async function () {
  const grid = document.getElementById("cards");
  let idx;
  try {
    idx = await (await fetch("lessons/index.json")).json();
  } catch (e) {
    grid.textContent = "The lesson index failed to load: " + e;
    return;
  }
  const accents = ["var(--steel)", "var(--gold)", "var(--red-hot)"];
  idx.lessons.forEach((l, i) => {
    const a = document.createElement("a");
    a.className = "card";
    a.href = l.href;
    a.style.setProperty("--accent", accents[i % accents.length]);
    a.innerHTML =
      "<span class='tag'>" + l.topic + (l.wasm ? " · wasm" : "") + "</span>" +
      "<h2>" + l.title + "</h2>" +
      "<p>" + l.card + "</p>" +
      "<div class='claim'>" + l.claim + "</div>" +
      "<div class='go'>Open &rarr;</div>";
    grid.appendChild(a);
  });
})();
