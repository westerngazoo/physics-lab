# RFC-003 — El estudio: fórmulas que se escriben, física que se ve, clases que se graban

- **Estado:** Propuesta (Discussing). La rebanada vertical de §7 ya está
  construida en esta rama, a propósito: una propuesta que cambia el ABI se
  juzga mejor funcionando que en papel. Nada de lo construido toca el
  comportamiento de las lecciones existentes (verificado bit a bit, §7.3).
- **Dueño:** Gustavo Delgadillo (westerngazoo)
- **Alcance:** physics-lab como el **entorno interactivo de Akademos** y como
  la **fuente de las tomas** que guion convierte en video.
- **Depende de / se alinea con:** [RFC-001](RFC-001-lesson-framework.md) (el
  framework), [RFC-002](RFC-002-mecanica.md) (mecanica), guion RFC-0001 §3.2
  (`Model` × `Source`, en `guion-video-creator`), el borrador *RFC-002
  simulated lessons* que vive en `guion-video-creator/docs/physics-lab/`, y
  Akademos R-0007 (`ContentItem::Interactive`).

---

## 0. TL;DR

1. **La visión cabe en una frase:** casi toda la matemática práctica es la
   descripción de un evento físico; el estudio pone la fórmula y el evento
   lado a lado y deja que cada uno corrija al otro — en clase, en YouTube y
   dentro de un curso de Akademos, **con el mismo modelo**.
2. **Casi todas las piezas ya existen**, repartidas en cinco repos. Lo que
   falta son las costuras: escribir matemática en una lección, grabar una
   sesión, e incrustar el lab en Akademos.
3. **Construido aquí:** el panel de fórmulas (crate `formulas`: analiza, nunca
   ejecuta; derivadas exactas por diferenciación automática), un ABI de texto
   aditivo en `lessons-common`, la lección *La derivada es una velocidad*
   (el coche de tu ejemplo, en 1D y 2D), y las **tomas**: se graban las
   entradas, no los píxeles, a través de un puerto (`window.physicsLab`) que
   sirve igual a un grabador, a un renderizador sin pantalla o a guion.
4. **Propuesto:** afinar el principio "sólo forma cerrada" a **"cada cuadro es
   una función pura de (entradas, t)"** — que es lo que de verdad compra el
   scrubbing sin deriva — y adoptar el borrador de lecciones simuladas como
   RFC-004. Con eso entran el doble péndulo y el entrenamiento de una red
   neuronal (el descenso por gradiente *es* un integrador).
5. **De paso, cuatro bugs del sitio en vivo** encontrados manejando las
   páginas, y corregidos (§7.4).

---

## 1. La visión, dicha para poder juzgarla

La idea del dueño, en sus palabras: *un lugar interactivo tipo Desmos donde de
un lado las funciones describan lo que pasa en el lab*. El ejemplo canónico:
el concepto de derivada con la velocidad — un cochecito en movimiento, su
gráfica, en una o dos dimensiones. Y la apuesta de fondo: **la mayoría de la
matemática práctica se puede describir por eventos físicos**.

Dicho como arquitectura, son dos paneles y una flecha en cada sentido:

```
   LO QUE ESCRIBES                         LO QUE PASA
   x(t) = 3t² − 0.2t³      ──────►        un coche que arranca, acelera,
   (la función)            ◄──────        frena y se detiene (el evento)
                        cada uno corrige
                           al otro
```

Hacia la derecha es Desmos: escribes y ves. Hacia la izquierda es PhET (*The
Moving Man*): mueves y aparece la función. Lo que ninguno de los dos tiene, y
esta casa sí, es la tercera pieza: **afirmaciones que se pueden romper** —
cada número de la página es una prueba de `cargo test` sobre el mismo código
que corre en el navegador.

Y tres audiencias con un solo modelo:

| Medio | Qué necesita | Quién lo hace hoy |
|---|---|---|
| **Clase** (en vivo) | manejar la lección, escribir fórmulas, romper una afirmación | physics-lab |
| **YouTube / reels** (grabado) | la misma lección, grabada, con voz (propia o TTS) | guion-video-creator (desde guiones TOML, no desde el lab) |
| **Curso** (Akademos) | la lección como un ítem de una lección de curso | nadie todavía |

La analogía de ingeniero: hoy hay un **simulador** (el lab), un **osciloscopio
que graba** (guion) y un **banco de pruebas** (Akademos), cada uno con su
propia copia de la señal. Esta propuesta los pone en el mismo bus.

---

## 2. Lo que ya existe (leído del código, no recordado)

| Repo | Qué es, de verdad | La costura con este RFC |
|---|---|---|
| **garust** | el kernel GA: `Cl(P,Q,R)`, motores, dinámica de sólidos | la curvatura de §4.1 ya sale de su `wedge` |
| **motoreel** | estudio de cine determinista *offline*: `Scene → Prim2 → frames` | el vocabulario de prims que el lab ya imita |
| **physics-lab** | el aula: lecciones Rust→wasm, puras, con afirmaciones como pruebas | este repo |
| **guion-video-creator** | *el* guion vivo: guiones TOML, `[[narration]]` con tiempos, motores TTS (`scaffold`, Piper offline), cama musical, `encode` a MP4 con ffmpeg, un estudio Tauri, y **`guion-motion` ya corre los `.wasm` de este repo con `wasmi`** (fuente `InPlace`) | §4.3: una toma del lab es otra fuente para guion |
| **guion** (privado) | un M0/M1 anterior del mismo marco; el trabajo se mudó a guion-video-creator | ninguna nueva |
| **akademos** | plataforma de cursos: grafo de árboles; una lección es una lista de `Video`, `Reading`, `Interactive { widget, spec }`, `Quiz` | §4.5: el lab como `Interactive` |

