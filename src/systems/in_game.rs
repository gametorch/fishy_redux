use super::main_menu::spawn_menu_button;
use crate::systems::main_menu::BackgroundObscuraSpawner;
use crate::{
    assets::{EnemyFishAssets, ObscuraAssets, PlayableFishTextures},
    components::*,
    constants,
    states::{GameState, InGameState},
};
use bevy::input::ButtonInput;
use bevy::prelude::*;
use rand::distributions::Distribution;
use rand::Rng;
use rand_distr::Exp1;
use bevy_light_2d::prelude::*;
use bevy::input::touch::Touches;

// ------------------------------------------------------------
// New imports for Game Over overlay & score icon
// ------------------------------------------------------------
use crate::components::{GameOverMainMenuButton, GameOverUI};
use crate::alpha_masks::{AlphaMasks, AlphaMask};
use crate::theme::Theme;

/// Base transform scale for a freshly spawned player fish.
const PLAYER_BASE_SCALE: f32 = 0.05;

/// Movement tuning parameters
const MAX_SPEED: f32 = 500.0; // world units / second
const ACCELERATION: f32 = 800.0; // world units / (second²) when mouse held
const WATER_RESISTANCE: f32 = 1.0; // proportion of velocity lost per second

/// Spawn Pause Menu UI.
pub fn setup_pause_menu_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Semi-transparent overlay
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            PauseMenuUI,
        ))
        .with_children(|parent| {
            spawn_menu_button(parent, &asset_server, "Continue", ContinueButton);
            // Button navigating back to the main menu
            spawn_menu_button(parent, &asset_server, "Main Menu", PauseMainMenuButton);
            // Skip Quit on web builds where closing the tab is easier / preferred.
            if !cfg!(target_arch = "wasm32") {
                spawn_menu_button(parent, &asset_server, "Quit", QuitGameButton);
            }
        });
}

/// Remove pause menu entities.
pub fn cleanup_pause_menu(mut commands: Commands, query: Query<Entity, With<PauseMenuUI>>) {
    for e in &query {
        commands.entity(e).despawn();
    }
}

/// Pressing ESC while playing opens the pause menu.
#[allow(clippy::needless_pass_by_value)]
pub fn esc_to_pause_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<InGameState>>, // Change nested state
) {
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(InGameState::PauseMenu);
    }
}

/// Pressing ESC while in pause resumes.
#[allow(clippy::needless_pass_by_value)]
pub fn esc_to_resume_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<InGameState>>, // Change nested state
) {
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(InGameState::Playing);
    }
}

/// Handle Continue button to resume gameplay.
pub fn continue_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ContinueButton>),
    >,
    mut next_state: ResMut<NextState<InGameState>>, // Change nested state
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = constants::PRESSED_COLOR.into();
                next_state.set(InGameState::Playing);
            }
            Interaction::Hovered => *color = constants::HOVER_COLOR.into(),
            Interaction::None => *color = constants::IDLE_COLOR.into(),
        }
    }
}

/// Quit from pause menu.
pub fn pause_quit_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<QuitGameButton>),
    >,
    mut exit: EventWriter<AppExit>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = constants::PRESSED_COLOR.into();
                exit.send(AppExit::Success);
            }
            Interaction::Hovered => *color = constants::HOVER_COLOR.into(),
            Interaction::None => *color = constants::IDLE_COLOR.into(),
        }
    }
}

/// Handle the "Main Menu" button in the pause menu.
pub fn pause_main_menu_button_system(
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<PauseMainMenuButton>)>,
    mut next_state: ResMut<NextState<GameState>>, // Switch top-level state
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = constants::PRESSED_COLOR.into();
                next_state.set(GameState::MainMenu);
            }
            Interaction::Hovered => *color = constants::HOVER_COLOR.into(),
            Interaction::None => *color = constants::IDLE_COLOR.into(),
        }
    }
}

// ---------------------------------------------------------------------
// Fish-picker UI (shown when entering InGameState::FishPicker)
// ---------------------------------------------------------------------

