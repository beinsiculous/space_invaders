//! Headless tests for the rules: pure formation, combat, bunker and flyby math, plus
//! physics simulations over the exact entity recipes the game spawns.

use std::collections::BTreeSet;

use engine_core::prelude::*;

use crate::art::*;
use crate::constants::*;
use crate::gameplay::{
    coop_defeated, fleet_has_landed, flyer_entry, flyer_kind_for, flyer_offscreen, invader_fire_rate,
    lose_block, march_speed, march_step, passes_through_hole, pick_shooter_column, player_fire_caps,
    rects_overlap, saucer_bonus, volley_fits, MarchOutcome,
};
use crate::menu::mode_hint;
use crate::spawning::{block_stands, invader_home, spawn_bunkers, spawn_cannon, spawn_flyer, spawn_invaders};
use crate::types::*;

const FORMATION_SIZE: usize = 40;

// --- the formation ---

#[test]
fn march_step_advances_without_descending_mid_field() {
    let out = march_step(0.0, 1.0, 5.0, -100.0, 100.0);
    assert_eq!(out, MarchOutcome { offset_x: 5.0, dir: 1.0, descended: false });
}

#[test]
fn march_step_bounces_off_either_bound_and_descends() {
    let live_max = 100.0;
    let out = march_step(MARCH_BOUND_X - live_max - 1.0, 1.0, 5.0, -100.0, live_max);
    assert_eq!(out, MarchOutcome { offset_x: MARCH_BOUND_X - live_max, dir: -1.0, descended: true });
    let live_min = -100.0;
    let out = march_step(-MARCH_BOUND_X - live_min + 1.0, -1.0, 5.0, live_min, 100.0);
    assert_eq!(out, MarchOutcome { offset_x: -MARCH_BOUND_X - live_min, dir: 1.0, descended: true });
}

#[test]
fn march_step_uses_live_extremes_so_a_thinned_fleet_marches_farther() {
    let out = march_step(MARCH_BOUND_X - 5.0, 1.0, 1.0, 0.0, 0.0);
    assert!(!out.descended, "a thin fleet keeps marching where a full one would bounce");
}

#[test]
fn the_full_fleet_has_136_px_of_travel_each_way() {
    assert_eq!(MARCH_BOUND_X - OUTERMOST_COLUMN_X, 136.0);
}

#[test]
fn march_speed_ramps_from_base_to_max_and_insane_multiplies() {
    assert_eq!(march_speed(FORMATION_SIZE, FORMATION_SIZE, false), MARCH_SPEED_BASE);
    assert_eq!(march_speed(1, FORMATION_SIZE, false), MARCH_SPEED_MAX);
    let mid = march_speed(FORMATION_SIZE / 2, FORMATION_SIZE, false);
    assert!(mid > MARCH_SPEED_BASE && mid < MARCH_SPEED_MAX);
    assert_eq!(march_speed(FORMATION_SIZE, FORMATION_SIZE, true), MARCH_SPEED_BASE * INSANE_MARCH_MULT);
    assert!(march_speed(0, 0, false) > 0.0 && march_speed(5, 3, false) > 0.0, "degenerate counts never panic");
}

/// The bottom row's body edge reaches the bunkers' top after 8 descents and the landing
/// line after 13 — the pacing four rows keep.
#[test]
fn the_fleet_reaches_the_bunkers_after_eight_descents_and_lands_after_thirteen() {
    let bottom_row = ROSTER.len() - 1;
    let start = invader_home(bottom_row, 0).y - half_size(Toasted::for_row(bottom_row).spec().sheet).y;
    let descents_to = |line: f32| (0..).find(|&descents| start - descents as f32 * DESCEND_STEP <= line).unwrap_or(0);
    assert_eq!(descents_to(BUNKER_TOP_Y), 8);
    assert_eq!(descents_to(INVASION_Y), 13);
}

