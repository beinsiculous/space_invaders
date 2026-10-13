use engine_core::prelude::*;

use crate::art::{Hero, Toasted, BUNKER_STATE_CLIPS};
use crate::constants::{BUNKER_COUNT, BUNKER_STATE_DAMAGE, STARTING_LIVES};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum GameState {
    TitleScreen { selection: u8 },
    ModeSelect { selection: u8 },
    Achievements,
    Playing,
    GameOver { won: bool },
}

/// How many cannons defend the counter: Deion alone, or Deion and Cubert sharing the
/// formation but keeping their own lives and scores.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum GameMode {
    SinglePlayer,
    TwoPlayerCoop,
}

impl GameMode {
    /// Number of cannons this mode fields.
    pub(crate) fn player_count(self) -> usize {
        match self {
            GameMode::SinglePlayer => 1,
            GameMode::TwoPlayerCoop => 2,
        }
    }
}

/// The way a cannon faces: the way it last moved. The heroes are drawn facing right.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Facing {
    Right,
    Left,
}

impl Facing {
    /// +1 facing right, -1 facing left: the x-scale's sign and the muzzle's mirror.
    pub(crate) fn sign(self) -> f32 {
        match self {
            Facing::Right => 1.0,
            Facing::Left => -1.0,
        }
    }
}

/// One cannon's per-match state.
pub(crate) struct PlayerState {
    /// The cannon's body; `None` once it runs out of lives.
    pub(crate) entity: Option<EntityId>,
    pub(crate) hero: Hero,
    /// The game's own x for the cannon — authoritative over the transform, so a
    /// respawn is never read back from where the body was.
    pub(crate) x: f32,
    pub(crate) facing: Facing,
    pub(crate) shots: Vec<EntityId>,
    pub(crate) fire_cooldown: f32,
    pub(crate) score: u32,
    pub(crate) lives: u32,
    pub(crate) shot_streak: u32,
    /// Seconds left jammed by a glob: no shot until it runs out.
    pub(crate) jammed_for: f32,
    /// While set, the cannon is hidden and inert for this many more seconds after a
    /// surviving hit; it reappears at its spawn when it runs out.
    pub(crate) respawn_in: Option<f32>,
}

impl PlayerState {
    /// A cannon at the start of a match: no body yet, full lives.
    pub(crate) fn fresh(hero: Hero, x: f32) -> Self {
        Self {
            entity: None,
            hero,
            x,
            facing: Facing::Right,
            shots: Vec::new(),
            fire_cooldown: 0.0,
            score: 0,
            lives: STARTING_LIVES,
            shot_streak: 0,
            jammed_for: 0.0,
            respawn_in: None,
        }
    }

    /// Whether this cannon can be hit, move and fire this frame.
    pub(crate) fn is_active(&self) -> bool {
        self.entity.is_some() && self.respawn_in.is_none()
    }
}

/// A live Toasted: its entity, the formation slot it marches in (fixed — the whole
/// formation moves as one offset), its kind and the hits it can still take.
pub(crate) struct Invader {
    pub(crate) entity: EntityId,
    pub(crate) row: usize,
    pub(crate) column: usize,
    pub(crate) kind: Toasted,
    pub(crate) hits_left: u32,
}

/// What an enemy shot is: a burnt crumb from the formation, or a jam glob from Jammer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EnemyShotKind {
    Crumb,
    JamGlob,
}

pub(crate) struct EnemyShot {
    pub(crate) entity: EntityId,
    pub(crate) kind: EnemyShotKind,
}

/// What crosses the flyby lane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FlyerKind {
    Saucer,
    Jammer,
}

pub(crate) struct Flyer {
    pub(crate) entity: EntityId,
    pub(crate) kind: FlyerKind,
    /// +1.0 flying right, -1.0 flying left.
    pub(crate) direction: f32,
    /// Seconds until Jammer's next glob.
    pub(crate) glob_in: f32,
}

/// The one message the HUD's transient slot shows; a newer one replaces the older.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Notice {
    SaucerBonus(u32),
    JammerBusted,
    Jammed { player: usize },
}

