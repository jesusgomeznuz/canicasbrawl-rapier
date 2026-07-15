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

# Jugar (única forma nativa; colisionadores siempre .compound precomputados)
cargo run

# El otro comando del juego: el convertidor del editor
cargo run -- --process-modules                      # raw JSON Figma → módulos finales

# Modificadores del pipeline (los lee el engine)
cargo run -- --record 30                            # graba 30 s → outputs/record_30s.mp4
cargo run -- --write-timeline 60 --slots 9 --seed 7 # casting: SOLO física, canicas anónimas → outputs/simulation_60s.timeline + voice_tracker con slot_N
cargo run -- --record 60 --play outputs/simulation_60s.timeline --seed 7 --characters A,B,...
                                                    # render desde timeline; el nombre i-ésimo viste al slot_i
# Regla: write-timeline y record de la MISMA duración y seed (la meta depende de la duración)
```

La grabación requiere `ffmpeg` instalado (`brew install ffmpeg`).

## Arquitectura

Dos crates: el **engine** reusable (`../rapier-bevy`) y el **juego** (`canicasbrawl-rapier`). Comparten target en `../rapier-bevy/target` (configurado en `.cargo/config.toml`).

El engine no conoce al juego. El juego consume el engine vía `game_app(GameAppConfig { title, resolution, seed })` y compone encima sus fases (`simulation::run`).

### `../rapier-bevy` — engine

| Módulo | Responsabilidad |
|---|---|
| `src/engine.rs` | `game_app(config) -> App` — arma el `App` según el modo del pipeline: física + `Dice` (nativo y write-timeline) o partitura (`--play`, donde duerme por necesidad ausente a quien declare choques o azar) |
| `src/timeline.rs` | El contrato bake/replay: `Timeline`, `Pose`, `TimelineKey`, `TimelineEvents`, `PlayEvent`, `Dice`, y la banda genérica (`run_the_event_band`, `EventBand`) — la aduana de cada juego es su derive de serde |
| `src/modes.rs` | La puerta del engine — preguntas puras sobre flags del pipeline: `record_duration()`, `write_timeline_duration()`, `timeline_path()`, `session_duration_secs()` |
| `src/plugins/record.rs` | `RecordPlugin` — captura GPU offscreen y pipe a ffmpeg via `crossbeam-channel`. Expone `OffscreenTarget` |
| `src/plugins/physics_stats.rs` | `PhysicsStatsPlugin` — overlay de estadísticas de física |
| `src/world_objects/mod.rs` | `spawn_object` (API principal), tipos `ObjectDef`, `ColliderShape`, `VisualDef`, `JointDef` |
| `src/world_objects/vehicle.rs` | `spawn_vehicle` con articulaciones tipo revoluta y motores |
| `src/world_objects/chain.rs` | `spawn_chain` con `ChainDef` / `ChainPath` |
| `src/world_objects/colliders.rs` | `build_collider` (mallas SIEMPRE desde `.compound`) + `preprocess_obj` (VHACD → `.compound`, lo usa quien fabrica assets) |
| `src/main.rs` | Demo del engine — consumidor del motor con su propia cámara y luces |

### `canicasbrawl-rapier` — juego

```
src/
  main.rs              parse_command + match top-level (2 comandos)
  args.rs              parseo CLI → Command
  simulation.rs        la vida del juego en 3 fases: on_start (nacer) /
                       on_update (el loop central: actos por semántica, cada
                       gorda declara su ritmo adentro) / on_exit (morir).
                       Cero ramas de modo: el juego nunca pregunta.
  process_modules/     el convertidor del editor (espejo escritor de spawn_module)
    mod.rs             run + transform — el flowchart, con las aduanas RawModule/WorldObject
    shapes.rs          un from_raw por forma + parseo de tags del nombre
    torus_assets.rs    fábrica de .obj/.compound del torus
  game/
    race_events.rs     ADUANA de eventos: enum RaceEvent con derive de serde
                       (ida y vuelta derivadas — la banda vive en el engine)
    staging.rs         escenografía única de ambos mundos (consume RaceEvent)
    marbles.rs         la canica: componentes, ensamblador, cuerpo, mesh
    labels.rs          etiquetas de nombre: spawn + seguimiento en pantalla
    faces.rs           la cara: disco + PNG del personaje + color dominante
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
```

## Flujo de arranque

```
main
└── match parse_command()
    ├── BuildModules    → process_modules::run()
    └── Simulation(...) → simulation::run(seed, roster, palette, video_secs)