Tres hechos que cambian el diseño:

1. **guion ya sabe correr una lección del lab** (`WasmLesson::load/eval` sobre
   el ABI de `lesson!`). El puente lab → video no hay que inventarlo: hay que
   darle algo que reproducir.
2. **Hay un RFC de lecciones simuladas escrito y sin adoptar**, preparado
   para "moverse verbatim a `physics-lab/docs/RFC-002`" — pero RFC-002 ya es
   `mecanica`. El número choca; el contenido sigue vigente (§4.2).
3. **Akademos ya tiene un segundo analizador de expresiones** (`cas.js`, en
   JavaScript, para los ejercicios con CAS). Dos analizadores son dos
   gramáticas que divergirán: §4.1 propone uno.

---

## 3. La brecha

| # | Falta | Consecuencia |
|---|---|---|
| 1 | **Escribir matemática.** Una lección sólo recibía números (deslizadores); las cadenas sólo *salían*, como índices a `labels`. | Imposible el "tipo Desmos". |
| 2 | **Ir más allá de la forma cerrada.** | Ni doble péndulo, ni red neuronal que aprende, ni caos en vivo. |
| 3 | **3D.** Prims 2D y un pintor SVG. | Ni paisajes de pérdida, ni sólidos que giran. |
| 4 | **Grabar.** El lab sólo existe en vivo; guion graba guiones, no sesiones. | La clase no se convierte en video sin rehacerla. |
| 5 | **Incrustar en Akademos.** La CSP del sitio dice `frame-ancestors 'none'`, `X-Frame-Options: DENY`, `Cross-Origin-Resource-Policy: same-origin`. | Hoy es imposible, por diseño. |
| 6 | **Densidad.** El pintor rehace el DOM SVG cada cuadro. | Bien hasta unos 2 000 elementos; una malla 3D o un campo vectorial no caben. |
| 7 | **La costura débil crece.** Orden de parámetros y número de ranura son convención (DESIGN.md §5). | Más fórmulas y más lecturas por lección = más formas de mentir en silencio. |
| 8 | **Reproducibilidad.** Los `.wasm` comprometidos no salen iguales en otra máquina (§7.5). | "Desplegable desde un checkout" es cierto; "verificable desde un checkout", no. |

---

## 4. Las ideas de arquitectura

### 4.1 El panel de fórmulas — construido

**Qué es.** Una lección declarada con `lesson!(draw, texto)` recibe, además
de los parámetros, lo que el lector escribió: UTF-8, un renglón por caja.
El ABI crece en cuatro exportaciones **aditivas** (`text_ptr`, `text_cap`,
`set_text_len`, `diag_ptr`); `state_at(n_params)` conserva su firma, así que
el `WasmLesson` de guion sigue manejando cualquier lección.

**La regla de oro: el texto se analiza, nunca se ejecuta.** El crate
`formulas` convierte el texto en un programa de una docena de operaciones en
notación polaca inversa. No hay `eval`, no hay JavaScript, no hay nombres que
el crate no conozca. Es la única pieza del lab que recibe entrada arbitraria
de un desconocido, y por eso tiene tres garantías con prueba:

- *nada entra sin ubicarse*: todo error sale como `(código, columna)` — la
  columna en **caracteres**, no bytes (`τ` ocupa dos bytes y una columna);
- *ninguna entrada tumba la página*: anidamiento y pila acotados; una
  `Formula` es un **invariante encapsulado** (como `Motor` en garust) que sólo
  existe si ya se comprobó que su programa no puede desbordar la pila —
  25 000 entradas aleatorias en la prueba, cero pánicos;
- *lo que sale son números*: la página recibe códigos y columnas, y tiene su
  propia tabla de mensajes, **comparada con la de Rust por una prueba** que lee
  `runtime.js`. La costura entre los dos idiomas tiene un guardián.

**Las derivadas son exactas.** Hay tres maneras de derivar lo que escribió
otro, y sólo una es honesta aquí:

| Método | Problema |
|---|---|
| la secante `(f(t+Δt) − f(t))/Δt` | es la *definición*; si también fuera "la derivada verdadera", mediríamos una regla con ella misma |
| derivación simbólica | hay que simplificar para que se lea, y un simplificador es un programa entero con sus propios errores |
| **diferenciación automática** (jets) | ninguno: cada número viaja con sus dos primeras derivadas y cada operación aplica la regla de la cadena en el momento; el resultado tiene el redondeo del valor mismo |

Es la idea de los números duales (`a + bε`, `ε² = 0`) llevada a segundo
orden (`ε³ = 0`): velocidad *y* aceleración de cualquier fórmula. La secante
queda donde debe: como aproximación, con su error en pantalla y un control
que la rompe (§7.2, D3).

