use std::collections::HashSet;

use crate::assets::ObscuraAssets;
use crate::components::BackgroundObscura;
use crate::assets::FloraAssets;
use crate::components::BackgroundFlora;
use crate::{components::*, constants, states::GameState};
use bevy::prelude::*;
use rand::Rng;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::render::render_asset::RenderAssetUsages;
use crate::assets::{PlayableFishTextures, EnemyFishAssets};
use crate::systems::theme_picker::PendingThemeSelectSfx;
use bevy::audio::{AudioPlayer, PlaybackSettings};

/// Build the main-menu UI with "Play!" and "Quit" buttons.
pub fn setup_menu_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Root node that fills the entire window and centers its children.
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
            MainMenuUI,
        ))
        .with_children(|parent| {
            // Play button (state transition)
            spawn_menu_button(parent, &asset_server, "Play!", PlayButton);

            // Theme picker button (return to theme selection screen)
            spawn_menu_button(parent, &asset_server, "Themes", ThemePickerButton);

            // Quit button (exit app) – desktop only
            if !cfg!(target_arch = "wasm32") {
                spawn_menu_button(parent, &asset_server, "Quit", QuitButton);
            }
        });
}

/// Helper to spawn a generic menu button with shared styling.
pub fn spawn_menu_button<C: Component + Copy + 'static>(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    label: &str,
    marker: C,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(220.0),
                height: Val::Px(90.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(constants::IDLE_COLOR),
            BorderRadius::all(Val::Px(constants::BUTTON_RADIUS)),
            marker,
        ))
        .with_children(|p| {
            // Use bold font specifically for the Play! button if available.
            let font_path = if label == "Play!" {
                "fonts/Fredoka-Bold.ttf"
            } else {
                "fonts/Fredoka.ttf"
            };

            p.spawn((
                Text::new(label),
                TextFont {
                    font: asset_server.load(font_path),
                    font_size: 42.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                TextLayout::new_with_justify(JustifyText::Center),
            ));
        });
}

/// Change button color on hover/press and quit when pressed.
pub fn quit_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<QuitButton>),
    >,
    mut exit: EventWriter<AppExit>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // Pressed: darker teal (#005F6B)
                *color = constants::PRESSED_COLOR.into();
                exit.send(AppExit::Success);
            }
            Interaction::Hovered => {
                // Hover: lighter teal (#00A5B4)
                *color = constants::HOVER_COLOR.into();
            }
            Interaction::None => {
                // Idle revert
                *color = constants::IDLE_COLOR.into();
            }
        }
    }
}

/// Handle the Play! button.
pub fn play_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<PlayButton>),
    >,
    mut next_state: ResMut<NextState<GameState>>, // Transition top-level game state
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = constants::PRESSED_COLOR.into();
                next_state.set(GameState::InGame);
            }
            Interaction::Hovered => *color = constants::HOVER_COLOR.into(),
            Interaction::None => *color = constants::IDLE_COLOR.into(),
        }
    }
}

/// Handle the Theme Picker button.
pub fn theme_picker_button_system(
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<ThemePickerButton>)>,
    mut next_state: ResMut<NextState<GameState>>, // Transition to ThemePicker
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = constants::PRESSED_COLOR.into();
                next_state.set(GameState::ThemePicker);
            }
            Interaction::Hovered => *color = constants::HOVER_COLOR.into(),
            Interaction::None => *color = constants::IDLE_COLOR.into(),
        }
    }
}

/// Despawn all entities tagged as part of the main menu.
pub fn cleanup_menu(
    mut commands: Commands,
    menu_query: Query<Entity, With<MainMenuUI>>,
    loading_query: Query<Entity, With<MainMenuLoadingUI>>,
) {
    for e in &menu_query {
        commands.entity(e).despawn();
    }

    // Ensure we also remove the bottom "Loading…" indicator if it is still
    // present when leaving the main menu.
    for e in &loading_query {
        commands.entity(e).despawn_recursive();
    }
}

