use engine_core::prelude::*;

pub(crate) const WIN_W: f32 = 800.0;
pub(crate) const WIN_H: f32 = 600.0;

// --- Player cannon ---
pub(crate) const PLAYER_W: f32 = 52.0;
pub(crate) const PLAYER_H: f32 = 18.0;
pub(crate) const PLAYER_SCALE: Vec2 = Vec2::new(PLAYER_W / RENDER_UNIT, PLAYER_H / RENDER_UNIT);
pub(crate) const PLAYER_Y: f32 = -250.0;
pub(crate) const PLAYER_SPEED: f32 = 420.0;
pub(crate) const PLAYER_MAX_X: f32 = WIN_W / 2.0 - 20.0 - PLAYER_W / 2.0;
/// Co-op: each cannon starts this far to either side of center (∓). Must
/// sit inside `PLAYER_MAX_X` so both spawn in-bounds.
pub(crate) const CANNON_COOP_OFFSET: f32 = 120.0;
/// Tint for player 2's cannon and its bullets (player 1 uses the chaos
/// theme's accent color, so the two cannons always read apart).
pub(crate) const PLAYER2_COLOR: Vec4 = Vec4::new(1.0, 0.75, 0.2, 1.0);

// --- Bullets ---
pub(crate) const PLAYER_BULLET_W: f32 = 5.0;
pub(crate) const PLAYER_BULLET_H: f32 = 14.0;
pub(crate) const PLAYER_BULLET_SPEED: f32 = 540.0;
pub(crate) const INVADER_BULLET_W: f32 = 6.0;
pub(crate) const INVADER_BULLET_H: f32 = 12.0;
pub(crate) const INVADER_BULLET_SPEED: f32 = 240.0;
/// Safety-net auto-despawn for any bullet that somehow escapes the
/// off-screen culling (engine `Lifetime` component).
pub(crate) const BULLET_LIFETIME: f32 = 2.5;
/// Seconds between player shots.
pub(crate) const FIRE_COOLDOWN: f32 = 0.4;
/// Live player bullets allowed at once (classic single-shot discipline).
pub(crate) const MAX_PLAYER_BULLETS: usize = 1;
/// Insane mode: live-bullet cap — extra single shots on screen compensate
/// for the faster-marching fleet.
pub(crate) const INSANE_MAX_PLAYER_BULLETS: usize = 4;
/// Ridiculous mode: live-bullet cap for the twin cannon (one volley of 2
/// in flight — the classic discipline with two barrels).
pub(crate) const RIDICULOUS_MAX_PLAYER_BULLETS: usize = 2;
/// Insiculous mode: live-bullet cap for the twin cannon (3 stacked volleys
/// against both fleet buffs at once).
pub(crate) const INSICULOUS_MAX_PLAYER_BULLETS: usize = 6;
/// Ridiculous mode: horizontal offset of each twin-cannon barrel.
pub(crate) const TWIN_CANNON_OFFSET: f32 = 12.0;
pub(crate) const INVADER_BULLET_COLOR: Vec4 = Vec4::new(1.0, 0.4, 0.3, 1.0);

// --- Invader formation ---
pub(crate) const INVADER_COLS: usize = 10;
pub(crate) const INVADER_ROWS: usize = 5;
pub(crate) const INVADER_W: f32 = 36.0;
pub(crate) const INVADER_H: f32 = 24.0;
pub(crate) const INVADER_SCALE: Vec2 = Vec2::new(INVADER_W / RENDER_UNIT, INVADER_H / RENDER_UNIT);
pub(crate) const INVADER_GAP_X: f32 = 16.0;
pub(crate) const INVADER_GAP_Y: f32 = 18.0;
/// Y position of the center of the top invader row at spawn.
pub(crate) const FORMATION_TOP_Y: f32 = 230.0;
/// The formation's leftmost/rightmost invader center never marches past ±this.
pub(crate) const MARCH_BOUND_X: f32 = WIN_W / 2.0 - 30.0;
/// Horizontal march speed with the formation intact.
pub(crate) const MARCH_SPEED_BASE: f32 = 30.0;
/// Horizontal march speed as the last invader standing.
pub(crate) const MARCH_SPEED_MAX: f32 = 240.0;
/// Insane mode: march speed multiplier.
pub(crate) const INSANE_MARCH_MULT: f32 = 1.8;
/// Vertical drop on every edge bounce.
pub(crate) const DESCEND_STEP: f32 = 24.0;
/// An invader center at or below this line means the fleet has landed.
pub(crate) const INVASION_Y: f32 = -215.0;