/// Build the fish-picker UI presenting three fish options.
pub fn setup_fish_picker_ui(mut commands: Commands, textures: Res<PlayableFishTextures>) {
    // Root container holding the three selectable fish images side-by-side.
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Row,
                // Horizontal spacing between the three options.
                column_gap: Val::Px(40.0),
                ..default()
            },
            FishPickerUI,
        ))
        .with_children(|parent| {
            // Spawn each fish option as an image button.

            // Fish option 1
            parent.spawn((
                Button,
                // Constrain only the width; height will be calculated automatically to preserve
                // the image's aspect ratio.
                Node {
                    width: Val::Percent(22.0),
                    height: Val::Auto,
                    // Add some breathing room between the border and the sprite itself.
                    padding: UiRect::all(Val::Px(8.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                ImageNode::new(textures.fish1.clone()),
                BackgroundColor(constants::PICKER_IDLE_COLOR.into()),
                BorderColor(Color::BLACK),
                BorderRadius::all(Val::Px(12.0)),
                FishOption1,
            ));

            // Fish option 2
            parent.spawn((
                Button,
                Node {
                    width: Val::Percent(22.0),
                    height: Val::Auto,
                    padding: UiRect::all(Val::Px(8.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                ImageNode::new(textures.fish2.clone()),
                BackgroundColor(constants::PICKER_IDLE_COLOR.into()),
                BorderColor(Color::BLACK),
                BorderRadius::all(Val::Px(12.0)),
                FishOption2,
            ));

            // Fish option 3
            parent.spawn((
                Button,
                Node {
                    width: Val::Percent(22.0),
                    height: Val::Auto,
                    padding: UiRect::all(Val::Px(8.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                ImageNode::new(textures.fish3.clone()),
                BackgroundColor(constants::PICKER_IDLE_COLOR.into()),
                BorderColor(Color::BLACK),
                BorderRadius::all(Val::Px(12.0)),
                FishOption3,
            ));
        });
}

/// Despawn the fish-picker UI when leaving the picker state.
pub fn cleanup_fish_picker(mut commands: Commands, query: Query<Entity, With<FishPickerUI>>) {
    for e in &query {
        commands.entity(e).despawn_recursive();
    }
}

// ---------------------------------------------------------------------
// Fish selection interaction
// ---------------------------------------------------------------------

/// Holds the texture handle of the fish chosen by the player.
#[derive(Resource, Default, Clone)]
pub struct SelectedFish(pub Option<Handle<Image>>);

/// Generic system to handle interaction on a specific fish option.
fn fish_option_interaction<M: Component>(
    mut interaction_query: Query<
        (&Interaction, Entity, &mut BackgroundColor),
        (Changed<Interaction>, With<M>),
    >,
    _commands: Commands,
    textures: Res<PlayableFishTextures>,
    mut next_state: ResMut<NextState<InGameState>>, // Transition to Playing
    mut selected: ResMut<SelectedFish>,             // store chosen fish
) {
    for (interaction, _entity, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = constants::PICKER_PRESSED_COLOR.into();

                // Determine which fish texture corresponds to this marker type.
                let handle = if std::any::TypeId::of::<M>() == std::any::TypeId::of::<FishOption1>()
                {
                    textures.fish1.clone()
                } else if std::any::TypeId::of::<M>() == std::any::TypeId::of::<FishOption2>() {
                    textures.fish2.clone()
                } else {
                    textures.fish3.clone()
                };

                selected.0 = Some(handle);

                // We can despawn root UI here to avoid flicker but cleanup runs on state exit.
                // commands.entity(entity).despawn_recursive();

                next_state.set(InGameState::Playing);
            }
            Interaction::Hovered => {
                *color = constants::PICKER_HOVER_COLOR.into();
            }
            Interaction::None => {
                *color = constants::PICKER_IDLE_COLOR.into();
            }
        }
    }
}

/// Wire up individual interaction systems for each fish option.
pub fn fish_option1_system(
    mut interaction_query: Query<
        (&Interaction, Entity, &mut BackgroundColor),
        (Changed<Interaction>, With<FishOption1>),
    >,
    commands: Commands,
    textures: Res<PlayableFishTextures>,
    next_state: ResMut<NextState<InGameState>>,
    selected: ResMut<SelectedFish>,
) {
    fish_option_interaction::<FishOption1>(
        interaction_query,
        commands,
        textures,
        next_state,
        selected,
    );
}

pub fn fish_option2_system(
    mut interaction_query: Query<
        (&Interaction, Entity, &mut BackgroundColor),
        (Changed<Interaction>, With<FishOption2>),
    >,
    commands: Commands,
    textures: Res<PlayableFishTextures>,
    next_state: ResMut<NextState<InGameState>>,
    selected: ResMut<SelectedFish>,
) {
    fish_option_interaction::<FishOption2>(
        interaction_query,
        commands,
        textures,
        next_state,
        selected,
    );
}

pub fn fish_option3_system(
    mut interaction_query: Query<
        (&Interaction, Entity, &mut BackgroundColor),
        (Changed<Interaction>, With<FishOption3>),
    >,
    commands: Commands,
    textures: Res<PlayableFishTextures>,
    next_state: ResMut<NextState<InGameState>>,
    selected: ResMut<SelectedFish>,
) {
    fish_option_interaction::<FishOption3>(
        interaction_query,
        commands,
        textures,
        next_state,
        selected,
    );
}

// ---------------------------------------------------------------------
// Player fish spawning & cleanup
// ---------------------------------------------------------------------

/// Spawn the chosen fish sprite when entering the Playing state.
pub fn spawn_player_fish_sprite(
    mut commands: Commands,
    images: Res<Assets<Image>>, // Needed to query image dimensions.
    windows: Query<&Window>,
    selected: Res<SelectedFish>,
    textures: Res<PlayableFishTextures>,
    theme: Res<Theme>,
    existing_fish: Query<Entity, With<PlayerFish>>, // avoid duplicates
) {
    // Prevent spawning duplicate player fish when resuming from pause.
    if !existing_fish.is_empty() {
        return;
    }

    if let Some(handle) = &selected.0 {
        if let Some(image) = images.get(handle) {
            // Original pixel dimensions of the texture.
            let width = image.texture_descriptor.size.width as f32;
            let height = image.texture_descriptor.size.height as f32;

            // Determine desired minimum rendered size based on window.
            let window = windows.single().expect("No primary window");

            let min_width = 64.0_f32.max(window.width() / 100.0);
            let min_height = 64.0_f32.max(window.height() / 100.0);

            // Scale factors required to reach the minimum size.
            let scale_w = min_width / width;
            let scale_h = min_height / height;

            // Ensure uniform scaling and at least PLAYER_BASE_SCALE.
            let desired_scale = scale_w.max(scale_h).max(PLAYER_BASE_SCALE);

            let scaled_width = width * desired_scale;
            let scaled_height = height * desired_scale;

            // Initial meat is area in pixel² after scaling.
            let area = (scaled_width * scaled_height) as usize;

            let base_area = width * height;

            let entity = commands
                .spawn(Sprite::from_image(handle.clone()))
                .insert(Transform::from_scale(Vec3::splat(desired_scale)))
                .insert(PlayerFish)
                .insert(Meat(area))
                .insert(BaseSpriteArea(base_area))
                .insert(Velocity::default())
                .insert(FishTexture(handle.clone()))
                .id();

            if handle.id() == textures.fish3.id() {
                // Theme-specific lure position (unscaled, px from top-left)
                let (ox_px, oy_px) = match *theme {
                    Theme::Chibi => (280.0, 205.0),
                    Theme::Crayon => (350.0, 290.0),
                    Theme::Retro => (180.0, 455.0),
                };
                let offset_x = -(width / 2.0 - ox_px);
                let offset_y = height / 2.0 - oy_px;

                // Tighter radius & warmer colour for point-source glow
                let radius = scaled_width.max(scaled_height);
                commands.entity(entity).with_children(|parent| {
                    parent.spawn((
                        PointLight2d {
                            intensity: 20.0,
                            radius,
                            color: Color::srgb(1.0, 0.85, 0.4),
                            ..default()
                        },
                        Transform::from_xyz(offset_x, offset_y, 1.0),
                        AnglerLight { base_x: offset_x, base_y: offset_y, base_radius: radius },
                    ));
                });
            }
        } else {
            // Asset not ready yet; skip spawning for this frame.
            warn!("Selected fish texture not yet loaded – delaying player fish spawn");
        }
    } else {
        eprintln!("No fish selected when entering Playing state");
    }
}

/// System that keeps a player's fish visual scale in sync with its Meat value.
/// If the meat changes (e.g., by eating in future gameplay), the sprite will grow
/// or shrink accordingly so that `scaled_width * scaled_height == meat`.
pub fn update_player_fish_scale(
    mut query: Query<(&Meat, &BaseSpriteArea, &mut Transform), With<PlayerFish>>,
) {
    for (meat, base_area, mut transform) in &mut query {
        if base_area.0 <= 0.0 {
            continue; // Avoid division by zero or invalid data.
        }

        // Desired uniform scale so that sprite area equals `meat`.
        let desired_scale = (meat.0 as f32 / base_area.0).sqrt();

        // Update the transform if it differs significantly to avoid tiny jitter.
        if (desired_scale - transform.scale.x).abs() > f32::EPSILON {
            transform.scale = Vec3::splat(desired_scale);
        }
    }
}

/// Remove player fish sprite when leaving gameplay.
pub fn cleanup_player_fish(mut commands: Commands, query: Query<Entity, With<PlayerFish>>) {
    for e in &query {
        commands.entity(e).despawn();
    }
}

// ---------------------------------------------------------------------
// State navigation from FishPicker
// ---------------------------------------------------------------------

/// Pressing ESC in the fish picker returns to the main menu.
pub fn esc_to_main_menu_from_picker_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>, // Switch top-level state
) {
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::MainMenu);
    }
}

// ---------------------------------------------------------------------
// Player movement systems
// ---------------------------------------------------------------------

/// Accelerate the fish towards the mouse cursor while the left button is held or clicked.
pub fn player_fish_acceleration_system(
    windows: Query<&Window>,
    mouse_buttons: Res<ButtonInput<MouseButton>>, // Mouse input state
    touches: Res<Touches>,                        // NEW: active touches
    time: Res<Time>,
    mut query: Query<(&Transform, &mut Velocity), With<PlayerFish>>,
) {
    // ---------------------------------------------------------------------
    // Determine whether the player is providing *any* form of pointing
    // input that should move the fish: either a left-mouse button press
    // (desktop) *or* at least one active touch (mobile).
    // ---------------------------------------------------------------------

    let touch_active = touches.iter().next().is_some();

    if !mouse_buttons.pressed(MouseButton::Left) && !touch_active {
        return; // no acceleration unless input is active
    }

    let window = match windows.single() {
        Ok(w) => w,
        Err(_) => return,
    };

    // ---------------------------------------------------------------------
    // Determine the target position in *window* coordinates. We prioritise a
    // touch location (if present) so that simultaneous mouse + touch input
    // behaves intuitively on hybrid devices.
    // ---------------------------------------------------------------------

    let cursor_screen: Option<Vec2> = if let Some(touch) = touches.iter().next() {
        Some(touch.position())
    } else {
        window.cursor_position()
    };

    // Convert to world coordinates and apply acceleration towards that point.
    if let Some(cursor) = cursor_screen {
        let cursor_world = Vec2::new(
            cursor.x - window.width() / 2.0,
            window.height() / 2.0 - cursor.y,
        );

        for (transform, mut vel) in &mut query {
            let pos = Vec2::new(transform.translation.x, transform.translation.y);
            let dir_vec = cursor_world - pos;
            if dir_vec.length_squared() > 1e-4 {
                let dir = dir_vec.normalize();
                // Apply acceleration scaled by frame time
                vel.0 += dir * ACCELERATION * time.delta_secs();
                // Clamp to max speed
                let speed = vel.0.length();
                if speed > MAX_SPEED {
                    vel.0 = vel.0.normalize() * MAX_SPEED;
                }
            }
        }
    }
}

/// Apply velocity to position and simulate water resistance.
pub fn player_fish_movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &mut Transform), With<PlayerFish>>,
) {
    for (mut vel, mut transform) in &mut query {
        // Update position
        transform.translation.x += vel.0.x * time.delta_secs();
        transform.translation.y += vel.0.y * time.delta_secs();

        // Apply simple linear damping representing water resistance
        let decay = (1.0 - WATER_RESISTANCE * time.delta_secs()).max(0.0);
        vel.0 *= decay;

        // Threshold small velocities to zero to avoid jitter
        if vel.0.length_squared() < 0.01 {
            vel.0 = Vec2::ZERO;
        }
    }
}

/// Adjust sprite orientation (flip horizontally) based on current velocity.
pub fn player_fish_orientation_system(
    mut query: Query<(&Velocity, &mut Sprite, &mut Transform), With<PlayerFish>>,
) {
    for (vel, mut sprite, mut transform) in &mut query {
        let vx = vel.0.x;
        if vx.abs() < 0.1 {
            continue; // avoid jitter for tiny movements
        }

        // Ensure uniform positive scale (orientation purely via flip_x)
        transform.scale.x = transform.scale.x.abs();

        // Default sprite faces left; flip when swimming right
        sprite.flip_x = vx > 0.0;
    }
}

/// ---------------------------------------------------------------------
/// Background obscura spawning & movement during gameplay
/// ---------------------------------------------------------------------

/// Helper to spawn a single obscura sprite with randomised parameters.
fn spawn_single_background_obscura_ingame(
    commands: &mut Commands,
    obscura: &ObscuraAssets,
    obscura_asset_index: usize,
    window: &Window,
    images: &Assets<Image>,
    rng: &mut impl Rng,
    spawn_inside: bool,
) {
    use bevy::prelude::*;

    let handle = obscura.images[obscura_asset_index].clone();

    // Randomise direction: 0 = right→left, 1 = left→right, 2 = bottom→top, 3 = top→bottom
    let dir = rng.gen_range(0..4);

    let speed = rng.gen_range(30.0..70.0); // pixels per second in drift direction
    let wiggle_amp = rng.gen_range(10.0..40.0);
    let wiggle_speed = rng.gen_range(0.5..1.5); // Hz
    let phase = rng.gen_range(0.0..std::f32::consts::TAU);

    // Determine velocity based on direction
    let velocity: Vec2 = match dir {
        0 => Vec2::new(-speed, 0.0), // leftwards
        1 => Vec2::new(speed, 0.0),  // rightwards
        2 => Vec2::new(0.0, speed),  // upwards
        _ => Vec2::new(0.0, -speed), // downwards
    };

    // Start position & baseline perpendicular coordinate
    let (start_pos, base_perp) = if spawn_inside {
        let half_w = window.width() / 2.0;
        let half_h = window.height() / 2.0;
        let x = rng.gen_range(-half_w..half_w);
        let y = rng.gen_range(-half_h..half_h);

        let base = if velocity.x.abs() > 0.0 { y } else { x };
        (Vec3::new(x, y, -1.0), base)
    } else {
        match dir {
            0 => {
                let x = window.width() / 2.0 + 100.0;
                let y = rng.gen_range(-window.height() / 2.0..window.height() / 2.0);
                (Vec3::new(x, y, -1.0), y)
            }
            1 => {
                let x = -window.width() / 2.0 - 100.0;
                let y = rng.gen_range(-window.height() / 2.0..window.height() / 2.0);
                (Vec3::new(x, y, -1.0), y)
            }
            2 => {
                let y = -window.height() / 2.0 - 100.0;
                let x = rng.gen_range(-window.width() / 2.0..window.width() / 2.0);
                (Vec3::new(x, y, -1.0), x)
            }
            _ => {
                let y = window.height() / 2.0 + 100.0;
                let x = rng.gen_range(-window.width() / 2.0..window.width() / 2.0);
                (Vec3::new(x, y, -1.0), x)
            }
        }
    };

    // Scale so that the tallest side is at most 1/25th of the screen height.
    let img_height = images
        .get(&handle)
        .map(|img| img.texture_descriptor.size.height as f32)
        .unwrap_or(200.0);

    let desired_height = window.height() / 25.0;
    let uniform_scale = desired_height / img_height;
    let scale = Vec3::splat(uniform_scale);

    commands.spawn((
        Sprite::from_image(handle.clone()),
        Transform::from_translation(start_pos).with_scale(scale),
        BackgroundObscura {
            velocity,
            wiggle_amp,
            wiggle_speed,
            phase,
            base_perp,
        },
    ));
}

/// Initial spawn of 1–2 obscura sprites when entering gameplay.
pub fn spawn_background_obscura_initial_ingame(
    mut commands: Commands,
    obscura_opt: Option<Res<ObscuraAssets>>,
    windows: Query<&Window>,
    images: Res<Assets<Image>>,
    existing: Query<&BackgroundObscura>,
) {
    // If obscura already present (e.g. we just un-paused) skip to avoid duplicates.
    if !existing.is_empty() {
        return;
    }

    commands.insert_resource(BackgroundObscuraSpawner::default());

    let Some(obscura) = obscura_opt else {
        return;
    };
    if obscura.images.is_empty() {
        return;
    }

    let window = windows.single().expect("No primary window");
    let mut rng = rand::thread_rng();

    let count = rng.gen_range(1..=2);
    for _ in 0..count {
        let idx = rng.gen_range(0..obscura.images.len());
        spawn_single_background_obscura_ingame(
            &mut commands,
            &obscura,
            idx,
            window,
            &images,
            &mut rng,
            true,
        );
    }
}

/// Periodically spawn additional obscura during gameplay, keeping at most 2 on screen.
pub fn background_obscura_spawn_system_ingame(
    mut commands: Commands,
    obscura_opt: Option<Res<ObscuraAssets>>,
    windows: Query<&Window>,
    mut spawner: ResMut<BackgroundObscuraSpawner>,
    time: Res<Time>,
    existing: Query<&BackgroundObscura>,
    images: Res<Assets<Image>>,
) {
    let Some(obscura) = obscura_opt else {
        return;
    };
    if obscura.images.is_empty() {
        return;
    }

    let window = windows.single().expect("No primary window");
    spawner.timer.tick(time.delta());

    // Maintain at most 2 obscura elements.
    if spawner.timer.finished() && existing.iter().count() < 2 {
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(0..obscura.images.len());
        spawn_single_background_obscura_ingame(
            &mut commands,
            &obscura,
            idx,
            window,
            &images,
            &mut rng,
            false,
        );

        // Schedule next spawn between 4–7 seconds for a calmer background.
        let next = rng.gen_range(4.0..7.0);
        spawner
            .timer
            .set_duration(std::time::Duration::from_secs_f32(next));
        spawner.timer.reset();
    }
}

// ---------------------------------------------------------------------
// Ambient moving fish (non-player)
// ---------------------------------------------------------------------

/// Simple marker for autonomous fish that swim straight across the screen.
#[derive(Component)]
pub struct MovingFish {
    pub velocity: Vec2,
    pub wiggle_amp: f32,   // radians
    pub wiggle_speed: f32, // Hz
    pub phase: f32,
}

/// Resource controlling Poisson-distributed spawning of ambient fish.
#[derive(Resource)]
pub struct MovingFishSpawner {
    pub timer: Timer,
}

impl Default for MovingFishSpawner {
    fn default() -> Self {
        // Set first interval to exponential(1) seconds (mean 1 s).
        let mut rng = rand::thread_rng();
        let dur: f32 = Exp1.sample(&mut rng);
        Self {
            timer: Timer::from_seconds(dur, bevy::time::TimerMode::Once),
        }
    }
}

/// Spawn a single moving fish entity.
fn spawn_single_moving_fish(
    commands: &mut Commands,
    enemy_assets: &EnemyFishAssets,
    images: &Assets<Image>,
    window: &Window,
) {
    use bevy::prelude::*;
    let mut rng = rand::thread_rng();

    if enemy_assets.images.is_empty() {
        return; // nothing to spawn
    }
    let handle = enemy_assets.images[rng.gen_range(0..enemy_assets.images.len())].clone();

    // Determine original image dimensions (fallback 200×200 if not yet loaded).
    let (img_w, img_h) = images
        .get(&handle)
        .map(|img| {
            let w = img.texture_descriptor.size.width as f32;
            let h = img.texture_descriptor.size.height as f32;
            (w, h)
        })
        .unwrap_or((200.0, 200.0));

    // Target area range: between 0.1× player base area and 1/9th of screen area.
    let screen_area = window.width() * window.height();
    let min_area = img_w * img_h * PLAYER_BASE_SCALE * PLAYER_BASE_SCALE * 0.1;
    let max_area = screen_area / 9.0;

    // Bias towards smaller fish by sampling with a quadratic distribution
    // Generate a uniform [0,1) random, square it (values cluster near 0), then
    // remap into [min_area, max_area].
    let t: f32 = rng.gen::<f32>().powf(2.0);
    let desired_area = min_area + t * (max_area - min_area);
    let scale = (desired_area / (img_w * img_h)).sqrt();

    // Speed inversely correlated with size, plus randomness.
    let base_speed = 25.0; // tuning constant
    let speed = (base_speed / scale).clamp(10.0, 150.0) * rng.gen_range(0.8..1.2);

    // Decide side: 0 = left → right, 1 = right → left.
    let side = rng.gen_bool(0.5);

    let half_sprite_width = images
        .get(&handle)
        .map(|img| img.texture_descriptor.size.width as f32)
        .unwrap_or(300.0)
        * scale
        / 2.0;
    let mut sprite = Sprite::from_image(handle.clone());
    let (x, vx, flip_x) = if side {
        (
            -window.width() / 2.0 - half_sprite_width,
            speed,
            true, // sprite faces right (default faces left)
        )
    } else {
        (window.width() / 2.0 + half_sprite_width, -speed, false)
    };
    sprite.flip_x = flip_x;

    let y = rng.gen_range(-window.height() / 2.0..window.height() / 2.0);

    // Calculate meat (area in pixel² after scaling) for collision logic.
    let meat_val = (img_w * img_h * scale * scale) as usize;

    // Random subtle rotational wiggle parameters
    let wiggle_amp = rng.gen_range(0.03..0.12); // radians (~1.7°–6.9°)
    let wiggle_speed = rng.gen_range(0.4..1.2); // Hz
    let phase = rng.gen_range(0.0..std::f32::consts::TAU);

    commands.spawn((
        sprite,
        Transform::from_translation(Vec3::new(x, y, -0.5)).with_scale(Vec3::splat(scale)),
        MovingFish {
            velocity: Vec2::new(vx, 0.0),
            wiggle_amp,
            wiggle_speed,
            phase,
        },
        Meat(meat_val),
        FishTexture(handle.clone()),
    ));
}

/// Initialise the spawner when gameplay starts.
pub fn setup_moving_fish_spawner(mut commands: Commands) {
    commands.insert_resource(MovingFishSpawner::default());
}

/// Tick spawner timer and create new fish when needed.
pub fn moving_fish_spawn_system(
    mut commands: Commands,
    mut spawner: ResMut<MovingFishSpawner>,
    time: Res<Time>,
    enemy_assets: Res<EnemyFishAssets>,
    images: Res<Assets<Image>>,
    windows: Query<&Window>,
) {
    let window = windows.single().expect("No primary window");

    spawner.timer.tick(time.delta());
    if spawner.timer.finished() {
        spawn_single_moving_fish(&mut commands, &enemy_assets, &images, window);

        // Schedule next interval from exponential(1) distribution.
        let mut rng = rand::thread_rng();
        let next: f32 = Exp1.sample(&mut rng);
        spawner
            .timer
            .set_duration(std::time::Duration::from_secs_f32(next));
        spawner.timer.reset();
    }
}

/// Move ambient fish every frame and despawn when off-screen.
pub fn moving_fish_movement_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &MovingFish)>,
    time: Res<Time>,
    windows: Query<&Window>,
) {
    let dt = time.delta_secs();
    let window = windows.single().expect("No primary window");
    let half_w = window.width() / 2.0;

    let elapsed = time.elapsed_secs();

    for (entity, mut transform, fish) in &mut query {
        transform.translation.x += fish.velocity.x * dt;

        // Apply gentle rotational wiggle around Z axis
        let angle = fish.wiggle_amp * (elapsed * fish.wiggle_speed + fish.phase).sin();
        transform.rotation = Quat::from_rotation_z(angle);

        if transform.translation.x < -half_w - 150.0 || transform.translation.x > half_w + 150.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn cleanup_moving_fish(mut commands: Commands, query: Query<Entity, With<MovingFish>>) {
    for e in &query {
        commands.entity(e).despawn();
    }
}

/// UI marker for the root node that holds the on-screen Meat score.
#[derive(Component)]
pub struct MeatScoreUI;

/// UI marker for the text displaying the Meat value.
#[derive(Component)]
pub struct MeatScoreText;

/// Spawn the meat score UI when entering the Playing state.
pub fn spawn_meat_score_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    theme: Res<Theme>,
) {
    let font = asset_server.load("fonts/Fredoka.ttf");
    let icon_handle = asset_server.load(theme.path("score_icon.png"));

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Start, // align at top edge
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(6.0),
                padding: UiRect {
                    top: Val::Px(10.0),
                    ..default()
                },
                ..default()
            },
            MeatScoreUI,
        ))
        .with_children(|parent| {
            // Numeric text
            parent.spawn((
                Text::new("0"),
                TextFont {
                    font,
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                TextLayout::new_with_justify(JustifyText::Center),
                MeatScoreText,
            ));

            // Icon sprite
            parent.spawn((
                ImageNode::new(icon_handle),
                Node {
                    height: Val::Px(48.0),
                    width: Val::Auto,
                    ..default()
                },
            ));
        });
}

