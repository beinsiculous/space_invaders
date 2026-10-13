//! The match through the engine's `GameHarness` over the synced art: the cannons'
//! facing, muzzles and `fire`, the three ways a hurt plays, the life icons, the HUD's
//! text in its boxes, and what a restart or a quit leaves behind.

use engine_core::prelude::*;

use crate::art::*;
use crate::constants::*;
use crate::hud::{slot_box, Slot};
use crate::test_support::*;
use crate::types::*;

fn cannon(harness: &engine_core::test_support::GameHarness<SpaceInvadersGame>, index: usize) -> EntityId {
    harness.game().players[index].entity.expect("the cannon stands")
}

#[test]
fn the_solo_cannon_is_deion_centred_on_the_ground_with_three_life_icons() {
    let harness = harness(GameMode::SinglePlayer, ChaosMode::Normal);
    let game = harness.game();
    assert_eq!(game.players.len(), 1);
    assert_eq!(game.players[0].hero, Hero::Deion);
    assert_eq!(position_of(&harness, cannon(&harness, 0)), Vec2::new(0.0, cannon_y(Hero::Deion)));
    assert_eq!(game.life_icons.len(), STARTING_LIVES as usize);
    assert_eq!(game.invaders.len(), 40);
    assert_eq!(game.bunker_blocks.len(), 40);
}

#[test]
fn a_cannon_faces_the_way_it_moved_and_its_muzzle_and_shot_mirror_with_it() {
    let mut harness = harness(GameMode::SinglePlayer, ChaosMode::Normal);
    let entity = cannon(&harness, 0);
    step_with(&mut harness, &[press(KeyCode::KeyA)], &[]);
    step_quietly(&mut harness, 10, &[]);
    step_with(&mut harness, &[release(KeyCode::KeyA)], &[]);
    step_quietly(&mut harness, 2, &[]);

    let game = harness.game();
    let player = &game.players[0];
    assert_eq!(player.facing, Facing::Left);
    let transform = harness.world().get::<Transform2D>(entity).expect("a transform");
    assert!(transform.scale.x < 0.0, "a negative x-scale mirrors the cell");
    assert_eq!(transform.position, Vec2::new(player.x, cannon_y(Hero::Deion)), "the body is the collider's centre");
    let sprite = harness.world().get::<Sprite>(entity).expect("a sprite");
    assert_eq!(sprite.offset.x, -DEION.sprite_offset().x);
    let offset = Hero::Deion.muzzle_offset();
    assert_eq!(game.muzzle(0), Vec2::new(player.x - offset.x, cannon_y(Hero::Deion) + offset.y));

    step_with(&mut harness, &[press(KeyCode::Space)], &[]);
    step_with(&mut harness, &[release(KeyCode::Space)], &[]);
    let game = harness.game();
    let shot = game.players[0].shots[0];
    let shot_x = harness.world().get::<Transform2D>(shot).map(|t| t.position.x).expect("a shot");
    assert_eq!(shot_x, game.muzzle(0).x, "the icicle leaves the mirrored muzzle");

    // And back: a move right turns the art, the offset and the muzzle back.
    step_with(&mut harness, &[press(KeyCode::KeyD)], &[]);
    step_quietly(&mut harness, 5, &[]);
    step_with(&mut harness, &[release(KeyCode::KeyD)], &[]);
    step_quietly(&mut harness, 2, &[]);
    let game = harness.game();
    let player = &game.players[0];
    assert_eq!(player.facing, Facing::Right);
    assert!(harness.world().get::<Transform2D>(entity).expect("a transform").scale.x > 0.0);
    assert_eq!(harness.world().get::<Sprite>(entity).expect("a sprite").offset.x, DEION.sprite_offset().x);
    assert_eq!(game.muzzle(0), Vec2::new(player.x + offset.x, cannon_y(Hero::Deion) + offset.y));
}

#[test]
fn crumbs_landing_on_both_coop_cannons_in_one_frame_cost_each_one_life() {
    let mut harness = harness(GameMode::TwoPlayerCoop, ChaosMode::Normal);
    // Spawned in this order the frame's hits read [0, 1, 0]: player one must still lose
    // only one life.
    let crumbs = [
        drop_on_cannon(&mut harness, 0, EnemyShotKind::Crumb),
        drop_on_cannon(&mut harness, 1, EnemyShotKind::Crumb),
        harness.context(|game, ctx| {
            let player = &game.players[0];
            let top = cannon_y(player.hero) + player.hero.half_envelope().y;
            game.spawn_enemy_shot(ctx.world, EnemyShotKind::Crumb, Vec2::new(player.x + 6.0, top + 20.0), CRUMB_SPEED);
            game.enemy_shots.last().map(|shot| shot.entity).expect("a crumb")
        }),
    ];
    step_quietly(&mut harness, 10, &crumbs);
    let lives: Vec<u32> = harness.game().players.iter().map(|player| player.lives).collect();
    assert_eq!(lives, [STARTING_LIVES - 1, STARTING_LIVES - 1]);
}

