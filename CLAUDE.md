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
cargo run -- --write-timeline 60 --slots 9 --seed 7 # casting: SOLO física, canicas anónimas → outputs/simulation_60s.timeline + voice_tracker con slot_N
cargo run -- --record 60 --play outputs/simulation_60s.timeline --seed 7 --characters A,B,...
                                                    # render desde timeline; el nombre i-ésimo viste al slot_i
# Regla: write-timeline y record de la MISMA duración y seed (la meta depende de la duración)

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
  main.rs              parse_command + match top-level (3 comandos)
  args.rs              parseo CLI → Command
  simulation.rs        arma el App por fases: on_start / on_step / on_frame_update
                       / after_frame_update / on_exit + if ¿no hay timeline que reproducir?
  process_modules.rs   raw JSON Figma → módulo final
  game/
    race_events.rs     ADUANA de eventos: enum RaceEvent, payload+parse juntos
    staging.rs         escenografía única de ambos mundos (consume RaceEvent)
    marbles.rs         la canica: cuerpo, mesh, cara, etiquetas
    roster.rs          casting: quién corre (build_roster / slots_roster)
    camera.rs          cámara, luces y checks de encuadre
    finish.rs          meta y orden de llegada
    leader.rs          quién va ganando + su corona
    world/
      level_generation.rs  el director: decide cuándo, cuál y dónde (LevelGen, pick_module)
      modules.rs           el constructor: qué ES un módulo — aduana del JSON + spawn_module
      pickups.rs           qué efecto cae en cada slot
      setup.rs / structures.rs  arranque del escenario; suelo y paredes
    sensors/           freeze, shrink, swap, bouncy + badges e icons compartidos
    background/        palette, sky, stars, clouds
  production/
    voice_tracker.rs   track_race_leader, save_voice_tracker_on_exit
    stall_detector.rs  watchdog wall-clock del solver
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
- La simulación corre en `FixedUpdate` a 60 steps/s (`TimestepMode::Fixed`, fijado en `engine.rs::game_app`). Toda la lógica de tiempo del juego vive en `FixedUpdate` para contar steps, no el wall-clock. En modo grabación, `RecordPlugin` usa `TimeUpdateStrategy::ManualDuration(1/60)`: cada frame avanza un step fijo y captura un frame de video (1 step = 1 frame), así la duración del MP4 == la del tiempo simulado. La aceleración de producción viene del loop headless (`run_loop(ZERO)`), no de un multiplicador de tiempo.

## Contrato bake/replay (al agregar contenido o efectos)

Play NO re-simula ni re-deriva nada: todo cruza de write-timeline a play como datos
(poses por TimelineKey, eventos tipados para lo demás). El contrato universal
vive en `../rapier-bevy/src/timeline.rs` (Timeline, Pose, TimelineKey); el
vocabulario del juego en `src/game/race_events.rs` (enum RaceEvent — payload y
parse juntos).

Los eventos son event-sourced: los contactos reales (física) y la partitura
(play) emiten el MISMO `RaceEvent` de Bevy; `staging::stage_race_events` monta
la escenografía igual en ambos mundos, y `send_race_events_to_timeline` los
escribe solo. Al extender el juego:

- **Módulo nuevo** (JSON via --process-modules): solo agregarlo al pool de
  `pick_module` con su peso. Spawn, BakeKeys y evento `Module` son genéricos.
- **Más sensores freeze/shrink/swap/bouncy**: cero cambios.
- **TIPO de efecto nuevo por colisión**: 3 pasos —
  1. variante nueva en `RaceEvent` (con su payload y su parse);
  2. sistema de contacto en `react_to_real_collisions` que aplica SOLO la parte
     física (RigidBody, grupos, teleports) y emite la variante;
  3. brazo en `staging::stage_race_events` con su utilería (visuales/despawns —
     el movimiento de cuerpos viene gratis en las poses).
- **Cuerpo RigidBody spawneado fuera de spawn_module/marbles**: asignarle una
  `TimelineKey` determinista (sin ella cae al índice de Entity, que diverge si
  hay despawns).

Nada falla en silencio: sin el brazo de escenografía el match del enum no
compila; un payload ilegible o keys que no cuadran hacen panic con mensaje.

## Cómo extender modos

- **Modo nuevo del engine** (afecta a todos los juegos): añadir variante a `EngineMode` y rama en `parse_engine_mode`.
- **Modo nuevo del juego** (solo canicasbrawl): añadir variante a `Command` en `main.rs`, una rama en `parse_command` y otra en el `match` de `main`.
- **Modificador ortogonal** (combina con cualquier modo): exponer query en el engine (`fn xxx_enabled() -> bool`) y consultarla donde aplique.
