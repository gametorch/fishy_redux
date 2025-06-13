use bevy::prelude::States;

/// Highest-level game flow.
#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    /// Temporary loading screen shown while assets load before we enter the real main menu.
    #[default]
    PreMainMenu,
    /// Theme picker shown after loading but before the main menu.
    ThemePicker,
    /// Main menu with buttons once a theme has been chosen.
    MainMenu,
    /// Actual gameplay running.
    InGame,
}

/// Sub-state while the game is running.
#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum InGameState {
    /// Pick your fish before gameplay begins.
    #[default]
    FishPicker,
    /// Normal gameplay after a fish has been chosen.
    Playing,
    /// Pause menu overlay.
    PauseMenu,
    /// Game over overlay shown when the player is eaten.
    GameOver,
} 