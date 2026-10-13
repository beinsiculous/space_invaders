//! Shots and what they hit: the cannons' trigger discipline, the formation's crumbs,
//! Bagelgeist's hole, Sour-Doom's two hits, and the shots cancelling in mid-air.

use engine_core::prelude::*;

use crate::art::*;
use crate::constants::*;
use crate::spawning::{spawn_effect, spawn_shot, Effect, ShotRecipe};
use crate::types::*;

use super::bunkers::Chip;
use super::players::ShotOutcome;
use super::{entity_position, rects_overlap, ripple_grid};

/// Which live column returns fire, from a pseudo-random draw. `live_columns` must be
/// non-empty; the draw wraps over it.
pub(crate) fn pick_shooter_column(live_columns: &[usize], rand: u32) -> usize {
    live_columns[rand as usize % live_columns.len()]
}

/// The player's trigger discipline for a chaos mode: `(shots_per_fire, max_live_shots)`.
///
/// Normal keeps the classic one shot in flight. Insane answers the faster fleet with
/// more single shots on screen; Ridiculous the relentless fire with a twin volley, one
/// in flight; Insiculous faces both, so it stacks twin volleys. An explicit match (not
/// the `is_*` predicates — both are true for Insiculous) so each cap reads directly.
pub(crate) fn player_fire_caps(mode: ChaosMode) -> (usize, usize) {
    match mode {
        ChaosMode::Normal => (1, MAX_PLAYER_SHOTS),
        ChaosMode::Insane => (1, INSANE_MAX_PLAYER_SHOTS),
        ChaosMode::Ridiculous => (2, RIDICULOUS_MAX_PLAYER_SHOTS),
        ChaosMode::Insiculous => (2, INSICULOUS_MAX_PLAYER_SHOTS),
    }
}

/// Average Toasted shots per second for a chaos mode: Ridiculous (and Insiculous) fleets
/// fire relentlessly; Insane's buff is march speed, not trigger-happiness.
pub(crate) fn invader_fire_rate(mode: ChaosMode) -> f32 {
    if mode.is_ridiculous() {
        INVADER_FIRE_RATE * RIDICULOUS_FIRE_MULT
    } else {
        INVADER_FIRE_RATE
    }
}

/// Whether a shot whose centre line is at `shot_x` goes through the hole of a Bagelgeist
/// whose cell's left edge is at `cell_left`.
pub(crate) fn passes_through_hole(shot_x: f32, cell_left: f32) -> bool {
    let (first, past_last) = BAGELGEIST_HOLE_COLUMNS;
    shot_x >= cell_left + first && shot_x < cell_left + past_last
}

impl SpaceInvadersGame {
    /// The formation's return fire: a per-frame pseudo-random draw against the fleet's
    /// fire rate; the crumb drops from the lowest Toasted of a pseudo-randomly chosen
    /// live column.
    pub(crate) fn update_invader_fire(&mut self, ctx: &mut GameContext) {
        if self.invaders.is_empty() {
            return;
        }
        if hash_f32(self.frame_count) >= invader_fire_rate(self.chaos_mode) * ctx.delta_time {
            return;
        }

        let mut live_columns: Vec<usize> = self.invaders.iter().map(|invader| invader.column).collect();
        live_columns.sort_unstable();
        live_columns.dedup();
        let column = pick_shooter_column(&live_columns, hash_u32(self.frame_count));

        let shooter = self.invaders.iter()
            .filter(|invader| invader.column == column)
            .max_by_key(|invader| invader.row);
        let Some(shooter) = shooter else { return };
        let half = half_size(shooter.kind.spec().sheet);
        if let Some(position) = entity_position(ctx.world, shooter.entity) {
            let below = position - Vec2::new(0.0, half.y + half_size(&BURNT_CRUMB).y + 1.0);
            self.spawn_enemy_shot(ctx.world, EnemyShotKind::Crumb, below, CRUMB_SPEED);
        }
    }

