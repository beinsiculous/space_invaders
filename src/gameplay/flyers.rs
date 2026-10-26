//! The flyby lane: one flyer at a time. Each spawn roll sends Maxwell's cake saucer
//! three times in four and Jammer once, and the flyer's kind branches everything it
//! does — the saucer pays a mystery bonus and never fires; Jammer drops jam globs that
//! jam a cannon, and pays a flat bonus.

use engine_core::prelude::*;

use crate::art::*;
use crate::constants::*;
use crate::spawning::{flyer_art, spawn_effect, spawn_flyer, Effect};
use crate::music::Moment;
use crate::types::*;

use super::players::ShotOutcome;
use super::entity_position;

// Distinct hash streams so the flyby draws don't correlate with the formation's fire
// (which draws on the bare frame count).
const SPAWN_SALT: u32 = 4093;
const DIRECTION_SALT: u32 = 8191;
const BONUS_SALT: u32 = 3301;
const KIND_SALT: u32 = 6113;

/// Which flyer a spawn roll sends: Jammer on one draw in four, the saucer otherwise.
pub(crate) fn flyer_kind_for(rand: u32) -> FlyerKind {
    if rand % 4 < JAMMER_SHARE_OF_FOUR { FlyerKind::Jammer } else { FlyerKind::Saucer }
}

/// Where a flyer's cell enters and which way it flies, from a pseudo-random draw: even →
/// the left edge flying right, odd → the right edge flying left. It starts a full cell
/// off screen, so it slides in instead of popping.
pub(crate) fn flyer_entry(rand: u32, kind: FlyerKind) -> (f32, f32) {
    let direction = if rand.is_multiple_of(2) { 1.0 } else { -1.0 };
    (-direction * (WIN_W / 2.0 + flyer_art(kind).0.cell.x), direction)
}

/// A flyer is done once its cell has fully crossed the far edge.
pub(crate) fn flyer_offscreen(cell_x: f32, direction: f32, kind: FlyerKind) -> bool {
    cell_x * direction > WIN_W / 2.0 + flyer_art(kind).0.cell.x
}

/// Whether the empty lane sends a flyer on frame `frame`: a pseudo-random draw against the
/// spawn rate.
pub(crate) fn flyer_spawn_rolls(frame: u32, delta_time: f32) -> bool {
    hash_f32(frame.wrapping_add(SPAWN_SALT)) < FLYER_SPAWN_RATE * delta_time
}

/// The saucer's mystery bonus for a kill, drawn from the classic table.
pub(crate) fn saucer_bonus(rand: u32) -> u32 {
    SAUCER_BONUS_VALUES[rand as usize % SAUCER_BONUS_VALUES.len()]
}

impl SpaceInvadersGame {
    /// Roll for a flyer when the lane is empty; fly the one in it, dropping Jammer's globs
    /// while it is over the field, and retire it past the far edge.
    pub(crate) fn update_flyer(&mut self, ctx: &mut GameContext) {
        let Some(flyer) = self.flyer.as_mut() else {
            if flyer_spawn_rolls(self.frame_count, ctx.delta_time) {
                let kind = flyer_kind_for(hash_u32(self.frame_count.wrapping_add(KIND_SALT)));
                let (cell_x, direction) = flyer_entry(hash_u32(self.frame_count.wrapping_add(DIRECTION_SALT)), kind);
                let entity = spawn_flyer(ctx.world, self.sheets.flyer(kind), kind, cell_x);
                self.flyer = Some(Flyer { entity, kind, direction, glob_in: JAM_GLOB_INTERVAL });
                self.music.play(ctx, Moment::Flyer(entity));
            }
            return;
        };

        let (entity, kind, direction) = (flyer.entity, flyer.kind, flyer.direction);
        let Some(position) = entity_position(ctx.world, entity) else {
            self.flyer = None;
            return;
        };
        let spec = flyer_art(kind).0;
        let x = position.x + direction * FLYER_SPEED * ctx.delta_time;
        if flyer_offscreen(x + spec.sprite_offset().x, direction, kind) {
            self.destroy_flyer(ctx.world);
            return;
        }
        self.physics.set_kinematic_target(entity, Vec2::new(x, position.y), 0.0);

        if kind == FlyerKind::Jammer {
            flyer.glob_in -= ctx.delta_time;
            let over_field = x.abs() < WIN_W / 2.0;
            if flyer.glob_in <= 0.0 && over_field {
                flyer.glob_in = JAM_GLOB_INTERVAL;
                let below = Vec2::new(x, position.y - half_size(spec).y - half_size(&JAM_GLOB).y - 1.0);
                self.spawn_enemy_shot(ctx.world, EnemyShotKind::JamGlob, below, JAM_GLOB_SPEED);
            }
        }
    }

    /// Pay out a player's shot that struck the flyer. The saucer pays its mystery bonus
    /// and earns Saucer Hunter; Jammer pays its flat bonus and splatters. Either counts as a
    /// kill for the streak.
    pub(crate) fn check_flyer_hit(&mut self, ctx: &mut GameContext, collisions: &[CollisionData]) {
        let Some(flyer) = self.flyer.as_ref() else { return };
        let (entity, kind) = (flyer.entity, flyer.kind);
        let hit = (0..self.players.len()).find_map(|index| {
            self.players[index].shots.iter().copied()
                .find(|&shot| collisions.iter().any(|collision| collision.event.started && collision.event.involves(shot, entity)))
                .map(|shot| (index, shot))
        });
        let Some((index, shot)) = hit else { return };

        let position = entity_position(ctx.world, entity);
        match kind {
            FlyerKind::Saucer => {
                let bonus = saucer_bonus(hash_u32(self.frame_count.wrapping_add(BONUS_SALT)));
                self.players[index].score += bonus;
                self.show_notice(Notice::SaucerBonus(bonus));
                ctx.achievements.unlock(crate::achievements::UFO_HUNTER);
                if let Some(position) = position {
                    let theme = ChaosTheme::for_mode(self.chaos_mode);
                    ctx.particles.spawn_burst(position, &crate::effects::saucer_burst(&theme, self.sheets.white));
                }
            }
            FlyerKind::Jammer => {
                self.players[index].score += JAMMER_POINTS;
                self.show_notice(Notice::JammerBusted);
                if let Some(position) = position {
                    let splatter = spawn_effect(ctx.world, &Effect {
                        name: "Jam Splatter", spec: &JAM_SPLATTER, sheet: &self.sheets.jam_splatter,
                        depth: JAM_SPLATTER_DEPTH, clip: SPLAT, position, facing: Facing::Right,
                    }, false);
                    self.transient_visuals.push(splatter);
                }
            }
        }
        self.destroy_flyer(ctx.world);
        self.music.play(ctx, Moment::Bonus);
        self.finish_player_shot(ctx, index, shot, ShotOutcome::Kill);
    }

    pub(crate) fn destroy_flyer(&mut self, world: &mut World) {
        if let Some(flyer) = self.flyer.take() {
            self.physics.destroy_entity(world, flyer.entity);
        }
    }
}