// -----------------------------------------------------------------------------
// Background obscura drifting across the main menu
// -----------------------------------------------------------------------------

/// Spawn an initial batch (6-8) of obscura sprites when entering the main menu.
pub fn spawn_background_obscura_initial(
    mut commands: Commands,
    obscura_opt: Option<Res<ObscuraAssets>>,
    windows: Query<&Window>,
    images: Res<Assets<Image>>,
    flora_opt: Option<Res<FloraAssets>>,
) {
    // Always create the spawner so that subsequent systems depending on it
    // won't panic even if the obscura assets haven't loaded yet.
    commands.insert_resource(BackgroundObscuraSpawner::default());

    // If the resource hasn't loaded yet we simply skip spawning fish for now.
    let Some(obscura) = obscura_opt else {
        return;
    };

    if obscura.images.is_empty() {
        // Nothing to spawn – either directory empty or failed to load.
        return;
    }

    // Wait for all obscura textures to load before spawning, just like flora.
    if obscura.images.iter().any(|h| images.get(h).is_none()) {
        trace!("spawn_background_obscura_initial: waiting for all obscura textures to load");
        return;
    }

    let window = windows.single().expect("No primary window");

    let mut rng = rand::thread_rng();

    // Spawn between 6 and 8 obscura elements.
    let count = rng.gen_range(6..=8);
    let mut previously_spawned_obscura_asset_indices = HashSet::new();
    for _ in 0..count {
        let obscura_asset_index = {
            let mut ret = None;
            for _ in 0..2 * obscura.images.len() {
                let random_index = rng.gen_range(0..obscura.images.len());
                if !previously_spawned_obscura_asset_indices.contains(&random_index) {
                    ret = Some(random_index);
                    break;
                }
            }
            let ret = ret.unwrap_or(rng.gen_range(0..obscura.images.len()));
            previously_spawned_obscura_asset_indices.insert(ret);
            ret
        };
        spawn_single_background_obscura(&mut commands, &obscura, obscura_asset_index, window, &images, &mut rng, true);
    }

    // Flora decorations are spawned by their own dedicated system.
}

/// Resource keeping track of the next spawn delay.
#[derive(Resource)]
pub struct BackgroundObscuraSpawner {
    pub timer: Timer,
}

impl Default for BackgroundObscuraSpawner {
    fn default() -> Self {
        Self {
            // The timer will be randomised each time we spawn a fish, but we
            // initialise with 2 seconds so we don't spawn immediately.
            timer: Timer::from_seconds(2.0, bevy::time::TimerMode::Once),
        }
    }
}

