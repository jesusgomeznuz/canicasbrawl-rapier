use bevy::prelude::*;

/// Paleta de color del juego — controla cielo, nubes, obstáculos y ClearColor.
/// Se inserta como recurso según el flag `--neon` / `--rosa` / (default azul).
#[derive(Resource, Clone)]
pub struct ColorPalette {
    pub sky_stops: [(f32, [u8; 3]); 5],
    pub cloud_near: Color,
    pub cloud_mid: Color,
    pub cloud_far: Color,
    /// Color del cielo que se mezcla con el blanco de los obstáculos.
    pub obstacle_tint: Color,
    /// Qué tanto domina el tinte sobre el blanco base (0.0 = puro blanco, 1.0 = color puro).
    pub tint_strength: f32,
}

impl ColorPalette {
    pub fn azul() -> Self {
        Self {
            sky_stops: [
                (0.00, [0x07, 0x3B, 0x4C]),
                (0.40, [0x0E, 0x7C, 0x92]),
                (0.66, [0x13, 0xA0, 0xAE]),
                (0.84, [0x1E, 0xB6, 0xBE]),
                (1.00, [0x1E, 0xB6, 0xBE]),
            ],
            cloud_near: Color::srgb(0.92, 0.96, 1.00),
            cloud_mid: Color::srgb(0.60, 0.82, 0.90),
            cloud_far: Color::srgb(0.22, 0.60, 0.72),
            obstacle_tint: Color::srgb(0.027, 0.231, 0.298), // #073B4C top del cielo (oscuro)
            tint_strength: 0.5,
        }
    }

    // Crepúsculo neón: cielo índigo-morado → naranja-coral.
    // Nubes en violeta-púrpura (#8E7AC2 → #3A2C66).
    pub fn neon() -> Self {
        Self {
            sky_stops: [
                (0.00, [0x17, 0x15, 0x40]),
                (0.40, [0x46, 0x26, 0x6E]),
                (0.66, [0xC0, 0x41, 0x7E]),
                (0.84, [0xFF, 0x7E, 0x6B]),
                (1.00, [0xFF, 0x7E, 0x6B]),
            ],
            cloud_near: Color::srgb(0.56, 0.48, 0.76),
            cloud_mid: Color::srgb(0.39, 0.32, 0.58),
            cloud_far: Color::srgb(0.23, 0.17, 0.40),
            obstacle_tint: Color::srgb(0.27, 0.15, 0.43), // #46266E morado mid-sky
            tint_strength: 0.5,
        }
    }

    // Rosa: top muy oscuro fucsia → mid vibrante → horizonte durazno claro.
    // El salto de tono oscuro-frío a rosa-cálido da la profundidad que el original perdía.
    pub fn rosa() -> Self {
        Self {
            sky_stops: [
                (0.00, [0x5C, 0x08, 0x44]),
                (0.35, [0xB8, 0x20, 0x78]),
                (0.62, [0xEE, 0x55, 0xA4]),
                (0.82, [0xFF, 0x99, 0xCC]),
                (1.00, [0xFF, 0xBB, 0xDD]),
            ],
            cloud_near: Color::srgb(1.00, 1.00, 1.00),
            cloud_mid: Color::srgb(0.98, 0.82, 0.92),
            cloud_far: Color::srgb(0.93, 0.65, 0.83),
            obstacle_tint: Color::srgb(0.361, 0.031, 0.267), // #5C0844 top del cielo (oscuro)
            tint_strength: 0.5,
        }
    }

    pub fn clear_color(&self) -> Color {
        let [r, g, b] = self.sky_stops[0].1;
        Color::srgb_u8(r, g, b)
    }

    /// Blanco base de los obstáculos (`white_matte`) mezclado con el tinte del cielo.
    /// `tint_strength = 0` → puro blanco. `tint_strength = 0.10` → gota de color.
    pub fn obstacle_color(&self) -> Color {
        let s = self.tint_strength;
        let w = 1.0 - s;
        let Srgba {
            red, green, blue, ..
        } = self.obstacle_tint.to_srgba();
        // Base: (0.92, 0.92, 0.90) de white_matte
        Color::srgb(
            0.92f32.mul_add(w, red * s),
            0.92f32.mul_add(w, green * s),
            0.90f32.mul_add(w, blue * s),
        )
    }
}