/// A bunker's four damage states, which its blocks draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum BunkerState {
    Intact,
    Damaged,
    Critical,
    Melted,
}

impl BunkerState {
    /// The state a bunker with `damage` blocks lost draws.
    pub(crate) fn for_damage(damage: u32) -> BunkerState {
        match BUNKER_STATE_DAMAGE.iter().filter(|&&threshold| damage >= threshold).count() {
            0 => BunkerState::Intact,
            1 => BunkerState::Damaged,
            2 => BunkerState::Critical,
            _ => BunkerState::Melted,
        }
    }

    pub(crate) fn index(self) -> usize {
        self as usize
    }

    pub(crate) fn clip(self) -> &'static str {
        BUNKER_STATE_CLIPS[self.index()]
    }
}

/// A butter bunker: how many of its blocks have been destroyed, and the state that
/// makes it draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Bunker {
    pub(crate) damage: u32,
    pub(crate) state: BunkerState,
}

impl Bunker {
    pub(crate) const FRESH: Bunker = Bunker { damage: 0, state: BunkerState::Intact };
}

/// One 16 px block of a bunker, drawing its slice of the bunker's current state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BunkerBlock {
    pub(crate) entity: EntityId,
    pub(crate) bunker: usize,
    pub(crate) row: usize,
    pub(crate) column: usize,
}

/// Every loaded sheet, plus the white texture the particles use.
pub(crate) struct Sheets {
    pub(crate) white: u32,
    pub(crate) sour_doom: SpriteSheet,
    pub(crate) bagelgeist: SpriteSheet,
    pub(crate) rye_i_p: SpriteSheet,
    pub(crate) plain_wraith: SpriteSheet,
    pub(crate) saucer: SpriteSheet,
    pub(crate) jammer: SpriteSheet,
    pub(crate) deion: SpriteSheet,
    pub(crate) cubert: SpriteSheet,
    pub(crate) icicle: SpriteSheet,
    pub(crate) ice_chip: SpriteSheet,
    pub(crate) burnt_crumb: SpriteSheet,
    pub(crate) jam_glob: SpriteSheet,
    pub(crate) crouton_burst: SpriteSheet,
    pub(crate) ice_burst: SpriteSheet,
    pub(crate) butter_splash: SpriteSheet,
    pub(crate) jam_splatter: SpriteSheet,
    pub(crate) butter_bunker: SpriteSheet,
    pub(crate) countertop: SpriteSheet,
    pub(crate) life_icon: SpriteSheet,
}

impl Sheets {
    pub(crate) fn toasted(&self, kind: Toasted) -> &SpriteSheet {
        match kind {
            Toasted::SourDoom => &self.sour_doom,
            Toasted::Bagelgeist => &self.bagelgeist,
            Toasted::RyeIP => &self.rye_i_p,
            Toasted::PlainWraith => &self.plain_wraith,
        }
    }

    pub(crate) fn hero(&self, hero: Hero) -> &SpriteSheet {
        match hero {
            Hero::Deion => &self.deion,
            Hero::Cubert => &self.cubert,
        }
    }

    pub(crate) fn hero_shot(&self, hero: Hero) -> &SpriteSheet {
        match hero {
            Hero::Deion => &self.icicle,
            Hero::Cubert => &self.ice_chip,
        }
    }

    pub(crate) fn flyer(&self, kind: FlyerKind) -> &SpriteSheet {
        match kind {
            FlyerKind::Saucer => &self.saucer,
            FlyerKind::Jammer => &self.jammer,
        }
    }

    pub(crate) fn enemy_shot(&self, kind: EnemyShotKind) -> &SpriteSheet {
        match kind {
            EnemyShotKind::Crumb => &self.burnt_crumb,
            EnemyShotKind::JamGlob => &self.jam_glob,
        }
    }
}

