# CanicasBrawl — rapier

Juego de canicas que se pelean en una arena. Rust + Bevy + Rapier. Figma como
editor de módulos de nivel.

El binario depende del crate local `rapier-bevy`, que es el engine (física,
grabación, timelines). **Los dos repos tienen que estar clonados como carpetas
hermanas** — la dependencia es por ruta relativa:

```
git clone https://github.com/jesusgomeznuz/rapier-bevy.git
git clone https://github.com/jesusgomeznuz/canicasbrawl-rapier.git
cd canicasbrawl-rapier
cargo run
```

---

## Requisitos

- **Rust** (stable) — `rustup`
- **ffmpeg** — solo para `--record`: `brew install ffmpeg`
- **FIGMA_TOKEN** — **solo** si vas a regenerar módulos con `--process-modules`.
  Para jugar y para grabar no hace falta.

---

## Comandos rápidos

```bash
cargo run                                  # ejecutar el juego
cargo run --release -- --record 60         # grabar 60 s → outputs/
cargo run -- --seed 12345                  # partida reproducible
cargo run -- --characters goku,bart,finn   # elenco fijo
cargo run -- --slots 5                     # N corredores anónimos
cargo run -- --neon                        # paleta neon (default: azul)
cargo run -- --process-modules             # Figma → assets/modules/*.json
```

### Flags del juego

| Flag | Efecto |
|---|---|
| `--seed <u64>` | Semilla de la partida. Sin él, se usa el reloj |
| `--characters <a,b,c>` | Elenco explícito, por nombre de PNG en `assets/characters/` |
| `--slots <n>` | `n` corredores anónimos. **Excluyente con `--characters`** |
| `--neon` / `--rosa` | Paleta de color. Sin flag, `azul` |
| `--process-modules` | Lee Figma y regenera `assets/modules/*.json`. No juega |

### Flags del engine (`rapier-bevy`)

Los expone el engine, así que valen para cualquier juego construido sobre él.

| Flag | Efecto |
|---|---|
| `--record <secs>` | Graba a mp4 en `outputs/`. Default 60 s |
| `--write-timeline <secs>` | Escribe la timeline de la partida. Default 60 s |
| `--play <ruta.timeline>` | Reproduce una timeline ya escrita |

> `--bake`, `--simulate` y `--replay` **fueron renombrados** a
> `--write-timeline` (los dos primeros) y `--play`. Si los usas, el binario
> aborta con el mensaje del reemplazo en vez de fallar raro.

---

## Grabación de video

```bash
cargo run --release -- --record 60
```

Requiere `ffmpeg`. La simulación corre acelerada para generar contenido largo
rápido. Usa `--release`: en `dev` la física va demasiado lenta para grabar.

---

## Pipeline de módulos (Figma)

El nivel ya no es un archivo único generado desde Figma: se arma con **módulos**
que viven en `assets/modules/*.json` y ya están en el repo. Solo necesitas este
pipeline si vas a **diseñar módulos nuevos**.

```
Editas Figma  →  cargo run -- --process-modules  →  cargo run
                 (lee REST API, guarda JSON)        (arma el nivel con módulos)
```

### 1. Token de Figma

Genera un personal access token en
[figma.com → Account Settings → Personal access tokens](https://www.figma.com/settings)
y agrégalo a `~/.zshrc`:

```bash
export FIGMA_TOKEN="figd_xxxxxxxxxxxxxxxxxxxx"
```

### 2. Plugin de Figma (figma-mcp-go)

Permite que Claude Code lea y escriba Figma desde la terminal.

1. Figma → menú hamburguesa → **Plugins → Development → New Plugin**
2. **"Link existing plugin"**, o búscalo en la comunidad: **figma-mcp-go**
3. Córrelo en el archivo de niveles antes de cada sesión

El MCP ya está configurado en `.mcp.json`. Conecta por WebSocket al plugin que
corre en el browser, así que **el plugin debe estar activo** o Claude no conecta.

---

## Convención de nombres de capas en Figma

| Nombre de capa | Efecto en el juego |
|---|---|
| `platform` | Plataforma estática. Si w≈h y w>25px → rombo (rot 45° automático) |
| `platform\|r45` | Plataforma estática con rotación explícita en grados |
| `platform\|w1.5` | Plataforma kinematic, gira 1.5 rad/s anti-horario |
| `platform\|w-1.5` | Plataforma kinematic, gira 1.5 rad/s horario |
| `platform\|r45\|w1.5` | Rotación inicial + velocidad angular combinadas |
| `floor` | Rectángulo cuyo **borde superior** define la Y del suelo |
| `spawn_area` | Rectángulo cuyo **centro** define dónde spawnea el grid de canicas |
| `ref_marble` | Círculo de 18px — referencia visual, ignorado por el export |

### Cruces giratorias

Dos rectángulos con **exactamente el mismo centro** y el **mismo `|w<val>`**. Al
compartir posición y velocidad angular se comportan como una unidad.

```
Barra horizontal:  platform|w1.5   hx=largo, hy=grosor
Barra vertical:    platform|w1.5   hx=grosor, hy=largo
(mismo centro exacto)
```

- Cruz en esquina izquierda → `w-1.5` (horario, barre hacia el centro)
- Cruz en esquina derecha → `w1.5` (anti-horario, barre hacia el centro)

### Sistema de coordenadas

```
1px Figma = 0.01m en el juego

Arena:  110px ancho × 960px alto  →  1.10m × 9.60m
Canica: diámetro 18px             →  radio 0.09m

Origen del juego = centro inferior de la arena
  game_x = (cx_px - 55) × 0.01
  game_y = (960 - cy_px) × 0.01
```

> **Bug conocido en la REST API de Figma:** `absoluteBoundingBox` devuelve las
> dimensiones **post-rotación**. Para rombos (cuadrado rotado 45°) el bbox mide
> `lado × √2`. El export aplica la corrección dividiendo por `√2`.

---

## Estructura de archivos

```
canicasbrawl-rapier/
  src/
    main.rs              ← el flowchart: parsea el comando y rutea
    args.rs              ← la puerta: flags → Command
    game/                ← mundo, escena, sensores, física del juego
    production/          ← voice_tracker y lo que consume el pipeline de video
    figma_to_modules/    ← Figma REST API → assets/modules/*.json
  assets/
    modules/             ← módulos de nivel (en el repo; `raw/` no se trackea)
    characters/          ← PNG por personaje + circle_white.png
    effects/  fonts/  img/
    torus_*.obj/.compound ← geometría precomputada de colisionadores
  tools/                 ← utilidades sueltas (generate_swap_icon.py)
  outputs/               ← grabaciones y timelines (no se trackea)
  Cargo.toml             ← rapier-bevy (path dep) + bevy + bevy_rapier3d + serde
  .mcp.json              ← configuración MCP para Claude Code
```

Cada repo compila a su propio `target/`. Si quieres compartir uno entre los dos
para ahorrar disco, ponlo en `.cargo/config.toml` — **está en `.gitignore` a
propósito**, porque es una decisión de tu máquina y una ruta absoluta ahí rompe
el build de cualquiera que clone.
