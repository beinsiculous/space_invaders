//! Combat through the engine's `GameHarness` over the synced art: Bagelgeist's hole at
//! the march's speed, Sour-Doom's two hits, a bunker chipped from below, Jammer's globs,
//! and the two flyers' payouts.

use engine_core::prelude::*;
use engine_core::test_support::GameHarness;

use crate::art::*;
use crate::constants::*;
use crate::spawning::{block_tex_region, invader_home, spawn_flyer};
use crate::test_support::*;
use crate::types::*;

/// Put a fresh shot of player `index` 2 px under the body `target` will have after the
/// next physics step, its centre line at `x_in_cell` of that body's cell, flying up. The
/// shot is spawned and stepped once first, so its body exists and moves on the very next
/// step — the step that decides first contact — and the target is predicted after it.
fn shoot_into(harness: &mut GameHarness<SpaceInvadersGame>, index: usize, target: Target, x_in_cell: f32) -> EntityId {
    let shot = harness.context(|game, ctx| {
        game.spawn_player_shot(ctx.world, index, Vec2::new(0.0, -250.0));
        *game.players[index].shots.last().expect("the shot")
    });
    step_quietly(harness, 1, &target.keep());
    let (body, spec) = target.next_body(harness);
    harness.context(|game, _| {
        let hero = game.players[index].hero;
        let cell_left = body.x + spec.sprite_offset().x - spec.cell.x / 2.0;
        let below = body.y - half_size(spec).y - player_shot_half_collider(hero).y - 2.0;
        game.physics.reset_body(shot, Vec2::new(cell_left + x_in_cell, below));
        game.physics.set_velocity(shot, Vec2::new(0.0, PLAYER_SHOT_SPEED), 0.0);
    });
    shot
}

/// What a test shoots at.
#[derive(Clone, Copy)]
enum Target {
    Invader(EntityId),
    Flyer(EntityId),
}

impl Target {
    fn keep(self) -> Vec<EntityId> {
        match self {
            Target::Invader(_) => Vec::new(),
            Target::Flyer(entity) => vec![entity],
        }
    }

