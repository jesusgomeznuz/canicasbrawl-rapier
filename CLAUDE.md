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

# Modos del juego
cargo run -- --process-modules                      # raw JSON Figma → módulos finales
cargo run -- --preprocess                           # OBJ → .compound VHACD
cargo run -- --sim-raw                              # física sin precomputing (variante de Sim)

# Modificadores (combinables con Sim)
cargo run -- --debug                                # activa RapierDebugRenderPlugin
cargo run -- --record 30                            # graba 30 s → outputs/record_30s.mp4

# Bench vive en el engine, no en el juego
cd ../rapier-bevy && cargo run -- --bench falling-spheres 200
```

La grabación requiere `ffmpeg` instalado (`brew install ffmpeg`).

## Arquitectura

Dos crates: el **engine** reusable (`../rapier-bevy`) y el **juego** (`canicasbrawl-rapier`). Comparten target en `../rapier-bevy/target` (configurado en `.cargo/config.toml`).

El engine no conoce al juego. El juego consume el engine vía `engine::game_app(mode, config).add_plugins(GamePlugin).run()`.

### `../rapier-bevy` — engine

| Módulo | Responsabilidad |
|---|---|
| `src/engine.rs` | `game_app(mode, config) -> App` — arma el `App` con DefaultPlugins, físicas, ventana, record si aplica |
| `src/modes.rs` | `EngineMode`, `SimMode`, `BenchScene`, `parse_engine_mode(args)`, `record_duration()`, `debug_enabled()` |
| `src/plugins/record.rs` | `RecordPlugin` — captura GPU offscreen y pipe a ffmpeg via `crossbeam-channel`. Expone `OffscreenTarget` |
| `src/plugins/benchmark.rs` | `BenchmarkPlugin` / `run_bench_mode` — mide FPS promedio y p01 durante 600 frames |
| `src/plugins/physics_stats.rs` | `PhysicsStatsPlugin` — overlay de estadísticas de física |
| `src/world_objects/mod.rs` | `spawn_object` (API principal), tipos `ObjectDef`, `ColliderShape`, `VisualDef`, `JointDef` |
| `src/world_objects/vehicle.rs` | `spawn_vehicle` con articulaciones tipo revoluta y motores |
| `src/world_objects/chain.rs` | `spawn_chain` con `ChainDef` / `ChainPath` |
| `src/world_objects/colliders.rs` | `build_collider` + `preprocess_obj` (VHACD → `.compound`) |
| `src/world_objects/bench.rs` | Escenas de benchmark: esferas en caída, cajas apiladas, rejilla de cadenas |
| `src/main.rs` | Demo del engine — consumidor del motor con su propia cámara y luces |

### `canicasbrawl-rapier` — juego

```
src/
  main.rs              parse_command + match top-level (4 modos)
  game/                simulación
    mod.rs             CanicasBrawlPlugin + game::run(mode)
    world.rs           spawn_level, paredes, suelo
    level.rs           load_module → ModuleData
    marbles.rs         spawn_marbles, MarbleName, MarbleLabel
    camera.rs          spawn_camera_and_lights, follow_lowest, update_labels
  production/          post-simulación
    voice_tracker.rs   track_race_leader, save_voice_tracker_on_exit
  content/             pipeline editor
    process_modules.rs raw JSON Figma → módulo final
```

## Flujo de arranque

```
main
└── match parse_command()
    ├── ProcessModules  → content::process_modules::run()
    ├── Preprocess      → engine::preprocess_assets()
    └── Sim(mode)       → run_sim(mode)  ← composición visible en main.rs
```

`run_sim` vive en `main.rs` y compone directamente: `game_app(mode, ...).insert_resource(...).add_systems(Startup, (...)).add_systems(Update, (...))...run()`. Cada system referenciado (`game::camera::spawn_camera_and_lights`, `production::voice_tracker::track_race_leader`, etc.) es un cmd+click para saltar a su flowchart. **No se usan Plugins-wrapper triviales** — la ficha técnica del juego es visible al abrir `main.rs`.

## Convenciones clave

- `spawn_object` es el constructor central: recibe `ObjectDef` con forma de colisionador, material visual, tipo de cuerpo, y joint opcional; todo lo demás (vehículos, cadenas) lo envuelve.
- Los archivos `.compound` son colisionadores VHACD precomputados que `build_collider` carga cuando `SimMode::Precomputed`; `SimMode::Raw` usa la geometría exacta del OBJ.
- `engine::game_app(mode, config)` se encarga de DefaultPlugins, RapierPhysicsPlugin, FrameTimeDiagnostics, PhysicsStatsPlugin, RecordPlugin (si `--record`) y el debug renderer (si `--debug`). El juego compone su propio `Plugin` con `add_plugins`.
- Cada juego pone su propia cámara, luces y `ClearColor`. Si necesita renderizar a `--record`, lee `Res<OffscreenTarget>` (recurso opcional inyectado por `RecordPlugin`).
- El tiempo virtual en modo grabación corre a `SPEED` (50×) para generar simulaciones largas rápidamente.

## Cómo extender modos

- **Modo nuevo del engine** (afecta a todos los juegos): añadir variante a `EngineMode` y rama en `parse_engine_mode`.
- **Modo nuevo del juego** (solo canicasbrawl): añadir variante a `Command` en `main.rs`, una rama en `parse_command` y otra en el `match` de `main`.
- **Modificador ortogonal** (combina con cualquier modo): exponer query en el engine (`fn xxx_enabled() -> bool`) y consultarla donde aplique.
