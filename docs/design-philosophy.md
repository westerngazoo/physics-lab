# Filosofía de diseño — physics-lab

## Principios

- Claim-first, not demo-first — derive, check, then draw.
- Closed forms only in browser — no integrators (pedagogy contract).
- One source of truth — native test = WASM = what students see.
- τ not π throughout.

## Contexto

Every lesson states falsifiable claims verified in browser, Python, and cargo test on same Rust.

## Trade-offs explícitos

Este proyecto prioriza coherencia con los principios anteriores sobre conveniencia ad-hoc.
Cuando una decisión contradice un principio, debe documentarse como ADR o RFC.

## Relación con el ecosistema

- **garust**
- **motoreel**
- **guion**
- **fitAI**
