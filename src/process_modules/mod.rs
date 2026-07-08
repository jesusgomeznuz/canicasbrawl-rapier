mod shapes;
mod torus_assets;

use shapes::world_object_from_raw;

pub fn run() {
    let raw_dir = std::path::Path::new("assets/modules/raw");
    let out_dir = std::path::Path::new("assets/modules");

    let entries = std::fs::read_dir(raw_dir)
        .unwrap_or_else(|_| panic!("No se encontró {}", raw_dir.display()));

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let json = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("No se pudo leer {}", path.display()));
        let raw: RawModule = serde_json::from_str(&json)
            .unwrap_or_else(|_| panic!("{} tiene formato inválido", path.display()));

        let (slug, frame_tags) = split_frame_name(&raw.frame);
        let name = to_snake_case(slug);
        let module = transform(raw, &frame_tags);

        let out_path = out_dir.join(format!("{}.json", name));
        let output = serde_json::to_string_pretty(&module).unwrap();
        std::fs::write(&out_path, output)
            .unwrap_or_else(|_| panic!("No se pudo escribir {}", out_path.display()));
        std::fs::remove_file(&path)
            .unwrap_or_else(|_| panic!("No se pudo borrar {}", path.display()));

        println!(
            "✓ {} → {} objetos",
            out_path.display(),
            module.objects.len()
        );
    }
}

fn transform(raw: RawModule, frame_tags: &str) -> ModuleData {
    let objects = raw
        .rects
        .iter()
        .map(|r| {
            let merged = RawRect {
                name: format!("{}{}", r.name, frame_tags),
                x: r.x,
                y: r.y,
                w: r.w,
                h: r.h,
            };
            world_object_from_raw(&merged, raw.frame_w, raw.frame_h)
        })
        .collect();
    ModuleData { objects }
}

#[derive(serde::Deserialize)]
struct RawRect {
    name: String,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

#[derive(serde::Deserialize)]
struct RawModule {
    frame: String,
    frame_w: f32,
    frame_h: f32,
    rects: Vec<RawRect>,
}

#[derive(serde::Serialize)]
#[serde(tag = "kind")]
enum WorldObject {
    Box {
        x: f32,
        y: f32,
        hx: f32,
        hy: f32,
        rot: f32,
        #[serde(skip_serializing_if = "is_zero_vec3")]
        angvel: [f32; 3],
        #[serde(skip_serializing_if = "Option::is_none")]
        border_radius: Option<f32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        friction: Option<f32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        restitution: Option<f32>,
        #[serde(skip_serializing_if = "std::ops::Not::not")]
        bouncy: bool,
    },
    Sphere {
        x: f32,
        y: f32,
        radius: f32,
        #[serde(skip_serializing_if = "Option::is_none")]
        friction: Option<f32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        restitution: Option<f32>,
        #[serde(skip_serializing_if = "std::ops::Not::not")]
        bouncy: bool,
    },
    Mesh {
        x: f32,
        y: f32,
        rot: f32,
        model_name: String,
        #[serde(skip_serializing_if = "is_zero_vec3")]
        angvel: [f32; 3],
        #[serde(skip_serializing_if = "Option::is_none")]
        friction: Option<f32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        restitution: Option<f32>,
    },
    Image {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        rot: f32,
        texture: String,
    },
    Effect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        rot: f32,
        variant: String,
    },
    EffectSlot {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        rot: f32,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        options: Vec<String>,
    },
}

#[derive(serde::Serialize)]
struct ModuleData {
    objects: Vec<WorldObject>,
}

fn to_snake_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    for (i, c) in s.chars().enumerate() {
        if c.is_ascii_uppercase() && i > 0 {
            out.push('_');
        }
        out.extend(c.to_lowercase());
    }
    out
}

fn split_frame_name(frame: &str) -> (&str, String) {
    match frame.find('|') {
        Some(i) => (&frame[..i], frame[i..].to_string()),
        None => (frame, String::new()),
    }
}

fn is_zero_vec3(v: &[f32; 3]) -> bool {
    v[0] == 0.0 && v[1] == 0.0 && v[2] == 0.0
}
