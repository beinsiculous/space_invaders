//! Formation marching: the fleet moves as one shared offset applied to every Toasted's
//! home slot. March right until the outermost live column touches the bound, then
//! descend one step and reverse — the classic accelerating heartbeat as the fleet thins.

use engine_core::prelude::*;

use crate::art::half_size;
use crate::constants::*;
use crate::spawning::invader_home;
use crate::types::*;

use super::{entity_position, rects_overlap};

/// Horizontal march speed for the current fleet strength: ramps linearly from
/// `MARCH_SPEED_BASE` (full fleet) to `MARCH_SPEED_MAX` (last Toasted). Insane mode
/// multiplies the result.
pub(crate) fn march_speed(alive: usize, total: usize, insane: bool) -> f32 {
    let total = total.max(1);
    let alive = alive.clamp(1, total);
    // 0.0 with the fleet intact → 1.0 when one Toasted remains.
    let thinned = (total - alive) as f32 / total.max(2).saturating_sub(1) as f32;
    let speed = MARCH_SPEED_BASE + (MARCH_SPEED_MAX - MARCH_SPEED_BASE) * thinned;
    if insane { speed * INSANE_MARCH_MULT } else { speed }
}

/// One march tick's outcome: the new horizontal offset and direction, and whether the
/// fleet hit a bound and must descend one step.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct MarchOutcome {
    pub(crate) offset_x: f32,
    pub(crate) dir: f32,
    pub(crate) descended: bool,
}

/// Advance the formation offset by `dx` in direction `dir`, bouncing off
/// `MARCH_BOUND_X`. `live_min_home_x` / `live_max_home_x` are the cell centres of the
/// outermost LIVE columns — dead columns don't shrink the playfield, they widen the
/// march span.
pub(crate) fn march_step(
    offset_x: f32,
    dir: f32,
    dx: f32,
    live_min_home_x: f32,
    live_max_home_x: f32,
) -> MarchOutcome {
    let new_x = offset_x + dir * dx;
    if dir > 0.0 && live_max_home_x + new_x >= MARCH_BOUND_X {
        MarchOutcome { offset_x: MARCH_BOUND_X - live_max_home_x, dir: -1.0, descended: true }
    } else if dir < 0.0 && live_min_home_x + new_x <= -MARCH_BOUND_X {
        MarchOutcome { offset_x: -MARCH_BOUND_X - live_min_home_x, dir: 1.0, descended: true }
    } else {
        MarchOutcome { offset_x: new_x, dir, descended: false }
    }
}

/// Has the fleet landed? True when any live Toasted's bottom edge has reached
/// `INVASION_Y` — the top of the taller cannon's envelope, above every cannon, so it is
/// the only landing rule.
pub(crate) fn fleet_has_landed(bottom_edges: &[f32]) -> bool {
    bottom_edges.iter().any(|&bottom| bottom <= INVASION_Y)
}

impl SpaceInvadersGame {
    /// March the fleet and retarget every Toasted's kinematic body to its home plus the
    /// shared offset. Low Toasted chew through the bunker blocks they overlap.
    pub(crate) fn march_formation(&mut self, ctx: &mut GameContext) {
        if self.invaders.is_empty() {
            return;
        }

        let live_min = self.invaders.iter().map(|invader| invader.column).min().map(column_cell_x).unwrap_or(0.0);
        let live_max = self.invaders.iter().map(|invader| invader.column).max().map(column_cell_x).unwrap_or(0.0);

        let speed = march_speed(
            self.invaders.len(),
            ROW_CELL_Y.len() * FORMATION_COLUMNS,
            self.chaos_mode.is_insane(),
        );
        let outcome = march_step(
            self.formation_offset.x, self.march_dir, speed * ctx.delta_time, live_min, live_max);
        self.formation_offset.x = outcome.offset_x;
        self.march_dir = outcome.dir;
        if outcome.descended {
            self.formation_offset.y -= DESCEND_STEP;
        }

        for invader in &self.invaders {
            let target = invader_home(invader.row, invader.column) + self.formation_offset;
            self.physics.set_kinematic_target(invader.entity, target, 0.0);
        }

        self.chomp_bunkers(ctx);
    }

    /// Marching Toasted destroy any bunker block their body overlaps. Rapier never
    /// reports kinematic-vs-static pairs, so this is a game-side rectangle test —
    /// skipped until some Toasted's body reaches the bunkers' top.
    fn chomp_bunkers(&mut self, ctx: &mut GameContext) {
        let low: Vec<(Vec2, Vec2)> = self.invaders.iter()
            .filter_map(|invader| {
                let half = half_size(invader.kind.spec().sheet);
                entity_position(ctx.world, invader.entity).map(|position| (position, half))
            })
            .filter(|(position, half)| position.y - half.y < BUNKER_TOP_Y)
            .collect();
        if low.is_empty() {
            return;
        }

        let block_half = Vec2::splat(BUNKER_BLOCK_PX / 2.0);
        let chomped: Vec<BunkerBlock> = self.bunker_blocks.iter().copied()
            .filter(|block| {
                entity_position(ctx.world, block.entity).is_some_and(|block_position| {
                    low.iter().any(|&(position, half)| rects_overlap(position, half, block_position, block_half))
                })
            })
            .collect();
        for block in chomped {
            self.damage_bunker(ctx, block, crate::gameplay::bunkers::Chip::Butter);
        }
    }

    /// The fleet lands — immediate defeat, whatever lives are left — when a live
    /// Toasted's body reaches the invasion line.
    pub(crate) fn check_invasion(&mut self, ctx: &mut GameContext) {
        if self.state != GameState::Playing {
            return;
        }
        let bottom_edges: Vec<f32> = self.invaders.iter()
            .filter_map(|invader| {
                let half = half_size(invader.kind.spec().sheet);
                entity_position(ctx.world, invader.entity).map(|position| position.y - half.y)
            })
            .collect();
        if fleet_has_landed(&bottom_edges) {
            self.finish_game(ctx, false);
        }
    }
}