#[test]
fn the_landing_reads_body_edges_against_the_top_of_deions_envelope() {
    assert_eq!(INVASION_Y, -235.0);
    assert_eq!(INVASION_Y, GROUND_Y + 2.0 * Hero::Deion.half_envelope().y, "the taller cannon's top");
    assert!(fleet_has_landed(&[INVASION_Y]));
    assert!(!fleet_has_landed(&[INVASION_Y + 0.5, 100.0]));
    assert!(!fleet_has_landed(&[]), "an empty fleet never lands");
    // The line lies above every cannon, so no separate cannon-contact rule is needed:
    // Cubert alone is landed on 12 px above his head.
    let cubert_top = GROUND_Y + 2.0 * Hero::Cubert.half_envelope().y;
    assert_eq!(INVASION_Y - cubert_top, 12.0);
}

// --- the roster ---

#[test]
fn the_roster_by_row_is_sour_doom_bagelgeist_rye_and_plain_wraith() {
    assert_eq!(ROSTER, [Toasted::SourDoom, Toasted::Bagelgeist, Toasted::RyeIP, Toasted::PlainWraith]);
    let points: Vec<u32> = ROSTER.iter().map(|kind| kind.spec().points).collect();
    assert_eq!(points, [30, 20, 20, 10]);
    let hits: Vec<u32> = ROSTER.iter().map(|kind| kind.spec().hits).collect();
    assert_eq!(hits, [2, 1, 1, 1], "only Sour-Doom takes two hits");
    assert_eq!(Toasted::for_row(99), Toasted::PlainWraith, "a row past the roster is its last kind");
}

#[test]
fn the_rows_clear_each_other_and_the_flyby_lane() {
    for row in 1..ROSTER.len() {
        let above = invader_home(row - 1, 0).y - half_size(Toasted::for_row(row - 1).spec().sheet).y;
        let below = invader_home(row, 0).y + half_size(Toasted::for_row(row).spec().sheet).y;
        assert!(above > below, "row {row} overlaps the row above");
    }
    let top = invader_home(0, 0).y + half_size(&SOUR_DOOM).y;
    let saucer_bottom = SAUCER.position_for_cell(Vec2::new(0.0, FLYBY_CELL_Y)).y - half_size(&SAUCER).y;
    assert!(top < saucer_bottom, "the top row {top} reaches the lane {saucer_bottom}");
}

// --- shots ---

#[test]
fn pick_shooter_column_wraps_over_live_columns() {
    let live = [2usize, 5, 7];
    assert_eq!([0, 1, 2, 3].map(|rand| pick_shooter_column(&live, rand)), [2, 5, 7, 2]);
}

#[test]
fn rects_overlap_detects_touch_and_separation() {
    let half = Vec2::new(10.0, 10.0);
    assert!(rects_overlap(Vec2::ZERO, half, Vec2::new(20.0, 0.0), half), "edge touch counts");
    assert!(!rects_overlap(Vec2::ZERO, half, Vec2::new(21.0, 0.0), half));
}

#[test]
fn player_fire_caps_per_chaos_mode() {
    assert_eq!(player_fire_caps(ChaosMode::Normal), (1, MAX_PLAYER_SHOTS));
    assert_eq!(player_fire_caps(ChaosMode::Insane), (1, INSANE_MAX_PLAYER_SHOTS));
    assert_eq!(player_fire_caps(ChaosMode::Ridiculous), (2, RIDICULOUS_MAX_PLAYER_SHOTS));
    assert_eq!(player_fire_caps(ChaosMode::Insiculous), (2, INSICULOUS_MAX_PLAYER_SHOTS));
    for mode in ChaosMode::ALL {
        let (shots, max_live) = player_fire_caps(mode);
        assert!(shots <= max_live, "{mode:?} volley larger than its cap");
    }
}

/// Insane owns march speed, Ridiculous fire rate, Insiculous both, Normal neither.
#[test]
fn fleet_buffs_split_across_chaos_modes() {
    let speed = |mode: ChaosMode| march_speed(FORMATION_SIZE, FORMATION_SIZE, mode.is_insane());
    let base = speed(ChaosMode::Normal);
    assert_eq!(invader_fire_rate(ChaosMode::Normal), INVADER_FIRE_RATE);
    assert!(speed(ChaosMode::Insane) > base);
    assert_eq!(invader_fire_rate(ChaosMode::Insane), INVADER_FIRE_RATE, "Insane must not fire faster");
    assert_eq!(speed(ChaosMode::Ridiculous), base, "Ridiculous must not march faster");
    assert!(invader_fire_rate(ChaosMode::Ridiculous) > INVADER_FIRE_RATE);
    assert!(speed(ChaosMode::Insiculous) > base && invader_fire_rate(ChaosMode::Insiculous) > INVADER_FIRE_RATE);
}