/// Average invader shots per second across the whole fleet.
pub(crate) const INVADER_FIRE_RATE: f32 = 0.7;
/// Ridiculous mode: fire-rate multiplier (Insane's buff is march speed).
pub(crate) const RIDICULOUS_FIRE_MULT: f32 = 2.4;

/// Points per kill, by row (0 = top row, scores the most — classic table).
pub(crate) const INVADER_ROW_VALUES: [u32; INVADER_ROWS] = [30, 20, 20, 10, 10];
/// Row tints, top to bottom.
pub(crate) const INVADER_ROW_COLORS: [Vec4; INVADER_ROWS] = [
    Vec4::new(1.0, 0.35, 0.75, 1.0), // magenta
    Vec4::new(1.0, 0.55, 0.25, 1.0), // orange
    Vec4::new(1.0, 0.9, 0.3, 1.0),   // yellow
    Vec4::new(0.35, 0.95, 0.5, 1.0), // green
    Vec4::new(0.35, 0.7, 1.0, 1.0),  // blue
];

// --- UFO (mystery ship) ---
pub(crate) const UFO_W: f32 = 44.0;
pub(crate) const UFO_H: f32 = 16.0;
pub(crate) const UFO_SCALE: Vec2 = Vec2::new(UFO_W / RENDER_UNIT, UFO_H / RENDER_UNIT);
/// Flight lane across the top, above the fresh formation.
pub(crate) const UFO_Y: f32 = 268.0;
pub(crate) const UFO_SPEED: f32 = 130.0;
/// Average UFO appearances per second (~one every 15s).
pub(crate) const UFO_SPAWN_RATE: f32 = 1.0 / 15.0;
/// Mystery bonus table — a hash draw picks one per UFO (classic values).
pub(crate) const UFO_BONUS_VALUES: [u32; 4] = [50, 100, 150, 300];
pub(crate) const UFO_COLOR: Vec4 = Vec4::new(1.0, 0.3, 0.45, 1.0);
pub(crate) const UFO_EMISSIVE: f32 = 1.8;
/// Seconds the "UFO +N" HUD flash stays up after a kill.
pub(crate) const UFO_FLASH_SECS: f32 = 1.5;

// --- Barriers (bunkers) ---
pub(crate) const BARRIER_COUNT: usize = 4;
/// Each barrier is a grid of small destructible blocks.
pub(crate) const BARRIER_BLOCK: f32 = 12.0;
pub(crate) const BARRIER_BLOCK_COLS: usize = 6;
pub(crate) const BARRIER_BLOCK_ROWS: usize = 3;
/// Y position of a barrier's center row.
pub(crate) const BARRIER_Y: f32 = -180.0;
/// X positions of the barrier centers.
pub(crate) const BARRIER_XS: [f32; BARRIER_COUNT] = [-270.0, -90.0, 90.0, 270.0];

pub(crate) const STARTING_LIVES: u32 = 3;
/// Consecutive kill shots (no misses in between) for the streak achievement.
pub(crate) const SHARPSHOOTER_TARGET: u32 = 10;

/// Extra margin past the window edge before a bullet is culled.
pub(crate) const BULLET_CULL_PAD: f32 = 30.0;

pub(crate) const PLAYER_EMISSIVE: f32 = 1.5;
pub(crate) const BULLET_EMISSIVE: f32 = 2.5;
pub(crate) const INVADER_EMISSIVE: f32 = 0.9;
pub(crate) const BARRIER_EMISSIVE: f32 = 0.6;

// Radial impulses kicked into the spring-mass background grid.
pub(crate) const GRID_IMPULSE_KILL_STRENGTH: f32 = 260.0;
pub(crate) const GRID_IMPULSE_KILL_RADIUS: f32 = 90.0;
pub(crate) const GRID_IMPULSE_PLAYER_HIT_STRENGTH: f32 = 700.0;
pub(crate) const GRID_IMPULSE_PLAYER_HIT_RADIUS: f32 = 160.0;

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
