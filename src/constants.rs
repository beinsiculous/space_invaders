//! Toast Busters' layout and tuning. The sheets, the roster and the draw depths are
//! `art.rs`'s; everything here is derived from them or is a rule's own number.
//!
//! The game is drawn at one art pixel per window pixel, pixel-snapped, in a fixed
//! 800 x 600 window. A position here is a cell centre on a whole pixel; the body
//! that stands there is placed by its sheet's measured offset
//! (`SheetSpec::position_for_cell`), so a collider centre may sit on a half pixel while
//! the art stays crisp.

use engine_core::prelude::*;

use crate::art::{half_size, Hero, DEION, BUNKER_CELL};

pub(crate) const WIN_W: f32 = 800.0;
pub(crate) const WIN_H: f32 = 600.0;

// --- the HUD band, the window's top 45 px, in screen pixels from its top ----------------
// Two text rows above the flyby lane. Row one holds each score with its life icons and
// the chaos banner between them; row two each streak under its score and one centred
// transient. No two labels that can show at once share a place.

pub(crate) const HUD_ROW_ONE: (f32, f32) = (4.0, 20.0);
pub(crate) const HUD_ROW_TWO: (f32, f32) = (24.0, 40.0);
pub(crate) const HUD_TEXT_MARGIN: f32 = 12.0;
pub(crate) const HUD_SCORE_WIDTH: f32 = 120.0;
pub(crate) const HUD_BANNER_WIDTH: f32 = 220.0;
pub(crate) const HUD_TRANSIENT_WIDTH: f32 = 240.0;
pub(crate) const HUD_FONT_SIZE: f32 = 14.0;
/// The gap between a score's box and its first life icon, and between icons.
pub(crate) const LIFE_ICON_GAP: f32 = 4.0;

// --- the flyby lane ------------------------------------------------------------------

/// The flyers' cell centre: the saucer's 65 px body spans 189–254, under the HUD band.
pub(crate) const FLYBY_CELL_Y: f32 = 221.0;
pub(crate) const FLYER_SPEED: f32 = 130.0;
/// Average flyer appearances per second (one every 15 s).
pub(crate) const FLYER_SPAWN_RATE: f32 = 1.0 / 15.0;
/// Of every four flybys, this many are Jammer's; the rest are Maxwell's saucer.
pub(crate) const JAMMER_SHARE_OF_FOUR: u32 = 1;
/// Maxwell's mystery bonus — a hash draw picks one per saucer (the classic table).
pub(crate) const SAUCER_BONUS_VALUES: [u32; 4] = [50, 100, 150, 300];
pub(crate) const JAMMER_POINTS: u32 = 100;
/// Jammer drops a jam glob this often while over the field.
pub(crate) const JAM_GLOB_INTERVAL: f32 = 1.2;
pub(crate) const JAM_GLOB_SPEED: f32 = 180.0;
/// A glob on a live cannon jams it: no shot for this long, a second glob refreshing it.
pub(crate) const JAM_SECONDS: f32 = 1.5;
/// Seconds a flyby flash or a jam notice stays in the HUD's transient slot.
pub(crate) const NOTICE_SECONDS: f32 = 1.5;

// --- the formation ----------------------------------------------------------------------

pub(crate) const FORMATION_COLUMNS: usize = 10;
/// The rows' cell centres, top down — pitch 54, the tallest Toasted being 49.
pub(crate) const ROW_CELL_Y: [f32; 4] = [150.0, 96.0, 42.0, -12.0];
pub(crate) const COLUMN_PITCH: f32 = 52.0;
/// The outermost columns' cell centres: ±234.
pub(crate) const OUTERMOST_COLUMN_X: f32 = (FORMATION_COLUMNS as f32 - 1.0) * COLUMN_PITCH / 2.0;
/// The outermost live column's centre never marches past ±this: 136 px of travel.
pub(crate) const MARCH_BOUND_X: f32 = 370.0;
pub(crate) const MARCH_SPEED_BASE: f32 = 30.0;
pub(crate) const MARCH_SPEED_MAX: f32 = 240.0;
pub(crate) const INSANE_MARCH_MULT: f32 = 1.8;
/// The drop on every edge bounce: the bunkers' top after 8 descents, the landing after 13.
pub(crate) const DESCEND_STEP: f32 = 16.0;

/// Average Toasted shots per second across the whole fleet.
pub(crate) const INVADER_FIRE_RATE: f32 = 0.7;
/// Ridiculous mode: fire-rate multiplier (Insane's buff is march speed).
pub(crate) const RIDICULOUS_FIRE_MULT: f32 = 2.4;

// --- the bunkers --------------------------------------------------------------------------

