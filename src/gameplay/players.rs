//! The defending cannons — Deion, and Cubert in co-op: movement and facing, the
//! trigger, being jammed, and taking a hit. Everything is parameterized by player index,
//! so single player (one cannon) and co-op (two) share one code path.

use engine_core::prelude::*;

use crate::art::*;
use crate::constants::*;
use crate::spawning::{face, spawn_effect, spawn_shot, Effect, ShotRecipe};
use crate::music::Moment;
use crate::types::*;

use super::combat::player_fire_caps;
use super::{clip_state, entity_position};

/// Whether a fresh volley of `shots` fits under `max_live` given the cannon's own shots
/// already in flight. Each cannon checks its own, so one player firing at cap never
/// blocks the other.
pub(crate) fn volley_fits(live_shots: usize, shots: usize, max_live: usize) -> bool {
    live_shots + shots <= max_live
}

/// The co-op match is lost only when every cannon is out of lives. An empty roster (no
/// match in progress) is not a defeat.
pub(crate) fn coop_defeated(lives: &[u32]) -> bool {
    !lives.is_empty() && lives.iter().all(|&remaining| remaining == 0)
}

/// How a player's shot ended, for the sharpshooter streak.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShotOutcome {
    /// It busted something: the streak grows.
    Kill,
    /// It struck a Sour-Doom that took the hit: neither counted nor reset.
    Absorbed,
    /// A miss, a bunker or a cancel: the streak resets.
    Miss,
}

impl SpaceInvadersGame {
    /// Drive every cannon from its player's input.
    pub(crate) fn update_cannons(&mut self, ctx: &mut GameContext) {
        for index in 0..self.players.len() {
            self.update_one_cannon(ctx, index);
        }
    }

    /// Move cannon `index` and turn it the way it moved. Keys or stick take over whenever
    /// pressed; the mouse takes over on movement but drives only the lone solo cannon.
    /// A cannon waiting out a hit does nothing until its respawn, which is authoritative:
    /// the body and the game's own x both go to its spawn.
    fn update_one_cannon(&mut self, ctx: &mut GameContext, index: usize) {
        let Some(entity) = self.players[index].entity else { return };
        let hero = self.players[index].hero;
        self.players[index].jammed_for = (self.players[index].jammed_for - ctx.delta_time).max(0.0);

        if let Some(seconds_left) = self.players[index].respawn_in {
            let seconds_left = seconds_left - ctx.delta_time;
            if seconds_left > 0.0 {
                self.players[index].respawn_in = Some(seconds_left);
            } else {
                self.respawn_cannon(ctx.world, index);
            }
            return;
        }

        let x = self.players[index].x;
        let key_dx = self.cannon_axis(ctx, index) * PLAYER_SPEED;
        let mouse_active = index == 0
            && self.mode == GameMode::SinglePlayer
            && ctx.input.mouse_movement_delta().0.abs() > 0.0;
        let new_x = if key_dx != 0.0 {
            x + key_dx * ctx.delta_time
        } else if mouse_active {
            // Window pixels (origin top-left) → world (origin centre).
            ctx.input.mouse_position().x - ctx.window_size.x / 2.0
        } else {
            x
        };
        let max_x = cannon_max_x(hero);
        let new_x = new_x.clamp(-max_x, max_x);

        let moved = new_x != x;
        if moved {
            let facing = if new_x < x { Facing::Left } else { Facing::Right };
            self.players[index].facing = facing;
            face(ctx.world, entity, hero.spec().sheet, facing);
        }
        self.players[index].x = new_x;
        self.physics.set_kinematic_target(entity, Vec2::new(new_x, cannon_y(hero)), 0.0);

        // A shot's `fire` plays out: movement never cuts it short.
        if clip_state(ctx.world, entity).as_deref() != Some(FIRE) {
            set_clip_state(ctx.world, entity, if moved { WALK } else { IDLE });
        }
    }

