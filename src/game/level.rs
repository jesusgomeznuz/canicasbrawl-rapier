#[derive(serde::Deserialize, Clone)]
#[serde(tag = "kind")]
pub enum WorldObject {
    Box {
        x: f32, y: f32, hx: f32, hy: f32, rot: f32,
        #[serde(default)]
        angvel: [f32; 3],
        #[serde(default)]
        border_radius: Option<f32>,
        #[serde(default)]
        friction: Option<f32>,
        #[serde(default)]
        restitution: Option<f32>,
        #[serde(default)]
        bouncy: bool,
    },
    Sphere {
        x: f32, y: f32, radius: f32,
        #[serde(default)]
        friction: Option<f32>,
        #[serde(default)]
        restitution: Option<f32>,
        #[serde(default)]
        bouncy: bool,
    },
    Mesh {
        x: f32, y: f32, rot: f32, model_name: String,
        #[serde(default)]
        angvel: [f32; 3],
        #[serde(default)]
        friction: Option<f32>,
        #[serde(default)]
        restitution: Option<f32>,
    },
    Image {
        x: f32, y: f32, w: f32, h: f32, rot: f32, texture: String,
    },
}

#[derive(serde::Deserialize)]
pub struct ModuleData {
    pub objects: Vec<WorldObject>,
}

pub fn load_module(name: &str) -> ModuleData {
    let path = format!("assets/modules/{name}.json");
    let json = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("No se encontró {path}"));
    serde_json::from_str(&json).unwrap_or_else(|_| panic!("{path} tiene formato inválido"))
}

impl WorldObject {
    pub fn y_bounds(&self) -> (f32, f32) {
        match self {
            WorldObject::Box { y, hx, hy, rot, .. } => {
                let extent = (hx * rot.sin()).abs() + (hy * rot.cos()).abs();
                (y - extent, y + extent)
            }
            WorldObject::Sphere { y, radius, .. } => (y - radius, y + radius),
            WorldObject::Mesh { y, model_name, .. } => {
                let outer_r = torus_outer_radius(model_name).unwrap_or(0.0);
                (y - outer_r, y + outer_r)
            }
            WorldObject::Image { y, w, h, rot, .. } => {
                let extent = (w * 0.5 * rot.sin()).abs() + (h * 0.5 * rot.cos()).abs();
                (y - extent, y + extent)
            }
        }
    }
}

fn torus_outer_radius(model_name: &str) -> Option<f32> {
    let mut parts = model_name.split('_');
    if parts.next()? != "torus" { return None; }
    let major_mm: i32 = parts.next()?.trim_start_matches('R').parse().ok()?;
    let minor_mm: i32 = parts.next()?.trim_start_matches('r').parse().ok()?;
    Some((major_mm + minor_mm) as f32 / 1000.0)
}
