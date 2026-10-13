//! The art Toast Busters draws: one `SheetSpec` per synced sheet, the clip names the
//! game types (the contract with the sidecars), one draw depth per sheet, the
//! Toasted roster and the two heroes.
//!
//! Every box is measured from the synced PNG — bottom-right one past the last
//! opaque pixel — over the frames the subject plays, so a collider sized from it
//! covers what is drawn. Re-measure the art, never scale a sprite.

use engine_core::prelude::*;

const TOASTED_CELL: Vec2 = Vec2::new(64.0, 64.0);
const SAUCER_CELL: Vec2 = Vec2::new(96.0, 80.0);
const HERO_CELL: Vec2 = Vec2::new(64.0, 64.0);
const SHOT_CELL: Vec2 = Vec2::new(16.0, 16.0);
const SPLASH_CELL: Vec2 = Vec2::new(32.0, 32.0);
const BURST_CELL: Vec2 = Vec2::new(64.0, 64.0);
pub(crate) const BUNKER_CELL: Vec2 = Vec2::new(64.0, 48.0);
pub(crate) const COUNTERTOP_TILE_PX: f32 = 64.0;
const LIFE_ICON_CELL: Vec2 = Vec2::new(16.0, 16.0);

const fn whole_cell(cell: Vec2) -> (Vec2, Vec2) {
    (Vec2::ZERO, cell)
}

// --- the Toasted, each its `march` frames' union -------------------------------------

/// The top row's tank: 43 x 47.
pub(crate) const SOUR_DOOM: SheetSpec = SheetSpec {
    path: "sprites/ai_sour_doom.png",
    cell: TOASTED_CELL,
    bounds: (Vec2::new(10.0, 9.0), Vec2::new(53.0, 56.0)),
};
/// The ring with the hole a shot passes through: 41 x 47.
pub(crate) const BAGELGEIST: SheetSpec = SheetSpec {
    path: "sprites/ai_bagelgeist.png",
    cell: TOASTED_CELL,
    bounds: (Vec2::new(11.0, 9.0), Vec2::new(52.0, 56.0)),
};
/// 32 x 49, the tallest Toasted.
pub(crate) const RYE_I_P: SheetSpec = SheetSpec {
    path: "sprites/ai_rye_i_p.png",
    cell: TOASTED_CELL,
    bounds: (Vec2::new(16.0, 7.0), Vec2::new(48.0, 56.0)),
};
/// The bottom row: 35 x 45.
pub(crate) const PLAIN_WRAITH: SheetSpec = SheetSpec {
    path: "sprites/ai_plain_wraith.png",
    cell: TOASTED_CELL,
    bounds: (Vec2::new(14.0, 10.0), Vec2::new(49.0, 55.0)),
};

/// The columns of Bagelgeist's cell its hole spans at its widest, in every `march`
/// frame: x 27–35, half-open.
pub(crate) const BAGELGEIST_HOLE_COLUMNS: (f32, f32) = (27.0, 36.0);

// --- the flyers ------------------------------------------------------------------------

/// Maxwell's cake saucer, its `fly` frames' union: 76 x 65.
pub(crate) const SAUCER: SheetSpec = SheetSpec {
    path: "sprites/ai_maxwell_cake_saucer_96x80.png",
    cell: SAUCER_CELL,
    bounds: (Vec2::new(10.0, 7.0), Vec2::new(86.0, 72.0)),
};
/// Jammer crossing the lane, its `march` frames' union: 36 x 46.
pub(crate) const JAMMER: SheetSpec = SheetSpec {
    path: "sprites/ai_jammer.png",
    cell: TOASTED_CELL,
    bounds: (Vec2::new(13.0, 11.0), Vec2::new(49.0, 57.0)),
};

// --- the cannons -------------------------------------------------------------------------

/// Deion's collision envelope: the union of his `idle`, `walk` and `fire` boxes, 46 x 55.
/// The walk is wider and taller than the idle, so the envelope is the walk's box.
pub(crate) const DEION: SheetSpec = SheetSpec {
    path: "sprites/ai_deion.png",
    cell: HERO_CELL,
    bounds: (Vec2::new(9.0, 1.0), Vec2::new(55.0, 56.0)),
};
/// Cubert's collision envelope, the same union: 28 x 43.
pub(crate) const CUBERT: SheetSpec = SheetSpec {
    path: "sprites/ai_cubert.png",
    cell: HERO_CELL,
    bounds: (Vec2::new(18.0, 14.0), Vec2::new(46.0, 57.0)),
};