    /// Bring cannon `index` back from a surviving hit: at its spawn, facing right, at
    /// rest. Authoritative — the body is reset and the game's own x set, so the next
    /// frame's movement starts from the spawn, not from where the cannon fell.
    pub(super) fn respawn_cannon(&mut self, world: &mut World, index: usize) {
        let Some(entity) = self.players[index].entity else { return };
        let hero = self.players[index].hero;
        let spawn_x = cannon_spawn_x(self.players.len(), index);
        let player = &mut self.players[index];
        player.respawn_in = None;
        player.x = spawn_x;
        player.facing = Facing::Right;
        self.physics.reset_body(entity, Vec2::new(spawn_x, cannon_y(hero)));
        face(world, entity, hero.spec().sheet, Facing::Right);
        set_clip_state(world, entity, IDLE);
    }

    /// Merged horizontal axis for cannon `index`. The solo cannon answers to both
    /// players' devices; each co-op cannon to its own.
    fn cannon_axis(&self, ctx: &GameContext, index: usize) -> f32 {
        match self.mode {
            GameMode::SinglePlayer => (ctx.players.move_x(PlayerId::P1, ctx.input)
                + ctx.players.move_x(PlayerId::P2, ctx.input))
            .clamp(-1.0, 1.0),
            GameMode::TwoPlayerCoop => ctx.players.move_x(PlayerId(index as u8), ctx.input),
        }
    }

    /// Is the fire button held for cannon `index`? The solo cannon fires on either
    /// player's Action1; each co-op cannon on its own.
    fn fire_held(&self, ctx: &GameContext, index: usize) -> bool {
        match self.mode {
            GameMode::SinglePlayer => {
                ctx.players.is_active(PlayerId::P1, GameAction::Action1, ctx.input)
                    || ctx.players.is_active(PlayerId::P2, GameAction::Action1, ctx.input)
            }
            GameMode::TwoPlayerCoop => {
                ctx.players.is_active(PlayerId(index as u8), GameAction::Action1, ctx.input)
            }
        }
    }

    /// Where cannon `index`'s shot leaves from: the top of the mohawk or of the cube,
    /// mirrored with the facing.
    pub(crate) fn muzzle(&self, index: usize) -> Vec2 {
        let player = &self.players[index];
        let offset = player.hero.muzzle_offset();
        Vec2::new(player.x + offset.x * player.facing.sign(), cannon_y(player.hero) + offset.y)
    }

    /// Hold-to-fire for every cannon; each keeps its own cooldown and shot cap. The chaos
    /// mode sets the trigger discipline (see [`player_fire_caps`]).
    pub(crate) fn update_firing(&mut self, ctx: &mut GameContext) {
        for index in 0..self.players.len() {
            self.update_one_cannon_fire(ctx, index);
        }
    }

    fn update_one_cannon_fire(&mut self, ctx: &mut GameContext, index: usize) {
        self.players[index].fire_cooldown = (self.players[index].fire_cooldown - ctx.delta_time).max(0.0);

        let player = &self.players[index];
        if !player.is_active() || player.jammed_for > 0.0 || player.fire_cooldown > 0.0 || !self.fire_held(ctx, index) {
            return;
        }
        let (shots, max_live) = player_fire_caps(self.chaos_mode);
        if !volley_fits(player.shots.len(), shots, max_live) {
            return;
        }
        let Some(entity) = player.entity else { return };

        let muzzle = self.muzzle(index);
        let offsets: &[f32] = if shots == 2 { &[-TWIN_VOLLEY_OFFSET, TWIN_VOLLEY_OFFSET] } else { &[0.0] };
        for &offset in offsets {
            self.spawn_player_shot(ctx.world, index, Vec2::new(muzzle.x + offset, muzzle.y));
        }
        self.players[index].fire_cooldown = FIRE_COOLDOWN;
        set_clip_state(ctx.world, entity, FIRE);
        self.music.play(ctx, Moment::Shot(entity));
    }