```

`simulation::run` es la vida del juego en tres fases: `on_start` (nacer),
`on_update` (el loop central — actos agrupados por semántica, cada gorda
declara su ritmo adentro con vocabulario Bevy: FixedUpdate la verdad, Update
lo que se anima, PostUpdate los que persiguen) y `on_exit` (morir). Cada gorda
referenciada es un cmd+click a su flowchart en su carpeta de oficio. **No se
usan Plugins-wrapper triviales ni se envuelve lo que Bevy ya nombra bien** —
la ficha técnica es visible.

## Convenciones clave

- `spawn_object` es el constructor central: recibe `ObjectDef` con forma de colisionador, material visual, tipo de cuerpo, y joint opcional; todo lo demás (vehículos, cadenas) lo envuelve.
- Los colisionadores de malla vienen SIEMPRE de archivos `.compound` (VHACD precomputado). Quien fabrica el asset fabrica su compound: `torus_assets.rs` llama `preprocess_obj` durante `--process-modules`. El juego nunca descompone geometría en runtime.
- `game_app(config)` se encarga de DefaultPlugins, RapierPhysicsPlugin + `Dice` (cuando toca física), FrameTimeDiagnostics, PhysicsStatsPlugin, RecordPlugin (si `--record`) y PlayPlugin (si `--play`). El juego compone encima sus fases con `add_systems` directo — sin Plugins-wrapper.
- Cada juego pone su propia cámara, luces y `ClearColor`. Si necesita renderizar a `--record`, lee `Res<OffscreenTarget>` (recurso opcional inyectado por `RecordPlugin`).
- La simulación corre en `FixedUpdate` a 60 steps/s (`TimestepMode::Fixed`, fijado en `engine.rs`). Toda la lógica de tiempo del juego vive en `FixedUpdate` para contar steps, no el wall-clock. En modo grabación, `RecordPlugin` usa `TimeUpdateStrategy::ManualDuration(1/60)`: cada frame avanza un step fijo y captura un frame de video (1 step = 1 frame), así la duración del MP4 == la del tiempo simulado. La aceleración de producción viene del loop headless (`run_loop(ZERO)`), no de un multiplicador de tiempo.

## Contrato bake/replay (al agregar contenido o efectos)

Play NO re-simula ni re-deriva nada: todo cruza de write-timeline a play como datos
(poses por TimelineKey, eventos tipados para lo demás). El contrato universal
vive en `../rapier-bevy/src/timeline.rs` (Timeline, Pose, TimelineKey, Dice);
el vocabulario del juego en `src/game/race_events.rs`: el enum
RaceEvent con `#[derive(Event, Serialize, Deserialize)]` ES la aduana — la ida
y la vuelta se derivan de la estructura, sin formato a mano que desalinear.

LA LEY DEL JUEGO PURO: el código del juego jamás pregunta por modos. Cada
sistema DECLARA sus necesidades en la firma y el engine arma la mesa: la
verdad nueva solo nace del choque (`EventReader<CollisionEvent>`) o del azar
(`ResMut<Dice>`), y en `--play` no existe ninguno de los dos — quien los
declara se duerme solo (necesidad ausente → skip silencioso del engine).

La banda de eventos es ESTRUCTURA del engine:
`rapier_bevy::run_the_event_band::<RaceEvent>(app, stage_race_events)` toca
actuación → buzón → escenografía en el mismo tick; los que emiten en ese tick
(sensores, director) se ordenan `.before(rapier_bevy::EventBand)`. Al extender
el juego:

- **Módulo nuevo** (JSON via --process-modules): solo agregarlo al pool de
  `pick_module` con su peso. Spawn, BakeKeys y evento `Module` son genéricos.
- **Más sensores freeze/shrink/swap/bouncy**: cero cambios.
- **TIPO de efecto nuevo por colisión**: 3 pasos —
  1. variante nueva en `RaceEvent` (el derive de serde hace el resto);
  2. sistema de contacto en su carpeta de sensores (registrado en la gorda
     `run_the_sensors`) que aplica SOLO la parte física y emite la variante —
     su `EventReader<CollisionEvent>` ya lo duerme solo en play;
  3. brazo en `staging::stage_race_events` con su utilería (visuales/despawns —
     el movimiento de cuerpos viene gratis en las poses).
- **Cuerpo RigidBody spawneado fuera de spawn_module/marbles**: asignarle una
  `TimelineKey` determinista (sin ella cae al índice de Entity, que diverge si
  hay despawns).
- **Sistema que necesite azar**: declarar `ResMut<Dice>` (los dados del engine,
  sembrados del --seed) — nunca un RNG propio: los dados son también la marca
  de "creador de verdad" que duerme en play.

Nada falla en silencio: sin el brazo de escenografía el match del enum no
compila; un renglón ilegible o keys que no cuadran hacen panic con mensaje.

## Cómo extender modos

- **Modo nuevo del pipeline** (afecta a todos los juegos): pregunta pura nueva en `modes.rs` del engine y rama en el match de `game_app`. El juego jamás se entera.
- **Comando nuevo del juego** (solo canicasbrawl): añadir variante a `Command`, una rama en `parse_command` y otra en el `match` de `main`.
- La regla de asinceramiento (2026-07-15): lo que no se usa se AMPUTA, no se archiva — `--debug`, `--bench`, `--preprocess` y `--sim-raw` murieron; git los guarda si un día hacen falta.
