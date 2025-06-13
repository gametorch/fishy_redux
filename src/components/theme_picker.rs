use bevy::prelude::Component;

/// Root UI node for the theme picker menu.
#[derive(Component, Copy, Clone)]
pub struct ThemePickerUI;

/// Marker for the "Crayon" theme button.
#[derive(Component, Copy, Clone)]
pub struct CrayonButton;

/// Marker for the "Chibi" theme button.
#[derive(Component, Copy, Clone)]
pub struct ChibiButton;

/// Marker for the "Retro Pixelated" theme button.
#[derive(Component, Copy, Clone)]
pub struct RetroPixelButton; 