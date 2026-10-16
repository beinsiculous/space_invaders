//! Toast Busters — game crate: space invaders, re-skinned as Deion and Cubert holding
//! the counter against the Toasted.
//!
//! The library owns the whole game (`SpaceInvadersGame` + its `Game` impl) so
//! both entry points stay thin: `main.rs` (native window, filesystem saves,
//! optional editor) and `web_entry.rs` (wasm-bindgen start: fetch assets,
//! then the same `run_game`). This split also keeps `editor_integration`
//! behind the `editor` feature in both entry points — `main.rs` for the native
//! window, `web_entry.rs` for the browser's editor bundle.

mod achievements;
mod art;
#[cfg(test)]
mod art_tests;
#[cfg(test)]
mod combat_tests;
mod constants;
mod drawing;
mod effects;
mod gameplay;
#[cfg(test)]
mod gameplay_tests;
#[cfg(test)]
mod flow_tests;
mod hud;
mod menu;
mod spawning;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod title_screen_tests;
mod types;

#[cfg(target_arch = "wasm32")]
mod web_entry;

use engine_core::prelude::*;
use art::*;
use constants::*;
use spawning::spawn_countertop;
use types::*;

pub use types::SpaceInvadersGame;

/// Load a synced sheet; a missing or malformed one is a broken build, not a state
/// the game can play on.
fn load_sheet(assets: &mut AssetManager, spec: &SheetSpec) -> SpriteSheet {
    assets
        .load_sprite_sheet(spec.path)
        .unwrap_or_else(|error| panic!("{} does not load: {error}", spec.path))
}

/// The shared `GameConfig` for every target. Entry points add their own
/// platform extras on top — both configure the same save documents, just
/// addressed differently (native: file paths anchored to the game dir;
/// web: `beinsiculous.games.invaders.*` localStorage keys per the
/// engine's `docs/WEB_SAVES.md`).
///
/// `asset_base` must be an ANCHORED base: native callers pass an absolute
/// path (`main.rs` derives it from `game_root!()` so the cwd never
/// matters); the web entry passes the deploy URL base. Passing a bare
/// relative path like `"assets"` would silently resolve against the
/// current working directory.
pub fn game_config(asset_base: &str) -> GameConfig {
    GameConfig::new("Toast Busters")
        .with_size(WIN_W as u32, WIN_H as u32)
        .with_clear_color(0.0, 0.0, 0.0, 1.0)
        .with_fps(60)
        .with_pixel_snap(true)
        .with_startup_splashes(STARTUP_CARDS)
        .with_window_icon(WINDOW_ICON)
        .with_asset_base_path(asset_base)
}

impl Game for SpaceInvadersGame {
    fn register_achievements(&self, achievements: &mut AchievementManager, _strings: &Strings) {
        achievements::register_all(achievements);
    }

    fn init(&mut self, ctx: &mut GameContext) {
        // Resolve against the configured asset base so the same relative
        // path works natively (game dir) and on the web (VFS keys).
        let font_path = std::path::Path::new(ctx.assets.base_path()).join("fonts/font.ttf");
        if let Ok(font) = ctx.ui.load_font_file(&font_path.to_string_lossy()) {
            ctx.ui.set_default_font(font);
        }

        if let Ok(white) = ctx.assets.create_solid_color(1, 1, [255, 255, 255, 255]) {
            self.sheets.white = white.id;
        }
        self.title_art = TitleArt::load(ctx.assets, TITLE_ART);

        // Every sheet's path and cell is in `art.rs`; each PNG and its `.sheet.ron`
        // sidecar is a synced copy under `assets/sprites/`.
        let sheets = &mut self.sheets;
        for (field, spec) in [
            (&mut sheets.sour_doom, &SOUR_DOOM),
            (&mut sheets.bagelgeist, &BAGELGEIST),
            (&mut sheets.rye_i_p, &RYE_I_P),
            (&mut sheets.plain_wraith, &PLAIN_WRAITH),
            (&mut sheets.saucer, &SAUCER),
            (&mut sheets.jammer, &JAMMER),
            (&mut sheets.deion, &DEION),
            (&mut sheets.cubert, &CUBERT),
            (&mut sheets.icicle, &ICICLE),
            (&mut sheets.ice_chip, &ICE_CHIP),
            (&mut sheets.burnt_crumb, &BURNT_CRUMB),
            (&mut sheets.jam_glob, &JAM_GLOB),
            (&mut sheets.crouton_burst, &CROUTON_BURST),
            (&mut sheets.ice_burst, &ICE_BURST),
            (&mut sheets.butter_splash, &BUTTER_SPLASH),
            (&mut sheets.jam_splatter, &JAM_SPLATTER),
            (&mut sheets.butter_bunker, &BUTTER_BUNKER),
            (&mut sheets.countertop, &COUNTERTOP),
            (&mut sheets.life_icon, &LIFE_ICON),
        ] {
            *field = load_sheet(ctx.assets, spec);
        }

        // The countertop stays up under the menus; the cannons, the fleet and the bunkers
        // spawn fresh in `start_game()` once the menus pick a mode. No lattice draws over it:
        // its grout is the grid, and a second one over it read as two.
        self.countertop = Some(spawn_countertop(ctx.world, &self.sheets.countertop));
    }

    fn update(&mut self, ctx: &mut GameContext) {
        self.frame_count = self.frame_count.wrapping_add(1);

        match self.state.clone() {
            GameState::TitleScreen { selection } => self.update_title_input(ctx, selection),
            GameState::ModeSelect { selection } => self.update_mode_select_input(ctx, selection),
            GameState::Achievements => self.update_achievements_input(ctx),
            _ => self.update_gameplay(ctx),
        }

        self.update_entity_visibility(ctx);
        self.draw_ui(ctx);
    }
}