**Deslizadores que aparecen solos** (como en Desmos): un parámetro con
`"widget": "auto"` sólo se muestra mientras alguna fórmula lo nombra. La
lección reporta qué parámetros usa cada renglón como una máscara de bits.

**Decisiones de gramática** (explícitas, porque el lector las va a notar):
`−t²` es `−(t²)`; la multiplicación implícita pesa lo que la explícita
(`1/2t` = `(1/2)·t`); las funciones llevan paréntesis (`sin(t)`, no `sin t`);
`at²` es `a·t²` (el exponente se pega a la última letra); `sen`, `τ`, `π`,
`−`, `·`, `²` se aceptan porque el lector los va a pegar de la propia página;
el decimal es con punto, y la coma se rechaza con un mensaje que lo dice.

**Un solo motor para Akademos.** `cas.js` decide equivalencias evaluando en
puntos al azar — exactamente lo que `Formula::valor` permite. Y desde R-0022
akademos tiene además un *grapher* que reusa la gramática de `cas.js` y
calcula `f′` por diferencia central con `h = 10⁻⁵·max(1, |x|)`: ya son **tres**
motores (dos gramáticas y dos derivadas). Propuesta: que los ejercicios y el
grapher de Akademos usen el mismo `formulas` (vía un wasm pequeño) en vez
de mantener una segunda gramática en JavaScript y una derivada aproximada. Es la regla de
guion-video-creator aplicada al álgebra: *si el reel y la app dan números
distintos, uno de los dos miente*.

### 4.2 Pureza no es forma cerrada — propuesta de enmienda

RFC-001 dice "closed forms only — nothing is stepped in the browser", y da la
razón: *scrubbing cannot drift*. Pero la propiedad que compra eso no es la
forma cerrada; es la **pureza**: que cada cuadro sea una función de
`(entradas, t)`. La forma cerrada es una condición *suficiente*, no
*necesaria*.

Un integrador de paso fijo que arranca siempre de las mismas condiciones
iniciales también es una función pura de `(entradas, t)` — sólo que cara. Se
memoriza una vez (el `bake` del borrador de lecciones simuladas, el
`record()` de motoreel) y se muestrea: `O(log n)` por cuadro, sin deriva.
En términos de embebidos: una tabla en ROM y un cálculo en línea son la misma
función; una es memoria y la otra ciclos.

**Propuesta:** adoptar el borrador de `guion-video-creator/docs/physics-lab/`
como **RFC-004** (renumerado: RFC-002 ya es `mecanica`), con el principio
reformulado:

> **Cada cuadro es una función pura de (entradas, t). Cuando hace falta un
> integrador, su error está en pantalla y su paso está fijo.**

Esto no afloja nada: la lección con forma cerrada sigue sin deber banda de
error, y la simulada paga la suya (energía, orden medido, contra la forma
cerrada donde exista). Y abre la puerta grande de la visión: **el descenso por
gradiente es Euler explícito sobre el flujo del gradiente**, así que entrenar
una red es una lección simulada, con las mismas pruebas.

### 4.3 Tomas: grabar entradas, no píxeles — construido

Una grabación de pantalla es un archivo de audio: fija la resolución, el
formato y los errores. Una **toma** es un archivo MIDI: guarda la
interpretación (qué parámetro, qué fórmula, cuándo) y cualquier instrumento
la vuelve a tocar — 9:16 para reels, 16:9 para YouTube, 4K para el
proyector. Funciona **porque cada cuadro es puro**: si el cuadro es `f(entradas)`,
grabar las entradas es grabar la lección. (Es el mismo truco que `rr` usa para
depurar: registrar las entradas no deterministas y reproducir el resto.)

**El puerto.** `runtime.js` expone `window.physicsLab`: `estado()`,
`aplica(estado)` (dibuja sincrónicamente), `alCuadro(f)` y `detenerReloj()`.
Es un puerto de depuración — como JTAG: el objetivo no sabe quién está al
otro lado. El grabador (`js/estudio.js`, que sólo se descarga con `?estudio`
en la URL) es un cliente; un renderizador sin pantalla es otro; guion puede
ser el tercero. El alumno nunca descarga el estudio.

**El formato** (`physics-lab/toma@1`): el estado completo en `t = 0`, luego
sólo lo que cambió y cuándo, y marcas (tecla **M**) para la narración. El
estado en `τ` es el último `p` y el último `x` con `t ≤ τ`: retención de
orden cero, así que la toma es una función del tiempo definida en todas
partes. Cada toma lleva el **SHA-256 del `lesson.wasm`** con que se grabó:
procedencia, como los filmes de motoreel — quien la re-renderice puede
comprobar que usa la misma física.

**Dos caminos a video**, y cuándo usar cada uno:

| | A · navegador sin pantalla | B · guion (`wasmi` + motoreel) |
|---|---|---|
| Pinta | el mismo `runtime.js`: tipografía, acentos, CSS, idéntico a clase | `Prim2` → motoreel; su rasterizador aún no escribe acentos (lessons-common lo documenta) |
| Física | la del `.wasm`, en el navegador | la del mismo `.wasm`, en `wasmi`: bits idénticos |
| Hoy | **funciona** (§7.2: medido en esta rama) | falta: texto en `WasmLesson`, adaptador prims→`Prim2`, fuentes |
| Para | clases capturadas, demos del lab | reels de guion con marca, cama y mezcla |