pub(crate) const BUNKER_COUNT: usize = 4;
/// The bunkers' cell centres: x by bunker, and one y — the cells span −199…−151.
pub(crate) const BUNKER_XS: [f32; BUNKER_COUNT] = [-270.0, -90.0, 90.0, 270.0];
pub(crate) const BUNKER_CELL_Y: f32 = -175.0;
pub(crate) const BUNKER_BLOCK_PX: f32 = 16.0;
pub(crate) const BUNKER_BLOCK_COLUMNS: usize = 4;
pub(crate) const BUNKER_BLOCK_ROWS: usize = 3;
/// A block stands only where its slice of the current state draws at least this many
/// opaque pixels: a block with less would be an invisible wall.
pub(crate) const BLOCK_MIN_OPAQUE_PIXELS: u32 = 16;
/// Opaque pixels in each 16 px slice of the bunker art, by state, row and column —
/// measured on the synced sheet, which a test re-measures.
pub(crate) const BUNKER_SLICE_OPAQUE: [[[u32; BUNKER_BLOCK_COLUMNS]; BUNKER_BLOCK_ROWS]; 4] = [
    [[7, 64, 60, 2], [110, 256, 256, 63], [80, 161, 160, 65]],
    [[0, 0, 0, 0], [89, 256, 158, 8], [80, 161, 160, 65]],
    [[0, 0, 0, 0], [0, 134, 92, 1], [72, 161, 142, 53]],
    [[0, 0, 0, 0], [0, 0, 0, 0], [66, 146, 144, 57]],
];
/// The damage at which a bunker becomes `damaged`, `critical` and `melted`. Ten blocks
/// stand at the start, so every state is reachable.
pub(crate) const BUNKER_STATE_DAMAGE: [u32; 3] = [2, 4, 6];
/// The top of the bunkers' cells.
pub(crate) const BUNKER_TOP_Y: f32 = BUNKER_CELL_Y + BUNKER_CELL.y / 2.0;

// --- the cannons ----------------------------------------------------------------------------

/// The line every cannon's collision envelope stands on.
pub(crate) const GROUND_Y: f32 = -290.0;
pub(crate) const PLAYER_SPEED: f32 = 420.0;
/// The envelope never comes nearer the window's side than this.
pub(crate) const CANNON_SIDE_MARGIN: f32 = 20.0;
/// Co-op: each cannon starts this far to either side of centre (∓).
pub(crate) const CANNON_COOP_OFFSET: f32 = 120.0;
/// The Ridiculous and Insiculous twin volley: the muzzle ± this, inside the body.
pub(crate) const TWIN_VOLLEY_OFFSET: f32 = 10.0;
/// A surviving hit hides the cannon for its `hurt` (4 frames at 100 ms), then it
/// reappears at its spawn.
pub(crate) const RESPAWN_SECONDS: f32 = 0.4;
/// The fleet has landed when a live Toasted's bottom edge reaches this: the top of
/// Deion's envelope, the taller cannon. It lies above every cannon, so it is the only
/// landing rule.
pub(crate) const INVASION_Y: f32 = GROUND_Y + (DEION.bounds.1.y - DEION.bounds.0.y);

pub(crate) const STARTING_LIVES: u32 = 3;
/// Consecutive kill shots (no misses in between) for the streak achievement.
pub(crate) const SHARPSHOOTER_TARGET: u32 = 10;

// --- shots ------------------------------------------------------------------------------------

pub(crate) const PLAYER_SHOT_SPEED: f32 = 540.0;
/// A player's shot collides as its centre line, 2 px wide over the drawn height —
/// classic Invaders' thin shot, narrower than the icicle or chip it draws.
pub(crate) const PLAYER_SHOT_COLLIDER_WIDTH: f32 = 2.0;
pub(crate) const CRUMB_SPEED: f32 = 240.0;
/// Safety-net auto-despawn for any shot that somehow escapes the culling. Long enough for
/// the slowest shot — a jam glob from the lane — to fall past the cull line first, or a
/// glob that missed would blink out above the window's edge.
pub(crate) const SHOT_LIFETIME: f32 = 3.5;
/// Seconds between a cannon's shots.
pub(crate) const FIRE_COOLDOWN: f32 = 0.4;
pub(crate) const MAX_PLAYER_SHOTS: usize = 1;
/// Insane: extra single shots on screen against the faster fleet.
pub(crate) const INSANE_MAX_PLAYER_SHOTS: usize = 4;
/// Ridiculous: one twin volley in flight.
pub(crate) const RIDICULOUS_MAX_PLAYER_SHOTS: usize = 2;
/// Insiculous: three stacked twin volleys against both fleet buffs.
pub(crate) const INSICULOUS_MAX_PLAYER_SHOTS: usize = 6;
/// Extra margin past the window edge before a shot is culled.
pub(crate) const SHOT_CULL_PAD: f32 = 30.0;

