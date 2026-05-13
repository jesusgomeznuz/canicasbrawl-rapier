#[derive(serde::Deserialize)]
struct RawRect {
    name: String,
    x: f32, y: f32, w: f32, h: f32,
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
        x: f32, y: f32, hx: f32, hy: f32, rot: f32,
        #[serde(skip_serializing_if = "is_zero_vec3")]
        angvel: [f32; 3],
        #[serde(skip_serializing_if = "Option::is_none")]
        border_radius: Option<f32>,
    },
    Sphere { x: f32, y: f32, radius: f32 },
    Mesh   {
        x: f32, y: f32, rot: f32, model_name: String,
        #[serde(skip_serializing_if = "is_zero_vec3")]
        angvel: [f32; 3],
    },
    Image  { x: f32, y: f32, w: f32, h: f32, rot: f32, texture: String },
}

#[derive(serde::Serialize)]
struct ModuleData {
    objects: Vec<WorldObject>,
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

        println!("✓ {} → {} objetos", out_path.display(), module.objects.len());
    }
}

fn transform(raw: RawModule) -> ModuleData {
    let objects = raw.rects.iter()
        .map(|r| world_object_from_raw(r, raw.frame_w, raw.frame_h))
        .collect();
    ModuleData { objects }
}

fn world_object_from_raw(r: &RawRect, frame_w: f32, frame_h: f32) -> WorldObject {
    match base_name(&r.name) {
        "sphere" => sphere_from_raw(r, frame_w, frame_h),
        "torus"  => torus_from_raw(r, frame_w, frame_h),
        "image"  => image_from_raw(r, frame_w, frame_h),
        _        => box_from_raw(r, frame_w, frame_h),
    }
}

fn base_name(name: &str) -> &str {
    name.split('|').next().unwrap_or(name).trim()
}

fn box_from_raw(r: &RawRect, frame_w: f32, frame_h: f32) -> WorldObject {
    let rot = rot_from_name(&r.name).to_radians();
    let half_w = r.w / 2.0;
    let half_h = r.h / 2.0;
    let (sin, cos) = rot.sin_cos();
    let cx_figma = r.x + half_w * cos + half_h * sin;
    let cy_figma = r.y - half_w * sin + half_h * cos;

    WorldObject::Box {
        x: round4((cx_figma - frame_w / 2.0) * 0.01),
        y: round4((frame_h - cy_figma)       * 0.01),
        hx: round4(half_w * 0.01),
        hy: round4(half_h * 0.01),
        rot: round4(rot),
        angvel: angvel_from_name(&r.name),
        border_radius: br_from_name(&r.name),
    }
}

fn is_zero_vec3(v: &[f32; 3]) -> bool {
    v[0] == 0.0 && v[1] == 0.0 && v[2] == 0.0
}

fn torus_from_raw(r: &RawRect, frame_w: f32, frame_h: f32) -> WorldObject {
    if (r.w - r.h).abs() > 0.01 {
        panic!("Torus '{}' tiene w={} h={}, su bbox debe ser cuadrada", r.name, r.w, r.h);
    }
    let rot = rot_from_name(&r.name).to_radians();
    let outer_radius = r.w / 2.0 * 0.01;
    let minor_r = required_tag(&r.name, "|t");
    let major_r = outer_radius - minor_r;
    if major_r <= 0.0 {
        panic!("Torus '{}': tube radius ({}) >= radio exterior ({})", r.name, minor_r, outer_radius);
    }
    let model_name = ensure_torus_assets(major_r, minor_r);
    let cx_figma = r.x + r.w / 2.0;
    let cy_figma = r.y + r.h / 2.0;
    WorldObject::Mesh {
        x: round4((cx_figma - frame_w / 2.0) * 0.01),
        y: round4((frame_h - cy_figma)       * 0.01),
        rot: round4(rot),
        model_name,
        angvel: angvel_from_name(&r.name),
    }
}

fn required_tag(name: &str, tag: &str) -> f32 {
    let Some(start) = name.find(tag) else {
        panic!("'{}' requiere tag {} (ej. {}{}0.05)", name, tag, tag, "");
    };
    let rest = &name[start + tag.len()..];
    let end = rest.find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-')).unwrap_or(rest.len());
    rest[..end].parse::<f32>().unwrap_or_else(|_| panic!("'{}': tag {} sin número", name, tag))
}

