//! Per-frame gameplay orchestration. The pieces live in submodules: the marching
//! formation (`formation`), shots and what they hit (`combat`), the butter bunkers
//! (`bunkers`), the flyby lane (`flyers`), the cannons (`players`) and the match
//! lifecycle (`flow`).

mod bunkers;
mod combat;
mod flow;
mod flyers;
mod formation;
mod players;

#[cfg(test)]
pub(crate) use bunkers::{lose_block, Chip};
#[cfg(test)]
pub(crate) use combat::{invader_fire_rate, passes_through_hole, pick_shooter_column, player_fire_caps};
#[cfg(test)]
pub(crate) use flyers::{flyer_entry, flyer_kind_for, flyer_offscreen, flyer_spawn_rolls, saucer_bonus};
#[cfg(test)]
pub(crate) use formation::{fleet_has_landed, march_speed, march_step, MarchOutcome};
#[cfg(test)]
pub(crate) use players::{coop_defeated, volley_fits};

use engine_core::prelude::*;

use crate::constants::*;
use crate::types::*;

/// Axis-aligned overlap test between two rectangles given centres and half extents.
/// The game-side collision fallbacks (the fleet chomping bunker blocks, shot cancels)
/// use it — rapier does not report kinematic-vs-static pairs.
pub(crate) fn rects_overlap(center_a: Vec2, half_a: Vec2, center_b: Vec2, half_b: Vec2) -> bool {
    (center_a.x - center_b.x).abs() <= half_a.x + half_b.x
        && (center_a.y - center_b.y).abs() <= half_a.y + half_b.y
}

pub(crate) fn entity_position(world: &World, entity: EntityId) -> Option<Vec2> {
    world.get::<Transform2D>(entity).map(|transform| transform.position)
}

/// The state `entity`'s clip machine is in, if it has one.
pub(crate) fn clip_state(world: &World, entity: EntityId) -> Option<String> {
    world.get::<ClipStateMachine>(entity).map(|machine| machine.state().to_string())
}

impl SpaceInvadersGame {
    pub(crate) fn update_gameplay(&mut self, ctx: &mut GameContext) {
        // F1 toggles the collider debug overlay. Magenta outlines render on
        // top of sprites so any sprite-vs-collider mismatch is obvious.
        if ctx.input.is_key_just_pressed(KeyCode::F1) {
            self.debug_colliders = !self.debug_colliders;
        }

        // Pause gate: while paused the whole match is frozen — no physics step, no
        // march, no fire timers; the overlay is drawn in the UI pass.
        if self.state == GameState::Playing {
            let action = self.pause.update(ctx.players, ctx.input, ctx.window_size);
            ctx.time_scale = self.pause.time_scale();
            match action {
                PauseAction::Restart => { self.start_game(ctx); return; }
                PauseAction::QuitToTitle => { self.reset_to_title(ctx.world); return; }
                PauseAction::ExitGame => { ctx.request_exit(); return; }
                // Skip the rest of the frame so the resuming keypress can't leak into
                // gameplay; the world unfreezes next frame.
                PauseAction::Resumed => {
                    self.music.follow_pause(ctx, false);
                    return;
                }
                PauseAction::Idle => {}
            }
            self.music.follow_pause(ctx, self.pause.is_active());
            if self.pause.is_active() {
                // A dip frozen under the menu would thaw out of step with the song.
                self.music.rest_bob(ctx.world, &self.invaders);
                // The engine holds the world still; only the collider overlay is ours to draw.
                self.emit_collider_overlay(ctx);
                return;
            }
        }

        if self.state == GameState::Playing {
            self.update_cannons(ctx);
        }
        self.physics.update(ctx.world, ctx.delta_time);

        // Drain this frame's collision events once (take = the buffer is consumed, not
        // borrowed), then drop every contact of a shot passing a bagel's hole before
        // anything reads them.
        let collisions = self.physics.take_collision_events();
        let collisions = self.filter_passing_contacts(ctx.world, collisions);

        self.handle_state_input(ctx);
        if self.state == GameState::Playing {
            self.tick_notice(ctx.delta_time);
            self.update_firing(ctx);
            self.update_invader_fire(ctx);
            self.update_flyer(ctx);
            self.march_formation(ctx);
            self.resolve_player_shots(ctx, &collisions);
            self.check_flyer_hit(ctx, &collisions);
            self.resolve_enemy_shots(ctx, &collisions);
            self.cancel_crossing_shots(ctx);
            self.cull_stray_shots(ctx);
            self.check_invasion(ctx);
            self.check_win_condition(ctx);
        }
        // A frame that ended the match has put the fleet at rest behind the panel; a dip
        // written after it would be drawn there with no beat under it.
        if self.state == GameState::Playing {
            self.music.bob(ctx, &self.invaders);
        }

        self.emit_collider_overlay(ctx);
    }

    /// Outline every collider in bright magenta while F1 is on — the only line-buffer work
    /// the game owns, drawn over the art.
    pub(crate) fn emit_collider_overlay(&self, ctx: &mut GameContext) {
        if self.debug_colliders {
            debug::draw_colliders(ctx.world, ctx.lines, DEBUG_COLLIDER_COLOR, DEBUG_COLLIDER_EMISSIVE);
        }
    }

    /// Run down the HUD's transient slot.
    fn tick_notice(&mut self, delta_time: f32) {
        if let Some((_, seconds_left)) = self.notice.as_mut() {
            *seconds_left -= delta_time;
        }
        if self.notice.is_some_and(|(_, seconds_left)| seconds_left <= 0.0) {
            self.notice = None;
        }
    }

    /// Show `notice` in the HUD's transient slot, replacing whatever was there.
    pub(crate) fn show_notice(&mut self, notice: Notice) {
        self.notice = Some((notice, NOTICE_SECONDS));
    }

    /// Gameplay sprites only show in a match (playing or game over); the countertop stays
    /// up under the menus. A cannon waiting out a surviving hit is
    /// hidden, and each life icon shows only while its index is below its player's lives.
    pub(crate) fn update_entity_visibility(&self, ctx: &mut GameContext) {
        let in_match = matches!(self.state, GameState::Playing | GameState::GameOver { .. });
        let entities: Vec<EntityId> = self.invaders.iter().map(|invader| invader.entity)
            .chain(self.flyer.as_ref().map(|flyer| flyer.entity))
            .chain(self.bunker_blocks.iter().map(|block| block.entity))
            .chain(self.players.iter().flat_map(|player| player.shots.iter().copied()))
            .chain(self.enemy_shots.iter().map(|shot| shot.entity))
            .chain(self.transient_visuals.iter().copied())
            .collect();
        set_sprites_visible(ctx.world, entities, in_match);

        for player in &self.players {
            if let Some(entity) = player.entity {
                set_sprites_visible(ctx.world, [entity], in_match && player.respawn_in.is_none());
            }
        }
        for &(player, index, entity) in &self.life_icons {
            let lives = self.players.get(player).map(|state| state.lives).unwrap_or(0);
            set_sprites_visible(ctx.world, [entity], in_match && index < lives);
        }
    }
}