#[test]
fn a_players_shot_collides_as_its_two_pixel_centre_line_over_the_drawn_height() {
    assert_eq!(player_shot_half_collider(Hero::Deion), Vec2::new(1.0, 6.0), "the icicle is 12 tall");
    assert_eq!(player_shot_half_collider(Hero::Cubert), Vec2::new(1.0, 3.5), "the chip is 7 tall");
}

#[test]
fn the_hole_is_columns_27_to_35_of_the_bagels_cell() {
    let cell_left = 100.0;
    assert!(!passes_through_hole(cell_left + 26.9, cell_left));
    assert!(passes_through_hole(cell_left + 27.0, cell_left));
    assert!(passes_through_hole(cell_left + 35.9, cell_left));
    assert!(!passes_through_hole(cell_left + 36.0, cell_left));
}

// --- the cannons ---

#[test]
fn cannons_spawn_apart_inside_their_clamp_and_stand_on_the_ground() {
    assert_eq!(cannon_spawn_x(1, 0), 0.0, "the solo cannon is centred");
    let (left, right) = (cannon_spawn_x(2, 0), cannon_spawn_x(2, 1));
    assert!(left < 0.0 && right > 0.0 && (left + right).abs() < f32::EPSILON);
    for (index, hero) in [(0, Hero::Deion), (1, Hero::Cubert)] {
        assert!(cannon_spawn_x(2, index).abs() < cannon_max_x(hero));
        assert_eq!(cannon_y(hero) - hero.half_envelope().y, GROUND_Y);
        let cell = Vec2::new(0.0, cannon_y(hero)) + hero.spec().sheet.sprite_offset();
        assert_eq!(cell, cell.round(), "{hero:?}'s cell centre is a whole pixel");
    }
}

#[test]
fn coop_ends_only_when_every_cannon_is_out_and_caps_are_per_cannon() {
    assert!(!coop_defeated(&[3]) && coop_defeated(&[0]));
    assert!(!coop_defeated(&[0, 2]) && coop_defeated(&[0, 0]));
    assert!(!coop_defeated(&[]), "no roster is not a defeat");
    let (shots, max_live) = player_fire_caps(ChaosMode::Ridiculous);
    assert!(!volley_fits(max_live, shots, max_live));
    assert!(volley_fits(0, shots, max_live), "the other cannon fires independently");
}

// --- the bunkers ---

#[test]
fn a_bunkers_state_comes_from_its_damage_and_every_state_is_reachable() {
    let states: Vec<BunkerState> = (0..8).map(BunkerState::for_damage).collect();
    use BunkerState::*;
    assert_eq!(states, [Intact, Intact, Damaged, Damaged, Critical, Critical, Melted, Melted]);
    assert_eq!(lose_block(Bunker::FRESH), Bunker { damage: 1, state: Intact });
}

/// Destroy blocks of one bunker in `order` the way `damage_bunker` does, returning the
/// state after each chip and the blocks still standing at the end. A block already
/// melted is skipped, as a shot would find nothing there.
fn chip_sequence(order: &[(usize, usize)]) -> (Vec<BunkerState>, BTreeSet<(usize, usize)>) {
    let mut standing: BTreeSet<(usize, usize)> = (0..BUNKER_BLOCK_ROWS)
        .flat_map(|row| (0..BUNKER_BLOCK_COLUMNS).map(move |column| (row, column)))
        .filter(|&(row, column)| block_stands(BunkerState::Intact, row, column))
        .collect();
    let mut bunker = Bunker::FRESH;
    let mut states = Vec::new();
    for slot in order {
        if !standing.remove(slot) {
            continue;
        }
        let after = lose_block(bunker);
        if after.state != bunker.state {
            standing.retain(|&(row, column)| block_stands(after.state, row, column));
        }
        bunker = after;
        states.push(bunker.state);
    }
    (states, standing)
}