/// Update the meat score text (numeric value) every frame.
pub fn update_meat_score_ui(
    player: Query<&Meat, With<PlayerFish>>,
    mut texts: Query<&mut Text, With<MeatScoreText>>,
) {
    let meat_value = player.iter().next().map(|m| m.0).unwrap_or(0);

    for mut text in &mut texts {
        // Replace the displayed value only (no label).
        *text = Text::new(format!("{}", meat_value));
    }
}

/// Cleanup the meat score UI when leaving gameplay.
pub fn cleanup_meat_score_ui(mut commands: Commands, query: Query<Entity, With<MeatScoreUI>>) {
    for e in &query {
        commands.entity(e).despawn_recursive();
    }
}

// ------------------------------------------------------------
// Collision detection – player vs ambient moving fish
// ------------------------------------------------------------

/// Detect collisions between the player's fish and moving enemy fish. If the
/// enemy fish is smaller, despawn it and grow the player's meat by 10 % of the
/// enemy's meat.
pub fn collision_detection_system(
    mut commands: Commands,
    mut player_q: Query<(&Transform, &FishTexture, &mut Meat), (With<PlayerFish>, Without<MovingFish>)>,
    enemies: Query<(Entity, &Transform, &FishTexture, &Meat), (With<MovingFish>, Without<PlayerFish>)>,
    images: Res<Assets<Image>>, // for image info
    mut masks: ResMut<AlphaMasks>,
    mut next_state: ResMut<NextState<InGameState>>, // For triggering GameOver
) {
    let Ok((player_tf, player_tex, mut player_meat)) = player_q.get_single_mut() else {
        return;
    };
    let player_handle = &player_tex.0;

    // Approximate each fish with a circle whose radius is proportional to the
    // square-root of its meat (treating the sprite as roughly square).
    let player_radius = (player_meat.0 as f32).sqrt() * 0.5;

    for (entity, enemy_tf, enemy_tex, enemy_meat) in enemies.iter() {
        let enemy_handle = &enemy_tex.0;
        let enemy_radius = (enemy_meat.0 as f32).sqrt() * 0.5;

        let dx = enemy_tf.translation.x - player_tf.translation.x;
        let dy = enemy_tf.translation.y - player_tf.translation.y;
        let dist_sq = dx * dx + dy * dy;
        let radii = player_radius + enemy_radius;

        if dist_sq <= radii * radii {
            // Potential overlap – run pixel-level narrow phase.
            if pixel_perfect_overlap(
                player_tf,
                player_handle,
                enemy_tf,
                enemy_handle,
                &images,
                &mut masks,
            ) {
                // Confirmed overlap – evaluate size rule.
                if enemy_meat.0 < player_meat.0 {
                    // Increase by 25 % of the enemy's meat (rounded down).
                    player_meat.0 += (enemy_meat.0 as f32 * 0.25) as usize;
                    commands.entity(entity).despawn();
                } else {
                    next_state.set(InGameState::GameOver);
                }
            }
        }
    }
}