#[test]
fn a_match_ending_mid_hurt_stands_the_surviving_cannon_back_up() {
    let mut harness = harness(GameMode::SinglePlayer, ChaosMode::Normal);
    let entity = cannon(&harness, 0);
    let crumb = drop_on_cannon(&mut harness, 0, EnemyShotKind::Crumb);
    for _ in 0..10 {
        if harness.game().players[0].respawn_in.is_some() { break; }
        step_quietly(&mut harness, 1, &[crumb]);
    }
    assert!(harness.game().players[0].respawn_in.is_some(), "the cannon is mid-hurt");
    harness.context(|game, ctx| game.finish_game(ctx, false));
    step_quietly(&mut harness, 30, &[]);
    let game = harness.game();
    assert!(game.players[0].respawn_in.is_none());
    assert!(harness.world().get::<Sprite>(entity).expect("a sprite").visible, "it stands in the game-over scene");
    assert_eq!(position_of(&harness, entity).x, cannon_spawn_x(1, 0));
}

#[test]
fn a_ridiculous_twin_volley_leaves_the_muzzle_ten_pixels_either_side() {
    let mut harness = harness(GameMode::SinglePlayer, ChaosMode::Ridiculous);
    step_with(&mut harness, &[press(KeyCode::Space)], &[]);
    let game = harness.game();
    let muzzle = game.muzzle(0);
    let mut xs: Vec<f32> = game.players[0].shots.iter()
        .map(|&shot| position_of(&harness, shot).x)
        .collect();
    xs.sort_by(f32::total_cmp);
    assert_eq!(xs, [muzzle.x - TWIN_VOLLEY_OFFSET, muzzle.x + TWIN_VOLLEY_OFFSET]);
}

#[test]
fn a_fire_plays_out_before_movement_turns_it_into_a_walk() {
    let mut harness = harness(GameMode::SinglePlayer, ChaosMode::Normal);
    let entity = cannon(&harness, 0);
    step_with(&mut harness, &[press(KeyCode::Space)], &[]);
    step_with(&mut harness, &[release(KeyCode::Space), press(KeyCode::KeyD)], &[]);
    assert_eq!(machine_state(&harness, entity), FIRE, "moving does not cut the shot short");
    step_quietly(&mut harness, 15, &[]);
    assert_eq!(machine_state(&harness, entity), WALK, "the shot played out, then the walk");
}

#[test]
fn a_surviving_hit_hides_the_cannon_for_its_hurt_then_it_reappears_at_its_spawn() {
    let mut harness = harness(GameMode::SinglePlayer, ChaosMode::Normal);
    let entity = cannon(&harness, 0);
    harness.context(|game, _| game.players[0].x = 200.0);
    step_quietly(&mut harness, 2, &[]);
    let crumb = drop_on_cannon(&mut harness, 0, EnemyShotKind::Crumb);
    for _ in 0..10 {
        if harness.game().players[0].lives < STARTING_LIVES { break; }
        step_quietly(&mut harness, 1, &[crumb]);
    }
    // Hold a direction through the hurt: the respawn is still authoritative.
    step_with(&mut harness, &[press(KeyCode::KeyD)], &[]);
    let game = harness.game();
    assert_eq!(game.players[0].lives, STARTING_LIVES - 1);
    assert!(game.players[0].respawn_in.is_some());
    assert!(!harness.world().get::<Sprite>(entity).expect("a sprite").visible, "hidden while it hurts");
    let hurt = *game.transient_visuals.last().expect("the hurt plays");
    assert_eq!(machine_state(&harness, hurt), HURT);

    let mut frames_hidden = 0;
    while harness.game().players[0].respawn_in.is_some() {
        step_quietly(&mut harness, 1, &[]);
        frames_hidden += 1;
        assert!(frames_hidden < 60, "the cannon never came back");
    }
    assert_eq!(position_of(&harness, entity).x, cannon_spawn_x(1, 0), "it reappears at its spawn");
    assert!(harness.world().get::<Sprite>(entity).expect("a sprite").visible);
    assert!(!harness.world().entities().contains(&hurt), "the surviving hurt ended itself");
    step_quietly(&mut harness, 1, &[]);
    let after = position_of(&harness, entity).x;
    assert!(after > cannon_spawn_x(1, 0) && after < 20.0, "it moves on from the spawn, not from where it fell: {after}");
}

#[test]
fn one_coop_cannon_out_holds_its_hurt_while_the_other_plays_on() {
    let mut harness = harness(GameMode::TwoPlayerCoop, ChaosMode::Normal);
    harness.context(|game, _| game.players[1].lives = 1);
    let crumb = drop_on_cannon(&mut harness, 1, EnemyShotKind::Crumb);
    step_quietly(&mut harness, 10, &[crumb]);
    let game = harness.game();
    assert!(game.players[1].entity.is_none(), "Cubert's cannon is gone");
    assert_eq!(game.state, GameState::Playing, "Deion plays on");
    let hurt = *game.transient_visuals.last().expect("the hurt");
    step_quietly(&mut harness, 60, &[]);
    assert!(harness.world().entities().contains(&hurt), "the last life's hurt holds its final frame");
    assert_eq!(machine_state(&harness, hurt), HURT);
}