    /// Spawn a crumb or a glob at `position`, falling at `speed`.
    pub(crate) fn spawn_enemy_shot(&mut self, world: &mut World, kind: EnemyShotKind, position: Vec2, speed: f32) {
        let (spec, depth, name) = match kind {
            EnemyShotKind::Crumb => (&BURNT_CRUMB, BURNT_CRUMB_DEPTH, "Burnt Crumb"),
            EnemyShotKind::JamGlob => (&JAM_GLOB, JAM_GLOB_DEPTH, "Jam Glob"),
        };
        let recipe = ShotRecipe { name, spec, sheet: self.sheets.enemy_shot(kind), depth, half_collider: half_size(spec) };
        let entity = spawn_shot(world, &mut self.physics, &recipe, position, Vec2::new(0.0, -speed));
        self.enemy_shots.push(EnemyShot { entity, kind });
    }

    /// Keep the passing pairs current — a shot that first touches a Bagelgeist inside its
    /// hole passes that bagel until the two part or either goes — and drop every contact
    /// of a passing pair before anything resolves hits. Deciding once, at first contact,
    /// keeps the hole a hole at speed: a fixed gap would slide sideways under a fast shot
    /// crossing a moving bagel.
    pub(crate) fn filter_passing_contacts(&mut self, world: &World, collisions: Vec<CollisionData>) -> Vec<CollisionData> {
        let player_shots: Vec<EntityId> = self.players.iter().flat_map(|player| player.shots.iter().copied()).collect();
        let bagels: Vec<EntityId> = self.invaders.iter()
            .filter(|invader| invader.kind == Toasted::Bagelgeist)
            .map(|invader| invader.entity)
            .collect();
        let alive = |entity: EntityId| world.get::<Transform2D>(entity).is_some();

        self.passing.retain(|&(shot, bagel)| {
            alive(shot) && alive(bagel)
                && !collisions.iter().any(|collision| collision.event.stopped && collision.event.involves(shot, bagel))
        });
        for collision in collisions.iter().filter(|collision| collision.event.started) {
            for &shot in &player_shots {
                let Some(bagel) = collision.event.other(shot) else { continue };
                if !bagels.contains(&bagel) || self.passing.contains(&(shot, bagel)) {
                    continue;
                }
                let (Some(shot_position), Some(bagel_position)) = (entity_position(world, shot), entity_position(world, bagel)) else {
                    continue;
                };
                let cell_left = bagel_position.x + BAGELGEIST.sprite_offset().x - BAGELGEIST.cell.x / 2.0;
                if passes_through_hole(shot_position.x, cell_left) {
                    self.passing.push((shot, bagel));
                }
            }
        }

        collisions.into_iter()
            .filter(|collision| !self.passing.iter().any(|&(shot, bagel)| collision.event.involves(shot, bagel)))
            .collect()
    }

    /// Resolve every cannon's shots against this frame's started contacts: a Toasted
    /// struck, or a bunker chipped. Each shot is spent by its first hit.
    pub(crate) fn resolve_player_shots(&mut self, ctx: &mut GameContext, collisions: &[CollisionData]) {
        for index in 0..self.players.len() {
            let shots: Vec<EntityId> = self.players[index].shots.clone();
            for shot in shots {
                let started = |other: EntityId| {
                    collisions.iter().any(|collision| collision.event.started && collision.event.involves(shot, other))
                };
                if let Some(target) = self.invaders.iter().position(|invader| started(invader.entity)) {
                    let outcome = self.strike_invader(ctx, index, target, shot);
                    self.finish_player_shot(ctx, index, shot, outcome);
                    continue;
                }
                let block = self.bunker_blocks.iter().copied().find(|block| started(block.entity));
                if let Some(block) = block {
                    if let Some(position) = entity_position(ctx.world, shot) {
                        self.spawn_ice_burst(ctx.world, position);
                    }
                    self.damage_bunker(ctx, block, Chip::Butter);
                    self.finish_player_shot(ctx, index, shot, ShotOutcome::Miss);
                }
            }
        }
    }

