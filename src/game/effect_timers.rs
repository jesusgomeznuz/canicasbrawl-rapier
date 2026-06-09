use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::sprite::ColorMaterial;

use super::effects::{Frozen, Shrunk};
use super::marbles::Marble;

const FREEZE_COLOR: Color = Color::srgba(0.35, 0.88, 1.0, 1.0);
const SHRINK_COLOR: Color = Color::srgba(0.35, 1.0, 0.45, 1.0);
const RING_BG: Color = Color::srgba(1.0, 1.0, 1.0, 0.22);
const ARC_RADIUS_PX: f32 = 11.0;
const BADGE_OFFSET: Vec3 = Vec3::new(0.13, 0.15, 0.0);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EffectKind {
    Freeze,
    Shrink,
}

#[derive(Component)]
pub struct EffectTimerBadge {
    pub marble: Entity,
    pub expires_at: f32,
    pub started_at: f32,
    pub kind: EffectKind,
    // Handle al mesh del arco — se muta in-place cada frame para forzar re-upload al GPU
    pub arc_mesh: Handle<Mesh>,
}

// Marcadores en el marble para evitar doble-spawn y para cleanup
#[derive(Component)]
pub struct FreezeTimerMarker;

#[derive(Component)]
pub struct ShrinkTimerMarker;

// Detecta marbles recién congelados (sin badge todavía) usando polling en lugar de
// Added<Frozen> — más robusto cuando el componente viene de FixedUpdate.
pub fn manage_freeze_badges(
    mut commands: Commands,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<ColorMaterial>>,
    needs_badge: Query<(Entity, &Frozen), (With<Marble>, Without<FreezeTimerMarker>)>,
    lost_freeze: Query<Entity, (With<Marble>, With<FreezeTimerMarker>, Without<Frozen>)>,
    badges: Query<(Entity, &EffectTimerBadge)>,
) {
    // Spawn badge para nuevos congelados
    for (marble_e, frozen) in &needs_badge {
        spawn_badge(&mut commands, &time, &mut meshes, &mut mats, marble_e, frozen.expires_at, EffectKind::Freeze);
        commands.entity(marble_e).insert(FreezeTimerMarker);
    }
    // Despawnear badge cuando ya no está congelado
    for marble_e in &lost_freeze {
        for (badge_e, badge) in &badges {
            if badge.marble == marble_e && badge.kind == EffectKind::Freeze {
                commands.entity(badge_e).despawn();
            }
        }
        commands.entity(marble_e).remove::<FreezeTimerMarker>();
    }
}

pub fn manage_shrink_badges(
    mut commands: Commands,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<ColorMaterial>>,
    needs_badge: Query<(Entity, &Shrunk), (With<Marble>, Without<ShrinkTimerMarker>)>,
    lost_shrink: Query<Entity, (With<Marble>, With<ShrinkTimerMarker>, Without<Shrunk>)>,
    badges: Query<(Entity, &EffectTimerBadge)>,
) {
    for (marble_e, shrunk) in &needs_badge {
        spawn_badge(&mut commands, &time, &mut meshes, &mut mats, marble_e, shrunk.expires_at, EffectKind::Shrink);
        commands.entity(marble_e).insert(ShrinkTimerMarker);
    }
    for marble_e in &lost_shrink {
        for (badge_e, badge) in &badges {
            if badge.marble == marble_e && badge.kind == EffectKind::Shrink {
                commands.entity(badge_e).despawn();
            }
        }
        commands.entity(marble_e).remove::<ShrinkTimerMarker>();
    }
}

fn spawn_badge(
    commands: &mut Commands,
    time: &Time,
    meshes: &mut Assets<Mesh>,
    mats: &mut Assets<ColorMaterial>,
    marble_e: Entity,
    expires_at: f32,
    kind: EffectKind,
) {
    let arc_color = if kind == EffectKind::Freeze { FREEZE_COLOR } else { SHRINK_COLOR };
    // arc_mesh se crea aquí y se guarda en el componente; los hijos comparten el handle
    let arc_mesh_handle = meshes.add(build_arc_mesh(1.0));

    commands
        .spawn((
            EffectTimerBadge {
                marble: marble_e,
                expires_at,
                started_at: time.elapsed_secs(),
                kind,
                arc_mesh: arc_mesh_handle.clone(),
            },
            Transform::default(),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::VISIBLE,
            ViewVisibility::default(),
        ))
        .with_children(|parent| {
            // Fondo — anillo siempre completo
            parent.spawn((
                Mesh2d(meshes.add(build_arc_mesh(1.0))),
                MeshMaterial2d(mats.add(ColorMaterial::from_color(RING_BG))),
                Transform::from_scale(Vec3::splat(ARC_RADIUS_PX)),
            ));
            // Arco — mismo handle que badge.arc_mesh; se muta in-place en update_badges
            parent.spawn((
                Mesh2d(arc_mesh_handle),
                MeshMaterial2d(mats.add(ColorMaterial::from_color(arc_color))),
                Transform::from_scale(Vec3::splat(ARC_RADIUS_PX)).with_translation(Vec3::Z * 0.5),
            ));
        });
}

pub fn update_badges(
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    marbles: Query<&GlobalTransform, With<Marble>>,
    mut badges: Query<(&EffectTimerBadge, &mut Transform)>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
) {
    let Ok((camera, cam_gt)) = camera_q.single() else { return };
    let Some(viewport) = camera.logical_viewport_size() else { return };

    for (badge, mut badge_t) in &mut badges {
        let remaining = (badge.expires_at - time.elapsed_secs()).max(0.0);
        let total = (badge.expires_at - badge.started_at).max(0.01);
        let fraction = (remaining / total).clamp(0.0, 1.0);

        // Mutación in-place del mesh — get_mut marca el asset como cambiado y fuerza
        // el re-upload al GPU sin commands deferred
        if let Some(mesh) = meshes.get_mut(&badge.arc_mesh) {
            rebuild_arc_in_place(mesh, fraction);
        }

        let Ok(marble_gt) = marbles.get(badge.marble) else { continue };
        let world_pos = marble_gt.translation() + BADGE_OFFSET;
        if let Ok(screen) = camera.world_to_viewport(cam_gt, world_pos) {
            badge_t.translation.x = screen.x - viewport.x / 2.0;
            badge_t.translation.y = viewport.y / 2.0 - screen.y;
            badge_t.translation.z = 20.0;
        }
    }
}

fn rebuild_arc_in_place(mesh: &mut Mesh, fraction: f32) {
    let inner_r = 0.52_f32;
    let outer_r = 1.00_f32;
    let n_seg = 32_u32;

    let segments = ((fraction * n_seg as f32).ceil() as u32).clamp(1, n_seg);
    let arc_angle = fraction * std::f32::consts::TAU;
    let step = arc_angle / segments as f32;
    let start = std::f32::consts::FRAC_PI_2;

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for i in 0..=segments {
        let angle = start - i as f32 * step;
        let (s, c) = angle.sin_cos();
        positions.push([inner_r * c, inner_r * s, 0.0]);
        positions.push([outer_r * c, outer_r * s, 0.0]);
        normals.push([0.0, 0.0, 1.0]);
        normals.push([0.0, 0.0, 1.0]);
        uvs.push([0.0, 0.0]);
        uvs.push([1.0, 1.0]);
        if i > 0 {
            let j = (i - 1) * 2;
            indices.extend_from_slice(&[j, j + 2, j + 1, j + 1, j + 2, j + 3]);
        }
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
}

fn build_arc_mesh(fraction: f32) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD);
    rebuild_arc_in_place(&mut mesh, fraction);
    mesh
}
