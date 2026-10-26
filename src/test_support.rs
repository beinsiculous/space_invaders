//! Shared test fixtures.
//!
//! The synced sheets are read back through the engine's own GPU-free load path, so a
//! test can check the game's clip names against the committed art without a window, a
//! GPU or a `GameContext`. The fixtures also drive the whole game through the engine's
//! harness: the real frame, with no window.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use engine_core::assets::sprite_sheet::{prepare_sheet, PreparedSheet};
use engine_core::prelude::*;
use engine_core::test_support::{DrawCommand, GameHarness, InputEvent};

use crate::art::{half_size, Toasted};
use crate::constants::{player_shot_half_collider, FLYER_SPEED, PLAYER_SHOT_SPEED};
use crate::spawning::invader_home;
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

/// Put a fresh shot of player `index` 2 px under the body `target` will have after the
/// next physics step, its centre line at `x_in_cell` of that body's cell, flying up. The
/// shot is spawned and stepped once first, so its body exists and moves on the very next
/// step — the step that decides first contact — and the target is predicted after it.
pub(crate) fn shoot_into(harness: &mut GameHarness<SpaceInvadersGame>, index: usize, target: Target, x_in_cell: f32) -> EntityId {
    let shot = spawn_test_shot(harness, index);
    step_quietly(harness, 1, &target.keep());
    aim_shot(harness, index, shot, target, x_in_cell);
    shot
}

/// A fresh shot of player `index`, low over the counter, whose body exists after one step.
pub(crate) fn spawn_test_shot(harness: &mut GameHarness<SpaceInvadersGame>, index: usize) -> EntityId {
    harness.context(|game, ctx| {
        game.spawn_player_shot(ctx.world, index, Vec2::new(0.0, -250.0));
        *game.players[index].shots.last().expect("the shot")
    })
}

/// Put `shot`, stepped once since its spawn, under `target` as [`shoot_into`] says.
pub(crate) fn aim_shot(harness: &mut GameHarness<SpaceInvadersGame>, index: usize, shot: EntityId, target: Target, x_in_cell: f32) {
    let (body, spec) = target.next_body(harness);
    harness.context(|game, _| {
        let hero = game.players[index].hero;
        let cell_left = body.x + spec.sprite_offset().x - spec.cell.x / 2.0;
        let below = body.y - half_size(spec).y - player_shot_half_collider(hero).y - 2.0;
        game.physics.reset_body(shot, Vec2::new(cell_left + x_in_cell, below));
        game.physics.set_velocity(shot, Vec2::new(0.0, PLAYER_SHOT_SPEED), 0.0);
    });
}

/// What a test shoots at.
#[derive(Clone, Copy)]
pub(crate) enum Target {
    Invader(EntityId),
    Flyer(EntityId),
}

impl Target {
    pub(crate) fn keep(self) -> Vec<EntityId> {
        match self {
            Target::Invader(_) => Vec::new(),
            Target::Flyer(entity) => vec![entity],
        }
    }

    /// Where the target's body stands after the next physics step, and its sheet. A
    /// kinematic body reaches the target set the frame before: a Toasted's is its home
    /// plus the formation offset, a flyer's one frame of flight on from where it is.
    pub(crate) fn next_body(self, harness: &GameHarness<SpaceInvadersGame>) -> (Vec2, &'static SheetSpec) {
        let game = harness.game();
        match self {
            Target::Invader(entity) => {
                let invader = game.invaders.iter().find(|invader| invader.entity == entity).expect("alive");
                (invader_home(invader.row, invader.column) + game.formation_offset, invader.kind.spec().sheet)
            }
            Target::Flyer(entity) => {
                let flyer = game.flyer.as_ref().expect("a flyer");
                let step = Vec2::new(flyer.direction * FLYER_SPEED * FRAME, 0.0);
                (position_of(harness, entity) + step, crate::spawning::flyer_art(flyer.kind).0)
            }
        }
    }
}

/// Frames with wall time between them, so the song's virtual clock renders blocks and the
/// conductor drains what the frames sent.
pub(crate) fn settle(harness: &mut GameHarness<SpaceInvadersGame>) {
    for _ in 0..6 {
        std::thread::sleep(Duration::from_millis(3));
        harness.step(FRAME, &[]);
    }
}

/// Step with wall time between frames until `done` holds, failing past `limit`.
pub(crate) fn poll(
    harness: &mut GameHarness<SpaceInvadersGame>,
    limit: Duration,
    what: &str,
    done: impl Fn(&GameHarness<SpaceInvadersGame>) -> bool,
) {
    let started = Instant::now();
    while !done(harness) {
        assert!(started.elapsed() < limit, "{what} within {limit:?}");
        std::thread::sleep(Duration::from_millis(3));
        harness.step(FRAME, &[]);
    }
}

/// The live slice in formation slot (`row`, `column`).
pub(crate) fn slice(harness: &GameHarness<SpaceInvadersGame>, row: usize, column: usize) -> EntityId {
    harness.game().invaders.iter().find(|invader| invader.row == row && invader.column == column).expect("the slice").entity
}

/// Where a test's icicle meets a slice of `kind`: the middle of its cell, or the ring's side
/// of a Bagelgeist, whose middle is its hole.
pub(crate) fn aim_for(kind: Toasted) -> f32 {
    if kind == Toasted::Bagelgeist { 16.0 } else { kind.spec().sheet.cell.x / 2.0 }
}

/// Where the pause menu drew `label` in the last frame.
pub(crate) fn drawn_at(harness: &GameHarness<SpaceInvadersGame>, label: &str) -> Vec2 {
    harness.ui_commands().iter().find_map(|command| match command {
        DrawCommand::Text { data, .. } if data.text.contains(label) => {
            Some(data.position + Vec2::new(4.0, data.height / 2.0))
        }
        _ => None,
    }).unwrap_or_else(|| panic!("'{label}' was drawn"))
}
