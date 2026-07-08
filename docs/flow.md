# Flujos del repo

Diagramas vivos. Cuando el código cambie, corre `/update-flow` y Claude actualiza esto.

## 1. Flowchart de `main`

```mermaid
flowchart TD
    Start([cargo run]) --> Parse["cli::parse_command()<br/>--process-modules / --preprocess<br/>--sim-raw / --seed N (random si falta)<br/>--slots N ⊻ --characters A,B,... (excluyentes)<br/>--rosa / --neon (paleta; default azul)<br/>--bake T / --record T / --replay path (los lee el engine)"]
    Parse --> Match{match Command}
    Match -- ProcessModules --> Content["content::process_modules::run()"]
    Match -- Preprocess --> Pre["preprocess_assets"]
    Match -- "Sim(mode, seed, roster, palette)" --> Sim["run_sim<br/>roster: Default | Characters | Slots(n)"]

    Sim --> SimDetail["game_app + GameAppConfig<br/>━━━━━━━━━━━━━<br/>resources: ClearColor, ColorPalette,<br/>VoiceTracker, StallDetector, RaceResult,<br/>FinishLineY, RaceLeader, Roster,<br/>LevelSeed, FinishTarget<br/>━━━━━━━━━━━━━<br/>Startup:<br/>• camera::spawn_camera_and_lights + leader_crown<br/>• world::setup + set_gravity<br/>• background::spawn_sky / stars / clouds<br/>• hud::spawn_hud<br/>━━━━━━━━━━━━━<br/>FixedUpdate (60 steps/s, after Writeback):<br/>• finish::check_finish_crossing →<br/>&nbsp;&nbsp;leader::update_race_leader →<br/>&nbsp;&nbsp;voice_tracker::track_race_leader (chain)<br/>• effects::unfreeze / unshrink / fade_swap_rings / spin_icons<br/>• bouncy::animate_pulse / tick_cooldown<br/>• camera::camera_follows_lowest_marble<br/>━━━━━━━━━━━━━<br/>Update (wall-clock):<br/>• background::update_sky / twinkle / clouds<br/>• effect_timers::badges (freeze/shrink)<br/>• hud::update_hud<br/>• stall_detector::detect_stall<br/>━━━━━━━━━━━━━<br/>PostUpdate (after TransformPropagate):<br/>• camera::update_marble_labels + leader_crown<br/>━━━━━━━━━━━━━<br/>Last:<br/>• voice_tracker::save_voice_tracker_on_exit"]

    SimDetail --> Branch{"replay_path()?"}
    Branch -- "física (bake / sim)" --> Fisica["FixedUpdate extra:<br/>• world::generate_level (incremental)<br/>• world::disable_modules_above_screen<br/>• effects::on_freeze/shrink/swap_contact<br/>• bouncy::trigger_bouncy_pulse"]
    Branch -- "replay (sin física)" --> Replay["FixedUpdate extra:<br/>• replay_effects::apply_replay_effects<br/>• replay_effects::expire_replay_freezes<br/>(utilería re-actuada desde la timeline;<br/>CollisionEvent no existe en replay)"]

    Content --> End([exit])
    Pre --> End
    Fisica --> Loop["app.run<br/>(loop hasta AppExit)"]
    Replay --> Loop
    Loop --> End
```

`run_sim` es una función en `main.rs` — todo lo que se ve arriba **vive en `main.rs` literal**, no escondido en un Plugin. Cmd+click sobre cualquier nombre de system salta a su implementación.

`--bench` no es modo del juego. Para correrlo: `cd ../rapier-bevy && cargo run -- --bench falling-spheres 200`. El bench es herramienta del engine para probar features, no del juego.

## 2. Composición engine ↔ juego

```mermaid
flowchart LR
    Game[canicasbrawl-rapier<br/>main + game/ + production/ + content/] -->|game_app + add_plugins| Engine
    Engine[rapier-bevy<br/>game_app, plugins, world_objects] -->|expone| API["API:<br/>spawn_object, RecordPlugin, OffscreenTarget<br/>parse_engine_mode, SimMode<br/>bake_duration / record_duration / replay_path"]
    Demo[rapier-bevy/main.rs<br/>demo: vehículo + escalera] -->|game_app + setup| Engine
```

El engine no conoce a ningún juego. Cada juego (canicasbrawl, demo) pone su propia cámara, luces, sistemas y resources.

## 3. Pipeline editor → juego

