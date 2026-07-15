# Flujos del repo

Diagramas vivos. Cuando el código cambie, corre `/update-flow` y Claude actualiza esto.

## 1. Flowchart de `main`

```mermaid
flowchart TD
    Start([cargo run]) --> Parse["args::parse_command()<br/>--process-modules<br/>--seed N (random si falta)<br/>--slots N o --characters A,B,... (excluyentes)<br/>--rosa / --neon (paleta; default azul)<br/>--write-timeline T / --play X.timeline / --record T (los lee el engine)"]
    Parse --> Match{match Command}
    Match -- BuildModules --> Editor["process_modules::run()"]
    Match -- "Simulation(seed, roster, palette, video_secs)" --> Run["simulation::run<br/>roster: Default | Characters | Slots(n)"]

    Run --> Fases["game_app(config con seed)<br/>━━━━━━━━━━━━━<br/>on_start — nacer:<br/>prepare_the_race (elenco + marcador + reglas + el micrófono de producción),<br/>build_the_world (mundo, gravedad, cámara, corona),<br/>prepare_the_scene (semilla del telón, paleta, cielo, estrellas, nubes)<br/>━━━━━━━━━━━━━<br/>on_update — el loop central, actos por semántica<br/>(cada uno declara su ritmo adentro):<br/>• generate_the_level — el director tira los Dice del engine (FixedUpdate)<br/>• run_the_sensors — oídos, relojes e insignias<br/>• run_the_event_band::&lt;RaceEvent&gt; (engine) + stage_race_events<br/>• track_the_leader — meta → líder → voz (chain), cámara,<br/>&nbsp;&nbsp;corona persiguiendo (PostUpdate)<br/>• animate_the_backdrop + follow_the_marbles — lo visual persigue<br/>━━━━━━━━━━━━━<br/>on_exit — morir: save_voice_tracker_on_exit"]

    Fases --> Dark["el ENGINE decide el mundo, a oscuras del juego:<br/>nativo y --write-timeline → física + Dice en la mesa<br/>--play → sin física ni Dice: la partitura dicta poses y eventos,<br/>y todo sistema que declare choques (EventReader&lt;CollisionEvent&gt;)<br/>o azar (ResMut&lt;Dice&gt;) se duerme solo"]

    Editor --> End([exit])
    Dark --> Loop["app.run"]
    Loop --> End
```

`simulation.rs` es la vida del juego en tres fases (`on_start` → `on_update` →
`on_exit`), sin una sola rama de modos: el juego declara necesidades y el
engine arma la mesa. Cmd+click sobre cualquier system salta a su implementación.

Asinceramiento 2026-07-15: `--preprocess`, `--sim-raw`, `--debug` y `--bench`
murieron (nunca se usaban; git los guarda). Los colisionadores de malla vienen
SIEMPRE de `.compound` — quien fabrica el asset fabrica su compound.

## 2. Composición engine ↔ juego

```mermaid
flowchart LR
    Game["canicasbrawl-rapier<br/>main + args + simulation<br/>+ game/ + production/ + process_modules/"] -->|game_app + add_systems| Engine
    Engine["rapier-bevy<br/>engine, modes, timeline, plugins, world_objects"] -->|expone| API["API:<br/>spawn_object, ObjectDef, ColliderShape...<br/>timeline.rs: Timeline, Pose, TimelineKey,<br/>TimelineEvents, PlayEvent, Dice,<br/>run_the_event_band + EventBand<br/>WriteTimelinePlugin / PlayPlugin / RecordPlugin<br/>timeline_path / write_timeline_duration / record_duration<br/>/ session_duration_secs"]
    Demo["rapier-bevy/main.rs<br/>demo: vehículo + escalera"] -->|game_app + setup| Engine
```

El engine no conoce a ningún juego: mueve cuerpos, escribe y actúa timelines,
pone (o no) los Dice en la mesa, y toca la banda de eventos completa — el
juego solo aporta su aduana (`#[derive(Event, Serialize, Deserialize)]` en su enum) y
su escenografía. En `--play` duerme, por necesidad ausente, a todo sistema que
declare choques o azar: el juego jamás pregunta por modos.

## 3. Pipeline editor → juego