Y un tercero, que resultó ser el destino (§7.2): **C · el SVG del pintor,
rasterizado fuera del navegador** con un rasterizador determinista (`resvg`)
y las mismas fuentes — el dibujo de A con la reproducibilidad de B.

**La voz.** Las marcas de la toma son los *cues* de `[[narration]]` de
guion. Propuesta concreta para guion-video-creator: `guion import-toma` —
de una toma, un esqueleto de guion con un bloque `[[narration]]` por marca,
listo para que escribas el texto y elijas Piper (TTS offline, ya integrado)
o tu propia voz grabada contra el reloj de la toma. Los mismos cues dan los
subtítulos.

### 4.4 3D sin romper el ABI

La regla de motoreel es la correcta y se hereda: **no hay fuga silenciosa de
3D** — los prims siguen siendo 2D. La lección proyecta en Rust: una cámara
es un `Motor3` de garust, la órbita son dos parámetros (azimut, elevación)
que el lector arrastra, el orden de pintado se resuelve en Rust (pintor por
profundidad), y las curvas de nivel salen del mismo `curve` de siempre.

Lo que sí cambia es **el sumidero**. motoreel ya tiene la idea correcta —
`FrameSink`, con SVG y PPM detrás — y el navegador la necesita igual: el mismo
buffer de prims, dos pintores. SVG para lo nítido y exportable; **canvas**
para lo denso (una malla de 40 × 40 son 3 200 segmentos, fuera del
presupuesto de un DOM que se rehace a 60 Hz) — y canvas además se captura
más rápido para video. Se declara por vista en el manifiesto; la lección no
se entera.

### 4.5 Akademos: el lab como `ContentItem::Interactive` — ya existe

*(Reescrito el 2026-10-07, al leer akademos `main`.)* Lo que este RFC
proponía como opción (b) — mismo origen, versión fija — **ya está
construido** en akademos (R-0016, R-0037…R-0044):

- `scripts/vendor-sims.sh` copia `lesson.json`, `lesson.wasm` y las notas
  de un **commit fijo** de physics-lab a `apps/student/sims/<slug>/`, con la
  procedencia en `sims/index.json` (hoy: `d106e24`, de `feat/lab-hosted`,
  siete lecciones).
- Un curso las usa como `Interactive { widget: "sim", spec: { lesson } }`,
  servidas por el mismo binario, sin iframe.
- Las maneja **un segundo anfitrión del ABI**: `apps/student/src/sim.js`
  (puro, con pruebas) más `buildSim` en `main.js`, que llaman a las
  exportaciones del wasm directamente. No copia `runtime.js`: lo reescribe.
- Alrededor crecieron predicciones (R-0031), el banco (R-0037), elecciones
  (R-0038), una tabla de corridas **guardada en el servidor** (R-0039, la
  tabla `LAB_RUNS` de redb), gráfica y ley (R-0040), protocolo, retos,
  reporte y editor (R-0041…R-0044).

Lo que eso implica para este RFC:

1. **Dos implementaciones de un contrato.** `sim.js` dice dibujar
   "exactamente como el runtime de physics-lab", y nada lo comprueba. La
   propuesta es una **batería de conformidad**: casos dorados (manifiesto +
   parámetros + texto → buffer de prims y lecturas) que `cargo test`
   escribe aquí y `node --test` verifica allá. Es la disciplina N-versión
   de L2, aplicada al anfitrión.
2. **El brazo de texto no existe allá.** `checkManifest` no conoce
   `expresiones`: si hoy se vendorizara `velocidad`, cargaría sin error y
   dibujaría marcos vacíos — una falla silenciosa. Antes de vendorizarla,
   `sim.js` tiene que implementar el brazo (escribir en `text_ptr`,
   `set_text_len`, leer `diag_ptr`, unas 40 líneas) y las cajas de fórmula;
   como mínimo, `checkManifest` debe **rechazar** `expresiones` mientras no
   lo haga. Para lo que venga, se propone un campo `"requiere": ["texto"]`
   que todo anfitrión entienda o rechace — como los bits de capacidad de un
   descriptor USB: el driver que no conoce un bit no finge que no está.
3. **El puerto y las tomas** viven en `runtime.js`; `sim.js` no los expone.
   akademos ya graba algo parecido — una `LabRun` guarda parámetros y filas
   — pero no el tiempo. Converger es natural: una corrida puede llevar una
   toma.
4. **Los slugs son contrato.** Los cursos apuntan a una lección por su slug,
   y akademos ya vendorizó `derivada` (*La derivada como pendiente*, de
   `main`). Por eso la lección de este RFC vive en `velocidad`: reutilizar
   el slug habría cambiado la lección debajo de cursos ya escritos.

### 4.6 El mapa: matemática ↔ evento físico

La apuesta del dueño, convertida en currículo. Cada fila es una lección
posible; la columna *tipo* dice si cabe hoy (forma cerrada) o espera a
RFC-004 (simulada).

