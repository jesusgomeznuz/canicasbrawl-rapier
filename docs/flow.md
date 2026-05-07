# Flujos del repo

Diagramas vivos. Cuando el código cambie, corre `/update-flow` y Claude actualiza esto.

## 1. Flowchart de `main`

```mermaid
flowchart TD
    Start([cargo run]) --> Parse["parse_command()"]
    Parse --> Match{match Command}
    Match -- ProcessModules --> Content["content::process_modules::run()"]
    Match -- Preprocess --> Pre["preprocess_assets"]
    Match -- Sim --> Sim["run_sim(mode)"]

    Sim --> SimDetail["game_app + GameAppConfig<br/>━━━━━━━━━━━━━<br/>insert ClearColor<br/>insert VoiceTracker<br/>━━━━━━━━━━━━━<br/>Startup:<br/>• camera::spawn_camera_and_lights<br/>• world::setup<br/>• world::set_gravity<br/>━━━━━━━━━━━━━<br/>Update:<br/>• camera::camera_follows_lowest_marble<br/>• voice_tracker::track_race_leader<br/>━━━━━━━━━━━━━<br/>PostUpdate:<br/>• camera::update_marble_labels<br/>━━━━━━━━━━━━━<br/>Last:<br/>• voice_tracker::save_voice_tracker_on_exit"]

    Content --> End([exit])
    Pre --> End
    SimDetail --> Loop["app.run<br/>(loop hasta AppExit)"]
    Loop --> End
```

`run_sim` es una función en `main.rs` — todo lo que se ve arriba **vive en `main.rs` literal**, no escondido en un Plugin. Cmd+click sobre cualquier nombre de system salta a su implementación.

`--bench` no es modo del juego. Para correrlo: `cd ../rapier-bevy && cargo run -- --bench falling-spheres 200`. El bench es herramienta del engine para probar features, no del juego.

## 2. Composición engine ↔ juego

```mermaid
flowchart LR
    Game[canicasbrawl-rapier<br/>main + game/ + production/ + content/] -->|game_app + add_plugins| Engine
    Engine[rapier-bevy<br/>game_app, plugins, world_objects] -->|expone| API[API:<br/>spawn_object, RecordPlugin,<br/>OffscreenTarget, parse_engine_mode]
    Demo[rapier-bevy/main.rs<br/>demo: vehículo + escalera] -->|game_app + setup| Engine
```

El engine no conoce a ningún juego. Cada juego (canicasbrawl, demo) pone su propia cámara, luces, sistemas y resources.

## 3. Pipeline editor → juego

```mermaid
flowchart LR
    Figma[Figma página Modules<br/>frames Crosses, Zigzag, ...] -->|/export-module| Raw[assets/modules/raw/*.json]
    Raw -->|cargo run -- --process-modules| Final[assets/modules/*.json]
    Final -->|game::world::spawn_level| World[Mundo Bevy<br/>plataformas + paredes + canicas]
    World -->|cargo run| Window[Ventana interactiva]
    World -->|cargo run -- --record N| Pipe[ffmpeg pipe]
    Pipe --> MP4[outputs/record_Ns.mp4]
```

## 4. Pipeline producción de video (estado actual)

```mermaid
flowchart TD
    Run[cargo run --release -- --record 60] --> Sim[Simulación headless 60s]
    Sim --> Tracker[outputs/voice_tracker.json<br/>segmentos por líder]
    Sim --> RawMP4[outputs/record_60s.mp4<br/>video sin audio]
    Tracker --> AudioScript[script Python audio<br/>concat voces por segmento + instrumental]
    RawMP4 --> AudioScript
    AudioScript --> Final[outputs/final_*.mp4<br/>video completo]
```

## 5. Pipeline producción de video (futuro)

```mermaid
flowchart TD
    Roster[(roster de canciones)] --> Discover[agente descubrimiento YouTube<br/>algoritmo + canciones que pegaron]
    Discover --> Segments[mejores segmentos 1min<br/>por canción / por parte]
    Segments --> Roster
    Roster --> VoiceConv[conversión de voces<br/>roster personajes → audio]
    VoiceConv --> Render[render batch 5–30 videos<br/>cargo run --release -- --record]
    Render --> Mix[mix audio + video por video]
    Mix --> Publish[publicar en plataformas]
    Publish --> Feedback[métricas / feedback]
    Feedback --> Roster
```

## 6. Módulos del juego

```mermaid
flowchart TD
    main[main.rs<br/>parse_command + run_sim<br/>composición visible]
    main --> world[game/world.rs<br/>setup, set_gravity<br/>spawn_level, paredes, suelo]
    main --> marbles[game/marbles.rs<br/>spawn_marbles<br/>MarbleName, MarbleLabel]
    main --> camera[game/camera.rs<br/>spawn_camera_and_lights<br/>follow_lowest, update_labels]
    main --> tracker[production/voice_tracker.rs<br/>track_race_leader<br/>save_on_exit]
    main --> content[content/process_modules.rs<br/>raw JSON → final JSON]
    main -->|Sim, Preprocess| engine[(rapier-bevy<br/>game_app, world_objects,<br/>RecordPlugin, OffscreenTarget)]
    world --> level[game/level.rs<br/>load_module → ModuleData]
    world --> marbles
    camera --> marbles
    tracker --> marbles
    level -->|lee| ModulesJSON[(assets/modules/*.json)]
```

`main.rs` referencia directamente cada system por su ruta completa (`game::camera::spawn_camera_and_lights`, `production::voice_tracker::track_race_leader`, etc.). `game/mod.rs` y `production/mod.rs` son solo organizadores de namespace — sin Plugin ni función de composición.