#[test]
fn chipped_from_below_a_bunker_is_carved_away_as_it_melts() {
    let bottom_first: Vec<(usize, usize)> =
        (0..BUNKER_BLOCK_ROWS).rev().flat_map(|row| (0..BUNKER_BLOCK_COLUMNS).map(move |column| (row, column))).collect();
    let (states, standing) = chip_sequence(&bottom_first);
    use BunkerState::*;
    // Two chips make it damaged, which melts the top row and the middle row's right
    // sliver; two more make it critical, which melts the middle-left; the sixth chip
    // takes the last block, and the bunker is gone as it reaches melted.
    assert_eq!(states, [Intact, Damaged, Damaged, Critical, Critical, Melted]);
    assert!(standing.is_empty(), "the shot-out bunker is gone: {standing:?}");
}

#[test]
fn eaten_from_above_a_bunker_reaches_melted_on_its_bottom_row() {
    // Only five chips land above the bottom row (the right sliver melts on the way), so
    // the sixth, the first bottom block, is the one that makes it melted.
    let top_first: Vec<(usize, usize)> =
        (0..BUNKER_BLOCK_ROWS).flat_map(|row| (0..BUNKER_BLOCK_COLUMNS).map(move |column| (row, column))).collect();
    let (states, standing) = chip_sequence(&top_first[..9]);
    assert_eq!(states.len(), 6);
    assert_eq!(states.last(), Some(&BunkerState::Melted));
    let rest_of_bottom: BTreeSet<(usize, usize)> = (1..BUNKER_BLOCK_COLUMNS).map(|column| (2, column)).collect();
    assert_eq!(standing, rest_of_bottom, "melted draws only the bottom row");
}

#[test]
fn a_melt_never_counts_as_damage() {
    // Damaged melts three blocks at once; the bunker's damage is still the chips.
    let (states, _) = chip_sequence(&[(2, 0), (2, 1)]);
    assert_eq!(states, [BunkerState::Intact, BunkerState::Damaged]);
    assert_eq!(lose_block(lose_block(Bunker::FRESH)).damage, 2);
}

// --- the flyby lane ---

#[test]
fn the_flyby_sends_jammer_one_roll_in_four_and_the_saucer_otherwise() {
    let kinds: Vec<FlyerKind> = (0..8).map(flyer_kind_for).collect();
    assert_eq!(kinds.iter().filter(|&&kind| kind == FlyerKind::Jammer).count(), 2);
    assert_eq!(kinds.iter().filter(|&&kind| kind == FlyerKind::Saucer).count(), 6);
}

#[test]
fn a_flyer_enters_off_screen_and_leaves_only_past_the_far_edge() {
    for kind in [FlyerKind::Saucer, FlyerKind::Jammer] {
        let (left, rightward) = flyer_entry(0, kind);
        let (right, leftward) = flyer_entry(1, kind);
        assert!(left < -WIN_W / 2.0 && rightward == 1.0);
        assert!(right > WIN_W / 2.0 && leftward == -1.0);
        assert!(!flyer_offscreen(left, rightward, kind) && !flyer_offscreen(right, leftward, kind));
        assert!(!flyer_offscreen(0.0, 1.0, kind));
        assert!(flyer_offscreen(-left + 1.0, 1.0, kind) && flyer_offscreen(-right - 1.0, -1.0, kind));
    }
}

#[test]
fn a_glob_that_misses_falls_past_the_cull_line_before_its_lifetime_ends() {
    let jammer_bottom = JAMMER.position_for_cell(Vec2::new(0.0, FLYBY_CELL_Y)).y - half_size(&JAMMER).y;
    let cull_line = -(WIN_H / 2.0 + SHOT_CULL_PAD);
    let seconds_to_cull = (jammer_bottom - cull_line) / JAM_GLOB_SPEED;
    assert!(seconds_to_cull < SHOT_LIFETIME, "a glob needs {seconds_to_cull} s, its lifetime is {SHOT_LIFETIME} s");
}