| Matemática | El evento que la describe | Tipo | Donde GA paga | Estado |
|---|---|---|---|---|
| derivada | la pendiente de una secante que gira (`derivada`, en `main`); la velocidad de un coche y la cinta registradora (`velocidad`, este RFC) | cerrada | — | **construidas las dos** |
| segunda derivada, curvatura | la aceleración; una curva en la carretera | cerrada | `κ = |v∧a|/|v|³` | **construida** (2D) |
| integral, teorema fundamental | el odómetro contra el velocímetro; área bajo v(t) | cerrada | — | sumas de Riemann **construida** en `main` (`riemann`); el odómetro, siguiente (reusa `formulas`) |
| exponencial | un capacitor que se carga; café que se enfría | cerrada | — | |
| logaritmo | decibeles; la escala de un sonido | cerrada | — | |
| trigonometría | movimiento circular, resorte (MAS) | cerrada | rotores | |
| números complejos | fasores, una rueda que gira | cerrada | el subálgebra par de Cl(2) **es** ℂ | |
| series de Fourier | epiciclos: ruedas sobre ruedas; la cuerda que vibra | cerrada (la DFT es una suma finita) | rotores que giran | |
| transformada de Fourier | un prisma; el espectro de una nota | cerrada | | |
| producto punto / cuña | trabajo; torque y área orientada | cerrada | `a·b`, `a∧b` | |
| matrices | un lienzo que se deforma | cerrada | versores | |
| eigenvalores | modos normales (dos masas, tres resortes); ejes principales de inercia | cerrada | `f(v) ∧ v = 0`: la dirección que la transformación no gira | la cadena de ondas ya los usa |
| EDO lineal | el oscilador amortiguado; un RLC | cerrada | | |
| EDO no lineal, caos | péndulo grande, doble péndulo | **simulada** | motores | espera RFC-004 |
| gradiente, optimización | una bola que rueda en un paisaje | cerrada en cuadráticas | | |
| red neuronal | el plano que se dobla capa por capa; XOR | **simulada** | | espera RFC-004 |
| probabilidad, difusión | caminata aleatoria, un gas (semilla fija: pura) | simulada | | |
| multiplicadores de Lagrange | una cuenta ensartada en un alambre | cerrada | | |

### 4.7 La red neuronal como material observable

La intuición del dueño — *una visualización que podemos imaginar como
material en un espacio 3D observable* — tiene una forma exacta, y es física:

1. **El paisaje de pérdida es un terreno**: energía potencial sobre el
   espacio de pesos. Dos direcciones del espacio de pesos y la pérdida como
   altura: una superficie 3D (§4.4), con curvas de nivel.
2. **El descenso por gradiente es una partícula sobreamortiguada** que baja
   por ese terreno; **momentum es una bola pesada con fricción** (Polyak).
3. **La tasa de aprendizaje es un paso de integración**, y su límite de
   estabilidad es el de Euler explícito: sobre una cuadrática con hessiano
   `H`, el iterado es `w_k = (I − ηH)^k w₀`, que converge sólo si
   `η < 2/λ_max(H)`. El eigenvalor más grande del hessiano es el resorte más
   rígido del paisaje. **Esto es forma cerrada**: `w_k` es una función pura de
   `k` — cabe hoy, sin RFC-004, y cada afirmación es una prueba.
4. **Eigenvalores a la vista**: los ejes del tazón cuadrático son los
   eigenvectores; sus curvaturas, los eigenvalores; el número de condición
   `λ_max/λ_min` es el zigzag que se ve.
5. **Una capa dobla el plano**: afín + no linealidad. Con XOR se ve por qué
   hacen falta dos capas. Entrenarla ya es simulada (RFC-004), con la misma
   disciplina: paso fijo, semilla fija, pérdida en pantalla.

La primera lección de esta fila no necesita nada nuevo: **"La tasa de
aprendizaje es un paso de tiempo"**, con el paisaje cuadrático escrito por el
lector en el panel de fórmulas.

---

## 5. Lo que no cambia

Las afirmaciones primero; τ como la constante del círculo; Rust para la física
y ningún JavaScript por lección; cuadros puros; la CSP estricta
(`script-src 'self' 'wasm-unsafe-eval'`, nada de terceros); verificar
manejando la página, no mirando capturas.

## 6. Lo que sí cambia (enmiendas explícitas, con su costo)

| Cambio | Costo honesto |
|---|---|
| El texto **entra** al wasm (sólo como fórmulas, analizadas) | cuatro exportaciones nuevas y un buffer de diagnósticos; aditivo, pero es superficie de ABI |
| `runtime.js` crece de 414 a 617 líneas | se aleja del "tubo de ~80 líneas" de RFC-001; el puerto es el contrapeso: el estudio y los renderizadores viven fuera |
| Un **segundo archivo JS del framework** (`estudio.js`, 215 líneas) | escrito una vez, nunca por lección, y sólo con `?estudio`; aun así rompe la letra de "THE one JS file" |
| Campos nuevos en el manifiesto: `expresiones`, `"widget": "auto"`, `unit` en lecturas, formatos `fix0…fix3` | y un `fmt` desconocido **falla al cargar** en vez de caer en silencio a `fix3` |
| Una lección con fórmulas pesa ~129 KB (59 KB en gzip), contra 33–46 KB | es toda la libm que el lector puede escribir, el analizador de flotantes de `core` y el asignador — medido en §7.1 |

---

## 7. La rebanada vertical construida (esta rama)

### 7.1 Qué hay

