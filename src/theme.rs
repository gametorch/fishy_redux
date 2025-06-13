use bevy::prelude::*;

/// Available visual themes for the game assets.
///
/// The enum implements [`Resource`] so the currently selected theme can be stored
/// globally via [`insert_resource`](bevy::prelude::Commands::insert_resource).
#[derive(Resource, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum Theme {
    /// Hand-drawn crayon look.
    #[default]
    Crayon,
    /// Cute chibi style.
    Chibi,
    /// Retro pixel-art vibe.
    Retro,
}

impl Theme {
    /// Return the lowercase prefix that all files belonging to this theme are
    /// stored under in the `assets/images` directory.
    pub const fn prefix(&self) -> &'static str {
        match self {
            Theme::Crayon => "crayon",
            Theme::Chibi => "chibi",
            Theme::Retro => "retro",
        }
    }

    /// Helper that prefixes the supplied relative path with the theme directory.
    ///
    /// ```
    /// # use fishy_redux::theme::Theme;
    /// let t = Theme::Crayon;
    /// assert_eq!(t.path("playable_fish/fish_1.png"), "images/crayon/playable_fish/fish_1.png");
    /// ```
    pub fn path(&self, rel: &str) -> String {
        format!("images/{}/{}", self.prefix(), rel)
    }

    /// Helper that returns the absolute on-disk path (inside the `assets` folder)
    /// for a given relative image path. This is handy for the native build where
    /// we iterate over directories at runtime.
    pub fn assets_dir(&self, subdir: &str) -> String {
        format!("assets/images/{}/{}", self.prefix(), subdir)
    }
} 