#[derive(serde::Deserialize)]
pub struct PlatformData {
    pub x: f32, pub y: f32, pub hx: f32, pub hy: f32, pub rot: f32, pub angvel_z: f32,
}

#[derive(serde::Deserialize)]
pub struct SpawnData {
    pub cx: f32, pub cy: f32,
}

#[derive(serde::Deserialize)]
pub struct LevelData {
    pub platforms: Vec<PlatformData>,
    pub floor_y: Option<f32>,
    pub spawn: Option<SpawnData>,
}

pub fn load_level() -> LevelData {
    let json = std::fs::read_to_string("assets/levels/level_01.json")
        .expect("No se encontró assets/levels/level_01.json — corre: cargo run -- --process-figma");
    serde_json::from_str(&json).expect("level_01.json tiene formato inválido")
}
