#[derive(serde::Deserialize, Clone)]
#[serde(tag = "kind")]
pub enum WorldObject {
    Box {
        x: f32, y: f32, hx: f32, hy: f32, rot: f32,
        #[serde(default)]
        angvel: [f32; 3],
        #[serde(default)]
        border_radius: Option<f32>,
    },
    Sphere { x: f32, y: f32, radius: f32 },
    Mesh {
        x: f32, y: f32, rot: f32, model_name: String,
        #[serde(default)]
        angvel: [f32; 3],
    },
}

#[derive(serde::Deserialize)]
pub struct ModuleData {
    pub height: f32,
    pub objects: Vec<WorldObject>,
}

pub fn load_module(name: &str) -> ModuleData {
    let path = format!("assets/modules/{name}.json");
    let json = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("No se encontró {path}"));
    serde_json::from_str(&json).unwrap_or_else(|_| panic!("{path} tiene formato inválido"))
}
