use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

fn main() {
    // Themes to iterate over – must match the runtime list in `Theme::prefix`.
    const THEMES: &[&str] = &["crayon", "chibi", "retro"];

    // Helper function to gather files for a given sub-directory across all themes.
    fn gather_thematic_assets(subdir: &str) -> Vec<String> {
        let mut result = Vec::new();
        for theme in THEMES {
            let dir = Path::new("assets/images").join(theme).join(subdir);
            for file in list_png_files(&dir) {
                result.push(format!("{theme}/{subdir}/{file}"));
            }
        }
        result
    }

    let flora_files = gather_thematic_assets("flora");
    let obscura_files = gather_thematic_assets("obscura");
    let enemy_fish_files = gather_thematic_assets("fish");

    // Generate Rust source containing arrays for flora and obscura.
    let generated = format!(
        "pub const FLORA_ASSET_PATHS: &[&str] = &[{}];\n\n\
         pub const OBSCURA_ASSET_PATHS: &[&str] = &[{}];\n",
        flora_files
            .iter()
            .map(|f| format!("\"images/{}\"", f))
            .collect::<Vec<_>>()
            .join(", "),
        obscura_files
            .iter()
            .map(|f| format!("\"images/{}\"", f))
            .collect::<Vec<_>>()
            .join(", "),
    );

    // Generate Rust source for enemy fish list
    let enemy_fish_generated = format!(
        "pub const ENEMY_FISH_ASSET_PATHS: &[&str] = &[{}];\n",
        enemy_fish_files
            .iter()
            .map(|f| format!("\"images/{}\"", f))
            .collect::<Vec<_>>()
            .join(", "),
    );

    // Write the generated code to $OUT_DIR/asset_list.rs
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR env variable not set"));
    let dest_path = out_dir.join("asset_list.rs");
    let mut file = File::create(&dest_path).expect("Unable to create asset_list.rs");
    file.write_all(generated.as_bytes())
        .expect("Failed to write asset_list.rs");

    // Write enemy fish list file
    let enemy_dest_path = out_dir.join("enemy_fish_asset_list.rs");
    let mut enemy_file =
        File::create(&enemy_dest_path).expect("Unable to create enemy_fish_asset_list.rs");
    enemy_file
        .write_all(enemy_fish_generated.as_bytes())
        .expect("Failed to write enemy_fish_asset_list.rs");

    // Re-run the build script if any PNG files change.
    println!("cargo:rerun-if-changed=assets/images");
}

fn list_png_files(dir: &Path) -> Vec<String> {
    fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| e.path().is_file())
                .filter_map(|e| {
                    e.path()
                        .extension()
                        .and_then(|ext| if ext == "png" { Some(e) } else { None })
                })
                .filter_map(|e| {
                    e.path()
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.to_owned())
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
} 