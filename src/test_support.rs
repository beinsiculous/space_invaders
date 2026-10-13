//! Shared test fixtures.
//!
//! The synced sheets are read back through the engine's own GPU-free load path, so a
//! test can check the game's clip names against the committed art without a window, a
//! GPU or a `GameContext`. The fixtures also drive the whole game through the engine's
//! harness: the real frame, with no window.

use std::path::{Path, PathBuf};

use engine_core::assets::sprite_sheet::{prepare_sheet, PreparedSheet};
use engine_core::prelude::*;
use engine_core::test_support::{GameHarness, InputEvent};

use crate::types::*;

pub(crate) const FRAME: f32 = 1.0 / 60.0;

/// The synced art's directory, anchored to the crate so the working directory never
/// matters.
pub(crate) fn asset_base() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("assets")
}

/// A synced sheet read through the engine's own GPU-free load path.
pub(crate) fn sidecar(spec: &SheetSpec) -> PreparedSheet {
    prepare_sheet(&asset_base(), spec.path)
        .unwrap_or_else(|error| panic!("{} does not load: {error}", spec.path))
}

/// The title screen's world: the game's own config over the synced art, with no save
/// paths so achievements and scores stay in memory, and one frame run so `init` has
/// loaded the sheets and spawned the countertop.
pub(crate) fn title_harness() -> GameHarness<SpaceInvadersGame> {
    let base = asset_base();
    let config = crate::game_config(base.to_str().expect("the asset path is UTF-8"));
    let mut harness = GameHarness::new(SpaceInvadersGame::default(), config);
    harness.step(FRAME, &[]);
    harness
}

/// A match started as the mode select does it, in `mode` and `chaos`.
pub(crate) fn harness(mode: GameMode, chaos: ChaosMode) -> GameHarness<SpaceInvadersGame> {
    let mut harness = title_harness();
    harness.context(|game, ctx| {
        game.mode = mode;
        game.chaos_mode = chaos;
        ctx.chaos_mode = chaos;
        game.start_game(ctx);
    });
    harness
}

/// Step `frames` frames with nothing pressed, sweeping away after each one every crumb
/// the formation drops and any flyer the lane rolls — both are pseudo-random draws — but
/// the shots and the flyer in `keep`, which the test placed.
pub(crate) fn step_quietly(harness: &mut GameHarness<SpaceInvadersGame>, frames: usize, keep: &[EntityId]) {
    for _ in 0..frames {
        harness.step(FRAME, &[]);
        sweep_strays(harness, keep);
    }
}

fn sweep_strays(harness: &mut GameHarness<SpaceInvadersGame>, keep: &[EntityId]) {
    harness.context(|game, ctx| {
        let (kept, strays): (Vec<EnemyShot>, Vec<EnemyShot>) =
            game.enemy_shots.drain(..).partition(|shot| keep.contains(&shot.entity));
        game.enemy_shots = kept;
        for shot in strays {
            game.physics.destroy_entity(ctx.world, shot.entity);
        }
        if game.flyer.as_ref().is_some_and(|flyer| !keep.contains(&flyer.entity)) {
            game.destroy_flyer(ctx.world);
        }
    });
}

/// Remove every Toasted but those `keep` selects, and the bunkers, so a test's shot meets
/// only what it aims at.
pub(crate) fn clear_field(harness: &mut GameHarness<SpaceInvadersGame>, keep: impl Fn(&Invader) -> bool) {
    harness.context(|game, ctx| {
        let (kept, gone): (Vec<Invader>, Vec<Invader>) = game.invaders.drain(..).partition(|invader| keep(invader));
        for invader in gone {
            game.physics.destroy_entity(ctx.world, invader.entity);
        }
        game.invaders = kept;
        for block in game.bunker_blocks.drain(..) {
            game.physics.destroy_entity(ctx.world, block.entity);
        }
    });
}

/// The world position of `entity`, which must exist.
pub(crate) fn position_of(harness: &GameHarness<SpaceInvadersGame>, entity: EntityId) -> Vec2 {
    harness.world().get::<Transform2D>(entity).map(|transform| transform.position).expect("the entity exists")
}

/// The state `entity`'s clip machine is in.
pub(crate) fn machine_state(harness: &GameHarness<SpaceInvadersGame>, entity: EntityId) -> String {
    harness.world().get::<ClipStateMachine>(entity).map(|machine| machine.state().to_string()).expect("a clip machine")
}

pub(crate) fn press(key: KeyCode) -> InputEvent {
    InputEvent::KeyPressed(key)
}

pub(crate) fn release(key: KeyCode) -> InputEvent {
    InputEvent::KeyReleased(key)
}

/// Step one frame with `events` arriving before it, then sweep as `step_quietly` does.
pub(crate) fn step_with(harness: &mut GameHarness<SpaceInvadersGame>, events: &[InputEvent], keep: &[EntityId]) {
    harness.step(FRAME, events);
    sweep_strays(harness, keep);
}

/// Drop a crumb (or a glob) straight onto cannon `index` from just above its envelope,
/// and say which shot it is.
pub(crate) fn drop_on_cannon(harness: &mut GameHarness<SpaceInvadersGame>, index: usize, kind: EnemyShotKind) -> EntityId {
    harness.context(|game, ctx| {
        let player = &game.players[index];
        let top = crate::constants::cannon_y(player.hero) + player.hero.half_envelope().y;
        game.spawn_enemy_shot(ctx.world, kind, Vec2::new(player.x, top + 20.0), crate::constants::CRUMB_SPEED);
        game.enemy_shots.last().map(|shot| shot.entity).expect("the shot spawned")
    })
}
