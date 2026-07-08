use bevy::prelude::*;

pub struct MarbleConfig {
    pub nickname: String,
    pub image: Option<String>,
}

#[derive(Resource)]
pub struct Roster(pub Vec<MarbleConfig>);

pub fn build_roster(characters: Option<Vec<String>>) -> Result<Vec<MarbleConfig>, String> {
    let Some(names) = characters else {
        return Ok(default_roster());
    };
    if names.len() > 9 {
        return Err(format!(
            "Pediste {} personajes pero el grid de salida es 3×3 (máximo 9).",
            names.len(),
        ));
    }
    names.iter().map(|name| character_config(name)).collect()
}

pub fn slots_roster(n: usize) -> Result<Vec<MarbleConfig>, String> {
    if n == 0 || n > 9 {
        return Err(format!("--slots {n}: el grid de salida es 3×3 (entre 1 y 9)."));
    }
    Ok((0..n)
        .map(|i| MarbleConfig { nickname: format!("slot_{i}"), image: None })
        .collect())
}

fn character_config(name: &str) -> Result<MarbleConfig, String> {
    let image = format!("characters/{}.png", name.to_lowercase());
    if !std::path::Path::new("assets").join(&image).exists() {
        return Err(format!(
            "Personaje '{name}' sin imagen: falta assets/{image}. \
             Revisa el nombre (CamelCase canónico) o agrega el asset.",
        ));
    }
    Ok(MarbleConfig {
        nickname: name.to_string(),
        image: Some(image),
    })
}

fn default_roster() -> Vec<MarbleConfig> {
    [
        "Marceline",
        "Perla",
        "Steven",
        "Wendy",
        "Naruto",
        "Ben10",
        "Patricio",
        "Finn",
        "Bart",
    ]
    .iter()
    .map(|name| character_config(name).expect("default roster asset missing"))
    .collect()
}
