# Protocolo: tus medidas para la sentadilla

Cómo tomar en casa, con una cinta métrica y una foto de lado, las medidas
que usa el modelo de la sentadilla, y cómo escribirlas para que sirvan igual
en los reels y en la app.

- **Plantilla para llenar:** [`medidas/plantilla-sentadilla.toml`](../medidas/plantilla-sentadilla.toml)
- **Ejemplo lleno (valores inventados):** [`medidas/ejemplo-sentadilla.toml`](../medidas/ejemplo-sentadilla.toml)
- **Correrlo:** `cargo run -q -p mecanica -- sentadilla --medidas medidas/P-7K3Q.toml --carga 100`
  (agrega `--json` para la salida que va a la app)
- **Lo que la app necesita y lo que falta allá:** [`propuestas/sargentai-r0045-medidas.md`](propuestas/sargentai-r0045-medidas.md)

## 1. La idea en una línea

Se miden **alturas desde el piso**, de pie y descalzo, hasta cuatro
referencias óseas que se encuentran con los dedos. Cada segmento es la resta
de dos alturas. Es la misma construcción de la tabla de Winter
(*Biomechanics and Motor Control of Human Movement*, 4.ª ed., 2009, fig. 4.1),
así que lo que no midas se puede estimar con su proporción de tu estatura, y
el resultado dice qué fue medido y qué estimado.

| segmento del modelo | se calcula como |
|---|---|
| fémur | altura del trocánter − altura de la rodilla |
| tibia | altura de la rodilla − altura del maléolo |
| tronco (cadera → barra) | altura de la barra − altura del trocánter |

## 2. Lo que necesitas

- Cinta métrica flexible en centímetros, pegada **vertical** a una pared o
  al marco de una puerta, con el cero exactamente en el piso.
- Una escuadra, o un libro de tapa dura, para llevar cada referencia a la
  cinta en horizontal.
- Una báscula.
- Un teléfono para la foto de lado (§4).
- Un palo de escoba (§3.6).
- Unas calcomanías redondas o un plumón lavable para marcar la piel.
- Short o ropa ajustada, y descalzo. Una persona que ayude hace todo más
  preciso; sin ayuda se puede, con un espejo.

**Cada altura se mide dos veces.** Si las dos difieren más de 1 cm, mide una
tercera y usa la mediana. En `como` anota las lecturas (por ejemplo
`"91.3 y 91.7; cinta en el marco; con ayuda"`).

### 2.1 Estatura (`[estatura]`, cm) — obligatoria

Descalzo, talones, glúteos y espalda contra la pared, mirada al frente. La
escuadra sobre la cabeza, contra la pared; lee la cinta.

### 2.2 Masa (`[masa]`, kg) — obligatoria

En báscula, en la mañana y con ropa ligera. El modelo de la barra no la usa
(no pone el peso del cuerpo en los momentos, §7); la app sí.

## 3. Las alturas, de pie

De pie, de **lado** a la pared con la cinta, peso repartido en los dos pies,
pies a la anchura de la cadera. Mide el lado derecho (o el que quieras, pero
siempre el mismo, y anótalo).

### 3.1 Hombro: acromion (`[altura_hombro]`)

Sigue la clavícula con los dedos hacia afuera hasta el borde óseo en la punta
de arriba del hombro: es el acromion. Brazo relajado. Marca el punto más alto
de ese borde y llévalo a la cinta con la escuadra.

*Típico:* 0.818 de tu estatura (143 cm si mides 1.75 m).

### 3.2 Cadera: trocánter mayor (`[altura_trocanter]`)

En el costado del muslo, una mano abajo del hueso de la cintura (la cresta
ilíaca), está la prominencia del fémur. Para encontrarla, gira el pie hacia
adentro y hacia afuera sobre el talón: el hueso que rueda bajo los dedos es el
trocánter. Marca su punto **más alto**.

*Típico:* 0.530 de tu estatura (93 cm a 1.75 m).

### 3.3 Rodilla: interlínea articular lateral (`[altura_rodilla]`)

Sentado, rodilla a 90°: por fuera, junto al tendón de la rótula, hay un
hueco blando entre el fémur y la tibia. Es la interlínea. Pon ahí la
calcomanía, ponte de pie y comprueba con los dedos que sigue sobre el hueco
(la piel se mueve un poco al estirar la rodilla); corrige si hace falta y
mide.

*Típico:* 0.285 de tu estatura (50 cm a 1.75 m).