| Pieza | Dónde | Pruebas |
|---|---|---|
| `formulas`: léxico, sintaxis → RPN, jets de 2º orden, errores ubicados | `formulas/` (sin dependencias) | 14, incluida una batería de 11 fórmulas contra derivadas a mano y 25 000 entradas aleatorias |
| ABI de texto aditivo: `lesson!(draw, texto)`, `Diagnosticos`, `texto_valido` | `lessons-common/src/lib.rs` | 2 nuevas: el texto cruza el ABI real y los diagnósticos regresan; UTF-8 cortado a media letra |
| *La derivada es una velocidad* | `lessons/velocidad/`, `public/lessons/velocidad/` | 9: afirmaciones D1–D7, el manifiesto contra el Rust, y el arranque limpio |
| Cajas de fórmula, deslizadores automáticos, el puerto | `public/js/runtime.js`, `public/css/lesson.css` | L3 |
| El estudio: grabar, marcar, guardar, reproducir, `cuadro(t)` | `public/js/estudio.js` | L3 |

La lección: escribes `x(t)` y el coche se mueve, con **ruedas que ruedan sin
patinar** (son rotores: giran lo que el coche avanza entre su radio) y una
**cinta registradora** que marca dónde estaba cada 0.5 s. Al lado, `x(t)` con
su tangente — cuya pendiente *es* la velocidad — y la secante con su escalón
Δt/Δx; abajo, `v(t)` exacta con la secante como un punto rojo: la distancia
entre los dos es el error. Escribe también `y(t)` y el coche sale al plano:
velocidad tangente, la cuerda `Δr/Δt` a la misma escala que `v`, aceleración,
y la curvatura como producto cuña de garust.

El peso del wasm, medido por función: el código son 83 KB y los datos 21 KB;
lo grande es `libm` (reducción de argumento de sin/cos, pow, exp…), el
analizador de flotantes de `core` y `dlmalloc`. La lógica de la lección y de
`formulas` es la minoría. Si se quisiera recortar, el primer candidato no es
código: es la sección `name` (nombres de función para las trazas), que
`strip` quitaría a todas las lecciones — una decisión de toda la casa.

### 7.2 Verificación

| Capa | Qué | Resultado |
|---|---|---|
| L1 | `cargo test --workspace` | verde |
| L1a | `cargo clippy --workspace --all-targets -- -D warnings` | verde (código de salida, no un letrero) |
| L1b | build `wasm32-unknown-unknown --release` | verde |
| L3 | Chromium manejando la página **con las cabeceras reales de `_headers`** (CSP incluida), leyendo DOM y lecturas tras eventos sintéticos | 22 comprobaciones de la lección + 27 del estudio y de regresión en las seis lecciones: todas pasan, sin un error de consola ni una violación de CSP |

Algunas de las L3, porque dicen algo de la física y no sólo del software:
el error de la secante hacia adelante baja ÷10 por década de Δt y la
centrada ÷100 (medido en la página: razón 100.0); por debajo de Δt ≈ 10⁻⁸ el
error **sube** (de 3.7·10⁻⁹ en k = −8 a 7.8·10⁻⁵ en k = −12: el piso del
redondeo, D3); el círculo de radio 3 lee κ = 0.333; escribir `a t²` hace
aparecer el deslizador de `a`, y moverlo a 2 da v = 10.000 m/s en t = 2.5.

**Tomas, medido.** Una toma grabada por la interfaz (clic, cambios, tecla M,
clic) se descarga como JSON válido con la huella del wasm; `cuadro(0)` y
`cuadro(fin)` restauran exactamente los estados de los extremos, y la
retención de orden cero cambia de fórmula *exactamente* en el instante del
evento. Una toma de otra lección se rechaza con su nombre.

**El camino A, de punta a punta, medido.** Una toma de 11 s (el coche
arranca; se congela y Δt se encoge hasta que la secante se acuesta sobre la
tangente; salta al plano) se renderizó cuadro a cuadro con
`physicsLabEstudio.cuadro(i/30)` en Chromium sin pantalla, en un lienzo
vertical de 1080 × 1920 hecho **sólo con CSS** sobre el mismo pintor:

| Medida | Valor |
|---|---|
| cuadros | 331 a 30 fps |
| tiempo | 32.1 s — 97 ms por cuadro, ~3.2× más lento que tiempo real |
| video | WebM (VP8), 2.6 MB |
| la toma | 32 KB para 11 s muestreados a 60 Hz (~3 KB/s: una clase de 10 min ≈ 2 MB, sin comprimir) |
| determinismo del **dibujo** | el DOM completo del cuadro 60, alcanzado en secuencia o saltando desde el cuadro 300, es idéntico — 3 de 3 corridas |
| determinismo de la **captura** | **no garantizado**: de cuatro comparaciones re-renderizando en desorden (dos corridas × dos cuadros), una salió distinta — con ffmpeg codificando en paralelo. No se reprodujo en 3 corridas sin esa carga. No es una tesela vieja del cuadro anterior (sólo 1 % de los píxeles distintos coincide con él) ni un desplazamiento de 1–3 px. Causa sin determinar |