/// Pixel-perfect overlap test ignoring rotation and using cached alpha masks.
fn pixel_perfect_overlap(
    a_tf: &Transform,
    a_handle: &Handle<Image>,
    b_tf: &Transform,
    b_handle: &Handle<Image>,
    images: &Assets<Image>,
    masks: &mut AlphaMasks,
) -> bool {
    // Ensure both masks are present in the cache, inserting if necessary.
    if !masks.0.contains_key(a_handle) {
        if let Some(img) = images.get(a_handle) {
            masks.0.insert(a_handle.clone(), AlphaMask::from_image(img));
        }
    }
    if !masks.0.contains_key(b_handle) {
        if let Some(img) = images.get(b_handle) {
            masks.0.insert(b_handle.clone(), AlphaMask::from_image(img));
        }
    }

    let Some(mask_a) = masks.0.get(a_handle) else { return true; };
    let Some(mask_b) = masks.0.get(b_handle) else { return true; };

    // Sprite world half-sizes.
    let a_half_w = mask_a.width as f32 * a_tf.scale.x * 0.5;
    let a_half_h = mask_a.height as f32 * a_tf.scale.y * 0.5;
    let b_half_w = mask_b.width as f32 * b_tf.scale.x * 0.5;
    let b_half_h = mask_b.height as f32 * b_tf.scale.y * 0.5;

    // Determine overlap rectangle in world space.
    let left = (a_tf.translation.x - a_half_w).max(b_tf.translation.x - b_half_w);
    let right = (a_tf.translation.x + a_half_w).min(b_tf.translation.x + b_half_w);
    let bottom = (a_tf.translation.y - a_half_h).max(b_tf.translation.y - b_half_h);
    let top = (a_tf.translation.y + a_half_h).min(b_tf.translation.y + b_half_h);

    if right <= left || top <= bottom {
        return false; // no overlap
    }

    // Choose sampling step based on the finer scale for performance (≥1 px in world units).
    let step = a_tf.scale.x.min(b_tf.scale.x).max(0.5); // clamp min 0.5 world units

    let mut y = bottom;
    while y <= top {
        let mut x = left;
        while x <= right {
            // Convert world (x,y) to pixel coords in each image.
            let ax = ((x - a_tf.translation.x) / a_tf.scale.x) + mask_a.width as f32 / 2.0;
            let ay = ((y - a_tf.translation.y) / a_tf.scale.y) + mask_a.height as f32 / 2.0;
            let bx = ((x - b_tf.translation.x) / b_tf.scale.x) + mask_b.width as f32 / 2.0;
            let by = ((y - b_tf.translation.y) / b_tf.scale.y) + mask_b.height as f32 / 2.0;

            if mask_a.is_opaque(ax as u32, ay as u32) && mask_b.is_opaque(bx as u32, by as u32) {
                return true;
            }

            x += step;
        }
        y += step;
    }

    false
}

