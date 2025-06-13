use bevy::prelude::*;

/// Tag for static flora decorations that sit at the sea floor in the main menu.
#[derive(Component)]
pub struct BackgroundFlora {
    /// Base uniform scale applied at rest.
    pub base_scale: f32,
    /// Amplitude of the pulsating scale animation (fraction of `base_scale`).
    pub pulse_amp: f32,
    /// Frequency of both pulsation and wiggle, in Hertz.
    pub pulse_speed: f32,
    /// Phase offset so each flora animates independently.
    pub phase: f32,
    /// Amplitude of the subtle rotational wiggle (in radians).
    pub wiggle_amp: f32,
} 