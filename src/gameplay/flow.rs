//! Match lifecycle: state input during play, starting and ending a match, and what a
//! restart or a quit clears.

use engine_core::prelude::*;

use crate::art::Hero;
use crate::constants::*;
use crate::hud::life_icon_position;
use crate::spawning;
use crate::types::*;

impl SpaceInvadersGame {
    /// Actions that change the game state while the match screens are up. Play pauses
    /// and restarts through the pause gate in `update_gameplay`; here only game over
    /// reacts — Action1 restarts and Menu returns to the title.
    pub(crate) fn handle_state_input(&mut self, ctx: &mut GameContext) {
        if let GameState::GameOver { .. } = &self.state {
            if ctx.players.just_activated_any(GameAction::Action1, ctx.input) {
                self.start_game(ctx);
            } else if ctx.players.just_activated_any(GameAction::Menu, ctx.input) {
                self.reset_to_title(ctx.world);
            }
        }
    }

    /// Build a fresh roster for the chosen mode, rebuild the fleet, the bunkers and the
    /// cannons, and start the march. Called from the mode select and from a restart.
    pub(crate) fn start_game(&mut self, ctx: &mut GameContext) {
        self.clear_match(ctx.world);
        self.march_dir = 1.0;
        self.formation_offset = Vec2::ZERO;

        let count = self.mode.player_count();
        self.players = (0..count)
            .map(|index| PlayerState::fresh(Hero::for_player(index), cannon_spawn_x(count, index)))
            .collect();
        for index in 0..count {
            let (hero, x) = (self.players[index].hero, self.players[index].x);
            let entity = spawning::spawn_cannon(ctx.world, self.sheets.hero(hero), hero, x);
            self.players[index].entity = Some(entity);
            for life in 0..STARTING_LIVES {
                let icon = spawning::spawn_life_icon(
                    ctx.world, &self.sheets.life_icon, hero, life_icon_position(index, life));
                self.life_icons.push((index, life, icon));
            }
        }

        self.invaders = spawning::spawn_invaders(ctx.world, &self.sheets);
        self.bunker_blocks = spawning::spawn_bunkers(ctx.world, &self.sheets.butter_bunker);
        self.state = GameState::Playing;
        self.music.restart(ctx, &self.invaders);
    }

    /// End the match, win or lose. The scene stays on screen behind the game-over panel;
    /// the next start rebuilds it.
    pub(crate) fn finish_game(&mut self, ctx: &mut GameContext, won: bool) {
        // Both co-op cannons can fall to the same frame's crumbs: the first call ends the
        // match, a second must not re-submit scores.
        if let GameState::GameOver { .. } = self.state {
            return;
        }
        // A surviving cannon caught mid-hurt by the match's end stands back up for the
        // game-over scene: nothing runs its respawn timer after this.
        for index in 0..self.players.len() {
            if self.players[index].respawn_in.is_some() {
                self.respawn_cannon(ctx.world, index);
            }
        }
        self.destroy_all_shots(ctx.world);
        self.destroy_flyer(ctx.world);
        self.music.rest_bob(ctx.world, &self.invaders);
        let mode = match self.mode {
            GameMode::SinglePlayer => "single",
            GameMode::TwoPlayerCoop => "coop",
        };
        for player in &self.players {
            let _ = ctx.scores.submit(mode, u64::from(player.score));
        }
        self.state = GameState::GameOver { won };
    }

    /// Leave the match for the title: every shot, flyer and one-shot goes, so nothing
    /// plays on under the menus.
    pub(crate) fn reset_to_title(&mut self, world: &mut World) {
        self.destroy_all_shots(world);
        self.destroy_flyer(world);
        self.music.rest_bob(world, &self.invaders);
        self.clear_transient_visuals(world);
        self.notice = None;
        self.state = GameState::TitleScreen { selection: 0 };
    }

    /// Everything a match spawned, gone: cannons, fleet, bunkers, shots, the flyer, the
    /// life icons and every one-shot still playing.
    fn clear_match(&mut self, world: &mut World) {
        self.destroy_all_shots(world);
        self.destroy_flyer(world);
        self.clear_transient_visuals(world);
        self.notice = None;
        self.passing.clear();
        let bodies: Vec<EntityId> = self.players.iter_mut().filter_map(|player| player.entity.take())
            .chain(self.invaders.drain(..).map(|invader| invader.entity))
            .chain(self.bunker_blocks.drain(..).map(|block| block.entity))
            .collect();
        for entity in bodies {
            self.physics.destroy_entity(world, entity);
        }
        for (_, _, icon) in self.life_icons.drain(..) {
            world.remove_entity(&icon).ok();
        }
        self.bunkers = [Bunker::FRESH; BUNKER_COUNT];
    }

    pub(crate) fn destroy_all_shots(&mut self, world: &mut World) {
        let mut shots: Vec<EntityId> = self.enemy_shots.drain(..).map(|shot| shot.entity).collect();
        for player in &mut self.players {
            shots.append(&mut player.shots);
        }
        for shot in shots {
            self.physics.destroy_entity(world, shot);
        }
        self.passing.clear();
    }

    /// End every burst, splash and hurt still playing or holding.
    pub(crate) fn clear_transient_visuals(&mut self, world: &mut World) {
        for entity in self.transient_visuals.drain(..) {
            world.remove_entity(&entity).ok();
        }
    }
}