// ---------------------------------------------------------------------
// Game Over overlay UI
// ---------------------------------------------------------------------

/// Spawn the Game Over UI overlay when entering `InGameState::GameOver`.
pub fn setup_game_over_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font_bold = asset_server.load("fonts/Fredoka-Bold.ttf");
    let font = asset_server.load("fonts/Fredoka.ttf");

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)), // semi-transparent black
            GameOverUI,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Game Over!"),
                TextFont {
                    font: font_bold.clone(),
                    font_size: 72.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                TextLayout::new_with_justify(JustifyText::Center),
            ));

            parent.spawn((
                Text::new("You were eaten by a fish that's bigger than you!"),
                TextFont {
                    font,
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                TextLayout::new_with_justify(JustifyText::Center),
            ));

            // Main Menu button
            spawn_menu_button(parent, &asset_server, "Main Menu", GameOverMainMenuButton);
        });
}

/// Remove all Game Over UI entities.
pub fn cleanup_game_over(mut commands: Commands, query: Query<Entity, With<GameOverUI>>) {
    for e in &query {
        commands.entity(e).despawn_recursive();
    }
}

/// Handle the "Main Menu" button on the Game Over screen.
pub fn game_over_main_menu_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<GameOverMainMenuButton>),
    >,
    mut next_state: ResMut<NextState<GameState>>, // Switch to main menu
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = constants::PRESSED_COLOR.into();
                next_state.set(GameState::MainMenu);
            }
            Interaction::Hovered => *color = constants::HOVER_COLOR.into(),
            Interaction::None => *color = constants::IDLE_COLOR.into(),
        }
    }
}

