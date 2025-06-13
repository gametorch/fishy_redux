use bevy::prelude::*;

/// Tag for drifting obscura sprites that float through the main menu background.
///
/// Compared to the previous `BackgroundFish` component this variant supports
/// movement from any screen edge by storing a full 2-D velocity instead of just
/// a horizontal speed.
#[derive(Component)]
pub struct BackgroundObscura {
    /// Constant velocity in logical pixels/second.
    pub velocity: Vec2,
    /// Amplitude of the sinusoidal wiggle **perpendicular** to the movement
    /// direction.
    pub wiggle_amp: f32,
    /// Frequency of the wiggle, measured in Hertz.
    pub wiggle_speed: f32,
    /// Phase offset so that each obscura element wiggles independently.
    pub phase: f32,
    /// Baseline coordinate around which the sprite wiggles. This is the *y*
    /// coordinate for horizontal movement and the *x* coordinate for vertical
    /// movement.
    pub base_perp: f32,
} 