    /// Where the target's body stands after the next physics step, and its sheet. A
    /// kinematic body reaches the target set the frame before: a Toasted's is its home
    /// plus the formation offset, a flyer's one frame of flight on from where it is.
    fn next_body(self, harness: &GameHarness<SpaceInvadersGame>) -> (Vec2, &'static SheetSpec) {
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

fn invader_of(harness: &GameHarness<SpaceInvadersGame>, kind: Toasted) -> Option<EntityId> {
    harness.game().invaders.iter().find(|invader| invader.kind == kind).map(|invader| invader.entity)
}

/// A bagel in row 1 with its Sour-Doom straight above, alone on the counter and marching
/// `direction` at the thinned fleet's pace.
fn bagel_under_sour_doom(mode: GameMode, direction: f32) -> GameHarness<SpaceInvadersGame> {
    let mut harness = harness(mode, ChaosMode::Insane);
    clear_field(&mut harness, |invader| invader.row <= 1 && invader.column == 4);
    harness.context(|game, _| game.march_dir = direction);
    step_quietly(&mut harness, 3, &[]);
    harness
}

const HOLE_CENTRE: f32 = (BAGELGEIST_HOLE_COLUMNS.0 + BAGELGEIST_HOLE_COLUMNS.1) / 2.0;

/// At the fastest march the formation slides about 42 px while a shot crosses the 47 px
/// bagel, so a hole decided frame by frame would close on it; decided at first contact,
/// it holds whichever way the fleet marches and whoever fired.
#[test]
fn a_shot_through_the_hole_passes_the_bagel_at_the_fastest_march_both_ways_with_both_shots() {
    for (mode, index) in [(GameMode::SinglePlayer, 0), (GameMode::TwoPlayerCoop, 1)] {
        for direction in [1.0, -1.0] {
            let mut harness = bagel_under_sour_doom(mode, direction);
            let bagel = invader_of(&harness, Toasted::Bagelgeist).expect("the bagel");
            let shot = shoot_into(&mut harness, index, Target::Invader(bagel), HOLE_CENTRE);
            let bagel_top = position_of(&harness, bagel).y + half_size(&BAGELGEIST).y;
            let mut cleared = false;
            for _ in 0..12 {
                step_quietly(&mut harness, 1, &[]);
                if harness.world().entities().contains(&shot) && position_of(&harness, shot).y > bagel_top + 8.0 {
                    cleared = true;
                    break;
                }
            }
            let case = format!("player {index} marching {direction}");
            assert!(invader_of(&harness, Toasted::Bagelgeist).is_some(), "{case}: the bagel was passed, not busted");
            assert!(cleared, "{case}: the shot flew on out of the ring");
            assert!(harness.game().passing.is_empty(), "{case}: the pass ended when the two parted");
        }
    }
}

/// With two full rows standing the fleet marches about 2.3 px a frame, and the shot
/// that passed the bagel strikes the Sour-Doom a 7 px gap above it.
#[test]
fn a_shot_through_the_hole_goes_on_to_strike_the_sour_doom_above() {
    let mut harness = harness(GameMode::SinglePlayer, ChaosMode::Normal);
    clear_field(&mut harness, |invader| invader.row <= 1);
    step_quietly(&mut harness, 3, &[]);
    let bagel = harness.game().invaders.iter()
        .find(|invader| invader.kind == Toasted::Bagelgeist && invader.column == 4)
        .map(|invader| invader.entity)
        .expect("the bagel");
    let shot = shoot_into(&mut harness, 0, Target::Invader(bagel), HOLE_CENTRE);
    step_quietly(&mut harness, 20, &[]);
    let game = harness.game();
    assert!(game.invaders.iter().any(|invader| invader.entity == bagel), "the bagel was passed");
    let struck: Vec<&Invader> = game.invaders.iter().filter(|invader| invader.hits_left == 1 && invader.kind == Toasted::SourDoom).collect();
    assert_eq!(struck.len(), 1, "one Sour-Doom took the shot");
    assert_eq!(struck[0].column, 4, "the one straight above");
    assert!(!harness.world().entities().contains(&shot));
}

#[test]
fn a_shot_into_the_ring_busts_the_bagel() {
    let mut harness = bagel_under_sour_doom(GameMode::SinglePlayer, 1.0);
    let bagel = invader_of(&harness, Toasted::Bagelgeist).expect("the bagel");
    shoot_into(&mut harness, 0, Target::Invader(bagel), 16.0);
    step_quietly(&mut harness, 10, &[]);
    assert!(invader_of(&harness, Toasted::Bagelgeist).is_none(), "the ring stops the shot");
    assert_eq!(harness.game().players[0].score, Toasted::Bagelgeist.spec().points);
}

#[test]
fn sour_doom_absorbs_its_first_hit_without_touching_the_streak_and_pays_on_its_second() {
    let mut harness = harness(GameMode::SinglePlayer, ChaosMode::Normal);
    clear_field(&mut harness, |invader| invader.row == 0 && invader.column == 0);
    harness.context(|game, _| game.players[0].shot_streak = 5);
    let sour_doom = invader_of(&harness, Toasted::SourDoom).expect("a Sour-Doom");

    shoot_into(&mut harness, 0, Target::Invader(sour_doom), 32.0);
    step_quietly(&mut harness, 3, &[]);
    let game = harness.game();
    assert_eq!(game.invaders[0].hits_left, 1);
    assert_eq!(game.players[0].score, 0, "the first hit pays nothing");
    assert_eq!(game.players[0].shot_streak, 5, "neither counted nor reset");
    assert_eq!(machine_state(&harness, sour_doom), HURT);
    step_quietly(&mut harness, 30, &[]);
    assert_eq!(machine_state(&harness, sour_doom), MARCH, "the hurt plays once and marches on");

    shoot_into(&mut harness, 0, Target::Invader(sour_doom), 32.0);
    step_quietly(&mut harness, 3, &[]);
    let game = harness.game();
    assert!(game.invaders.is_empty());
    assert_eq!(game.players[0].score, 30);
    assert_eq!(game.players[0].shot_streak, 6);
}

#[test]
fn a_shot_chips_a_bunker_block_from_below_and_a_state_change_redraws_the_rest() {
    let mut harness = harness(GameMode::SinglePlayer, ChaosMode::Normal);
    let block = *harness.game().bunker_blocks.iter()
        .find(|block| block.bunker == 0 && block.row == 2 && block.column == 1)
        .expect("a bottom block");
    let below = position_of(&harness, block.entity) - Vec2::new(0.0, 30.0);
    harness.context(|game, ctx| game.spawn_player_shot(ctx.world, 0, below));
    step_quietly(&mut harness, 10, &[]);
    let game = harness.game();
    assert!(!game.bunker_blocks.contains(&block));
    assert_eq!(game.bunkers[0], Bunker { damage: 1, state: BunkerState::Intact });
    assert!(game.players[0].shots.is_empty());

    // A second chip makes it damaged: the top row melts, and what stands draws damaged.
    let next = *game.bunker_blocks.iter().find(|block| block.bunker == 0 && block.row == 2).expect("another");
    harness.context(|game, ctx| game.damage_bunker(ctx, next, crate::gameplay::Chip::Butter));
    let game = harness.game();
    assert_eq!(game.bunkers[0].state, BunkerState::Damaged);
    assert!(!game.bunker_blocks.iter().any(|block| block.bunker == 0 && block.row == 0));
    let standing = game.bunker_blocks.iter().find(|block| block.bunker == 0 && block.row == 1 && block.column == 1).expect("stands");
    let region = harness.world().get::<Sprite>(standing.entity).expect("a sprite").tex_region;
    assert_eq!(region, block_tex_region(&game.sheets.butter_bunker, BunkerState::Damaged, 1, 1));
    assert_eq!(game.bunkers[1], Bunker::FRESH, "the other bunkers are untouched");
}

#[test]
fn a_glob_falls_through_the_formation_and_jams_a_cannon_without_costing_a_life() {
    let mut harness = harness(GameMode::SinglePlayer, ChaosMode::Normal);
    let wraith = invader_of(&harness, Toasted::PlainWraith).expect("a wraith");
    let above = position_of(&harness, wraith) + Vec2::new(0.0, 40.0);
    let glob = harness.context(|game, ctx| {
        game.spawn_enemy_shot(ctx.world, EnemyShotKind::JamGlob, above, JAM_GLOB_SPEED);
        game.enemy_shots.last().map(|shot| shot.entity).expect("a glob")
    });
    step_quietly(&mut harness, 30, &[glob]);
    assert_eq!(harness.game().invaders.len(), 40, "a glob passes the Toasted");
    assert!(harness.game().enemy_shots.iter().any(|shot| shot.entity == glob), "and keeps falling");

    let glob = drop_on_cannon(&mut harness, 0, EnemyShotKind::JamGlob);
    step_quietly(&mut harness, 10, &[glob]);
    let game = harness.game();
    assert_eq!(game.players[0].lives, STARTING_LIVES);
    assert!(game.players[0].jammed_for > 0.0);
    assert_eq!(game.notice.map(|(notice, _)| notice), Some(Notice::Jammed { player: 0 }));

    step_with(&mut harness, &[press(KeyCode::Space)], &[]);
    step_quietly(&mut harness, 30, &[]);
    assert!(harness.game().players[0].shots.is_empty(), "no shot while jammed");
    step_quietly(&mut harness, 90, &[]);
    assert!(harness.game().players[0].shots.len() + harness.game().players[0].fire_cooldown.ceil() as usize > 0, "firing again after the jam");
}

#[test]
fn jammer_drops_globs_over_the_field() {
    let mut harness = harness(GameMode::SinglePlayer, ChaosMode::Normal);
    harness.context(|game, ctx| {
        let entity = spawn_flyer(ctx.world, &game.sheets.jammer, FlyerKind::Jammer, 0.0);
        game.flyer = Some(Flyer { entity, kind: FlyerKind::Jammer, direction: 1.0, glob_in: JAM_GLOB_INTERVAL });
    });
    let mut globs = 0;
    for _ in 0..80 {
        harness.step(FRAME, &[]);
        globs = globs.max(harness.game().enemy_shots.iter().filter(|shot| shot.kind == EnemyShotKind::JamGlob).count());
    }
    assert!(globs >= 1, "Jammer dropped a glob within 1.3 s");
}

#[test]
fn the_saucer_pays_its_mystery_bonus_and_jammer_its_flat_one() {
    for kind in [FlyerKind::Saucer, FlyerKind::Jammer] {
        let mut harness = harness(GameMode::SinglePlayer, ChaosMode::Normal);
        // One Toasted far from the lane keeps the match going.
        clear_field(&mut harness, |invader| invader.row == ROSTER.len() - 1 && invader.column == 0);
        let entity = harness.context(|game, ctx| {
            let entity = spawn_flyer(ctx.world, game.sheets.flyer(kind), kind, 0.0);
            game.flyer = Some(Flyer { entity, kind, direction: 1.0, glob_in: 10.0 });
            entity
        });
        step_quietly(&mut harness, 1, &[entity]);
        let spec = crate::spawning::flyer_art(kind).0;
        let shot = shoot_into(&mut harness, 0, Target::Flyer(entity), spec.cell.x / 2.0);
        for _ in 0..10 {
            step_quietly(&mut harness, 1, &[entity, shot]);
        }
        let game = harness.game();
        assert!(game.flyer.is_none(), "{kind:?} busted");
        let unlocked = harness.context(|_, ctx| ctx.achievements.is_unlocked(crate::achievements::UFO_HUNTER));
        let game = harness.game();
        match kind {
            FlyerKind::Saucer => {
                assert!(SAUCER_BONUS_VALUES.contains(&game.players[0].score));
                assert_eq!(game.notice.map(|(notice, _)| notice), Some(Notice::SaucerBonus(game.players[0].score)));
                assert!(unlocked, "the saucer earns Saucer Hunter");
            }
            FlyerKind::Jammer => {
                assert_eq!(game.players[0].score, JAMMER_POINTS);
                assert_eq!(game.notice.map(|(notice, _)| notice), Some(Notice::JammerBusted));
                assert!(!unlocked, "Saucer Hunter stays the saucer's");
            }
        }
        assert_eq!(game.players[0].shot_streak, 1, "{kind:?} counts as a kill for the streak");
    }
}

