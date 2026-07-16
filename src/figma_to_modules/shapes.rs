use super::{RawRect, WorldObject};
use super::torus_assets::ensure_torus_assets;

pub fn world_object_from_raw(r: &RawRect, frame_w: f32, frame_h: f32) -> WorldObject {
    match base_name(&r.name).as_str() {
        "sphere" => sphere_from_raw(r, frame_w, frame_h),
        "torus" => torus_from_raw(r, frame_w, frame_h),
        "image" => image_from_raw(r, frame_w, frame_h),
        "effect" => effect_from_raw(r, frame_w, frame_h),
        "slot" => slot_from_raw(r, frame_w, frame_h),
        _ => box_from_raw(r, frame_w, frame_h),
    }
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
        y: round4((frame_h - cy_figma) * 0.01),
        hx: round4(half_w * 0.01),
        hy: round4(half_h * 0.01),
        rot: round4(rot),
        angvel: angvel_from_name(&r.name),
        border_radius: optional_tag(&r.name, "|br"),
        friction: optional_tag(&r.name, "|fr"),
        restitution: optional_tag(&r.name, "|re"),
        bouncy: r.name.contains("|bouncy"),
    }
}

fn sphere_from_raw(r: &RawRect, frame_w: f32, frame_h: f32) -> WorldObject {
    if (r.w - r.h).abs() > 0.01 {
        panic!(
            "Sphere '{}' tiene w={} h={}, debe ser circular (w == h)",
            r.name, r.w, r.h
        );
    }
    let cx_figma = r.x + r.w / 2.0;
    let cy_figma = r.y + r.h / 2.0;
    WorldObject::Sphere {
        x: round4((cx_figma - frame_w / 2.0) * 0.01),
        y: round4((frame_h - cy_figma) * 0.01),
        radius: round4(r.w / 2.0 * 0.01),
        friction: optional_tag(&r.name, "|fr"),
        restitution: optional_tag(&r.name, "|re"),
        bouncy: r.name.contains("|bouncy"),
    }
}

fn torus_from_raw(r: &RawRect, frame_w: f32, frame_h: f32) -> WorldObject {
    if (r.w - r.h).abs() > 0.01 {
        panic!(
            "Torus '{}' tiene w={} h={}, su bbox debe ser cuadrada",
            r.name, r.w, r.h
        );
    }
    let rot = rot_from_name(&r.name).to_radians();
    let outer_radius = r.w / 2.0 * 0.01;
    let minor_r = required_tag(&r.name, "|t");
    let major_r = outer_radius - minor_r;
    if major_r <= 0.0 {
        panic!(
            "Torus '{}': tube radius ({}) >= radio exterior ({})",
            r.name, minor_r, outer_radius
        );
    }
    let model_name = ensure_torus_assets(major_r, minor_r);
    let cx_figma = r.x + r.w / 2.0;
    let cy_figma = r.y + r.h / 2.0;
    WorldObject::Mesh {
        x: round4((cx_figma - frame_w / 2.0) * 0.01),
        y: round4((frame_h - cy_figma) * 0.01),
        rot: round4(rot),
        model_name,
        angvel: angvel_from_name(&r.name),
        friction: optional_tag(&r.name, "|fr"),
        restitution: optional_tag(&r.name, "|re"),
    }
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
        panic!(
            "Image '{}' requiere filename tras 'image|' (ej. image|canicas_logo)",
            r.name
        );
    }
    WorldObject::Image {
        x: round4((cx_figma - frame_w / 2.0) * 0.01),
        y: round4((frame_h - cy_figma) * 0.01),
        w: round4(r.w * 0.01),
        h: round4(r.h * 0.01),
        rot: round4(rot),
        texture: format!("img/{}.png", texture),
    }
}

fn effect_from_raw(r: &RawRect, frame_w: f32, frame_h: f32) -> WorldObject {
    let rot = rot_from_name(&r.name).to_radians();
    let half_w = r.w / 2.0;
    let half_h = r.h / 2.0;
    let (sin, cos) = rot.sin_cos();
    let cx_figma = r.x + half_w * cos + half_h * sin;
    let cy_figma = r.y - half_w * sin + half_h * cos;
    let variant = r.name.split('|').nth(1).map(|s| s.trim()).unwrap_or("");
    if variant.is_empty() {
        panic!(
            "Effect '{}' requiere variante tras 'effect|' (ej. effect|freeze)",
            r.name
        );
    }
    WorldObject::Effect {
        x: round4((cx_figma - frame_w / 2.0) * 0.01),
        y: round4((frame_h - cy_figma) * 0.01),
        w: round4(r.w * 0.01),
        h: round4(r.h * 0.01),
        rot: round4(rot),
        variant: variant.to_string(),
    }
}

fn slot_from_raw(r: &RawRect, frame_w: f32, frame_h: f32) -> WorldObject {
    let rot = rot_from_name(&r.name).to_radians();
    let half_w = r.w / 2.0;
    let half_h = r.h / 2.0;
    let (sin, cos) = rot.sin_cos();
    let cx_figma = r.x + half_w * cos + half_h * sin;
    let cy_figma = r.y - half_w * sin + half_h * cos;
    let options = parse_slot_options(&r.name);
    WorldObject::EffectSlot {
        x: round4((cx_figma - frame_w / 2.0) * 0.01),
        y: round4((frame_h - cy_figma) * 0.01),
        w: round4(r.w * 0.01),
        h: round4(r.h * 0.01),
        rot: round4(rot),
        options,
    }
}

fn base_name(name: &str) -> String {
    name.split('|').next().unwrap_or(name).trim().to_lowercase()
}

fn parse_slot_options(name: &str) -> Vec<String> {
    let sensor_variants = ["freeze", "shrink", "swap"];
    let Some(first_tag) = name.split('|').nth(1) else {
        return vec![];
    };
    let candidates: Vec<String> = first_tag
        .split(',')
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect();
    if candidates
        .iter()
        .all(|c| sensor_variants.contains(&c.as_str()))
    {
        candidates
    } else {
        vec![]
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

fn optional_tag(name: &str, tag: &str) -> Option<f32> {
    let start = name.find(tag)?;
    let rest = &name[start + tag.len()..];
    let end = rest
        .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
        .unwrap_or(rest.len());
    rest[..end].parse::<f32>().ok()
}

fn required_tag(name: &str, tag: &str) -> f32 {
    let Some(start) = name.find(tag) else {
        panic!("'{}' requiere tag {} (ej. {}{}0.05)", name, tag, tag, "");
    };
    let rest = &name[start + tag.len()..];
    let end = rest
        .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
        .unwrap_or(rest.len());
    rest[..end]
        .parse::<f32>()
        .unwrap_or_else(|_| panic!("'{}': tag {} sin número", name, tag))
}

fn parse_tagged_number(name: &str, tag: &str) -> f32 {
    let Some(start) = name.find(tag) else {
        return 0.0;
    };
    let rest = &name[start + tag.len()..];
    let end = rest
        .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
        .unwrap_or(rest.len());
    rest[..end].parse::<f32>().unwrap_or(0.0)
}

fn round4(v: f32) -> f32 {
    (v * 10_000.0).round() / 10_000.0
}
