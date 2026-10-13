//! Particle presets — the accent layer over the drawn bursts and splashes. Each event's
//! look is tuned here in one place; the backdrop grid is the engine's, and gameplay only
//! ripples it.

use engine_core::prelude::*;

use crate::art::{ICE_WATER_COLOR, SAUCER_BURST_COLOR};

/// One radial burst of `count` particles in `color`, fading out.
fn radial_burst(color: Vec4, count: f32, theme: &ChaosTheme, texture: u32) -> ParticleConfig {
    ParticleConfig::burst((count * theme.particle_count_mult).round() as usize)
        .with_direction(Vec2::Y, std::f32::consts::PI)
        .with_color(color, Vec4::new(color.x, color.y, color.z, 0.0))
        .with_texture(texture)
}

/// Croutons flying off a busted Toasted, in its crust's colour.
pub(crate) fn crouton_burst(color: Vec4, theme: &ChaosTheme, texture: u32) -> ParticleConfig {
    radial_burst(color, 26.0, theme, texture)
        .with_lifetime(0.25, 0.6)
        .with_speed(100.0, 320.0)
        .with_scale(5.0, 0.5)
        .with_drag(2.2)
}

/// A cannon's lost life: its water spraying out.
pub(crate) fn player_hit_burst(theme: &ChaosTheme, texture: u32) -> ParticleConfig {
    radial_burst(ICE_WATER_COLOR, 70.0, theme, texture)
        .with_lifetime(0.4, 0.9)
        .with_speed(120.0, 480.0)
        .with_scale(8.0, 0.5)
        .with_drag(1.7)
}

/// Maxwell's saucer going down: a shower of frosting.
pub(crate) fn saucer_burst(theme: &ChaosTheme, texture: u32) -> ParticleConfig {
    radial_burst(SAUCER_BURST_COLOR, 46.0, theme, texture)
        .with_lifetime(0.3, 0.8)
        .with_speed(110.0, 400.0)
        .with_scale(7.0, 0.5)
        .with_drag(1.9)
}

/// A tiny flash where two shots cancel each other in mid-air.
pub(crate) fn shot_cancel_burst(theme: &ChaosTheme, texture: u32) -> ParticleConfig {
    radial_burst(ICE_WATER_COLOR, 14.0, theme, texture)
        .with_lifetime(0.12, 0.3)
        .with_speed(90.0, 240.0)
        .with_scale(4.0, 0.5)
        .with_drag(2.6)
}
