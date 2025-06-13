use crate::{components::*, constants, states::GameState};
use bevy::prelude::*;
use crate::theme::Theme;
use bevy::audio::AudioSource;
use crate::assets::ThemeSelectAudio;
use bevy::audio::{AudioPlayer, PlaybackSettings};

/// Resource inserted when the player clicks a theme button to defer the
/// theme-select SFX until all heavy image assets finish loading.
#[derive(Resource)]
pub struct PendingThemeSelectSfx {
    pub handle: Handle<AudioSource>,
}

/// Helper function to spawn a themed image button used in the picker.
fn spawn_theme_image_button<C: Component + Copy + 'static>(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    image_path: &str,
    marker: C,
) {
    use bevy::ui::*;

    let handle = asset_server.load(image_path);

    parent
        .spawn((
            Button,
            // Fixed width with auto height to preserve aspect ratio.
            Node {
                width: Val::Px(200.0),
                height: Val::Auto,
                padding: UiRect::all(Val::Px(0.0)),
                border: UiRect::all(Val::Px(20.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ImageNode::new(handle),
            BackgroundColor(constants::THEME_PICKER_IDLE_COLOR.into()),
            BorderColor(Color::BLACK),
            BorderRadius::all(Val::Px(constants::BUTTON_RADIUS)),
            marker,
        ));
}

/// Build the theme picker UI with three vertically stacked buttons.
pub fn setup_theme_picker_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Full-screen root container, centred.
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
            ThemePickerUI,
        ))
        .with_children(|parent| {
            // Title above the theme buttons.
            parent.spawn((
                Text::new("Choose a Theme!"),
                TextFont {
                    font: asset_server.load("fonts/Fredoka-Bold.ttf"),
                    font_size: 54.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                TextLayout::new_with_justify(JustifyText::Center),
            ));

            // Replace text with representative fish icons for each theme.
            spawn_theme_image_button(
                parent,
                &asset_server,
                "images/crayon/playable_fish/fish_1.png",
                CrayonButton,
            );
            spawn_theme_image_button(
                parent,
                &asset_server,
                "images/chibi/playable_fish/fish_2.png",
                ChibiButton,
            );
            spawn_theme_image_button(
                parent,
                &asset_server,
                "images/retro/playable_fish/fish_2.png",
                RetroPixelButton,
            );
        });
}

/// Generic interaction handler for a theme button.
#[derive(Component)]
pub struct ThemeHoverSfx;

fn theme_button_interaction<B: Component>(
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<B>)>,
    mut commands: Commands,
    hover_audio_query: Query<Entity, With<ThemeHoverSfx>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut theme: ResMut<Theme>,
    selected_theme: Theme,
    audio_handle: Handle<AudioSource>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = constants::THEME_PICKER_PRESSED_COLOR.into();

                // Store chosen theme then transition to main menu.
                *theme = selected_theme;
                next_state.set(GameState::MainMenu);
            }
            Interaction::Hovered => {
                *color = constants::THEME_PICKER_HOVER_COLOR.into();

                // Avoid overlapping hover SFX – remove any currently playing one.
                for entity in &hover_audio_query {
                    commands.entity(entity).despawn();
                }

                // Spawn a new audio player for the hover SFX (auto-despawns when finished).
                commands.spawn((
                    AudioPlayer::new(audio_handle.clone()),
                    PlaybackSettings::DESPAWN,
                    ThemeHoverSfx,
                ));
            }
            Interaction::None => {
                *color = constants::THEME_PICKER_IDLE_COLOR.into();
            }
        }
    }
}

pub fn crayon_button_system(
    interaction_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<CrayonButton>)>,
    commands: Commands,
    hover_audio_query: Query<Entity, With<ThemeHoverSfx>>,
    next_state: ResMut<NextState<GameState>>,
    theme: ResMut<Theme>,
    audio: Res<ThemeSelectAudio>,
) {
    theme_button_interaction::<CrayonButton>(
        interaction_query,
        commands,
        hover_audio_query,
        next_state,
        theme,
        Theme::Crayon,
        audio.crayon.clone(),
    );
}

pub fn chibi_button_system(
    interaction_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<ChibiButton>)>,
    commands: Commands,
    hover_audio_query: Query<Entity, With<ThemeHoverSfx>>,
    next_state: ResMut<NextState<GameState>>,
    theme: ResMut<Theme>,
    audio: Res<ThemeSelectAudio>,
) {
    theme_button_interaction::<ChibiButton>(
        interaction_query,
        commands,
        hover_audio_query,
        next_state,
        theme,
        Theme::Chibi,
        audio.chibi.clone(),
    );
}

pub fn retro_button_system(
    interaction_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<RetroPixelButton>)>,
    commands: Commands,
    hover_audio_query: Query<Entity, With<ThemeHoverSfx>>,
    next_state: ResMut<NextState<GameState>>,
    theme: ResMut<Theme>,
    audio: Res<ThemeSelectAudio>,
) {
    theme_button_interaction::<RetroPixelButton>(
        interaction_query,
        commands,
        hover_audio_query,
        next_state,
        theme,
        Theme::Retro,
        audio.retro.clone(),
    );
}

/// Despawn all theme picker UI entities on state exit.
pub fn cleanup_theme_picker(mut commands: Commands, query: Query<Entity, With<ThemePickerUI>>) {
    for e in &query {
        commands.entity(e).despawn_recursive();
    }
} 