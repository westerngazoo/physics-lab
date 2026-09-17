# Flujos — physics-lab

## Flujo principal (Mermaid)

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
2. **Student** — runtime reads lesson.json → WASM params → paint primitives.
3. **Verify** — Claim in Rust test = browser = Python cross-check.
4. **Deploy** — wrangler deploy → physics.goosethropic.systems.

## Secuencia (PlantUML)

Fuente: [`diagrams/flow-sequence.puml`](./diagrams/flow-sequence.puml)

```plantuml
@startuml
title physics-lab — secuencia principal

participant "Author" as Author0
participant "Student" as Student1
participant "Verify" as Verify2
participant "Deploy" as Deploy3

Author0 -> Student1: runtime reads lesson.json → WASM params → paint primitives.
Student1 -> Verify2: Claim in Rust test = browser = Python cross-check.
Verify2 -> Deploy3: wrangler deploy → physics.goosethropic.systems.

@enduml
```

## Componentes / estados (PlantUML)

Fuente: [`diagrams/flow-architecture.puml`](./diagrams/flow-architecture.puml)

```plantuml
@startuml
title physics-lab — flujo de componentes
start
:RUSTLesson;
:WASMwasm32;
:RUST;
:CARGOcargo;
:PYchecks/run.py;
:WASM;
:BROWSERruntime.js;
:CARGO;
:CICI;
:PY;
:CI;
:BROWSER;
:STUDENTStudent;
stop

@enduml
```

## Estados y casos borde

Consulta los tests de integración y los RFC/requirements del proyecto para flujos de error y recuperación.