    /// Cannon `index`'s shot struck the Toasted at `target`. A Sour-Doom with a hit to
    /// spare winces through `hurt` and pays nothing; anything else is busted into
    /// croutons and pays its points.
    fn strike_invader(&mut self, ctx: &mut GameContext, index: usize, target: usize, shot: EntityId) -> ShotOutcome {
        if self.invaders[target].hits_left > 1 {
            self.invaders[target].hits_left -= 1;
            set_clip_state(ctx.world, self.invaders[target].entity, HURT);
            if let Some(position) = entity_position(ctx.world, shot) {
                self.spawn_ice_burst(ctx.world, position);
            }
            return ShotOutcome::Absorbed;
        }

        let invader = self.invaders.remove(target);
        let spec = invader.kind.spec();
        self.players[index].score += spec.points;
        if let Some(position) = entity_position(ctx.world, invader.entity) {
            let theme = ChaosTheme::for_mode(self.chaos_mode);
            ctx.particles.spawn_burst(position, &crate::effects::crouton_burst(spec.burst_color, &theme, self.sheets.white));
            let burst = spawn_effect(ctx.world, &Effect {
                name: "Crouton Burst", spec: &CROUTON_BURST, sheet: &self.sheets.crouton_burst,
                depth: CROUTON_BURST_DEPTH, clip: BURST_DIE, position, facing: Facing::Right,
            }, false);
            self.transient_visuals.push(burst);
            ripple_grid(ctx.world, position, GRID_IMPULSE_KILL_STRENGTH, GRID_IMPULSE_KILL_RADIUS);
        }
        self.physics.destroy_entity(ctx.world, invader.entity);
        ShotOutcome::Kill
    }

    pub(crate) fn spawn_ice_burst(&mut self, world: &mut World, position: Vec2) {
        let burst = spawn_effect(world, &Effect {
            name: "Ice Burst", spec: &ICE_BURST, sheet: &self.sheets.ice_burst,
            depth: ICE_BURST_DEPTH, clip: BURST_HIT, position, facing: Facing::Right,
        }, false);
        self.transient_visuals.push(burst);
    }

    /// Resolve the crumbs and globs: a crumb on a live cannon costs it a life, a glob
    /// jams it; either chips a bunker block it touches. A glob passes the Toasted, and
    /// a cannon waiting out a hit is never struck.
    pub(crate) fn resolve_enemy_shots(&mut self, ctx: &mut GameContext, collisions: &[CollisionData]) {
        let mut spent: Vec<EntityId> = Vec::new();
        let mut hit: Vec<usize> = Vec::new();
        let mut jammed: Vec<usize> = Vec::new();
        let mut chips: Vec<(BunkerBlock, Chip)> = Vec::new();
        for shot in &self.enemy_shots {
            let started = |other: EntityId| {
                collisions.iter().any(|collision| collision.event.started && collision.event.involves(shot.entity, other))
            };
            let struck = self.players.iter().position(|player| {
                player.is_active() && player.entity.is_some_and(started)
            });
            if let Some(index) = struck {
                spent.push(shot.entity);
                match shot.kind {
                    EnemyShotKind::Crumb => hit.push(index),
                    EnemyShotKind::JamGlob => jammed.push(index),
                }
                continue;
            }
            if let Some(block) = self.bunker_blocks.iter().copied().find(|block| started(block.entity)) {
                spent.push(shot.entity);
                chips.push((block, if shot.kind == EnemyShotKind::JamGlob { Chip::Jam } else { Chip::Butter }));
            }
        }

        for shot in spent {
            self.enemy_shots.retain(|live| live.entity != shot);
            self.physics.destroy_entity(ctx.world, shot);
        }
        for (block, chip) in chips {
            self.damage_bunker(ctx, block, chip);
        }
        for index in jammed {
            self.jam_cannon(index);
        }
        // One crumb a frame is a hit; interleaved hits ([0, 1, 0]) must not cost a life twice.
        hit.sort_unstable();
        hit.dedup();
        for index in hit {
            self.player_hit(ctx, index);
        }
    }