```mermaid
flowchart LR
    Figma["Figma página Modules<br/>frames Crosses, Zigzag, ..."] -->|/export-module| Raw["assets/modules/raw/*.json"]
    Raw -->|"cargo run -- --process-modules<br/>(process_modules/: shapes + torus_assets)"| Final["assets/modules/*.json"]
    Final -->|"world::modules::load_module<br/>spawn vía RaceEvent::Module → staging"| World["Mundo Bevy<br/>plataformas + paredes + canicas"]
    World -->|cargo run| Window["Ventana interactiva"]
    World -->|"cargo run -- --record N"| Pipe["ffmpeg pipe"]
    Pipe --> MP4["outputs/record_Ns.mp4"]
```

## 4. Pipeline producción de video (write-timeline anónimo → casting → play vestido)

Este repo es **uno de los dos juegos** que orquesta `~/canicasbrawl-production`
(el otro es `~/musical-path-rapier`). El contrato: producir timeline +
voice_tracker (write-timeline) y un mp4 crudo (record); todo lo demás (covers
RVC, mezcla, subtítulos, card, B2, publicación) vive en production.

```mermaid
flowchart TD
    Sim["cargo run --release --<br/>--write-timeline 60 --slots 9 --seed S<br/>física anónima slot_0..slot_8, sin render"] --> Timeline["outputs/simulation_60s.timeline"]
    Sim --> VT["outputs/voice_tracker.json<br/>líderes por slot"]
    Timeline --> Prod[("canicasbrawl-production:<br/>alberca de timelines compartida,<br/>judge elige el mejor run,<br/>sampler decide el cast")]
    VT --> Prod
    Prod --> Render["cargo run --release --<br/>--record 60 --play chosen.timeline<br/>--seed S --characters A,B,...<br/>(la posición i viste al slot_i)"]
    Render --> RawMP4["outputs/record_60s.mp4"]
    RawMP4 --> Ensamble["production: covers RVC + mezcla<br/>+ subtítulos karaoke + song card<br/>→ B2 → TikTok / Instagram"]
```

Detalle del contrato en la sección "Contrato bake/replay" de `CLAUDE.md` y el
contrato de datos en `../rapier-bevy/src/timeline.rs`.

## 5. Pipeline general (materializado en canicasbrawl-production)

El "pipeline futuro" que vivía aquí ya existe: es el repo
`~/canicasbrawl-production` (discovery → planner → runs → audio → video →
publisher, con feedback de métricas al planner). Sus diagramas viven en
`~/canicasbrawl-production/docs/flow.md`; este repo solo aporta el área `runs`.

## 6. Módulos del juego

```mermaid
flowchart TD
    main["main.rs<br/>match: 2 comandos"] --> args["args.rs<br/>Command, RosterSpec"]
    main --> sim["simulation.rs<br/>3 fases de vida:<br/>on_start / on_update / on_exit<br/>cero ramas de modo"]
    main --> pm["process_modules/<br/>mod: run + transform<br/>shapes: un from_raw por forma<br/>torus_assets: .obj/.compound"]

    sim --> aduana["race_events.rs<br/>ADUANA: enum RaceEvent<br/>derive de serde = ida y vuelta<br/>desde la estructura misma<br/>(la banda vive en el engine)"]
    sim --> worlddir["world/ — EL MUNDO (interactuable)<br/>level_generation: el director de pista<br/>modules: aduana JSON + spawn_module<br/>marbles: el cuerpo de la canica<br/>staging: la escenografía de ambos mundos<br/>sensors/: freeze, shrink, swap, bouncy<br/>pickups + setup + structures"]
    sim --> racedir["race/ — LA CARRERA<br/>roster: casting · faces: la cara<br/>labels: etiquetas de nombre<br/>finish: meta + FinishTarget<br/>leader: el líder + su corona"]
    sim --> scenedir["scene/ — LA ESCENA<br/>background/: telón + BackdropSeed<br/>camera: encuadre y luces"]
    sim --> tracker["production/ — EL OBSERVADOR<br/>voice_tracker.rs"]

    worlddir --> ModulesJSON[("assets/modules/*.json")]
    aduana --> engine[("rapier-bevy<br/>timeline.rs + plugins")]
    sim --> engine
```

`simulation.rs` referencia cada system por su ruta completa
(`game::sensors::freeze::on_freeze_contact`, `game::staging::stage_race_events`,
...). Los `mod.rs` son índices puros — sin Plugins ni funciones de composición.
