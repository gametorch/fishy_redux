use bevy::ecs::schedule::common_conditions::resource_exists;
use bevy::prelude::*;
use bevy_light_2d::prelude::*;

mod assets;
mod components;
mod constants;
mod states;
mod systems;
mod alpha_masks;
mod theme;

use assets::*;
use states::*;
use systems::*;
use alpha_masks::AlphaMasks;
use theme::Theme;

fn main() {
    App::new()
        // Deep-water navy background
        .insert_resource(ClearColor(constants::CLEAR_COLOR))
        .insert_resource(SelectedFish::default())
        .insert_resource(AlphaMasks::default())
        .insert_resource(Theme::Crayon)
        .add_plugins(DefaultPlugins.set(bevy::window::WindowPlugin {
            primary_window: Some(bevy::window::Window {
                title: "Fishy Redux!".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(Light2dPlugin)
        // Global game state machines
        .init_state::<GameState>()
        .init_state::<InGameState>()
        // Asset loading (runs once at PreStartup to ensure resources exist before state transitions)
        .add_systems(
            bevy::app::PreStartup,
            (
                load_playable_fish_textures,
                load_enemy_fish_assets,
                load_obscura_assets,
                load_flora_assets,
                load_theme_select_audio_assets,
                // Preload *all* assets for every theme upfront.
                preload_all_assets,
            ),
        )
        // Camera exists for all states
        .add_systems(Startup, setup_camera)
        // ---------------- Loading splash ----------------
        .add_systems(OnEnter(GameState::PreMainMenu), spawn_loading_splash_ui)
        .add_systems(
            Update,
            (
                loading_text_animation_system,
                // Transition once all required assets are loaded.
                check_assets_ready_system,
            )
                .run_if(in_state(GameState::PreMainMenu)),
        )
        .add_systems(OnExit(GameState::PreMainMenu), cleanup_loading_splash)
        // ---------------- Theme picker ----------------
        .add_systems(
            OnEnter(GameState::ThemePicker),
            (
                spawn_gradient_background,
                spawn_background_obscura_initial,
                spawn_background_flora_initial,
                setup_theme_picker_ui,
            ),
        )
        .add_systems(
            Update,
            (
                crayon_button_system,
                chibi_button_system,
                retro_button_system,
                background_obscura_movement_system,
                background_obscura_despawn_system,
                background_obscura_spawn_system,
                background_flora_animation_system,
            )
                .run_if(in_state(GameState::ThemePicker)),
        )
        .add_systems(
            OnExit(GameState::ThemePicker),
            (
                cleanup_theme_picker,
                cleanup_gradient_background,
                cleanup_background_obscura,
                cleanup_background_flora,
                // Reload game assets for the newly selected theme.
                load_playable_fish_textures,
                load_enemy_fish_assets,
                load_obscura_assets,
                load_flora_assets,
            ),
        )
        // ---------------- Main menu ----------------
        .add_systems(
            OnEnter(GameState::MainMenu),
            (
                spawn_gradient_background,
                spawn_background_obscura_initial,
                setup_menu_ui,
                // Bottom-of-screen loading indicator while theme assets stream in
                spawn_menu_loading_indicator,
            ),
        )
        // Button interaction systems only run while we're in the menu
        .add_systems(
            Update,
            (
                quit_button_system,
                play_button_system,
                theme_picker_button_system,
                play_pending_theme_select_sfx_system,
                // background fish systems run while in main menu
                background_obscura_movement_system,
                background_obscura_despawn_system,
                background_obscura_spawn_system,
                background_flora_animation_system,
                // Animate the dots in the loading indicator text
                loading_text_animation_system,
                // Despawn loading indicator once all theme assets are ready
                menu_loading_indicator_check_system,
                // Spawn flora once when assets are ready and none exist yet.
                spawn_background_flora_initial
                    .run_if(resource_exists::<FloraAssets>)
                    .run_if(in_state(GameState::MainMenu)),
            )
                .run_if(in_state(GameState::MainMenu)),
        )
        // Cleanup menu when leaving state
        .add_systems(
            OnExit(GameState::MainMenu),
            (
                cleanup_menu,
                cleanup_gradient_background,
                cleanup_background_obscura,
                cleanup_background_flora,
            ),
        )
        // --------------- Fish selection ----------------
        // Spawn gradient background first, then the fish picker UI when entering InGame
        .add_systems(
            OnEnter(GameState::InGame),
            (spawn_gradient_background, setup_fish_picker_ui),
        )
        .add_systems(
            OnExit(InGameState::FishPicker),
            (
                cleanup_fish_picker,
                cleanup_background_flora,
                cleanup_gradient_background,
            ),
        )
        .add_systems(OnExit(GameState::InGame), cleanup_gradient_background)
        .add_systems(
            Update,
            (
                fish_option1_system,
                fish_option2_system,
                fish_option3_system,
                esc_to_main_menu_from_picker_system,
                // Animate flora decorations while in the picker menu
                background_flora_animation_system,
                // Spawn flora once when assets are ready and none exist yet.
                spawn_background_flora_initial.run_if(resource_exists::<FloraAssets>),
            )
                .run_if(in_state(GameState::InGame))
                .run_if(in_state(InGameState::FishPicker)),
        )
        // --------------- Gameplay ----------------
        .add_systems(
            OnEnter(InGameState::Playing),
            (
                spawn_gradient_background,
                spawn_player_fish_sprite,
                spawn_background_flora_initial,
                spawn_background_obscura_initial_ingame,
                setup_moving_fish_spawner,
                spawn_meat_score_ui,
            ),
        )
        // Clean up player sprite when leaving overall InGame state (e.g., back to main menu)
        .add_systems(OnExit(GameState::InGame), cleanup_player_fish)
        .add_systems(
            OnExit(GameState::InGame),
            (
                cleanup_background_flora,
                cleanup_background_obscura,
                cleanup_moving_fish,
                cleanup_fish_picker,
                cleanup_meat_score_ui,
                // Ensure any lingering Game Over overlay is removed when we leave gameplay
                cleanup_game_over,
            ),
        )
        // Keep player fish scale in sync with its 'Meat'
        .add_systems(
            Update,
            update_player_fish_scale
                .run_if(in_state(GameState::InGame))
                .run_if(in_state(InGameState::Playing)),
        )
        .add_systems(
            Update,
            (
                player_fish_acceleration_system,
                player_fish_movement_system,
                player_fish_orientation_system,
                update_angler_light_position,
                animate_angler_light_system,
            )
                .run_if(in_state(GameState::InGame))
                .run_if(in_state(InGameState::Playing)),
        )
        // -------- In-game and pause handling --------
        .add_systems(OnEnter(InGameState::PauseMenu), setup_pause_menu_ui)
        .add_systems(OnExit(InGameState::PauseMenu), cleanup_pause_menu)
        // Game over overlay
        .add_systems(OnEnter(InGameState::GameOver), setup_game_over_ui)
        .add_systems(OnExit(InGameState::GameOver), cleanup_game_over)
        // Input handling
        .add_systems(
            Update,
            (
                esc_to_pause_system
                    .run_if(in_state(GameState::InGame))
                    .run_if(in_state(InGameState::Playing)),
                esc_to_resume_system
                    .run_if(in_state(GameState::InGame))
                    .run_if(in_state(InGameState::PauseMenu)),
                continue_button_system.run_if(in_state(InGameState::PauseMenu)),
                pause_main_menu_button_system.run_if(in_state(InGameState::PauseMenu)),
                pause_quit_button_system.run_if(in_state(InGameState::PauseMenu)),
                game_over_main_menu_button_system.run_if(in_state(InGameState::GameOver)),
            ),
        )
        .add_systems(
            Update,
            (
                background_obscura_movement_system,
                background_obscura_despawn_system,
                background_obscura_spawn_system_ingame,
                background_flora_animation_system,
                moving_fish_spawn_system,
                moving_fish_movement_system,
                collision_detection_system,
                update_meat_score_ui,
            )
                .run_if(in_state(GameState::InGame))
                .run_if(in_state(InGameState::Playing)),
        )
        .add_systems(
            OnExit(GameState::InGame),
            reset_in_game_state_system
        )
        .run();
}

/// 2-D camera used for both UI and gameplay.
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

// ------------------------------------------------------------
// System that checks whether all required main-menu assets have
// finished loading and transitions to the real main menu.
// ------------------------------------------------------------

fn check_assets_ready_system(
    asset_server: Res<AssetServer>,
    obscura: Res<assets::ObscuraAssets>,
    flora: Res<assets::FloraAssets>,
    playable: Res<assets::PlayableFishTextures>,
    enemy: Res<assets::EnemyFishAssets>,
    audio: Res<assets::ThemeSelectAudio>,
    prefetched: Res<assets::PrefetchedAssets>,
    mut next_state: ResMut<NextState<states::GameState>>,
) {
    use states::GameState;
    use bevy::asset::LoadState;

    let obscura_ready = obscura.images.iter().all(|h| asset_server.is_loaded(h));
    let flora_ready = flora.images.iter().all(|h| asset_server.is_loaded(h));

    let playable_ready = asset_server.is_loaded(&playable.fish1)
        && asset_server.is_loaded(&playable.fish2)
        && asset_server.is_loaded(&playable.fish3);

    let enemy_ready = enemy.images.iter().all(|h| asset_server.is_loaded(h));

    let audio_ready = asset_server.is_loaded(&audio.crayon)
        && asset_server.is_loaded(&audio.chibi)
        && asset_server.is_loaded(&audio.retro);

    // Ensure every prefetched asset reached the Loaded state.
    let prefetched_ready = prefetched.handles.iter().all(|h| {
        matches!(asset_server.get_load_state(h.id()), Some(LoadState::Loaded))
    });

    if obscura_ready
        && flora_ready
        && playable_ready
        && enemy_ready
        && audio_ready
        && prefetched_ready
    {
        info!("All assets loaded – switching to ThemePicker state");
        next_state.set(GameState::ThemePicker);
    }
}

// ---------------------------------------------------------------------
// Helper systems
// ---------------------------------------------------------------------

/// Reset the nested InGameState back to its default (FishPicker) whenever we
/// leave the overall `GameState::InGame`. This ensures that a fresh fish
/// selection screen is shown the next time the player starts a new game after
/// returning to the main menu or quitting to desktop.
fn reset_in_game_state_system(mut next_state: ResMut<NextState<InGameState>>) {
    next_state.set(InGameState::FishPicker);
}