```mermaid
flowchart LR
    Figma[Figma página Modules<br/>frames Crosses, Zigzag, ...] -->|/export-module| Raw[assets/modules/raw/*.json]
    Raw -->|cargo run -- --process-modules| Final[assets/modules/*.json]
    Final -->|game::world::generate_level<br/>incremental durante la carrera| World[Mundo Bevy<br/>plataformas + paredes + canicas]
    World -->|cargo run| Window[Ventana interactiva]
    World -->|cargo run -- --record N| Pipe[ffmpeg pipe]
    Pipe --> MP4[outputs/record_Ns.mp4]
```

## 4. Pipeline producción de video (bake anónimo → casting → replay vestido)

Este repo es **uno de los dos juegos** que orquesta `~/canicasbrawl-production`
(el otro es `~/musical-path-rapier`). El contrato: producir timeline +
voice_tracker (bake) y un mp4 crudo (record); todo lo demás (covers RVC,
mezcla, subtítulos, card, B2, publicación) vive en production.

```mermaid
flowchart TD
    Bake["cargo run --release --<br/>--bake 60 --slots 9 --seed S<br/>física anónima slot_0..slot_8, sin render"] --> Timeline[outputs/bake_60s.timeline]
    Bake --> VT[outputs/voice_tracker.json<br/>líderes por slot]
    Timeline --> Prod[(canicasbrawl-production:<br/>alberca de bakes compartida,<br/>judge elige el mejor run,<br/>sampler decide el cast)]
    VT --> Prod
    Prod --> Render["cargo run --release --<br/>--record 60 --replay chosen.timeline<br/>--seed S --characters A,B,...<br/>(la posición i viste al slot_i)"]
    Render --> RawMP4[outputs/record_60s.mp4]
    RawMP4 --> Ensamble[production: covers RVC + mezcla<br/>+ subtítulos karaoke + song card<br/>→ B2 → TikTok / Instagram]
```

Detalle del contrato bake/replay en la sección "Contrato bake/replay" de
`CLAUDE.md`.

## 5. Pipeline general (materializado en canicasbrawl-production)

El "pipeline futuro" que vivía aquí ya existe: es el repo
`~/canicasbrawl-production` (discovery → planner → runs → audio → video →
publisher, con feedback de métricas al planner). Sus diagramas viven en
`~/canicasbrawl-production/docs/flow.md`; este repo solo aporta el área `runs`.

## 6. Módulos del juego

```mermaid
flowchart TD
    main[main.rs<br/>fn main + run_sim<br/>composición visible]
    main --> cli[cli.rs<br/>Command, RosterSpec<br/>parse_command y parseo de flags]
    main --> world[game/world.rs<br/>setup, set_gravity<br/>generate_level incremental<br/>disable_modules_above_screen]
    main --> marbles[game/marbles.rs<br/>build_roster / slots_roster<br/>spawn_marbles]
    main --> camera[game/camera.rs<br/>spawn_camera_and_lights<br/>follow_lowest, labels, crown]
    main --> background[game/background.rs<br/>ColorPalette azul/rosa/neon<br/>sky + stars + clouds]
    main --> hud[game/hud.rs<br/>spawn_hud + update_hud]
    main --> bouncy[game/bouncy.rs<br/>BouncyOnContact + pulse/cooldown]
    main --> effects[game/effects.rs<br/>freeze / shrink / swap<br/>collision groups]
    main --> timers[game/effect_timers.rs<br/>badges de freeze/shrink]
    main --> finish[game/finish.rs<br/>check_finish_crossing<br/>RaceResult + FinishLineY]
    main --> leader[game/leader.rs<br/>update_race_leader]
    main --> replayfx[game/replay_effects.rs<br/>re-actúa efectos horneados<br/>solo en replay]
    main --> tracker[production/voice_tracker.rs<br/>track_race_leader<br/>save_on_exit]
    main --> stall[production/stall_detector.rs<br/>detect_stall: frames patológicos del solver → AppExit]
    main --> content[content/process_modules.rs<br/>raw JSON → final JSON]
    main -->|Sim, Preprocess| engine[(rapier-bevy<br/>game_app, world_objects,<br/>RecordPlugin, OffscreenTarget,<br/>bake/record/replay helpers)]
    world --> level[game/level.rs<br/>load_module → ModuleData]
    world --> marbles
    world --> finish
    camera --> marbles
    tracker --> marbles
    effects --> marbles
    bouncy --> marbles
    finish --> marbles
    leader --> finish
    timers --> effects
    level -->|lee| ModulesJSON[(assets/modules/*.json)]
```

`main.rs` referencia directamente cada system por su ruta completa (`game::camera::spawn_camera_and_lights`, `production::voice_tracker::track_race_leader`, etc.). `game/mod.rs` y `production/mod.rs` son solo organizadores de namespace — sin Plugin ni función de composición.