// Add component to track angler light offset
#[derive(Component)]
pub struct AnglerLight {
    base_x: f32,
    base_y: f32,
    base_radius: f32, // NEW: store the spawn radius
}

// System to mirror light when fish flips direction
pub fn update_angler_light_position(
    mut fish_q: Query<(&Sprite, &Children), With<PlayerFish>>,
    mut light_q: Query<(&mut Transform, &AnglerLight)>,
) {
    for (sprite, children) in &mut fish_q {
        let sign = if sprite.flip_x { -1.0 } else { 1.0 };
        for child in children.iter() {
            if let Ok((mut tf, angler)) = light_q.get_mut(child) {
                tf.translation.x = angler.base_x * sign;
                tf.translation.y = angler.base_y;
            }
        }
    }
}

// System to animate angler light intensity and radius
pub fn animate_angler_light_system(
    time: Res<Time>,
    mut query: Query<(&mut PointLight2d, &AnglerLight)>,
) {
    let t = time.elapsed_secs();
    let intensity = 3.5 + 2.5 * (t * 1.2).sin();
    let radius_coeff = 0.5 + 0.05 * (t * 1.2).sin();
    for (mut light, angler) in &mut query {
        light.intensity = intensity;
        light.radius = angler.base_radius * radius_coeff;
    }
}
