#!/usr/bin/env python3
"""Generate public/lessons/index.json from the lesson manifests, so the
hub renders itself and can never forget or misdescribe a lesson. Legacy
JS-era examples are listed here until their ports retire them."""
import json, glob, os

entries = []
for mf in sorted(glob.glob("public/lessons/*/lesson.json")):
    j = json.load(open(mf))
    entries.append({
        "href": "lessons/%s/index.html" % j["slug"],
        "topic": j.get("topic", ""), "wasm": True,
        "title": j["title"], "card": j.get("card", ""),
        "claim": j.get("cardClaim", ""),
    })

LEGACY = [
    { "href": "examples/optics/index.html", "topic": "geometric optics", "wasm": False,
      "title": "Real or Virtual",
      "card": "Four devices — converging and diverging lenses, concave and convex mirrors — over one equation. The deep category is not lens vs mirror: it is the sign of f.",
      "claim": "A convex lens and a concave mirror share the same five cases; their diverging twins share a single one." },
    { "href": "examples/bouncing-ball/index.html", "topic": "gravity", "wasm": False,
      "title": "Infinitely Many Bounces",
      "card": "A ball keeping a fixed fraction of its speed each bounce never has a last bounce. Watch the apexes fall away geometrically, and the clock stop anyway.",
      "claim": "Infinitely many bounces, finite total time: T = √(2h₀/g)·(1+e)/(1−e)." },
]

out = { "lessons": entries + LEGACY }
json.dump(out, open("public/lessons/index.json", "w"), indent=1, ensure_ascii=False)
print("indexed", len(out["lessons"]), "entries")
