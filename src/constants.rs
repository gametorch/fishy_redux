//! Central repository for project-wide constants.

use bevy::prelude::Color;

/// Window clear color – deep-water navy (#002633).
pub const CLEAR_COLOR: Color = Color::srgb(0.0, 0.15, 0.20);

/// Button idle teal (#008C99).
pub const IDLE_COLOR: Color = Color::srgb(0.0, 0.55, 0.60);

/// Button hover – lighter teal (#00A5B4).
pub const HOVER_COLOR: Color = Color::srgb(0.0, 0.65, 0.71);

/// Button pressed – darker teal (#005F6B).
pub const PRESSED_COLOR: Color = Color::srgb(0.0, 0.37, 0.42);

/// Corner radius for rounded UI buttons (in logical pixels).
pub const BUTTON_RADIUS: f32 = 12.0;

// -----------------------------------------------------------------------------
// Fish-picker palette (soft underwater hues)
// -----------------------------------------------------------------------------

/// Fish-picker idle – fully transparent background.
pub const PICKER_IDLE_COLOR: Color = Color::NONE;

/// Fish-picker hover – semi-transparent to keep the image visible underneath.
pub const PICKER_HOVER_COLOR: Color = Color::srgba(0.43, 0.79, 0.70, 0.25);

/// Fish-picker pressed – deep turquoise (#1A7280).
pub const PICKER_PRESSED_COLOR: Color = Color::srgb(0.10, 0.45, 0.50);

// -----------------------------------------------------------------------------
// Theme-picker button colors (more transparent, like fish picker)
// -----------------------------------------------------------------------------

/// Theme-picker hover – semi-transparent blue-green.
pub const THEME_PICKER_HOVER_COLOR: Color = Color::srgba(0.20, 0.65, 0.90, 0.22);

/// Theme-picker pressed – semi-transparent deeper blue.
pub const THEME_PICKER_PRESSED_COLOR: Color = Color::srgba(0.10, 0.45, 0.80, 0.32); 

pub const THEME_PICKER_IDLE_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.0);