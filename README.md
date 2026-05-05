# CanicasBrawl — rapier

Juego de canicas que se pelean en una arena. Rust + Bevy + Rapier. Figma como editor de niveles.

---

## Requisitos

- **Rust** (stable) — `rustup`
- **ffmpeg** — solo para grabación: `brew install ffmpeg`
- **Python 3** — para exportar niveles desde Figma
- **Node.js / npx** — para el MCP de Figma
- **FIGMA_TOKEN** en el entorno (ver abajo)

---

## Comandos rápidos

```bash
cargo run                                    # ejecutar el juego
cargo run -- --process-figma                 # exportar nivel desde Figma → JSON → juego
cargo run -- --record 60                     # grabar 60 s → outputs/record_60s.mp4
cargo run -- --debug                         # colisionadores visibles (RapierDebug)
cargo run -- --sim-raw                       # física sin precomputing
```

---

## Configuración de Figma como editor de niveles

### 1. Token de Figma

Genera un personal access token en [figma.com → Account Settings → Personal access tokens](https://www.figma.com/settings).

Agrégalo a `~/.zshrc`:

```bash
export FIGMA_TOKEN="figd_xxxxxxxxxxxxxxxxxxxx"
```

Recarga el shell:

```bash
source ~/.zshrc
```

### 2. Plugin de Figma (figma-mcp-go)

Este plugin permite que Claude Code lea y escriba Figma directamente desde la terminal.

**Instalación del plugin en Figma:**

1. Abre Figma → menú hamburguesa → **Plugins → Development → New Plugin**
2. Selecciona **"Link existing plugin"** o busca en la comunidad: **figma-mcp-go**
3. Ejecuta el plugin en el archivo "Levels" antes de cada sesión con Claude

**MCP en Claude Code** (ya configurado en `.mcp.json`):

```json
{
  "mcpServers": {
    "figma-mcp-go": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@vkhanhqui/figma-mcp-go@latest"]
    }
  }
}
```

El MCP conecta vía WebSocket al plugin de Figma que corre en el browser. Claude puede leer nodos, moverlos, renombrarlos, etc.

> **Importante:** el plugin de Figma debe estar corriendo en el browser para que el MCP funcione. Si Claude no puede conectar, revisar que el plugin esté activo.

---

## Pipeline de niveles

```
Editas Figma  →  cargo run -- --process-figma  →  cargo run
               (lee REST API, guarda JSON)         (carga JSON, sin recompilar)
```

1. Diseña el nivel en Figma (archivo **"Levels"**, frame **"Level_01"**)
2. Corre `cargo run -- --process-figma` → genera `assets/levels/level_01.json`
3. Corre `cargo run` → el juego carga el JSON al iniciar

El JSON es la fuente de verdad del nivel. **No editar `level_01.json` a mano.**

---

## Convención de nombres de capas en Figma

El script `figma_export.py` interpreta las capas del frame **Level_01** según estos nombres:

| Nombre de capa | Efecto en el juego |
|---|---|
| `platform` | Plataforma estática. Si w≈h y w>25px → se convierte en rombo (rot 45° automático) |
| `platform\|r45` | Plataforma estática con rotación explícita en grados |
| `platform\|w1.5` | Plataforma kinematic, gira 1.5 rad/s anti-horario |
| `platform\|w-1.5` | Plataforma kinematic, gira 1.5 rad/s horario |
| `platform\|r45\|w1.5` | Rotación inicial + velocidad angular combinadas |
| `floor` | Rectángulo cuyo **borde superior** define la Y del suelo |
| `spawn_area` | Rectángulo cuyo **centro** define dónde spawnea el grid 3×3 de canicas |
| `ref_marble` | Círculo de 18px — referencia visual en Figma, ignorado por el script |

### Cruces giratorias

Para crear una cruz giratoria: dos rectángulos con **exactamente el mismo centro** (x, y) y el **mismo `|w<val>`**. Al compartir posición y velocidad angular se comportan como una unidad.

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

> **Bug conocido en la REST API de Figma:** `absoluteBoundingBox` devuelve las dimensiones **post-rotación**. Para rombos (cuadrado rotado 45°), el bbox mide `lado × √2`. El script aplica la corrección automáticamente dividiendo por `√2`.

---

## Estructura de archivos

```
canicasbrawl-rapier/
  src/main.rs              ← lógica del juego, carga level_01.json en startup
  figma_export.py          ← traduce Figma REST API → assets/levels/level_01.json
  assets/
    levels/
      level_01.json        ← nivel generado (no editar a mano)
    characters/            ← PNGs de cada personaje + circle_white.png
  Cargo.toml               ← rapier-bevy (path dep) + serde + serde_json
  .mcp.json                ← configuración MCP para Claude Code
```

El binario depende del crate local `../rapier-bevy`. Comparten `target/` (configurado en `.cargo/config.toml`).

---

## Grabación de video

```bash
cargo run --release -- --record 60    # graba 60 s → outputs/record_60s.mp4
```

Requiere `ffmpeg`. La simulación corre a velocidad acelerada (50×) para generar contenido largo rápidamente.
