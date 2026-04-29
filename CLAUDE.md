# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Estado del proyecto

El archivo de verdad del progreso de CanicasBrawl es:

**`/Users/jesus/Documents/Obsidian Vault/Proyectos/CanicasBrawl-rapier.md`**

Leerlo siempre al inicio de cada sesión para saber qué está hecho y qué sigue.

## Comandos

```bash
# Compilar
cargo build

# Ejecutar en modo por defecto (Precomputed)
cargo run

# Modos de ejecución
cargo run -- --sim-raw                              # física sin precomputing
cargo run -- --bench falling-spheres 200            # benchmark (escenas: falling-spheres, stacked-boxes, chain-grid)
cargo run -- --preprocess                           # genera archivos .compound desde OBJ
cargo run -- --debug                                # activa RapierDebugRenderPlugin (colisionadores visibles)
cargo run -- --record 30                            # graba 30 s de simulación → outputs/record_30s.mp4
```

La grabación requiere `ffmpeg` instalado (`brew install ffmpeg`).

## Estructura del workspace

Este repo (`canicasbrawl-rapier`) es el binario de juego y depende del crate hermano local `rapier-bevy` en `../rapier-bevy`. Ambos comparten el directorio de compilación en `../rapier-bevy/target` (configurado en `.cargo/config.toml`).

### `../rapier-bevy` — crate de librería

| Módulo | Responsabilidad |
|---|---|
| `src/modes.rs` | `SimMode` (recurso Bevy), parseo de args CLI, `debug_enabled()`, `record_duration()` |
| `src/plugins/graphics.rs` | `GraphicsPlugin` — cámara, luces; redirige al `OffscreenTarget` cuando existe |
| `src/plugins/record.rs` | `RecordPlugin` — captura GPU offscreen y pipe a ffmpeg via `crossbeam-channel` |
| `src/plugins/benchmark.rs` | `BenchmarkPlugin` / `run_bench_mode` — mide FPS promedio y p01 durante 600 frames |
| `src/plugins/physics_stats.rs` | `PhysicsStatsPlugin` — overlay de estadísticas de física |
| `src/world_objects/mod.rs` | `spawn_object` (API principal), tipos `ObjectDef`, `ColliderShape`, `VisualDef`, `JointDef` |
| `src/world_objects/vehicle.rs` | `spawn_vehicle` con articulaciones tipo revoluta y motores |
| `src/world_objects/chain.rs` | `spawn_chain` con `ChainDef` / `ChainPath` |
| `src/world_objects/colliders.rs` | `build_collider` + `preprocess_obj` (VHACD → `.compound`) |
| `src/world_objects/bench.rs` | Escenas de benchmark: esferas en caída, cajas apiladas, rejilla de cadenas |

### Flujo de arranque

```
parse_mode()
  ├── Preprocess  → preprocess_assets()          # sin ventana
  ├── Bench       → run_bench_mode(mode)          # con ventana, mide FPS
  └── Sim         → run_world_mode(mode)
        ├── sin --record  → DefaultPlugins + FrameTimeDiagnostics + PhysicsStatsPlugin
        └── con --record  → headless (sin WinitPlugin) + RecordPlugin → ffmpeg pipe
```

## Convenciones clave

- `spawn_object` es el constructor central: recibe `ObjectDef` con forma de colisionador, material visual, tipo de cuerpo, y joint opcional; todo lo demás (vehículos, cadenas) lo envuelve.
- Los archivos `.compound` son colisionadores VHACD precomputados que `build_collider` carga cuando `SimMode::Precomputed`; `SimMode::Raw` usa la geometría exacta del OBJ.
- `RecordPlugin` debe insertarse **antes** de `GraphicsPlugin` para que `OffscreenTarget` exista cuando la cámara se crea.
- El tiempo virtual en modo grabación corre a `SPEED` (50×) para generar simulaciones largas rápidamente.
