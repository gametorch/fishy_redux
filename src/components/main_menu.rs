use bevy::prelude::Component;

/// Marker for the Quit button in the main menu.
#[derive(Component, Copy, Clone)]
pub struct QuitButton;

/// Marker for the Play button in the main menu.
#[derive(Component, Copy, Clone)]
pub struct PlayButton;

/// Marker for the Theme Picker button in the main menu.
#[derive(Component, Copy, Clone)]
pub struct ThemePickerButton;

/// Tag for all UI entities that belong to the main menu.
#[derive(Component, Copy, Clone)]
pub struct MainMenuUI;

/// Marker component for the small "Loading…" indicator shown at the bottom of
/// the main menu while theme assets are (re)loading after a new theme has been
/// selected.
#[derive(Component, Copy, Clone)]
pub struct MainMenuLoadingUI; 