/// Helper that actually creates a single background fish entity with randomised
/// attributes.
fn spawn_single_background_obscura(
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

    // Determine velocity based on dir first; start position handled afterwards.
    let velocity: Vec2 = match dir {
        0 => Vec2::new(-speed, 0.0), // leftwards
        1 => Vec2::new(speed, 0.0),  // rightwards
        2 => Vec2::new(0.0, speed),  // upwards
        _ => Vec2::new(0.0, -speed), // downwards
    };

    // Calculate start position and baseline depending on whether we spawn inside.
    let (start_pos, base_perp) = if spawn_inside {
        let half_w = window.width() / 2.0;
        let half_h = window.height() / 2.0;
        let x = rng.gen_range(-half_w..half_w);
        let y = rng.gen_range(-half_h..half_h);

        let base = if velocity.x.abs() > 0.0 { y } else { x };
        (Vec3::new(x, y, -1.0), base)
    } else {
        match dir {
            // From right edge moving leftwards
            0 => {
                let x = window.width() / 2.0 + 100.0;
                let y = rng.gen_range(-window.height() / 2.0..window.height() / 2.0);
                (Vec3::new(x, y, -1.0), y)
            }
            // From left edge moving rightwards
            1 => {
                let x = -window.width() / 2.0 - 100.0;
                let y = rng.gen_range(-window.height() / 2.0..window.height() / 2.0);
                (Vec3::new(x, y, -1.0), y)
            }
            // From bottom edge moving upwards
            2 => {
                let y = -window.height() / 2.0 - 100.0;
                let x = rng.gen_range(-window.width() / 2.0..window.width() / 2.0);
                (Vec3::new(x, y, -1.0), x)
            }
            // From top edge moving downwards
            _ => {
                let y = window.height() / 2.0 + 100.0;
                let x = rng.gen_range(-window.width() / 2.0..window.width() / 2.0);
                (Vec3::new(x, y, -1.0), x)
            }
        }
    };

    // --- NEW SIZE CONSTRAINT LOGIC ---
    // Compute uniform scale so that the sprite fits within max(96x96, 1/20th the screen), preserving aspect ratio.
    let (img_w, img_h) = images
        .get(&handle)
        .map(|img| {
            let w = img.texture_descriptor.size.width as f32;
            let h = img.texture_descriptor.size.height as f32;
            (w, h)
        })
        .unwrap_or((200.0, 200.0));

    let max_screen_w = window.width() / 20.0;
    let max_screen_h = window.height() / 20.0;
    let max_w = max_screen_w.max(96.0);
    let max_h = max_screen_h.max(96.0);
    let scale = {
        let scale_w = max_w / img_w;
        let scale_h = max_h / img_h;
        let uniform = scale_w.min(scale_h);
        Vec3::splat(uniform)
    };

    commands.spawn((
        Sprite::from_image(handle),
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

/// Move background fish every frame, applying slow drift + sinusoidal wiggle.
pub fn background_obscura_movement_system(
    mut query: Query<(&mut Transform, &BackgroundObscura)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let elapsed = time.elapsed_secs();

    for (mut transform, obscura) in &mut query {
        // Linear drift
        transform.translation.x += obscura.velocity.x * dt;
        transform.translation.y += obscura.velocity.y * dt;

        // Sinusoidal wiggle perpendicular to drift direction.
        if obscura.velocity.x.abs() > 0.0 {
            // Horizontal drift → wiggle vertically.
            transform.translation.y = obscura.base_perp
                + obscura.wiggle_amp * (elapsed * obscura.wiggle_speed + obscura.phase).sin();
        } else {
            // Vertical drift → wiggle horizontally.
            transform.translation.x = obscura.base_perp
                + obscura.wiggle_amp * (elapsed * obscura.wiggle_speed + obscura.phase).sin();
        }
    }
}

/// Despawn background fish that are fully off-screen to the left.
///
/// When a fish despawns we schedule the spawner timer so a replacement will be
/// spawned after a random delay (1-3 seconds).
pub fn background_obscura_despawn_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform), With<BackgroundObscura>>, // need transform for position
    windows: Query<&Window>,
    mut spawner: ResMut<BackgroundObscuraSpawner>,
) {
    let window = windows.single().expect("No primary window");

    let mut rng = rand::thread_rng();

    let half_w = window.width() / 2.0;
    let half_h = window.height() / 2.0;

    for (entity, transform) in &mut query {
        // Despawn once the sprite is well outside any screen edge (+buffer).
        if transform.translation.x < -half_w - 120.0
            || transform.translation.x > half_w + 120.0
            || transform.translation.y < -half_h - 120.0
            || transform.translation.y > half_h + 120.0
        {
            commands.entity(entity).despawn();

            // Randomise next spawn between 1-3 s.
            let next = rng.gen_range(1.0..3.0);
            spawner
                .timer
                .set_duration(std::time::Duration::from_secs_f32(next));
            spawner.timer.reset();
        }
    }
}

/// Periodically spawn new background fish as governed by `BackgroundObscuraSpawner`.
pub fn background_obscura_spawn_system(
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

    // Maintain at most 8 fish.
    if spawner.timer.finished() && existing.iter().count() < 8 {
        let mut rng = rand::thread_rng();
        spawn_single_background_obscura(&mut commands, &obscura, rng.gen_range(0..obscura.images.len()), window, &images, &mut rng, false);

        // Set next spawn delay now so we don't spawn in the very next frame.
        let next = rng.gen_range(1.0..3.0);
        spawner
            .timer
            .set_duration(std::time::Duration::from_secs_f32(next));
        spawner.timer.reset();
    }
}

/// Clean up all background fish and associated resources when leaving the main menu.
pub fn cleanup_background_obscura(
    mut commands: Commands,
    fish_query: Query<Entity, With<BackgroundObscura>>,
) {
    for e in &fish_query {
        commands.entity(e).despawn();
    }

    // Also remove the spawner resource so the timer resets next time we open the menu.
    commands.remove_resource::<BackgroundObscuraSpawner>();
}

/// Spawn a batch of static flora decorations along the sea floor.
pub fn spawn_background_flora_initial(
    mut commands: Commands,
    flora_opt: Option<Res<FloraAssets>>, 
    windows: Query<&Window>,
    images: Res<Assets<Image>>,
    existing: Query<Entity, With<BackgroundFlora>>, // ensure we spawn only once per menu visit
) {
    // If we already have flora entities, skip (prevents duplicates when this system is in Update).
    if !existing.is_empty() {
        trace!("spawn_background_flora_initial: flora entities already present – skipping");
        return;
    }

    // ---------------------------------------------------------------------
    // Extensive logging to help debug why flora may not be visible.
    // ---------------------------------------------------------------------

    if flora_opt.is_none() {
        info!(
            "spawn_background_flora_initial: FloraAssets resource not yet loaded – skipping spawn."
        );
        return;
    }

    let flora = flora_opt.unwrap();

    if flora.images.is_empty() {
        info!(
            "spawn_background_flora_initial: FloraAssets loaded but contains 0 images – skipping spawn."
        );
        return;
    }

    // Ensure **all** flora textures are fully loaded before spawning so that
    // we can calculate a correct scale from the real image dimensions. If we
    // spawn before assets finish loading Bevy returns `None` from `images.get`.
    // This triggered the bug where sprites were massively over-scaled on the
    // very first visit to a newly-selected theme. We therefore delay the spawn
    // until every handle resolves successfully; the system runs every frame so
    // the decorations will appear automatically as soon as the textures are
    // ready.
    if flora
        .images
        .iter()
        .any(|h| images.get(h).is_none())
    {
        trace!("spawn_background_flora_initial: waiting for all flora textures to load");
        return;
    }

    let window = windows.single().expect("No primary window");
    let mut rng = rand::thread_rng();

    // Spawn slightly fewer decorations than before so the sea floor looks cleaner.
    let count = rng.gen_range(6..=14);
    let half_w = window.width() / 2.0;

    info!(
        "spawn_background_flora_initial: window = {:.1}×{:.1} px, spawning {} flora decorations ({} available images).",
        window.width(),
        window.height(),
        count,
        flora.images.len()
    );

    // Track x-positions we have already placed flora at so we can enforce a
    // minimum horizontal separation between decorations. This avoids the
    // situation where multiple sprites overlap heavily or sit directly in
    // front of one another.
    let mut placed_x: Vec<f32> = Vec::with_capacity(count as usize);
    // Track which flora indices have already been picked for this batch
    let mut picked_indices = HashSet::with_capacity(count as usize);

    for i in 0..count {
        // Try up to 20 times to pick a unique flora index
        let img_idx = {
            let mut idx = None;
            for _ in 0..20 {
                let candidate = rng.gen_range(0..flora.images.len());
                if !picked_indices.contains(&candidate) {
                    idx = Some(candidate);
                    break;
                }
            }
            // If we failed to find a unique one, just pick any (with replacement)
            idx.unwrap_or_else(|| rng.gen_range(0..flora.images.len()))
        };
        picked_indices.insert(img_idx);
        let handle = flora.images[img_idx].clone();

        // Scale the sprite so that its height does not exceed 1/4 of the current
        // window height. This guarantees flora never towers over gameplay UI
        // on very small or large screens while still preserving the original
        // pixel aspect ratio.

        let img_height_px = images
            .get(&handle)
            .map(|img| img.texture_descriptor.size.height as f32)
            .unwrap_or(200.0); // reasonable fallback until asset loads

        // Desired maximum height in world units (pixels) for this flora sprite.
        let desired_height = window.height() / 4.0;

        // Uniform scale factor so that `img_height_px * scale == desired_height`.
        // If the source image is smaller than 1/4 of the screen we may scale
        // it *up* so it still reaches the desired visual weight.
        let base_scale = desired_height / img_height_px;

        // Minimum horizontal distance to any previously placed decoration.
        let min_spacing = 140.0 * base_scale; // increased spacing to reduce overlap.

        // Pick an x coordinate that respects the spacing constraint. To avoid
        // potential infinite loops we cap the number of retry attempts.
        let mut x;
        let mut attempts = 0u8;
        loop {
            x = rng.gen_range(-half_w..half_w);
            if placed_x.iter().all(|px| (x - *px).abs() > min_spacing) || attempts >= 10 {
                break;
            }
            attempts += 1;
        }
        placed_x.push(x);

        // -----------------------------------------------------------------
        // Vertical placement: make sure the *bottom* edge of the sprite is a
        // little below the bottom of the screen (so it can appear anchored on
        // the sea floor) rather than positioning by the sprite's centre.
        // -----------------------------------------------------------------

        // Random downward offset (2‒10 px) from the screen bottom.
        let offset = rng.gen_range(5.0..=15.0);

        // Determine the original (un-scaled) image height in pixels. If the
        // asset hasn't fully loaded yet we fall back to 0 which will still
        // place the sprite roughly at the bottom without crashing.
        let img_height = images
            .get(&handle)
            .map(|img| img.texture_descriptor.size.height as f32)
            .unwrap_or(0.0);

        // Adjust for the scale we are going to apply to the sprite so that the
        // calculation is done in world units (1 px = 1 world unit in Bevy's 2D
        // camera by default).
        let scaled_height = img_height * base_scale;

        // Center y coordinate such that the bottom of the sprite is `offset`
        // pixels below the bottom of the screen.
        let y = -window.height() / 2.0 - offset + scaled_height / 2.0;

        let pulse_amp = rng.gen_range(0.03..0.07);
        let pulse_speed = rng.gen_range(0.3..0.8);
        let phase = rng.gen_range(0.0..std::f32::consts::TAU);
        let wiggle_amp = rng.gen_range(2.0f32.to_radians()..5.0f32.to_radians());

        debug!(
            "Flora #{}/{} → pos=({:.1},{:.1}), base_scale={:.3}, img_idx={} (handle={:?})",
            i + 1,
            count,
            x,
            y,
            base_scale,
            img_idx,
            handle
        );

        commands.spawn((
            Sprite::from_image(handle),
            Transform::from_xyz(x, y, -2.0).with_scale(Vec3::splat(base_scale)),
            BackgroundFlora {
                base_scale,
                pulse_amp,
                pulse_speed,
                phase,
                wiggle_amp,
            },
        ));
    }
}

/// Animate flora pulsation and slight rotation.
pub fn background_flora_animation_system(
    mut query: Query<(&mut Transform, &BackgroundFlora)>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();
    let mut processed = 0u32;
    for (mut transform, flora) in &mut query {
        let sin = (t * flora.pulse_speed + flora.phase).sin();
        transform.scale = Vec3::splat(flora.base_scale * (1.0 + flora.pulse_amp * sin));
        transform.rotation = Quat::from_rotation_z(sin * flora.wiggle_amp);

        if processed == 0 {
            debug!(
                "background_flora_animation_system: example entity pos=({:.1},{:.1}) scale={:?}",
                transform.translation.x,
                transform.translation.y,
                transform.scale
            );
        }
        processed += 1;
    }

    if processed == 0 {
        debug!("background_flora_animation_system: no flora entities present this frame");
    }
}

/// Clean up flora sprites when leaving the main menu.
pub fn cleanup_background_flora(
    mut commands: Commands,
    flora_query: Query<Entity, With<BackgroundFlora>>, 
) {
    for e in &flora_query { commands.entity(e).despawn(); }
}

// -----------------------------------------------------------------------------
// Background gradient for main menu
// -----------------------------------------------------------------------------

/// Marker component for the procedurally generated gradient background.
#[derive(Component)]
pub struct GradientBackground;

/// Spawn a full-screen vertical gradient (dark deep water at the bottom → bright surface at the top).
pub fn spawn_gradient_background(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>, 
    windows: Query<&Window>,
    existing: Query<Entity, With<GradientBackground>>, // existing gradient layers
) {
    // Remove any pre-existing gradient background (both backdrop and overlay) to
    // prevent multiple transparent overlays stacking up when this system runs
    // again (e.g. after pausing/unpausing or state transitions).
    for entity in &existing {
        commands.entity(entity).despawn();
    }

    let window = windows.single().expect("No primary window");

    // Texture dimensions (1×N to minimise memory, scaled to window size).
    const HEIGHT: usize = 512; // adequate vertical resolution for smoothness
    const WIDTH: usize = 1;

    // Bottom (deep) and top (surface) colours expressed in sRGB 0-1 range.
    let bottom = [0.0, 0.15, 0.20]; // deep cold water (#002633)
    let top = [0.20, 0.65, 0.90];   // sunlit surface blue

    let mut data = Vec::with_capacity(WIDTH * HEIGHT * 4);
    // NOTE: In image data, the very first row ends up at the top of the sprite. We therefore
    // invert the interpolation so that the bright ``top`` colour is written first and the
    // dark ``bottom`` colour appears last. This yields a light surface and dark depths.
    for y in 0..HEIGHT {
        let t = 1.0 - y as f32 / (HEIGHT - 1) as f32; // 0 = top, 1 = bottom
        let r = bottom[0] * (1.0 - t) + top[0] * t;
        let g = bottom[1] * (1.0 - t) + top[1] * t;
        let b = bottom[2] * (1.0 - t) + top[2] * t;
        data.extend_from_slice(&[
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8,
            255,
        ]);
    }

    let size = Extent3d {
        width: WIDTH as u32,
        height: HEIGHT as u32,
        depth_or_array_layers: 1,
    };
    let image = Image::new_fill(size, TextureDimension::D2, &data, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default());
    let handle = images.add(image);

    // Create a sprite component from the generated texture handle.
    let mut sprite = Sprite::from_image(handle.clone());
    sprite.custom_size = Some(Vec2::new(window.width(), window.height()));

    commands.spawn((
        sprite,
        Transform::from_xyz(0.0, 0.0, -5.0), // behind fish & UI
        GradientBackground,
    ));

    // -----------------------------------------------------------------
    // Foreground overlay gradient (transparent top → darker bottom).
    // Adds subtle shading so the sea floor appears darker and the surface
    // area lighter without additional lighting complexity.
    // -----------------------------------------------------------------

    // Re-use the same texture dimensions for the overlay.
    let mut overlay_data = Vec::with_capacity(WIDTH * HEIGHT * 4);
    for y in 0..HEIGHT {
        let t = y as f32 / (HEIGHT - 1) as f32; // 0 = top, 1 = bottom
        // Alpha ramps from 0 (fully transparent at the top) to ~0.7 at bottom.
        let alpha = (t * t * t * 204.0) as u8; // 0‒204 ≈ 0%‒80% opacity
        overlay_data.extend_from_slice(&[0, 15, 30, alpha]);
    }

    let overlay_size = Extent3d {
        width: WIDTH as u32,
        height: HEIGHT as u32,
        depth_or_array_layers: 1,
    };
    let overlay_image = Image::new_fill(
        overlay_size,
        TextureDimension::D2,
        &overlay_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    let overlay_handle = images.add(overlay_image);

    let mut overlay_sprite = Sprite::from_image(overlay_handle.clone());
    overlay_sprite.custom_size = Some(Vec2::new(window.width(), window.height()));

    commands.spawn((
        overlay_sprite,
        // Positive z puts the overlay *in front* of fish & background but *behind* UI.
        Transform::from_xyz(0.0, 0.0, 2.0),
        GradientBackground,
    ));
}

/// Despawn the gradient background when leaving the main menu.
pub fn cleanup_gradient_background(
    mut commands: Commands,
    query: Query<Entity, With<GradientBackground>>, 
) {
    for e in &query {
        commands.entity(e).despawn();
    }
}

// -----------------------------------------------------------------------------
// Loading splash (PreMainMenu) --------------------------------------------------
// -----------------------------------------------------------------------------

/// Marker for the root UI node of the loading splash.
#[derive(Component)]
pub struct LoadingSplashUI;

/// Component for the animated loading text ("Loading", "Loading.", ...).
#[derive(Component)]
pub struct LoadingText {
    timer: Timer,
    phase: u8, // 0-3
}

/// Spawn a centered "Loading…" text while we wait for assets.
pub fn spawn_loading_splash_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    use bevy::ui::*;

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            LoadingSplashUI,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("Loading"),
                TextFont {
                    font: asset_server.load("fonts/Fredoka-Bold.ttf"),
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                TextLayout::new_with_justify(JustifyText::Center),
                LoadingText {
                    timer: Timer::from_seconds(0.5, bevy::time::TimerMode::Repeating),
                    phase: 0,
                },
            ));
        });
}