    /// Spawn one of cannon `index`'s shots with its tail at `muzzle`.
    pub(crate) fn spawn_player_shot(&mut self, world: &mut World, index: usize, muzzle: Vec2) {
        let hero = self.players[index].hero;
        let spec = hero.spec();
        let recipe = ShotRecipe {
            name: if hero == Hero::Deion { "Icicle" } else { "Ice Chip" },
            spec: spec.shot,
            sheet: self.sheets.hero_shot(hero),
            depth: spec.shot_depth,
            half_collider: player_shot_half_collider(hero),
        };
        let position = muzzle + Vec2::new(0.0, half_size(spec.shot).y);
        let shot = spawn_shot(world, &mut self.physics, &recipe, position, Vec2::new(0.0, PLAYER_SHOT_SPEED));
        self.players[index].shots.push(shot);
    }

    /// Cannon `index` took a crumb: clear every shot, spend one of its lives, and play
    /// its `hurt` as a detached one-shot at its spot and facing — it never collides and
    /// nothing preempts it. A surviving cannon is hidden and inert for the hurt's length,
    /// then reappears at its spawn; on the last life the hurt holds its final frame until
    /// a restart or a quit, the cannon goes, and the match ends once every cannon is out.
    pub(super) fn player_hit(&mut self, ctx: &mut GameContext, index: usize) {
        if !self.players[index].is_active() {
            return;
        }
        let Some(entity) = self.players[index].entity else { return };
        let hero = self.players[index].hero;
        let facing = self.players[index].facing;
        let position = entity_position(ctx.world, entity).unwrap_or(Vec2::new(self.players[index].x, cannon_y(hero)));
        let theme = ChaosTheme::for_mode(self.chaos_mode);
        ctx.particles.spawn_burst(position, &crate::effects::player_hit_burst(&theme, self.sheets.white));

        self.destroy_all_shots(ctx.world);
        let player = &mut self.players[index];
        player.shot_streak = 0;
        player.jammed_for = 0.0;
        player.lives = player.lives.saturating_sub(1);
        let last_life = player.lives == 0;

        let spec = hero.spec();
        let hurt = spawn_effect(ctx.world, &Effect {
            name: "Hurt", spec: spec.sheet, sheet: self.sheets.hero(hero), depth: spec.depth,
            clip: HURT, position, facing,
        }, last_life);
        self.transient_visuals.push(hurt);
        self.music.play(ctx, Moment::Hit);

        if last_life {
            if let Some(entity) = self.players[index].entity.take() {
                self.physics.destroy_entity(ctx.world, entity);
            }
            let lives: Vec<u32> = self.players.iter().map(|player| player.lives).collect();
            if coop_defeated(&lives) {
                self.finish_game(ctx, false);
            }
        } else {
            self.players[index].respawn_in = Some(RESPAWN_SECONDS);
        }
    }

    /// Cannon `index` took a jam glob: no shot until the jam runs out, a second glob
    /// refreshing it. No life is lost.
    pub(super) fn jam_cannon(&mut self, index: usize) {
        self.players[index].jammed_for = JAM_SECONDS;
        self.show_notice(Notice::Jammed { player: index });
    }

    /// Retire one of cannon `index`'s shots and account for it.
    pub(super) fn finish_player_shot(&mut self, ctx: &mut GameContext, index: usize, shot: EntityId, outcome: ShotOutcome) {
        self.players[index].shots.retain(|&live| live != shot);
        self.physics.destroy_entity(ctx.world, shot);
        match outcome {
            ShotOutcome::Kill => {
                self.players[index].shot_streak += 1;
                if self.players[index].shot_streak == SHARPSHOOTER_TARGET {
                    ctx.achievements.unlock(crate::achievements::SHARPSHOOTER);
                }
            }
            ShotOutcome::Absorbed => {}
            ShotOutcome::Miss => self.players[index].shot_streak = 0,
        }
    }
}
