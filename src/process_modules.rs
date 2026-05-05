#[derive(serde::Deserialize)]
struct RawRect {
    name: String,
    x: f32, y: f32, w: f32, h: f32,
    #[serde(default)]
    inner_w: Option<f32>,
    #[serde(default)]
    inner_h: Option<f32>,
}

#[derive(serde::Deserialize)]
struct RawModule {
    frame: String,
    frame_w: f32,
    frame_h: f32,
    rects: Vec<RawRect>,
}

#[derive(serde::Serialize)]
struct PlatformData {
    x: f32, y: f32, hx: f32, hy: f32, rot: f32, angvel_z: f32,
}

#[derive(serde::Serialize)]
struct ModuleData {
    height: f32,
    platforms: Vec<PlatformData>,
}

pub fn run() {
    let raw_dir = std::path::Path::new("assets/modules/raw");
    let out_dir = std::path::Path::new("assets/modules");

    let entries = std::fs::read_dir(raw_dir)
        .unwrap_or_else(|_| panic!("No se encontró {}", raw_dir.display()));

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") { continue; }

        let json = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("No se pudo leer {}", path.display()));
        let raw: RawModule = serde_json::from_str(&json)
            .unwrap_or_else(|_| panic!("{} tiene formato inválido", path.display()));

        let name = raw.frame.to_lowercase();
        let module = transform(raw);

        let out_path = out_dir.join(format!("{}.json", name));
        let output = serde_json::to_string_pretty(&module).unwrap();
        std::fs::write(&out_path, output)
            .unwrap_or_else(|_| panic!("No se pudo escribir {}", out_path.display()));
        std::fs::remove_file(&path)
            .unwrap_or_else(|_| panic!("No se pudo borrar {}", path.display()));

        println!("✓ {} → {} plataformas", out_path.display(), module.platforms.len());
    }
}

fn transform(raw: RawModule) -> ModuleData {
    let platforms = raw.rects.iter().map(|r| platform_from_raw(r, raw.frame_w, raw.frame_h)).collect();
    ModuleData { height: round4(raw.frame_h * 0.01), platforms }
}

fn platform_from_raw(r: &RawRect, frame_w: f32, frame_h: f32) -> PlatformData {
    let cx_figma = r.x + r.w / 2.0;
    let cy_figma = r.y + r.h / 2.0;
    let collider_w = r.inner_w.unwrap_or(r.w);
    let collider_h = r.inner_h.unwrap_or(r.h);

    let game_x = (cx_figma - frame_w / 2.0) * 0.01;
    let game_y = (frame_h - cy_figma)        * 0.01;
    let hx     = collider_w / 2.0            * 0.01;
    let hy     = collider_h / 2.0            * 0.01;
    let rot    = rot_from_name(&r.name).to_radians();
    let angvel_z = angvel_from_name(&r.name);

    PlatformData {
        x: round4(game_x), y: round4(game_y),
        hx: round4(hx),    hy: round4(hy),
        rot: round4(rot),  angvel_z,
    }
}

fn rot_from_name(name: &str) -> f32 {
    parse_tagged_number(name, "|r")
}

fn angvel_from_name(name: &str) -> f32 {
    parse_tagged_number(name, "|w")
}

fn parse_tagged_number(name: &str, tag: &str) -> f32 {
    let Some(start) = name.find(tag) else { return 0.0; };
    let rest = &name[start + tag.len()..];
    let end = rest.find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-')).unwrap_or(rest.len());
    rest[..end].parse::<f32>().unwrap_or(0.0)
}

fn round4(v: f32) -> f32 {
    (v * 10_000.0).round() / 10_000.0
}