/// Animate the dots after the "Loading" label (cycles 0‒3 dots).
pub fn loading_text_animation_system(
    mut query: Query<(&mut Text, &mut LoadingText)>,
    time: Res<Time>,
) {
    for (mut text, mut loader) in &mut query {
        loader.timer.tick(time.delta());
        if loader.timer.just_finished() {
            loader.phase = (loader.phase + 1) % 4;
            let dots: String = ".".repeat(loader.phase as usize);
            // In Bevy 0.16 `Text` dereferences to its underlying `String`, so we can
            // update the contents directly rather than editing text sections.
            *text = Text::new(format!("Loading{}", dots));
        }
    }
}

/// Remove loading UI when leaving PreMainMenu.
pub fn cleanup_loading_splash(mut commands: Commands, query: Query<Entity, With<LoadingSplashUI>>) {
    for e in &query {
        commands.entity(e).despawn_recursive();
    }
}

// -----------------------------------------------------------------------------
// Helper run conditions ---------------------------------------------------------
// -----------------------------------------------------------------------------

/// Run condition: true when at least one `FloraAssets` resource exists and has images.
pub fn flora_assets_ready(flora_opt: Option<Res<FloraAssets>>) -> bool {
    flora_opt.as_ref().map_or(false, |f| !f.images.is_empty())
}