// --- the shots, each its `fly` frames' union ------------------------------------------

/// Deion's shot: 8 x 12.
pub(crate) const ICICLE: SheetSpec = SheetSpec {
    path: "sprites/ai_icicle_16x16.png",
    cell: SHOT_CELL,
    bounds: (Vec2::new(4.0, 2.0), Vec2::new(12.0, 14.0)),
};
/// Cubert's shot: 8 x 7.
pub(crate) const ICE_CHIP: SheetSpec = SheetSpec {
    path: "sprites/ai_ice_chip_16x16.png",
    cell: SHOT_CELL,
    bounds: (Vec2::new(4.0, 5.0), Vec2::new(12.0, 12.0)),
};
/// What the Toasted drop: 9 x 8.
pub(crate) const BURNT_CRUMB: SheetSpec = SheetSpec {
    path: "sprites/ai_burnt_crumb_16x16.png",
    cell: SHOT_CELL,
    bounds: (Vec2::new(3.0, 4.0), Vec2::new(12.0, 12.0)),
};
/// What Jammer drops: 9 x 11.
pub(crate) const JAM_GLOB: SheetSpec = SheetSpec {
    path: "sprites/ai_jam_glob_16x16.png",
    cell: SHOT_CELL,
    bounds: (Vec2::new(4.0, 3.0), Vec2::new(13.0, 14.0)),
};

// --- the one-shots, each drawn centred on where it happens -----------------------------

pub(crate) const CROUTON_BURST: SheetSpec = SheetSpec {
    path: "sprites/ai_crouton_burst_64x64.png",
    cell: BURST_CELL,
    bounds: whole_cell(BURST_CELL),
};
pub(crate) const ICE_BURST: SheetSpec = SheetSpec {
    path: "sprites/ai_ice_burst_32x32.png",
    cell: SPLASH_CELL,
    bounds: whole_cell(SPLASH_CELL),
};
pub(crate) const BUTTER_SPLASH: SheetSpec = SheetSpec {
    path: "sprites/ai_butter_splash_32x32.png",
    cell: SPLASH_CELL,
    bounds: whole_cell(SPLASH_CELL),
};
pub(crate) const JAM_SPLATTER: SheetSpec = SheetSpec {
    path: "sprites/ai_jam_splatter_32x32.png",
    cell: SPLASH_CELL,
    bounds: whole_cell(SPLASH_CELL),
};

// --- the counter, the bunkers and the HUD ------------------------------------------------

/// The bunker's 64 x 48 art, which its blocks slice: the whole cell, so a block's
/// slice of it lands where the block is.
pub(crate) const BUTTER_BUNKER: SheetSpec = SheetSpec {
    path: "sprites/ai_butter_bunker_64x48.png",
    cell: BUNKER_CELL,
    bounds: whole_cell(BUNKER_CELL),
};
pub(crate) const COUNTERTOP: SheetSpec = SheetSpec {
    path: "sprites/ai_countertop_64x64.png",
    cell: Vec2::new(COUNTERTOP_TILE_PX, COUNTERTOP_TILE_PX),
    bounds: whole_cell(Vec2::new(COUNTERTOP_TILE_PX, COUNTERTOP_TILE_PX)),
};
/// The two heads the HUD counts lives with, one frame each.
pub(crate) const LIFE_ICON: SheetSpec = SheetSpec {
    path: "sprites/ai_busters_life_icon_16x16.png",
    cell: LIFE_ICON_CELL,
    bounds: whole_cell(LIFE_ICON_CELL),
};

// --- the clip names the game types ------------------------------------------------------

