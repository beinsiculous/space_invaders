//! The butter bunkers: every destruction — a shot, a crumb, a glob or the chomping
//! fleet — goes through one `damage_bunker`. A bunker's state comes from how many of
//! its blocks were destroyed; when the state changes, every block whose slice of the
//! new state draws too little melts away without adding damage, so the state never
//! feeds back on itself, and every block left draws its slice of the new state.

use engine_core::prelude::*;

use crate::art::*;
use crate::spawning::{block_stands, block_tex_region, spawn_effect, Effect};
use crate::types::*;

use super::entity_position;

/// What a chip throws: butter for a shot, a crumb or a chomp; jam for a glob.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Chip {
    Butter,
    Jam,
}

/// A bunker after one more of its blocks is destroyed.
pub(crate) fn lose_block(bunker: Bunker) -> Bunker {
    let damage = bunker.damage + 1;
    Bunker { damage, state: BunkerState::for_damage(damage) }
}

impl SpaceInvadersGame {
    /// Destroy `block`, splash it, and count it against its bunker. A block already gone
    /// this frame — two things struck it at once — counts once.
    pub(crate) fn damage_bunker(&mut self, ctx: &mut GameContext, block: BunkerBlock, chip: Chip) {
        if !self.bunker_blocks.contains(&block) {
            return;
        }
        if let Some(position) = entity_position(ctx.world, block.entity) {
            let (spec, sheet, depth) = match chip {
                Chip::Butter => (&BUTTER_SPLASH, &self.sheets.butter_splash, BUTTER_SPLASH_DEPTH),
                Chip::Jam => (&JAM_SPLATTER, &self.sheets.jam_splatter, JAM_SPLATTER_DEPTH),
            };
            let splash = spawn_effect(ctx.world, &Effect {
                name: "Splash", spec, sheet, depth, clip: SPLAT, position, facing: Facing::Right,
            }, false);
            self.transient_visuals.push(splash);
        }
        self.remove_block(ctx.world, block);

        let before = self.bunkers[block.bunker];
        let after = lose_block(before);
        self.bunkers[block.bunker] = after;
        if after.state != before.state {
            self.redraw_bunker(ctx.world, block.bunker);
        }
    }

    fn remove_block(&mut self, world: &mut World, block: BunkerBlock) {
        self.bunker_blocks.retain(|standing| *standing != block);
        self.physics.destroy_entity(world, block.entity);
    }

    /// Melt every block of `bunker` that its state no longer draws, and point the rest
    /// at their slice of that state.
    fn redraw_bunker(&mut self, world: &mut World, bunker: usize) {
        let state = self.bunkers[bunker].state;
        let blocks: Vec<BunkerBlock> =
            self.bunker_blocks.iter().copied().filter(|block| block.bunker == bunker).collect();
        for block in blocks {
            if !block_stands(state, block.row, block.column) {
                self.remove_block(world, block);
                continue;
            }
            let region = block_tex_region(&self.sheets.butter_bunker, state, block.row, block.column);
            if let Some(sprite) = world.get_mut::<Sprite>(block.entity) {
                sprite.tex_region = region;
            }
        }
    }
}
