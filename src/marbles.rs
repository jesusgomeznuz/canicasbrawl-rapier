use bevy::prelude::*;
use rapier_bevy::{
    AssetsLoading, BodyType, ColliderShape, LockedAxes, ObjectDef, SimMode,
    VisualAppearance, VisualDef, spawn_object,
};

#[derive(Component)]
pub struct Marble;

#[derive(Component)]
pub struct MarbleLabel(pub Entity);

pub struct MarbleConfig {
    pub nickname: &'static str,
    pub color: Color,
    pub image: &'static str,
}

pub fn spawn_marbles(
    commands: &mut Commands,
    mode: &SimMode,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    spawn_cx: f32,
    spawn_cy: f32,
    assets_loading: &mut Option<ResMut<AssetsLoading>>,
) {
    let roster = marble_roster();
    let grid = spawn_grid(spawn_cx, spawn_cy);
    for (cfg, pos) in roster.iter().zip(grid.iter()) {
        let entity = spawn_marble_body(commands, mode, asset_server, meshes, materials, cfg, *pos);
        spawn_marble_label(commands, entity, cfg.nickname);
        attach_marble_face(commands, entity, cfg.image, asset_server, meshes, materials, assets_loading);
    }
}

fn marble_roster() -> Vec<MarbleConfig> {
    let white = Color::srgb(0.92, 0.92, 0.90);
    vec![
        MarbleConfig { nickname: "Goku",     color: white, image: "characters/Goku.png" },
        MarbleConfig { nickname: "Bart",      color: white, image: "characters/bart.png" },
        MarbleConfig { nickname: "Naruto",    color: white, image: "characters/naruto.png" },
        MarbleConfig { nickname: "Finn",      color: white, image: "characters/finn.png" },
        MarbleConfig { nickname: "Rick",      color: white, image: "characters/rick.png" },
        MarbleConfig { nickname: "Shrek",     color: white, image: "characters/shrek.png" },
        MarbleConfig { nickname: "Vegeta",    color: white, image: "characters/vegeta.png" },
        MarbleConfig { nickname: "Patricio",  color: white, image: "characters/patricio.png" },
        MarbleConfig { nickname: "Gumball",   color: white, image: "characters/gumball.png" },
    ]
}

fn spawn_grid(cx: f32, cy: f32) -> [(f32, f32); 9] {
    let dx = 0.25;
    let dy = 0.30;
    [
        (cx - dx, cy + dy), (cx, cy + dy), (cx + dx, cy + dy),
        (cx - dx, cy),      (cx, cy),      (cx + dx, cy),
        (cx - dx, cy - dy), (cx, cy - dy), (cx + dx, cy - dy),
    ]
}

fn spawn_marble_body(
    commands: &mut Commands,
    mode: &SimMode,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    cfg: &MarbleConfig,
    (x, y): (f32, f32),
) -> Entity {
    let radius = 0.09;
    let half_depth = crate::UNIT / 4.0;
    let entity = spawn_object(
        commands,
        ObjectDef {
            shape: ColliderShape::Cylinder { half_height: half_depth, radius, axis: Vec3::Z },
            position: Vec3::new(x, y, 0.0),
            body: BodyType::Dynamic,
            restitution: Some(0.6),
            friction: Some(0.4),
            linear_damping: Some(0.15),
            angular_damping: Some(0.4),
            ccd: true,
            locked_axes: Some(
                LockedAxes::TRANSLATION_LOCKED_Z
                    | LockedAxes::ROTATION_LOCKED_X
                    | LockedAxes::ROTATION_LOCKED_Y,
            ),
            visual: Some(VisualDef {
                appearance: VisualAppearance::Color(cfg.color),
                ..VisualDef::white_matte()
            }),
            ..Default::default()
        },
        mode,
        asset_server,
        meshes,
        materials,
    );
    commands.entity(entity).insert(Marble);
    entity
}

fn spawn_marble_label(commands: &mut Commands, marble_entity: Entity, nickname: &'static str) {
    commands.spawn((
        Text2d::new(nickname),
        TextFont { font_size: 20.0, ..default() },
        TextColor(Color::BLACK),
        TextLayout::new_with_justify(JustifyText::Center),
        MarbleLabel(marble_entity),
    ));
}

fn attach_marble_face(
    commands: &mut Commands,
    entity: Entity,
    image_path: &'static str,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    assets_loading: &mut Option<ResMut<AssetsLoading>>,
) {
    let radius = 0.09;
    let half_depth = crate::UNIT / 4.0;
    let quad_size = radius * 2.0;

    let bg_handle: Handle<Image> = asset_server.load("characters/circle_white.png");
    let img_handle: Handle<Image> = asset_server.load(image_path);

    if let Some(al) = assets_loading.as_deref_mut() {
        al.0.push(bg_handle.clone().untyped());
        al.0.push(img_handle.clone().untyped());
    }

    commands.entity(entity).with_children(|parent| {
        parent.spawn((
            Mesh3d(meshes.add(Rectangle::new(quad_size, quad_size))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(bg_handle),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.0, half_depth + 0.001),
        ));
        parent.spawn((
            Mesh3d(meshes.add(Rectangle::new(quad_size, quad_size))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(img_handle),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.0, half_depth + 0.002),
        ));
    });
}
