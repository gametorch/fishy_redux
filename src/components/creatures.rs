use bevy::prelude::Component;
use bevy::math::Vec2;
use bevy::prelude::Handle;
use bevy::prelude::Image;

/// Marker component for the player's fish sprite.
#[derive(Component, Copy, Clone)]
pub struct PlayerFish;

// NEW: Component representing the edible "meat" of a fish (area in px²).
#[derive(Component, Copy, Clone, Debug)]
pub struct Meat(pub usize);

/// Stores the original (unscaled) sprite area in pixels². Used to derive
/// how much to scale the sprite to match a desired `Meat`.
#[derive(Component, Copy, Clone, Debug)]
pub struct BaseSpriteArea(pub f32);

#[derive(Component, Copy, Clone, Debug, Default)]
pub struct Velocity(pub Vec2);

#[derive(Component, Clone, Debug)]
pub struct FishTexture(pub Handle<Image>); 