#[test]
fn the_final_defeat_holds_the_hurt_until_a_restart_or_a_quit() {
    for quit in [false, true] {
        let mut harness = harness(GameMode::SinglePlayer, ChaosMode::Normal);
        harness.context(|game, _| game.players[0].lives = 1);
        let crumb = drop_on_cannon(&mut harness, 0, EnemyShotKind::Crumb);
        step_quietly(&mut harness, 10, &[crumb]);
        assert_eq!(harness.game().state, GameState::GameOver { won: false });
        let hurt = *harness.game().transient_visuals.last().expect("the hurt");
        step_quietly(&mut harness, 60, &[]);
        assert!(harness.world().entities().contains(&hurt));

        harness.context(|game, ctx| if quit { game.reset_to_title(ctx.world) } else { game.start_game(ctx) });
        assert!(!harness.world().entities().contains(&hurt), "quit {quit}: the held hurt ends");
        assert!(harness.game().transient_visuals.is_empty());
    }
}

#[test]
fn a_life_icon_shows_only_while_its_index_is_below_its_players_lives_and_only_in_a_match() {
    let mut harness = harness(GameMode::TwoPlayerCoop, ChaosMode::Normal);
    harness.context(|game, _| game.players[0].lives = 1);
    step_quietly(&mut harness, 1, &[]);
    let visible = |harness: &engine_core::test_support::GameHarness<SpaceInvadersGame>, entity: EntityId| {
        harness.world().get::<Sprite>(entity).expect("an icon").visible
    };
    for &(player, index, icon) in &harness.game().life_icons {
        let lives = harness.game().players[player].lives;
        assert_eq!(visible(&harness, icon), index < lives, "player {player} icon {index}");
    }
    harness.context(|game, ctx| game.reset_to_title(ctx.world));
    step_quietly(&mut harness, 1, &[]);
    for &(_, _, icon) in &harness.game().life_icons {
        assert!(!visible(&harness, icon), "no icon under the menus");
    }
}

#[test]
fn every_hud_string_fits_its_box_in_the_real_font() {
    let mut harness = harness(GameMode::TwoPlayerCoop, ChaosMode::Insiculous);
    let longest: Vec<(Slot, String)> = vec![
        (Slot::Score { player: 0 }, "SCORE 9999999".to_string()),
        (Slot::Score { player: 1 }, "P2 9999999".to_string()),
        (Slot::Streak { player: 0 }, "STREAK x999".to_string()),
        (Slot::Transient, "HE JAMMED ME!".to_string()),
        (Slot::Transient, "SAUCER +300".to_string()),
        (Slot::Transient, "JAMMER +100".to_string()),
    ];
    let banners: Vec<(Slot, String)> = ChaosMode::ALL.iter()
        .filter_map(|&mode| ChaosTheme::for_mode(mode).banner_text)
        .map(|banner| (Slot::Banner, banner.to_string()))
        .collect();
    harness.context(|_, ctx| {
        for (slot, text) in longest.iter().chain(&banners) {
            let size = ctx.ui.measure_text_styled(text, HUD_FONT_SIZE);
            let bounds = slot_box(*slot);
            assert!(size.x <= bounds.width, "{text:?} is {} wide in a {} box", size.x, bounds.width);
            assert!(size.y <= bounds.height + 2.0, "{text:?} is {} tall in a {} row", size.y, bounds.height);
        }
    });
}

#[test]
fn a_restart_or_a_quit_ends_every_one_shot_and_every_shot() {
    for quit in [false, true] {
        let mut harness = harness(GameMode::SinglePlayer, ChaosMode::Normal);
        harness.context(|game, ctx| {
            game.spawn_ice_burst(ctx.world, Vec2::ZERO);
            game.spawn_enemy_shot(ctx.world, EnemyShotKind::JamGlob, Vec2::new(0.0, 100.0), JAM_GLOB_SPEED);
            game.spawn_player_shot(ctx.world, 0, Vec2::new(0.0, -100.0));
        });
        let effects = harness.game().transient_visuals.clone();
        harness.context(|game, ctx| if quit { game.reset_to_title(ctx.world) } else { game.start_game(ctx) });
        let game = harness.game();
        assert!(game.transient_visuals.is_empty() && game.enemy_shots.is_empty() && game.players[0].shots.is_empty());
        for effect in effects {
            assert!(!harness.world().entities().contains(&effect), "quit {quit}");
        }
    }
}

#[test]
fn the_countertop_and_the_grid_stay_up_under_the_menus() {
    let harness = title_harness();
    let game = harness.game();
    let countertop = game.countertop.expect("the countertop");
    let map = harness.world().get::<Tilemap>(countertop).expect("a map");
    assert!(map.width as f32 * COUNTERTOP_TILE_PX >= WIN_W && map.height as f32 * COUNTERTOP_TILE_PX >= WIN_H);
    let backdrop = game.backdrop.expect("the backdrop");
    let grid = harness.world().get::<GridBackdrop>(backdrop).expect("a grid");
    assert_eq!(grid.draw_order, GridDrawOrder::OverSprites);
    assert_eq!(grid.color.w, BACKDROP_ALPHA);
}
