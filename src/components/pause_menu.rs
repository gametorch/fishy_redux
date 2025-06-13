use bevy::prelude::Component;

/// Marker for the Pause Menu root node.
#[derive(Component, Copy, Clone)]
pub struct PauseMenuUI;

/// Button markers
#[derive(Component, Copy, Clone)]
pub struct ContinueButton;

#[derive(Component, Copy, Clone)]
pub struct SaveButton;

#[derive(Component, Copy, Clone)]
pub struct QuitGameButton;

/// Marker for the "Main Menu" button shown in the pause menu.
#[derive(Component, Copy, Clone)]
pub struct PauseMainMenuButton; 