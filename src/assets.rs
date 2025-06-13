use bevy::prelude::*;
use bevy::audio::AudioSource;
use crate::theme::Theme;
use bevy::asset::UntypedHandle;

// -----------------------------------------------------------------------------
//  Playable fish textures (the three options shown in the picker)
// -----------------------------------------------------------------------------

/// Collection of texture handles for the *player-controllable* fish options.
#[derive(Resource)]
pub struct PlayableFishTextures {
    pub fish1: Handle<Image>,
    pub fish2: Handle<Image>,
    pub fish3: Handle<Image>,
}

/// Load all playable fish PNGs and store them in a [`PlayableFishTextures`] resource.
#[allow(clippy::needless_pass_by_value)]
pub fn load_playable_fish_textures(mut commands: Commands, asset_server: Res<AssetServer>, theme: Res<Theme>) {
    let textures = PlayableFishTextures {
        fish1: asset_server.load(theme.path("playable_fish/fish_1.png")),
        fish2: asset_server.load(theme.path("playable_fish/fish_2.png")),
        fish3: asset_server.load(theme.path("playable_fish/fish_3.png")),
    };

    commands.insert_resource(textures);
}

// -----------------------------------------------------------------------------
//  Enemy fish textures (ambient moving fish during gameplay)
// -----------------------------------------------------------------------------

/// Collection of textures found under `images/fish/*.png`.
#[derive(Resource, Default, Clone)]
pub struct EnemyFishAssets {
    pub images: Vec<Handle<Image>>, // arbitrary number of PNGs
}

/// Load all PNGs in `assets/images/fish` into an [`EnemyFishAssets`] resource (PreStartup).
pub fn load_enemy_fish_assets(mut commands: Commands, asset_server: Res<AssetServer>, theme: Res<Theme>) {
    #[cfg(target_arch = "wasm32")]
    {
        // Embedded asset list generated in build.rs for wasm.
        // include!(concat!(env!("OUT_DIR"), "/enemy_fish_asset_list.rs"));
        let prefix = format!("images/{}/", theme.prefix());
        let handles = ENEMY_FISH_ASSET_PATHS
            .iter()
            .filter(|p| p.starts_with(&prefix))
            .map(|path| asset_server.load(*path))
            .collect();
        commands.insert_resource(EnemyFishAssets { images: handles });
        return;
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::ffi::OsStr;
        use std::fs;

        let mut handles = Vec::new();

        if let Ok(entries) = fs::read_dir(theme.assets_dir("fish")) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension() == Some(OsStr::new("png")) {
                    if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                        let rel_path = theme.path(&format!("fish/{file_name}"));
                        handles.push(asset_server.load(rel_path));
                    }
                }
            }
        }

        commands.insert_resource(EnemyFishAssets { images: handles });
    }
}

// -----------------------------------------------------------------------------
//  Obscura textures (background fish drifting through the main menu)
// -----------------------------------------------------------------------------

/// Collection of all textures found under `images/obscura/*.png`.
///
/// These are loaded once at application start so they are instantly available
/// when we need to spawn drifting background fish while the main menu is
/// visible.
#[derive(Resource, Default, Clone)]
pub struct ObscuraAssets {
    pub images: Vec<Handle<Image>>, // arbitrary number of PNGs
}

/// Load **all** PNG files located in `assets/images/obscura/` and store their
/// handles in an [`ObscuraAssets`] resource.
///
/// This runs during the `PreStartup` schedule so the assets are ready before
/// the first frame is rendered.
#[cfg(target_arch = "wasm32")]
include!(concat!(env!("OUT_DIR"), "/asset_list.rs"));

pub fn load_obscura_assets(mut commands: Commands, asset_server: Res<AssetServer>, theme: Res<Theme>) {
    #[cfg(target_arch = "wasm32")]
    {
        let prefix = format!("images/{}/", theme.prefix());
        let handles = OBSCURA_ASSET_PATHS
            .iter()
            .filter(|p| p.starts_with(&prefix))
            .map(|path| asset_server.load(*path))
            .collect();
        commands.insert_resource(ObscuraAssets { images: handles });
        return;
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::ffi::OsStr;
        use std::fs;

        let mut handles = Vec::new();

        if let Ok(entries) = fs::read_dir(theme.assets_dir("obscura")) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension() == Some(OsStr::new("png")) {
                    if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                        let rel_path = theme.path(&format!("obscura/{file_name}"));
                        handles.push(asset_server.load(rel_path));
                    }
                }
            }
        }

        commands.insert_resource(ObscuraAssets { images: handles });
    }
}