// --- the backdrop grid --------------------------------------------------------------------

/// The backdrop grid's alpha, well under the preset's resting value: the lattice reads
/// over the countertop without veiling it.
pub(crate) const BACKDROP_ALPHA: f32 = 0.25;
pub(crate) const GRID_IMPULSE_KILL_STRENGTH: f32 = 260.0;
pub(crate) const GRID_IMPULSE_KILL_RADIUS: f32 = 90.0;
pub(crate) const GRID_IMPULSE_PLAYER_HIT_STRENGTH: f32 = 700.0;
pub(crate) const GRID_IMPULSE_PLAYER_HIT_RADIUS: f32 = 160.0;

/// Collider outlines for the F1 overlay.
pub(crate) const DEBUG_COLLIDER_COLOR: Vec4 = Vec4::new(1.0, 0.2, 1.0, 0.9);
pub(crate) const DEBUG_COLLIDER_EMISSIVE: f32 = 2.0;

// --- derived positions ------------------------------------------------------------------------

/// The cell centre of formation column `column`, before any march.
pub(crate) fn column_cell_x(column: usize) -> f32 {
    -OUTERMOST_COLUMN_X + column as f32 * COLUMN_PITCH
}

/// Where `hero`'s cannon stands: its envelope's centre, the envelope on the ground.
pub(crate) fn cannon_y(hero: Hero) -> f32 {
    GROUND_Y + hero.half_envelope().y
}

/// The farthest `hero`'s cannon may stand from centre.
pub(crate) fn cannon_max_x(hero: Hero) -> f32 {
    WIN_W / 2.0 - CANNON_SIDE_MARGIN - hero.half_envelope().x
}

/// Where cannon `index` starts for a roster of `player_count`: solo at centre, co-op
/// split to ∓`CANNON_COOP_OFFSET`.
pub(crate) fn cannon_spawn_x(player_count: usize, index: usize) -> f32 {
    if player_count <= 1 {
        0.0
    } else if index == 0 {
        -CANNON_COOP_OFFSET
    } else {
        CANNON_COOP_OFFSET
    }
}

/// The half size a player's shot collides with: its centre line over the drawn height.
pub(crate) fn player_shot_half_collider(hero: Hero) -> Vec2 {
    Vec2::new(PLAYER_SHOT_COLLIDER_WIDTH / 2.0, half_size(hero.spec().shot).y)
}

// --- the startup cards and the window icon ---------------------------------------
// Synced from deion_assets like every sheet (`assets/sprites/sync.list`).

/// The cards every Insiculous game opens on, in order: the studio's, then the
/// engine's. The engine shows them before `init` (`GameConfig::with_startup_splashes`).
pub(crate) const STARTUP_CARDS: [&str; 2] = [
    "sprites/ai_be_insiculous_320x192.png",
    "sprites/ai_insiculous_2d_maxwell_splash_320x192.png",
];
/// The engine's icon: the window's until the game draws one of its own.
pub(crate) const WINDOW_ICON: &str = "sprites/ai_insiculous_2d_maxwell_icon_64x64.png";

#[cfg(test)]
mod startup_card_tests {
    use super::*;
    use engine_core::{AssetConfig, AssetManager};

    #[test]
    fn the_startup_cards_are_the_studio_then_the_engine_on_their_contract_backdrops() {
        let assets_directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
        let config = crate::game_config(assets_directory.to_str().expect("the asset path is UTF-8"));
        assert_eq!(config.startup_splashes, STARTUP_CARDS, "the studio's card, then the engine's");
        assert_eq!(config.window_icon.as_deref(), Some(WINDOW_ICON));

        // The corners are BRANDING.md's named fills, which the engine letterboxes each
        // card in; a redrawn backdrop that drifts from the contract fails here.
        let assets = AssetManager::headless(AssetConfig::from(&config));
        for (path, size, corner) in [
            (STARTUP_CARDS[0], (320, 192), Some([0x14, 0x10, 0x1F, 0xFF])),
            (STARTUP_CARDS[1], (320, 192), Some([0x4A, 0x44, 0x58, 0xFF])),
            (WINDOW_ICON, (64, 64), None),
        ] {
            let image = assets.image_backdrop(path).unwrap_or_else(|| panic!("{path} is synced"));
            assert_eq!((image.size.x, image.size.y), size, "{path}");
            if let Some(corner) = corner {
                assert_eq!(image.corner.to_rgba8(), corner, "{path}");
            }
        }
    }
}