impl Default for Sheets {
    fn default() -> Self {
        Self {
            white: 0,
            sour_doom: SpriteSheet::placeholder(),
            bagelgeist: SpriteSheet::placeholder(),
            rye_i_p: SpriteSheet::placeholder(),
            plain_wraith: SpriteSheet::placeholder(),
            saucer: SpriteSheet::placeholder(),
            jammer: SpriteSheet::placeholder(),
            deion: SpriteSheet::placeholder(),
            cubert: SpriteSheet::placeholder(),
            icicle: SpriteSheet::placeholder(),
            ice_chip: SpriteSheet::placeholder(),
            burnt_crumb: SpriteSheet::placeholder(),
            jam_glob: SpriteSheet::placeholder(),
            crouton_burst: SpriteSheet::placeholder(),
            ice_burst: SpriteSheet::placeholder(),
            butter_splash: SpriteSheet::placeholder(),
            jam_splatter: SpriteSheet::placeholder(),
            butter_bunker: SpriteSheet::placeholder(),
            countertop: SpriteSheet::placeholder(),
            life_icon: SpriteSheet::placeholder(),
        }
    }
}

pub struct SpaceInvadersGame {
    pub(crate) physics: PhysicsSystem,
    pub(crate) sheets: Sheets,

    pub(crate) invaders: Vec<Invader>,
    /// What is crossing the flyby lane, if anything: one flyer at a time.
    pub(crate) flyer: Option<Flyer>,
    /// The HUD's transient slot and the seconds it has left.
    pub(crate) notice: Option<(Notice, f32)>,
    pub(crate) bunkers: [Bunker; BUNKER_COUNT],
    pub(crate) bunker_blocks: Vec<BunkerBlock>,
    /// The Toasted's crumbs and Jammer's globs: one list, so a hit, a restart and the
    /// culling cover both.
    pub(crate) enemy_shots: Vec<EnemyShot>,
    /// A player shot that started touching a Bagelgeist through its hole, and that
    /// bagel: their contacts are ignored until the two part or either goes.
    pub(crate) passing: Vec<(EntityId, EntityId)>,
    /// The countertop's `Tilemap`, under everything; it stays up under the menus.
    pub(crate) countertop: Option<EntityId>,
    /// The deforming grid drawn over the art; the engine steps and draws it.
    pub(crate) backdrop: Option<EntityId>,
    /// The HUD's life icons, one per life a player started with: (player, index, entity).
    pub(crate) life_icons: Vec<(usize, u32, EntityId)>,
    /// Detached one-shots — bursts, splashes and the cannons' hurts — kept so a restart
    /// or a quit ends every one of them.
    pub(crate) transient_visuals: Vec<EntityId>,

    /// The defending cannons: one in single player, two in co-op.
    pub(crate) players: Vec<PlayerState>,
    pub(crate) mode: GameMode,
    pub(crate) state: GameState,
    /// Pause state + menu, driven from the `Playing` state each frame.
    pub(crate) pause: PauseMenu,
    pub(crate) chaos_mode: ChaosMode,
    pub(crate) frame_count: u32,

    /// Horizontal march direction: +1.0 right, -1.0 left.
    pub(crate) march_dir: f32,
    /// Formation displacement from the spawn layout (march + descents).
    pub(crate) formation_offset: Vec2,

    /// F1 toggles magenta collider outlines over the sprites.
    pub(crate) debug_colliders: bool,
}

impl Default for SpaceInvadersGame {
    fn default() -> Self {
        Self {
            physics: PhysicsSystem::with_config(PhysicsConfig::top_down()),
            sheets: Sheets::default(),
            invaders: Vec::new(),
            flyer: None,
            notice: None,
            bunkers: [Bunker::FRESH; BUNKER_COUNT],
            bunker_blocks: Vec::new(),
            enemy_shots: Vec::new(),
            passing: Vec::new(),
            countertop: None,
            backdrop: None,
            life_icons: Vec::new(),
            transient_visuals: Vec::new(),
            // One cannon until the title menu picks a mode and `start_game` rebuilds
            // the roster.
            players: vec![PlayerState::fresh(Hero::Deion, 0.0)],
            mode: GameMode::SinglePlayer,
            state: GameState::TitleScreen { selection: 0 },
            pause: PauseMenu::new(),
            chaos_mode: ChaosMode::Normal,
            frame_count: 0,
            march_dir: 1.0,
            formation_offset: Vec2::ZERO,
            debug_colliders: false,
        }
    }
}