### 3.4 Tobillo: maléolo lateral (`[altura_maleolo]`)

El hueso redondo por fuera del tobillo. Mide al **centro**, no al borde de
arriba.

*Típico:* 0.039 de tu estatura (7 cm a 1.75 m).

### 3.5 Largo del pie (`[largo_pie]`)

Una hoja en el piso contra la pared, talón contra la pared, marca la punta
del dedo más largo y mide. **El modelo de la barra no lo usa** (el tobillo es
un punto fijo del modelo, §7); la app sí, para poner el tobillo detrás del
medio pie.

*Típico:* 0.152 de tu estatura (27 cm a 1.75 m).

### 3.6 Barra (`[altura_barra]`)

Pon el palo de escoba donde llevas la barra: **barra alta** sobre el
trapecio, **barra baja** unos centímetros más abajo, sobre la parte de atrás
de los hombros. De pie, mide la altura del **centro** del palo. Anota en
`como` cuál de las dos es.

Si no la mides, el modelo pone la barra a la altura del hombro (es su
convención, y el resultado lo dice). La barra solo cambia el ángulo del
tronco, no los momentos (§7).

*Orientativo (convención, no un rango citado):* barra alta, de 0 a +5 cm
respecto del acromion; barra baja, de −3 a −8 cm. La revisión avisa fuera de
−12 a +10 cm.

## 4. El ángulo de la tibia en el fondo (`[angulo_tibia]`, grados)

Es el ángulo de la espinilla respecto de la **vertical** cuando estás abajo,
con el muslo paralelo al piso. Es la medida que más mueve el resultado (la
salida lo muestra en "Qué decide el resultado"), y no se puede estimar bien a
partir de la estatura.

1. Calcomanías en el maléolo lateral y en la interlínea de la rodilla.
2. El teléfono **de lado**, a la altura de la rodilla, a unos 3 m, vertical
   (usa el nivel de la cámara) y perpendicular a tu cuerpo. Encuadra de la
   cintura para abajo: la cara no hace falta.
3. Baja a la sentadilla hasta el muslo paralelo, con tu postura de siempre
   (pies, ancho y barra o palo como entrenas), y quédate 2 segundos.
4. En la foto, traza la línea maléolo → rodilla y mide su ángulo con la
   vertical (con una app de transportador, o con las coordenadas de los dos
   puntos: ángulo = atan(|Δx| / |Δy|)).

Anota en `como` si fue sin carga, con el palo o con la barra cargada. La
foto **no** se sube al repositorio ni a la app: solo el número va al
archivo.

*Típico:* de 10° a 45°. Si no lo mides, el modelo usa **22°**: es la
convención del modelo publicado, sin fuente, y el resultado lo dice.

**No se mide el ancho de los pies** ni su giro: el modelo es de lado (2D) y
no los usa (§7).

## 5. Llenar el archivo

1. Copia la plantilla con tu seudónimo como nombre:
   `cp medidas/plantilla-sentadilla.toml medidas/P-7K3Q.toml`. Los archivos
   de medidas reales **no se suben** al repositorio (`.gitignore` los
   excluye; solo la plantilla y el ejemplo se versionan).
2. `persona`: un seudónimo, `P-` y de 4 a 12 mayúsculas o cifras con al
   menos una cifra. Para inventarlo: `openssl rand -hex 2 | tr a-f A-F`.
   Nunca un nombre.
3. `fecha`: el día de la sesión, `AAAA-MM-DD`.
4. Para cada tabla que mediste: `fuente = "medido"`, el valor con la clave
   de su unidad (`valor_cm`, `valor_kg`, `valor_grados`), su `fecha` y
   `como` (instrumento, postura, lecturas, si hubo ayuda; sin nombres).
5. Corre `mecanica sentadilla --medidas medidas/P-7K3Q.toml`. Si algo está
   mal, te dice la línea y qué hacer, todo de una vez.

La unidad va en el nombre de la clave a propósito: si escribes `valor_m =
0.43` donde va `valor_cm`, es un error con pista, no un fémur de 0.43 cm.

## 6. Si no estás seguro

**Déjalo `estimado`.** Un número "más o menos" no se escribe como medido.
Un campo estimado no lleva valor (puede llevar `como` con la razón), y lo
pone:

| campo | si es estimado |
|---|---|
| alturas de hombro, trocánter, rodilla y maléolo; largo del pie | la proporción de Winter × tu estatura (tabla de §3) |
| barra | a la altura del hombro (convención del modelo) |
| ángulo de la tibia | 22° (convención del modelo, sin fuente) |
| estatura, masa | no se estiman: se miden siempre |

Con todo estimado, el resultado es exactamente el del modelo de siempre con
las proporciones de Winter. La salida marca cada campo como `medido` o
`estimado` y dice cuánto mueve cada uno el resultado, para que sepas qué
conviene medir primero.

## 7. Lo que el modelo es y lo que no

Es el modelo de la sentadilla de `mecanica`
([`mecanica/src/sentadilla.rs`](../mecanica/src/sentadilla.rs)), el mismo de
los reels:

- **2D, de lado** (plano sagital). Ni ancho de pies, ni giro, ni rodillas
  hacia adentro.
- **Una sola postura estática: el fondo**, muslo paralelo. No hay bajada ni
  subida, ni pico, ni "por repetición": ningún modelo publicado valida una
  cinemática de la bajada, y el modelo no la inventa.
- **Cuasi-estático:** sin aceleraciones. Subiendo rápido, el momento real es
  mayor.
- **La barra sobre el medio pie**, y el ángulo del tronco se **resuelve**
  para que quede ahí. Por eso la barra y el tronco cambian el ángulo del
  tronco, pero no los brazos de momento: esos los deciden el fémur, la tibia
  y su ángulo.
- **El tobillo es un punto fijo** del modelo (4 cm detrás del medio pie, a
  9 cm del piso): el largo del pie no entra.
- **Solo la barra.** El peso del tronco, la cabeza y los brazos no entra en
  los momentos; con él la cadera trabaja bastante más (la especificación del
  modelo de la app, SPEC-0045 §2.4.3, lo calcula para MECÁNICA 01: de
  274 N·m de barra a cerca de 366 N·m con el cuerpo). Cada salida lo dice (`bar_only`).
- **Grupos, no músculos:** extensores de cadera, extensores de rodilla.
  Repartir entre músculos es indeterminado sin un modelo musculoesquelético.
- **Física sí, clínica no:** dice qué carga cambia y dónde, en N·m. No dice
  qué es "seguro", ni nada de dolor o lesiones.
- **Los márgenes son convenciones** (±1 cm medido, ±2.5 cm estimado, ±3° o
  ±10° en el ángulo), no errores medidos. La salida suma lo que cada margen
  mueve el resultado: es una cota, no una distribución.
- **Los rangos de la revisión son convenciones:** "típico" (avisa) y
  "posible" (rechaza) alrededor de la media de Winter, para atrapar errores de
  cinta y de unidad. No son rangos poblacionales citados.

## 8. Qué sale

En palabras (sin `--json`) o en JSON (con `--json`):

- **La entrada de la app** (`app_input`): estatura, masa, carga y solo los
  largos **medidos**, en metros, con los nombres que usa la app.
- **El modelo** (`model`): la postura (tobillo, rodilla, cadera, hombro y la
  barra, en metros, `x` hacia la punta del pie desde el medio pie), el
  ángulo del tronco, y para cadera y rodilla el brazo de momento y el momento
  de la barra en N·m (positivo: trabajan los extensores), cada uno con su
  cota y la lista de supuestos.
- **Cada medida** (`measurements`): valor usado, `measured` o `estimated`,
  fecha, cómo, si el modelo la usa, cuánto mueve cada resultado por unidad y
  cuánto aporta su margen.
- **Las decisivas** (`drivers`): las medidas que más mueven el resultado.
- **Avisos** (`warnings`): lo raro que no impide calcular.

Sin `--carga` sale la postura y los brazos, pero no momentos: el modelo no
supone un peso.

## 9. Privacidad

- La persona es un seudónimo. El archivo no tiene lugar para nombres, caras
  ni fotos: una clave que no está en la plantilla es un error.
- Las fotos no se suben ni al repositorio ni a la app; solo el ángulo.
- Los archivos de medidas reales no se versionan.
- Las medidas de otra persona solo con su permiso por escrito; de un menor,
  el de sus tutores.

## 10. Para citar en un reel

La procedencia se dice en palabras: *"calculado con el modelo de sentadilla
de physics-lab (de lado, estático en el fondo, solo la barra), con medidas
tomadas el 2026-10-18 con cinta y foto; el ángulo de la tibia, estimado"*.
Si algún campo fue estimado, se dice cuál.