La conclusión, dicha sin adornos: **el camino A es determinista en lo que
dibuja, no en lo que fotografía**. Para cuadros idénticos bit a bit hay un
camino C que junta lo mejor de A y B: el pintor ya produce SVG, así que se
serializa el SVG de cada cuadro y se rasteriza fuera del navegador con un
rasterizador determinista (p. ej. `resvg`, en Rust) y las mismas fuentes
auto-hospedadas — el mismo dibujo que ve la clase, con la reproducibilidad de
motoreel. Mientras tanto, el camino A se vuelve verificable capturando cada
cuadro dos veces y comparando.

Un detalle práctico para productizarlo: el ffmpeg que trae Playwright sólo
decodifica MJPEG y no reconoce `-` como stdin; los cuadros van como JPEG por
`pipe:0`, con `-c:v mjpeg` declarado antes de `-i`.

### 7.3 Lo que NO cambió

El brazo `lesson!(draw)` se refactorizó para compartir sus buffers con el
brazo de texto (un solo generador, dos formas). Las cinco lecciones
existentes, recompiladas antes y después, dan **buffers de prims y lecturas
idénticos bit a bit en 1 347 combinaciones de parámetros**. Los binarios
difieren en exactamente 4 bytes de la sección de datos — cada uno +74: los
números de línea de los `Location` de pánico, porque `lessons-common/src/lib.rs`
creció 74 líneas por encima de `Prims`. Por eso **no** se recompilaron ni
recomprometieron sus `.wasm`.

### 7.4 Bugs del sitio en vivo, encontrados manejándolo

1. **Toda leyenda salía rota**: el runtime genera `.legend` y `.swatch`, y no
   había CSS para ninguna — las muestras de color medían cero y el texto salía
   pegado. (Afectaba a cinco de seis lecciones.)
2. **Rótulos con borde crema** sobre el escenario oscuro: el pintor usa
   `var(--paper, #f4efe3)` para el halo del texto, y `--paper` no existía.
   Ahora es un token, igual al color del escenario.
3. **Formatos ignorados en silencio**: el runtime sólo conocía `fix3`, `sci` y
   `turns3`; `fix0/1/2` caían a `fix3` y `unit` no se leía. *El plano
   inclinado* mostraba `46.092` donde su manifiesto pedía `46.1 N`.
4. **Las mayúsculas cambiaban la física**: `text-transform: uppercase` volvía
   `s` (segundo) en `S` (siemens), `kg` en `KG`, `x′(t)` en `X′(T)`. Los
   valores con unidad conservan su caja, y el manifiesto marca la matemática
   con `class='m'`.

### 7.5 Hallazgos que NO se corrigieron (fuera de alcance)

- **Reproducibilidad de los `.wasm`**: con rustc 1.94.1 aquí, ninguno de los
  cinco binarios comprometidos sale igual (y el de *plano-inclinado* pesa 39 303
  bytes comprometido contra 35 381 recompilado). Un `rust-toolchain.toml` —
  guion-video-creator ya lo tiene — haría la afirmación "desplegable desde un
  checkout" también *verificable* desde un checkout.
- **Mayúsculas en otras lecciones**: la clase `m` existe; falta aplicarla en
  sus manifiestos (por ejemplo, la lectura `θ` de *projectile* se pinta `Θ`).
- **Nativo contra wasm**: `cargo test` corre la libm de la plataforma y el
  navegador la de `compiler_builtins` (un port de musl). Pueden diferir en el
  último ulp en funciones trascendentes. Las afirmaciones usan tolerancias y no
  se ven afectadas; pero para re-renderizar *bit a bit* lo que vio el
  navegador, el camino es correr el mismo `.wasm` (como ya hace guion con
  `wasmi`), no el `rlib` nativo.

---

## 8. Plan por etapas (cada una se puede publicar)

| Etapa | Trabajo | Resultado |
|---|---|---|
| **E0** | esta rama | fórmulas, la derivada, tomas, cuatro bugs |
| **E1** | cálculo con `formulas`: la integral (odómetro/velocímetro), Fourier como epiciclos, eigenvalores como modos normales — `main` ya trae `derivada` (pendiente) y `riemann` | el tramo cerrado del mapa §4.6 |
| **E2** | RFC-004 (lecciones simuladas) adoptado; doble péndulo; "la tasa de aprendizaje es un paso de tiempo" (cerrada) y luego XOR (simulada) | la puerta a la red neuronal |
| **E3** | 3D por proyección + sumidero canvas | paisajes de pérdida, sólidos |
| **E4** | tomas → guion: `guion import-toma`, `[[narration]]` desde las marcas, camino A productizado, camino B en `guion-motion` | de la clase al reel |
| **E5** | Akademos: el alojamiento ya existe (§4.5); falta el brazo de texto en `sim.js`, la batería de conformidad, `"requiere"`, y las prácticas como predicados sobre el estado | el lab con fórmulas dentro de los cursos |
| **E6** | el estudio como producto: lienzos 9:16 y 16:9, teleprompter, "modo dedo" (mueves el coche con el dedo y aparece x(t)) | el estudio de física matemática |

---

## 9. Riesgos

- **El estudio es un producto, no una función.** El mayor riesgo es de
  alcance: cada etapa de §8 debe publicarse sola.
- **El runtime engorda.** El puerto es la defensa: lo que no sea montar la
  página y pintar, vive fuera de `runtime.js`.
