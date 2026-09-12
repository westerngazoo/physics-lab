# Flujos — physics-lab

## Flujo principal

```mermaid
flowchart TB
RUST[Lesson lib.rs] --> WASM[wasm32 build]
RUST --> CARGO[cargo test claims]
RUST --> PY[checks/run.py]
WASM --> BROWSER[runtime.js]
CARGO --> CI[CI gate]
PY --> CI
BROWSER --> STUDENT[Student view]
```

## Descripción paso a paso

1. **Author** — Write lib.rs + lesson.json + notes.html → build-wasm.sh.
1. **Student** — runtime reads lesson.json → WASM params → paint primitives.
1. **Verify** — Claim in Rust test = browser = Python cross-check.
1. **Deploy** — wrangler deploy → physics.goosethropic.systems.

## Diagrama PlantUML

Equivalente PlantUML del flujo principal (misma topología que el diagrama Mermaid):

```plantuml
@startuml
title physics-lab — flujo principal
note as N1
Ver flows.md Mermaid para detalle;
exportar con herramientas mermaid→plantuml si se prefiere editar en PlantUML.
end note
@enduml
```

## Estados y casos borde

Consulta los tests de integración y los RFC/requirements del proyecto para flujos de error y recuperación.
