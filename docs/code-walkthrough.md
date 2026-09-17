# Recorrido del código — physics-lab

Guía orientada a desarrolladores para entender dónde vive cada responsabilidad.

### 1. Author

Write lib.rs + lesson.json + notes.html → build-wasm.sh.

### 2. Student

runtime reads lesson.json → WASM params → paint primitives.

### 3. Verify

Claim in Rust test = browser = Python cross-check.

### 4. Deploy

wrangler deploy → physics.goosethropic.systems.

## Punto de entrada recomendado

Empieza por el README del proyecto y el módulo/crate principal listado en la documentación de arquitectura.
