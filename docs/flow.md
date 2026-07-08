# Flujos del repo

Diagramas vivos. Cuando el código cambie, corre `/update-flow` y Claude actualiza esto.

## 1. Flowchart de `main`

```mermaid
flowchart TD
    Start([cargo run]) --> Parse["args::parse_command()<br/>--process-modules / --preprocess / --sim-raw<br/>--seed N (random si falta)<br/>--slots N o --characters A,B,... (excluyentes)<br/>--rosa / --neon (paleta; default azul)<br/>--write-timeline T / --play X.timeline / --record T (los lee el engine)"]
    Parse --> Match{match Command}
    Match -- BuildModules --> Editor["process_modules::run()"]
    Match -- PreprocessConcaveColliders --> Pre["preprocess_concave_colliders"]
    Match -- "Simulation(mode, seed, roster, palette)" --> Run["simulation::run<br/>roster: Default | Characters | Slots(n)"]

    Run --> Fases["random_physics_game_app(mode, config)<br/>━━━━━━━━━━━━━<br/>on_start — dispara una vez al arrancar:<br/>10 resources + spawns: cámara, corona, mundo,<br/>gravedad, cielo, estrellas, nubes<br/>━━━━━━━━━━━━━<br/>on_step — cada step de física (60/s, after Writeback):<br/>• liderazgo: check_finish_crossing →<br/>&nbsp;&nbsp;update_race_leader → track_race_leader (chain)<br/>• banda de eventos: emit_race_events_from_timeline →<br/>&nbsp;&nbsp;send_race_events_to_timeline → stage_race_events (chain)<br/>• timers/animaciones en paralelo: try_unfreeze, try_unshrink,<br/>&nbsp;&nbsp;fade_swap_rings, spin_icons, bounce pulse y cooldown,<br/>&nbsp;&nbsp;camera_follows_lowest_marble<br/>━━━━━━━━━━━━━<br/>on_frame_update — cada frame de pantalla (wall-clock):<br/>fondo sigue cámara, twinkle, nubes, manage badges, stall_detector<br/>━━━━━━━━━━━━━<br/>after_frame_update — posiciones ya finales (PostUpdate):<br/>update_marble_labels, crown_follows_leader, update_badges<br/>━━━━━━━━━━━━━<br/>on_exit: save_voice_tracker_on_exit"]

    Fases --> If{"timeline_path().is_none()<br/>¿no hay timeline que reproducir?"}
    If -- "sí: mundo con física (dev y write-timeline)" --> Fisica["react_to_real_collisions:<br/>generate_level, disable_modules_above_screen,<br/>on_freeze/shrink/swap_contact, trigger_bouncy_pulse<br/>— detectan, aplican física y EMITEN RaceEvent"]
    If -- "no: mundo actuado (--play)" --> Play["nada extra: el PlayPlugin del engine<br/>escribe poses y re-emite PlayEvents;<br/>la banda común los escenifica"]

    Editor --> End([exit])
    Pre --> End
    Fisica --> Loop["app.run"]
    Play --> Loop
    Loop --> End
```

`simulation.rs` es el flowchart del juego: cinco fases con nombre de momento
(`on_start` → `on_step` → `on_frame_update` → `after_frame_update` → `on_exit`)
y una sola rama. Cmd+click sobre cualquier system salta a su implementación.

`--bench` no es modo del juego. Para correrlo: `cd ../rapier-bevy && cargo run -- --bench falling-spheres 200`.

## 2. Composición engine ↔ juego

```mermaid
flowchart LR
    Game["canicasbrawl-rapier<br/>main + args + simulation<br/>+ game/ + production/ + process_modules/"] -->|random_physics_game_app + add_systems| Engine
    Engine["rapier-bevy<br/>engine, modes, timeline, plugins, world_objects"] -->|expone| API["API:<br/>spawn_object, ObjectDef, ColliderShape...<br/>timeline.rs: Timeline, Pose, TimelineKey,<br/>TimelineEvents, PlayEvent<br/>WriteTimelinePlugin / PlayPlugin / RecordPlugin<br/>timeline_path / write_timeline_duration / record_duration"]
    Demo["rapier-bevy/main.rs<br/>demo: vehículo + escalera"] -->|random_physics_game_app + setup| Engine
```

El engine no conoce a ningún juego: mueve cuerpos, escribe y actúa timelines,
y transporta eventos como sobres opacos. Cada juego pone su cámara, sus
sistemas y su vocabulario (`RaceEvent`).

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
    main["main.rs<br/>match: 3 comandos"] --> args["args.rs<br/>Command, RosterSpec"]
    main --> sim["simulation.rs<br/>5 fases + banda de eventos<br/>+ if timeline_path().is_none()"]
    main --> pm["process_modules/<br/>mod: run + transform<br/>shapes: un from_raw por forma<br/>torus_assets: .obj/.compound"]

    sim --> race["race_events.rs<br/>ADUANA: enum RaceEvent<br/>payload + parse juntos<br/>+ puentes send/emit"]
    sim --> staging["staging.rs<br/>escenografía única de ambos mundos<br/>consume RaceEvent"]
    sim --> roster["roster.rs<br/>casting: build_roster / slots_roster"]
    sim --> finish["finish.rs<br/>meta y orden de llegada"]
    sim --> leader["leader.rs<br/>quién va ganando + su corona"]
    sim --> camera["camera.rs<br/>cámara, luces, checks de encuadre"]
    sim --> marbles["marbles.rs<br/>la canica: componentes,<br/>ensamblador, cuerpo, mesh"]
    sim --> labels["labels.rs<br/>etiquetas de nombre"]
    sim --> faces["faces.rs<br/>cara: PNG + color dominante"]
    sim --> tracker["production/voice_tracker.rs"]
    sim --> stall["production/stall_detector.rs"]

    sim --> worlddir["world/<br/>level_generation: el director<br/>modules: el constructor (aduana JSON + spawn)<br/>pickups: qué efecto cae en cada slot<br/>setup + structures"]
    sim --> sensorsdir["sensors/<br/>freeze, shrink, swap, bouncy<br/>+ badges e icons compartidos"]
    sim --> bgdir["background/<br/>palette, sky, stars, clouds"]

    staging --> worlddir
    staging --> sensorsdir
    worlddir --> ModulesJSON[("assets/modules/*.json")]
    marbles --> labels
    marbles --> faces
    marbles --> roster
    race --> engine[("rapier-bevy<br/>timeline.rs + plugins")]
    sim --> engine
```

`simulation.rs` referencia cada system por su ruta completa
(`game::sensors::freeze::on_freeze_contact`, `game::staging::stage_race_events`,
...). Los `mod.rs` son índices puros — sin Plugins ni funciones de composición.