pub(crate) const MARCH: &str = "march";
pub(crate) const HURT: &str = "hurt";
pub(crate) const FLY: &str = "fly";
pub(crate) const IDLE: &str = "idle";
pub(crate) const WALK: &str = "walk";
pub(crate) const FIRE: &str = "fire";
pub(crate) const BURST_DIE: &str = "die";
pub(crate) const BURST_HIT: &str = "hit";
pub(crate) const SPLAT: &str = "splat";
/// The bunker's four damage states, intact first: the clip names, and the order
/// `BunkerState` indexes.
pub(crate) const BUNKER_STATE_CLIPS: [&str; 4] = ["intact", "damaged", "critical", "melted"];
pub(crate) const LIFE_ICON_DEION: &str = "deion";
pub(crate) const LIFE_ICON_CUBERT: &str = "cubert";

// --- draw depths ----------------------------------------------------------------------
// One depth per sheet: the renderer draws each sheet's sprites as one batch, ordered by
// its lowest depth, and transparent texels write depth — so a sheet with entities both
// under and over another sheet's would punch holes in it. Nested from the counter up;
// every one stays under the particles the engine draws at 0.5. A jam glob draws behind
// the formation it falls through.

pub(crate) const COUNTERTOP_DEPTH: f32 = -2.0;
pub(crate) const BUNKER_DEPTH: f32 = -1.0;
pub(crate) const JAM_GLOB_DEPTH: f32 = -0.60;
pub(crate) const PLAIN_WRAITH_DEPTH: f32 = -0.50;
pub(crate) const RYE_I_P_DEPTH: f32 = -0.45;
pub(crate) const BAGELGEIST_DEPTH: f32 = -0.40;
pub(crate) const SOUR_DOOM_DEPTH: f32 = -0.35;
pub(crate) const SAUCER_DEPTH: f32 = -0.30;
pub(crate) const JAMMER_DEPTH: f32 = -0.25;
pub(crate) const DEION_DEPTH: f32 = -0.20;
pub(crate) const CUBERT_DEPTH: f32 = -0.15;
pub(crate) const ICICLE_DEPTH: f32 = -0.10;
pub(crate) const ICE_CHIP_DEPTH: f32 = -0.05;
pub(crate) const BURNT_CRUMB_DEPTH: f32 = 0.0;
pub(crate) const CROUTON_BURST_DEPTH: f32 = 0.10;
pub(crate) const ICE_BURST_DEPTH: f32 = 0.15;
pub(crate) const BUTTER_SPLASH_DEPTH: f32 = 0.20;
pub(crate) const JAM_SPLATTER_DEPTH: f32 = 0.25;
pub(crate) const LIFE_ICON_DEPTH: f32 = 0.40;

// --- the burst colours -------------------------------------------------------------------

/// Crouton particles when a Toasted is busted: the toast's own crust and crumb.
pub(crate) const TOAST_CRUST_COLOR: Vec4 = rgb(0xD8A05C);
pub(crate) const PLAIN_CRUMB_COLOR: Vec4 = rgb(0xF2D6A0);
/// Maxwell's saucer going down: the cake's frosting red.
pub(crate) const SAUCER_BURST_COLOR: Vec4 = rgb(0xD33F4C);
/// A cannon's lost life and a shot's ice: Deion's and Cubert's water.
pub(crate) const ICE_WATER_COLOR: Vec4 = rgb(0x29D1EA);

// --- the roster -------------------------------------------------------------------------

/// The four kinds of Toasted in the formation, top row first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Toasted {
    SourDoom,
    Bagelgeist,
    RyeIP,
    PlainWraith,
}

/// What one kind of Toasted is: the sheet it marches on and its depth, what it pays,
/// how many hits it takes, and the colour of its crouton burst.
pub(crate) struct ToastedSpec {
    pub(crate) sheet: &'static SheetSpec,
    pub(crate) depth: f32,
    pub(crate) points: u32,
    pub(crate) hits: u32,
    pub(crate) burst_color: Vec4,
    pub(crate) name: &'static str,
}

const SOUR_DOOM_SPEC: ToastedSpec = ToastedSpec {
    sheet: &SOUR_DOOM, depth: SOUR_DOOM_DEPTH, points: 30, hits: 2,
    burst_color: TOAST_CRUST_COLOR, name: "Sour-Doom",
};
const BAGELGEIST_SPEC: ToastedSpec = ToastedSpec {
    sheet: &BAGELGEIST, depth: BAGELGEIST_DEPTH, points: 20, hits: 1,
    burst_color: TOAST_CRUST_COLOR, name: "Bagelgeist",
};
const RYE_I_P_SPEC: ToastedSpec = ToastedSpec {
    sheet: &RYE_I_P, depth: RYE_I_P_DEPTH, points: 20, hits: 1,
    burst_color: TOAST_CRUST_COLOR, name: "Rye.I.P.",
};
const PLAIN_WRAITH_SPEC: ToastedSpec = ToastedSpec {
    sheet: &PLAIN_WRAITH, depth: PLAIN_WRAITH_DEPTH, points: 10, hits: 1,
    burst_color: PLAIN_CRUMB_COLOR, name: "Plain Wraith",
};