- **Superficie de ataque.** `formulas` recibe texto arbitrario; hoy está
  acotado y con fuzzing determinista en CI. Si algún día acepta funciones
  definidas por el usuario, la prueba de "ninguna entrada tumba la página"
  tiene que crecer con él.
- **Dos caminos a video.** Mantener A y B a la vez duplica el pintor. Por eso
  B sólo tiene sentido cuando motoreel escriba acentos.

---

## 10. Decisiones que te tocan

1. **¿Adoptas "cada cuadro es una función pura de (entradas, t)"** como el
   principio, en lugar de "sólo forma cerrada"? (Recomiendo sí; §4.2.)
2. **¿El borrador de lecciones simuladas entra como RFC-004?** El número
   RFC-002 que tenía previsto ya está tomado.
3. ~~¿`estudio.js` como segundo archivo del framework?~~ **Decidido**
   (2026-10-07): sí, "si es necesario" — ver el registro.
4. **Gramática de `formulas`**: ¿`1/2t` como `(1/2)·t` (hoy) o como
   `1/(2t)` (como muchas calculadoras)? ¿Coma decimal rechazada (hoy) o
   aceptada?
5. ~~Akademos: ¿iframe o mismo origen?~~ **Decidido por construcción**:
   mismo origen, con vendorización fija (§4.5). Lo abierto ahora es otro:
   ¿un anfitrión del ABI o dos con batería de conformidad?
6. **Video: ¿camino A ya (con doble captura para verificar), C (SVG +
   `resvg`) como destino, y B sólo para los reels con la marca de guion?**
7. **¿Arreglar la costura débil antes de crecer?** Parámetros y lecturas por
   nombre, con un enum generado por `lesson!` (DESIGN.md §5). La lección nueva
   ya compara su orden de parámetros con el manifiesto en una prueba; hacerlo
   para todas costaría poco.
8. **¿Idioma?** La lección nueva está en español, como la de mecánica; el hub y
   las primeras cuatro, en inglés. ¿El lab es bilingüe, o español primero?
9. **¿Fijar el toolchain** (`rust-toolchain.toml`) para que los `.wasm` se
   puedan verificar desde un checkout?

---

## 11. Estado del arte, y qué hace distinto a esto

- **Desmos, GeoGebra**: escribir y ver. Sin evento físico al otro lado, y sin
  afirmaciones que se puedan romper.
- **PhET** (Colorado; *The Moving Man* es la derivada con un muñeco): el
  evento, excelente; la matemática, no se escribe.
- **manim** (3Blue1Brown) y **manim-voiceover**: animación matemática
  programada y sincronizada con voz (propia o TTS). Es la referencia para E4 —
  pero es una película, no un aula: no se maneja.
- **Bret Victor** (*Up and Down the Ladder of Abstraction*, *Kill Math*),
  **Explorable Explanations**, **Distill** (*Why Momentum Really Works*, Goh
  2017: el descenso con momentum como una bola, con deslizadores).
- **TensorFlow Playground**; Olah, *Neural Networks, Manifolds, and Topology*
  (2014); Li et al., *Visualizing the Loss Landscape of Neural Nets* (2018).

Lo distinto aquí no es ninguna pieza, es el cierre: **el mismo modelo** corre
en la clase, en el video y en el curso; **cada número en pantalla es una
prueba** que un alumno puede romper; las grabaciones guardan **entradas, no
píxeles**; y la destinación del programa es el álgebra geométrica.

---

## Registro de decisiones

| Fecha | Decisión | Razón |
|---|---|---|
| 2026-09-29 | El texto entra al wasm sólo como fórmulas analizadas, nunca ejecutadas | la única entrada arbitraria del lab; sin `eval`, la CSP no se toca |
| 2026-09-29 | Derivadas por diferenciación automática, la secante como aproximación en pantalla | la definición no puede ser también la referencia |
| 2026-09-29 | Errores como `(código, columna)` en f64; mensajes en la página, comparados con Rust por una prueba | el buffer sigue siendo de números; la costura tiene guardián |
| 2026-09-29 | Tomas de entradas, no de píxeles, a través de un puerto; el estudio fuera de `runtime.js` | pureza ⇒ re-render exacto en cualquier formato; el alumno no descarga el estudio |
| 2026-09-29 | No recompilar los `.wasm` de lecciones no tocadas | no son reproducibles con este toolchain; su comportamiento se verificó idéntico |
| 2026-10-07 | `estudio.js` como segundo archivo del framework — **aprobado por el dueño** («está bien si es necesario») | lo necesario es el puerto; `estudio.js` es su cliente para grabar desde cualquier navegador sin instalar nada, y el alumno nunca lo descarga. Dentro de akademos no aplica (allá no hay puerto; ver §4.5) |
| 2026-10-07 | Al fusionar `main`, la lección de este RFC pasa de `derivada` a `velocidad` | `main` ya tenía *La derivada como pendiente* en `derivada`, y akademos ya la vendorizó con ese slug: un slug no se reutiliza |

## Changelog

- 2026-09-29 — creado (Discussing), con la rebanada vertical E0 construida.
- 2026-10-07 — fusionado con `main` (elecciones, `bool`, unidades, `derivada`
  de pendiente, `riemann`, `curva`, `difraccion`, `vehiculo`); la lección pasa a
  `velocidad`; §4.5 reescrito contra lo que akademos construyó; `estudio.js`
  aprobado.
