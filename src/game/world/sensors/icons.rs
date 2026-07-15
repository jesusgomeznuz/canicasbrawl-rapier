use bevy::prelude::*;

#[derive(Component)]
pub struct SpinningIcon {
    pub axis: Vec3,
    pub speed: f32,
}

pub fn spin_icons(time: Res<Time>, mut icons: Query<(&SpinningIcon, &mut Transform)>) {
    for (icon, mut transform) in &mut icons {
        transform.rotate_axis(Dir3::new(icon.axis).unwrap_or(Dir3::Y), icon.speed * time.delta_secs());
    }
}