/// The formation's rows, top to bottom: the roster by row.
pub(crate) const ROSTER: [Toasted; 4] =
    [Toasted::SourDoom, Toasted::Bagelgeist, Toasted::RyeIP, Toasted::PlainWraith];

impl Toasted {
    pub(crate) fn spec(self) -> &'static ToastedSpec {
        match self {
            Toasted::SourDoom => &SOUR_DOOM_SPEC,
            Toasted::Bagelgeist => &BAGELGEIST_SPEC,
            Toasted::RyeIP => &RYE_I_P_SPEC,
            Toasted::PlainWraith => &PLAIN_WRAITH_SPEC,
        }
    }

    /// The kind marching in formation `row`; a row past the roster is its last kind.
    pub(crate) fn for_row(row: usize) -> Toasted {
        ROSTER[row.min(ROSTER.len() - 1)]
    }
}

// --- the heroes -------------------------------------------------------------------------

/// The two cannons: Deion defends solo and as player one, Cubert as player two.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Hero {
    Deion,
    Cubert,
}

/// What one hero is: the sheet and depth, the shot it fires, the point of its cell a
/// shot leaves from, and its life icon.
pub(crate) struct HeroSpec {
    pub(crate) sheet: &'static SheetSpec,
    pub(crate) depth: f32,
    pub(crate) shot: &'static SheetSpec,
    pub(crate) shot_depth: f32,
    /// The top of the mohawk or of the cube, in cell pixels, Y down: the middle of the
    /// crest's top row of pixels in every `fire` frame, which is when a shot leaves.
    pub(crate) muzzle_in_cell: Vec2,
    pub(crate) life_icon_clip: &'static str,
    pub(crate) name: &'static str,
}

const DEION_SPEC: HeroSpec = HeroSpec {
    sheet: &DEION, depth: DEION_DEPTH, shot: &ICICLE, shot_depth: ICICLE_DEPTH,
    muzzle_in_cell: Vec2::new(38.5, 4.0), life_icon_clip: LIFE_ICON_DEION, name: "Deion",
};
const CUBERT_SPEC: HeroSpec = HeroSpec {
    sheet: &CUBERT, depth: CUBERT_DEPTH, shot: &ICE_CHIP, shot_depth: ICE_CHIP_DEPTH,
    muzzle_in_cell: Vec2::new(30.5, 17.0), life_icon_clip: LIFE_ICON_CUBERT, name: "Cubert",
};

impl Hero {
    pub(crate) fn spec(self) -> &'static HeroSpec {
        match self {
            Hero::Deion => &DEION_SPEC,
            Hero::Cubert => &CUBERT_SPEC,
        }
    }

    /// Who defends as player `index`.
    pub(crate) fn for_player(index: usize) -> Hero {
        if index == 0 { Hero::Deion } else { Hero::Cubert }
    }

    /// Half the collision envelope's size.
    pub(crate) fn half_envelope(self) -> Vec2 {
        half_size(self.spec().sheet)
    }

    /// Where a shot leaves from, relative to the cannon's position (the envelope's
    /// centre), facing right; facing left mirrors its x.
    pub(crate) fn muzzle_offset(self) -> Vec2 {
        let spec = self.spec();
        let (top_left, bottom_right) = spec.sheet.bounds;
        let envelope_centre = (top_left + bottom_right) * 0.5;
        Vec2::new(
            spec.muzzle_in_cell.x - envelope_centre.x,
            envelope_centre.y - spec.muzzle_in_cell.y,
        )
    }
}

/// Half a sheet's measured box, in world units.
pub(crate) fn half_size(spec: &SheetSpec) -> Vec2 {
    (spec.bounds.1 - spec.bounds.0) * 0.5
}