fn ensure_torus_assets(major_r: f32, minor_r: f32) -> String {
    let model_name = format!("torus_R{}_r{}",
        (major_r * 1000.0).round() as i32,
        (minor_r * 1000.0).round() as i32);
    let obj_path = format!("assets/{}.obj", model_name);
    let compound_path = format!("assets/{}.compound", model_name);
    if !std::path::Path::new(&obj_path).exists() {
        write_torus_obj(&obj_path, major_r, minor_r);
        println!("  ↳ generado {}", obj_path);
    }
    if !std::path::Path::new(&compound_path).exists() {
        rapier_bevy::preprocess_obj(&obj_path, &compound_path, None,
            rapier_bevy::VHACDParameters { resolution: 64, ..Default::default() });
    }
    model_name
}

fn write_torus_obj(path: &str, major_r: f32, minor_r: f32) {
    let n_major = 48usize;
    let n_minor = 24usize;
    let mut s = String::new();
    s.push_str("# torus generado por process_modules\n");
    for i in 0..n_major {
        let phi = std::f32::consts::TAU * (i as f32) / (n_major as f32);
        let (sin_phi, cos_phi) = phi.sin_cos();
        for j in 0..n_minor {
            let theta = std::f32::consts::TAU * (j as f32) / (n_minor as f32);
            let (sin_theta, cos_theta) = theta.sin_cos();
            let x = (major_r + minor_r * cos_theta) * cos_phi;
            let y = (major_r + minor_r * cos_theta) * sin_phi;
            let z = minor_r * sin_theta;
            s.push_str(&format!("v {:.6} {:.6} {:.6}\n", x, y, z));
        }
    }
    for i in 0..n_major {
        let i_next = (i + 1) % n_major;
        for j in 0..n_minor {
            let j_next = (j + 1) % n_minor;
            let a = i      * n_minor + j      + 1;
            let b = i_next * n_minor + j      + 1;
            let c = i_next * n_minor + j_next + 1;
            let d = i      * n_minor + j_next + 1;
            s.push_str(&format!("f {} {} {}\n", a, b, c));
            s.push_str(&format!("f {} {} {}\n", a, c, d));
        }
    }
    std::fs::write(path, s).unwrap_or_else(|_| panic!("No se pudo escribir {}", path));
}

fn image_from_raw(r: &RawRect, frame_w: f32, frame_h: f32) -> WorldObject {
    let rot = rot_from_name(&r.name).to_radians();
    let half_w = r.w / 2.0;
    let half_h = r.h / 2.0;
    let (sin, cos) = rot.sin_cos();
    let cx_figma = r.x + half_w * cos + half_h * sin;
    let cy_figma = r.y - half_w * sin + half_h * cos;
    let texture = r.name.split('|').nth(1).map(|s| s.trim()).unwrap_or("");
    if texture.is_empty() {
        panic!("Image '{}' requiere filename tras 'image|' (ej. image|canicas_logo)", r.name);
    }
    WorldObject::Image {
        x: round4((cx_figma - frame_w / 2.0) * 0.01),
        y: round4((frame_h - cy_figma)       * 0.01),
        w: round4(r.w * 0.01),
        h: round4(r.h * 0.01),
        rot: round4(rot),
        texture: format!("img/{}.png", texture),
    }
}

fn sphere_from_raw(r: &RawRect, frame_w: f32, frame_h: f32) -> WorldObject {
    if (r.w - r.h).abs() > 0.01 {
        panic!("Sphere '{}' tiene w={} h={}, debe ser circular (w == h)", r.name, r.w, r.h);
    }
    let cx_figma = r.x + r.w / 2.0;
    let cy_figma = r.y + r.h / 2.0;
    WorldObject::Sphere {
        x: round4((cx_figma - frame_w / 2.0) * 0.01),
        y: round4((frame_h - cy_figma)       * 0.01),
        radius: round4(r.w / 2.0 * 0.01),
    }
}

fn rot_from_name(name: &str) -> f32 {
    parse_tagged_number(name, "|r")
}

fn angvel_from_name(name: &str) -> [f32; 3] {
    [
        parse_tagged_number(name, "|wx"),
        parse_tagged_number(name, "|wy"),
        parse_tagged_number(name, "|wz"),
    ]
}

fn br_from_name(name: &str) -> Option<f32> {
    let start = name.find("|br")?;
    let rest = &name[start + 3..];
    let end = rest.find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-')).unwrap_or(rest.len());
    rest[..end].parse::<f32>().ok()
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
