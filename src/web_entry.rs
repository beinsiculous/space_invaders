//! Browser entry point: fetch all assets into the engine's VFS, then run
//! the exact same game `main.rs` runs natively.
//!
//! Saves persist to browser localStorage: on wasm the `GameConfig`
//! save-path strings below are localStorage keys, and the values are
//! byte-identical to the native JSON save files (achievements, high
//! scores, input bindings). The key naming and the `insiculous-save`
//! event the site listens for are the engine's `docs/WEB_SAVES.md`
//! contract — the site slug for this game is `invaders`.

use engine_core::prelude::run_game;
use engine_core::web::{init_web_logging, preload_assets, set_boot_status};
use wasm_bindgen::prelude::wasm_bindgen;

/// Where the deployed build serves its assets from; also the VFS key base.
///
/// VERSION-BUMP CHECKLIST — these must all agree (a mismatch 404s every
/// asset at boot with a "not in vfs" message):
/// 1. this constant (`/games/<slug>/v<N>/assets`),
/// 2. `scripts/build_wasm.sh`'s output dir (currently hardcoded `v1`),
/// 3. the site's `src/content/games/<slug>.md` `wasm:` path,
/// 4. the deployed dir `insiculous_web/public/games/<slug>/v<N>/`.
const ASSET_BASE: &str = "/games/invaders/v1/assets";

#[wasm_bindgen(start)]
pub fn start() {
    init_web_logging();
    wasm_bindgen_futures::spawn_local(async {
        if let Err(e) = preload_assets(ASSET_BASE).await {
            log::error!("asset preload failed: {e}");
            set_boot_status(&format!("Failed to load assets: {e}"));
            return;
        }
        if let Err(e) = run_game(
            crate::SpaceInvadersGame::default(),
            crate::game_config(ASSET_BASE)
                .with_achievement_save_path("beinsiculous.games.invaders.achievements")
                .with_input_settings_path("beinsiculous.games.invaders.input")
                .with_score_save_path("beinsiculous.games.invaders.scores"),
        ) {
            log::error!("failed to start game: {e}");
            set_boot_status(&format!("Failed to start: {e}"));
        }
    });
}