    /// Shots flying opposite ways cancel each other in a flash. Both are dynamic sensors,
    /// so rapier would report the pair, but at closing speeds a whole frame can step past
    /// the overlap — a rectangle test inflated by half a frame of closing speed is the
    /// reliable arcade answer.
    pub(crate) fn cancel_crossing_shots(&mut self, ctx: &mut GameContext) {
        if self.enemy_shots.is_empty() {
            return;
        }
        let theme = ChaosTheme::for_mode(self.chaos_mode);
        let mut cancelled: Vec<(usize, EntityId, EntityId)> = Vec::new();
        for index in 0..self.players.len() {
            let hero = self.players[index].hero;
            for &mine in &self.players[index].shots {
                let Some(my_position) = entity_position(ctx.world, mine) else { continue };
                let crossing = self.enemy_shots.iter().find(|theirs| {
                    let speed = if theirs.kind == EnemyShotKind::JamGlob { JAM_GLOB_SPEED } else { CRUMB_SPEED };
                    let pad = (PLAYER_SHOT_SPEED + speed) * ctx.delta_time / 2.0;
                    let my_half = player_shot_half_collider(hero) + Vec2::new(0.0, pad);
                    let their_spec = if theirs.kind == EnemyShotKind::JamGlob { &JAM_GLOB } else { &BURNT_CRUMB };
                    cancelled.iter().all(|&(_, _, taken)| taken != theirs.entity)
                        && entity_position(ctx.world, theirs.entity)
                            .is_some_and(|their_position| rects_overlap(my_position, my_half, their_position, half_size(their_spec)))
                });
                if let Some(theirs) = crossing {
                    ctx.particles.spawn_burst(my_position, &crate::effects::shot_cancel_burst(&theme, self.sheets.white));
                    cancelled.push((index, mine, theirs.entity));
                }
            }
        }
        for (index, mine, theirs) in cancelled {
            self.finish_player_shot(ctx, index, mine, ShotOutcome::Miss);
            self.enemy_shots.retain(|shot| shot.entity != theirs);
            self.physics.destroy_entity(ctx.world, theirs);
        }
    }

    /// Remove shots that left the playfield or were despawned by their `Lifetime` safety
    /// net. A player's shot ending this way is a miss.
    pub(crate) fn cull_stray_shots(&mut self, ctx: &mut GameContext) {
        let top = WIN_H / 2.0 + SHOT_CULL_PAD;
        let bottom = -(WIN_H / 2.0 + SHOT_CULL_PAD);

        for index in 0..self.players.len() {
            let stray: Vec<EntityId> = self.players[index].shots.iter().copied()
                .filter(|&shot| entity_position(ctx.world, shot).is_none_or(|position| !position.y.is_finite() || position.y > top))
                .collect();
            for shot in stray {
                self.finish_player_shot(ctx, index, shot, ShotOutcome::Miss);
            }
        }

        let stray: Vec<EntityId> = self.enemy_shots.iter().map(|shot| shot.entity)
            .filter(|&shot| entity_position(ctx.world, shot).is_none_or(|position| !position.y.is_finite() || position.y < bottom))
            .collect();
        for shot in stray {
            self.enemy_shots.retain(|live| live.entity != shot);
            self.physics.destroy_entity(ctx.world, shot);
        }
    }

    /// Victory the moment the last Toasted is busted.
    pub(crate) fn check_win_condition(&mut self, ctx: &mut GameContext) {
        if self.state == GameState::Playing && self.invaders.is_empty() {
            self.unlock_win_achievements(ctx);
            self.finish_game(ctx, true);
        }
    }
}
