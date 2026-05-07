#[derive(serde::Deserialize, Clone, Copy)]
pub struct PlatformData {
    pub x: f32, pub y: f32, pub hx: f32, pub hy: f32, pub rot: f32, pub angvel_z: f32,
    #[serde(default)]
    pub border_radius: Option<f32>,
}

#[derive(serde::Deserialize)]
pub struct ModuleData {
    pub height: f32,
    pub platforms: Vec<PlatformData>,
}

pub fn load_module(name: &str) -> ModuleData {
    let path = format!("assets/modules/{name}.json");
    let json = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("No se encontró {path}"));
    serde_json::from_str(&json).unwrap_or_else(|_| panic!("{path} tiene formato inválido"))
}