#[test]
fn the_saucer_draws_its_bonus_from_the_classic_table() {
    let drawn: BTreeSet<u32> = (0..4).map(saucer_bonus).collect();
    assert_eq!(drawn, SAUCER_BONUS_VALUES.into_iter().collect());
}

#[test]
fn every_mode_has_a_hint() {
    for mode in ChaosMode::ALL {
        assert!(!mode_hint(mode).is_empty(), "{mode:?} needs a mode-select hint");
    }
}

// --- physics over the exact recipes ---

/// Step `game`'s physics until `shot` starts touching `target`, or two seconds pass.
fn shot_reaches(game: &mut SpaceInvadersGame, world: &mut World, shot: EntityId, target: EntityId) -> bool {
    for _ in 0..120 {
        game.physics.update(world, 1.0 / 60.0);
        if game.physics.take_collision_events().iter().any(|c| c.event.started && c.event.involves(shot, target)) {
            return true;
        }
    }
    false
}

#[test]
fn a_players_shot_reaches_a_toasted_the_flyer_and_a_bunker_block() {
    let sheets = Sheets::default();
    let mut game = SpaceInvadersGame::default();
    let mut world = World::new();
    let invaders = spawn_invaders(&mut world, &sheets);
    let target = invaders.iter().find(|invader| invader.row == ROSTER.len() - 1).expect("a bottom row");
    let target_position = invader_home(target.row, target.column);
    game.spawn_player_shot(&mut world, 0, target_position - Vec2::new(0.0, 80.0));
    let shot = game.players[0].shots[0];
    assert!(shot_reaches(&mut game, &mut world, shot, target.entity));

    let mut world = World::new();
    let saucer = spawn_flyer(&mut world, &sheets.saucer, FlyerKind::Saucer, 0.0);
    game.players[0].shots.clear();
    game.spawn_player_shot(&mut world, 0, Vec2::new(0.0, FLYBY_CELL_Y - 100.0));
    let shot = game.players[0].shots[0];
    assert!(shot_reaches(&mut game, &mut world, shot, saucer));

    let mut world = World::new();
    let blocks = spawn_bunkers(&mut world, &sheets.butter_bunker);
    let block = blocks.iter().find(|block| block.row == BUNKER_BLOCK_ROWS - 1).expect("a bottom block");
    let block_position = world.get::<Transform2D>(block.entity).map(|t| t.position).expect("placed");
    game.players[0].shots.clear();
    game.spawn_player_shot(&mut world, 0, block_position - Vec2::new(0.0, 60.0));
    let shot = game.players[0].shots[0];
    assert!(shot_reaches(&mut game, &mut world, shot, block.entity));
}

#[test]
fn a_crumb_and_a_glob_reach_a_cannon() {
    let sheets = Sheets::default();
    for kind in [EnemyShotKind::Crumb, EnemyShotKind::JamGlob] {
        let mut game = SpaceInvadersGame::default();
        let mut world = World::new();
        let cannon = spawn_cannon(&mut world, &sheets.deion, Hero::Deion, 0.0);
        game.spawn_enemy_shot(&mut world, kind, Vec2::new(0.0, cannon_y(Hero::Deion) + 100.0), CRUMB_SPEED);
        let shot = game.enemy_shots[0].entity;
        assert!(shot_reaches(&mut game, &mut world, shot, cannon), "{kind:?}");
    }
}

/// The engine's frame tail steps the `Lifetime` pass; the game owns no lifetime system,
/// so the test drives one over the frames it steps by hand.
#[test]
fn a_stray_shot_expires_through_its_lifetime() {
    let mut game = SpaceInvadersGame::default();
    let mut world = World::new();
    let mut lifetimes = LifetimeSystem::new();
    game.spawn_player_shot(&mut world, 0, Vec2::ZERO);
    let shot = game.players[0].shots[0];
    assert!(world.get::<Lifetime>(shot).is_some(), "shots must carry a Lifetime");
    for _ in 0..(SHOT_LIFETIME * 60.0) as usize + 10 {
        game.physics.update(&mut world, 1.0 / 60.0);
        lifetimes.update(&mut world, 1.0 / 60.0);
    }
    assert!(!world.entities().contains(&shot));
}