// -----------------------------------------------------------------------------
//  Flora textures (static background decorations along the sea floor)
// -----------------------------------------------------------------------------

/// Collection of textures located under `images/flora/*.png`.
#[derive(Resource, Default, Clone)]
pub struct FloraAssets {
    pub images: Vec<Handle<Image>>, // arbitrary number of PNGs
}

/// Load **all** PNG files located in `assets/images/flora/` and store their
/// handles in a [`FloraAssets`] resource. Runs during `PreStartup`.
pub fn load_flora_assets(mut commands: Commands, asset_server: Res<AssetServer>, theme: Res<Theme>) {
    #[cfg(target_arch = "wasm32")]
    {
        let prefix = format!("images/{}/", theme.prefix());
        let handles = FLORA_ASSET_PATHS
            .iter()
            .filter(|p| p.starts_with(&prefix))
            .map(|path| asset_server.load(*path))
            .collect();
        commands.insert_resource(FloraAssets { images: handles });
        return;
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::ffi::OsStr;
        use std::fs;

        let mut handles = Vec::new();

        if let Ok(entries) = fs::read_dir(theme.assets_dir("flora")) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension() == Some(OsStr::new("png")) {
                    if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                        let rel_path = theme.path(&format!("flora/{file_name}"));
                        handles.push(asset_server.load(rel_path));
                    }
                }
            }
        }

        commands.insert_resource(FloraAssets { images: handles });
    }
}

// Enemy fish asset list (generated in build.rs)
#[cfg(target_arch = "wasm32")]
include!(concat!(env!("OUT_DIR"), "/enemy_fish_asset_list.rs"));

/// Preloaded audio handles for the three theme-select sound effects.
#[derive(Resource)]
pub struct ThemeSelectAudio {
    pub crayon: Handle<AudioSource>,
    pub chibi: Handle<AudioSource>,
    pub retro: Handle<AudioSource>,
}

/// Load all theme-select audio files once at application start and store them
/// in a [`ThemeSelectAudio`] resource so they can be played instantly later.
#[allow(clippy::needless_pass_by_value)]
pub fn load_theme_select_audio_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let audio = ThemeSelectAudio {
        crayon: asset_server.load("sounds/crayon_sketch.mp3"),
        chibi: asset_server.load("sounds/cute_and_chibi.mp3"),
        retro: asset_server.load("sounds/retro_pixelated.mp3"),
    };

    commands.insert_resource(audio);
}

// -----------------------------------------------------------------------------
//  Global preloading for *all* assets across every theme
// -----------------------------------------------------------------------------

/// Collection of untyped asset handles returned from [`AssetServer::load_folder`]
/// when preloading every asset at application start.
#[derive(Resource, Default)]
pub struct PrefetchedAssets {
    pub handles: Vec<UntypedHandle>,
}

/// Preload **all** assets found under each theme directory as well as common
/// folders such as `sounds/` and `fonts/`. This drastically reduces stutter
/// later when the player switches themes because the assets are already
/// resident in memory.
///
/// The returned handles are stored in a [`PrefetchedAssets`] resource so we can
/// later check their [`LoadState`](bevy::asset::LoadState) inside the loading
/// splash screen.
#[allow(clippy::needless_pass_by_value)]
pub fn preload_all_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let mut handles: Vec<UntypedHandle> = Vec::new();

    // Preload every asset under each theme directory.
    for theme in [Theme::Crayon, Theme::Chibi, Theme::Retro] {
        let folder_handle = asset_server.load_folder(format!("images/{}", theme.prefix()));
        handles.push(folder_handle.untyped());
    }

    // Preload common non-themed assets.
    for folder in ["sounds", "fonts"] {
        let folder_handle = asset_server.load_folder(folder);
        handles.push(folder_handle.untyped());
    }

    commands.insert_resource(PrefetchedAssets { handles });
} 