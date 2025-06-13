use bevy::prelude::Component;

/// Marker for the root node of the Game-Over overlay UI.
#[derive(Component, Copy, Clone)]
pub struct GameOverUI;

/// Marker for the "Main Menu" button shown on the Game-Over screen.
#[derive(Component, Copy, Clone)]
pub struct GameOverMainMenuButton; 