// -----------------------------------------------------------------------------
// Main-menu loading indicator (shown after selecting a new theme) -------------
// -----------------------------------------------------------------------------

/// Helper that returns `true` when **all** assets required for the main menu
/// have finished loading for the currently-selected theme.
fn theme_assets_ready(
    asset_server: &AssetServer,
    obscura: &ObscuraAssets,
    flora: &FloraAssets,
    playable: &PlayableFishTextures,
    enemy: &EnemyFishAssets,
) -> bool {
    let obscura_ready = obscura.images.iter().all(|h| asset_server.is_loaded(h));
    let flora_ready = flora.images.iter().all(|h| asset_server.is_loaded(h));

    let playable_ready = asset_server.is_loaded(&playable.fish1)
        && asset_server.is_loaded(&playable.fish2)
        && asset_server.is_loaded(&playable.fish3);

    let enemy_ready = enemy.images.iter().all(|h| asset_server.is_loaded(h));

    obscura_ready && flora_ready && playable_ready && enemy_ready
}

/// Spawn a small "Loading…" text centred near the bottom of the screen while
/// the newly-selected theme assets stream in on the first visit to the main
/// menu after changing theme.
pub fn spawn_menu_loading_indicator(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    obscura: Res<ObscuraAssets>,
    flora: Res<FloraAssets>,
    playable: Res<PlayableFishTextures>,
    enemy: Res<EnemyFishAssets>,
) {
    // If everything is already loaded there's no need to show the indicator.
    if theme_assets_ready(&asset_server, &obscura, &flora, &playable, &enemy) {
        return;
    }

    // Full-screen root node so we can place the child text at the bottom-centre
    // using flexbox alignment.
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                // Push children to the bottom and centre horizontally.
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::Center,
                ..default()
            },
            MainMenuLoadingUI,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("Loading"),
                TextFont {
                    font: asset_server.load("fonts/Fredoka-Bold.ttf"),
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                TextLayout::new_with_justify(JustifyText::Center),
                // Re-use the same dot-cycling component & animation system.
                LoadingText {
                    timer: Timer::from_seconds(0.5, bevy::time::TimerMode::Repeating),
                    phase: 0,
                },
            ));
        });
}

