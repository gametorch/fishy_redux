use bevy::prelude::Component;

/// Root node for the fish selection UI.
#[derive(Component, Copy, Clone)]
pub struct FishPickerUI;

/// Marker component for the first fish option.
#[derive(Component, Copy, Clone)]
pub struct FishOption1;

/// Marker component for the second fish option.
#[derive(Component, Copy, Clone)]
pub struct FishOption2;

/// Marker component for the third fish option.
#[derive(Component, Copy, Clone)]
pub struct FishOption3; 