/// Periodically checks if all theme assets are ready and, if so, removes the
/// menu loading indicator UI.
pub fn menu_loading_indicator_check_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    obscura: Res<ObscuraAssets>,
    flora: Res<FloraAssets>,
    playable: Res<PlayableFishTextures>,
    enemy: Res<EnemyFishAssets>,
    query: Query<Entity, With<MainMenuLoadingUI>>,
) {
    if query.is_empty() {
        return;
    }

    if theme_assets_ready(&asset_server, &obscura, &flora, &playable, &enemy) {
        for e in &query {
            commands.entity(e).despawn_recursive();
        }
    }
}

/// Plays the deferred theme-select sound effect once *all* theme assets have
/// finished loading (so audio playback isn't interrupted by large image
/// streaming) and then removes the pending resource.
pub fn play_pending_theme_select_sfx_system(
    mut commands: Commands,
    pending: Option<Res<PendingThemeSelectSfx>>, // present only immediately after click
    asset_server: Res<AssetServer>,
    obscura_opt: Option<Res<ObscuraAssets>>,
    flora_opt: Option<Res<FloraAssets>>,
    playable_opt: Option<Res<PlayableFishTextures>>,
    enemy_opt: Option<Res<EnemyFishAssets>>,
) {
    let Some(pending) = pending else { return; };
    let (Some(obscura), Some(flora), Some(playable), Some(enemy)) =
        (obscura_opt, flora_opt, playable_opt, enemy_opt)
    else {
        // Required resources not present yet – keep waiting.
        return;
    };

    // Re-use the helper defined below to ensure all assets are ready.
    if theme_assets_ready(&asset_server, &obscura, &flora, &playable, &enemy)
        && asset_server.is_loaded(&pending.handle)
    {
        // Safe to play the SFX now that everything else has streamed in.
        commands.spawn((
            AudioPlayer::new(pending.handle.clone()),
            PlaybackSettings::DESPAWN,
        ));

        // Remove the marker so we don't play it again.
        commands.remove_resource::<PendingThemeSelectSfx>();
    }
}
