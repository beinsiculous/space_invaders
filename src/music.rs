//! Toast Busters' music: Camille Saint-Saëns's *Danse Macabre*, which the engine's conductor
//! plays as a song.
//!
//! The background — the clarinet's waltz tune over the acoustic bass's oom-pah-pah — plays by
//! itself from `init` on, through the menus and under the pause menu. Ten moments trigger the
//! song's bindings, each on the source [`Music::play`] names for it; a move in the pause menu
//! jams the `jam` counter-line; the fleet bobs on the song's bars and beats, in its art only.
//! The rules every game's song shares are the engine's `GameMusic` (its `start` is called on
//! `game`); what is here is the game's moments, counts and bob. A frame sends its shots, its
//! flyer, its busts and singes (each cannon's shots in order), its bonus, its hits, its clear.

use engine_core::prelude::*;

use crate::art::Toasted;
use crate::types::Invader;

/// The song under the asset base: an agent's first cut, under `ai/music` until a person's
/// pass promotes it.
pub(crate) const SONG_PATH: &str = "ai/music/ai_toast_busters.mid";
/// The song's eleven bindings: one per Toasted kind on the one `bones` part, the Sour-Doom's
/// first hit, and the six other moments.
pub(crate) const SOUR_DOOM_BINDING: &str = "sour_doom";
pub(crate) const BAGELGEIST_BINDING: &str = "bagelgeist";
pub(crate) const RYE_IP_BINDING: &str = "rye_ip";
pub(crate) const PLAIN_WRAITH_BINDING: &str = "plain_wraith";
pub(crate) const SINGE_BINDING: &str = "singe";
pub(crate) const SHOT_BINDING: &str = "shot";
pub(crate) const FLYER_BINDING: &str = "flyer";
pub(crate) const BONUS_BINDING: &str = "bonus";
pub(crate) const HIT_BINDING: &str = "hit";
pub(crate) const CLEAR_BINDING: &str = "clear";
pub(crate) const MENU_BINDING: &str = "menu";
/// How far the fleet's art dips on a bar line and on another beat, in pixels; zero on the
/// beat bobs on the bars only.
pub(crate) const BEAT_BOB_BAR_PIXELS: f32 = 4.0;
pub(crate) const BEAT_BOB_BEAT_PIXELS: f32 = 2.0;
/// How long a dip takes to come back up, whichever peak started it.
pub(crate) const BEAT_BOB_SECONDS: f32 = 0.12;

/// A moment the game plays, with the entity whose source it plays on where it has one: a
/// cannon's fire (once a fire, a twin volley included), a slice of a kind busted, a Sour-Doom's
/// first hit, a flyer entering, the flyer busted, a cannon hit, the last slice busted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Moment {
    Shot(EntityId),
    Bust(Toasted),
    Singe(EntityId),
    Flyer(EntityId),
    Bonus,
    Hit,
    Clear,
}

/// What the game itself sent, beside the conductor's counters: what a listener's ear is
/// checked against. The jam and the pulse are counted by `GameMusic`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct MusicCounts {
    pub(crate) shots: u64,
    pub(crate) busted: u64,
    pub(crate) singes: u64,
    pub(crate) flyers: u64,
    pub(crate) bonuses: u64,
    pub(crate) hits: u64,
    pub(crate) clears: u64,
}

/// See the module docs.
#[derive(Debug, Clone)]
pub(crate) struct Music {
    pub(crate) game: GameMusic,
    pub(crate) counts: MusicCounts,
    /// How far below its spec's offset every live slice is drawn now, in pixels.
    pub(crate) bob_pixels: f32,
    /// The dip the last beat started, which sets how fast it comes back up.
    pub(crate) bob_peak: f32,
    /// The bindings this frame triggered, in the order the game sent them: production's
    /// call order recorded for the same-frame order tests, never reconstructed.
    #[cfg(test)]
    pub(crate) triggered: Vec<&'static str>,
    /// The dip this frame's beat started, if one did, as the slices were written with it.
    #[cfg(test)]
    pub(crate) bobbed_at: Option<f32>,
}

impl Default for Music {
    fn default() -> Self {
        Self {
            game: GameMusic::new(SONG_PATH, "toast busters", "TOAST_BUSTERS_TELEMETRY"),
            counts: MusicCounts::default(),
            bob_pixels: 0.0,
            bob_peak: 0.0,
            #[cfg(test)]
            triggered: Vec::new(),
            #[cfg(test)]
            bobbed_at: None,
        }
    }
}

impl Music {
    /// A match starts: the engine's identity rule decides by the song in the slot, and the
    /// new fleet stands at rest.
    pub(crate) fn restart(&mut self, ctx: &mut GameContext, invaders: &[Invader]) {
        self.game.restart(ctx);
        self.rest_bob(ctx.world, invaders);
    }

    /// Trigger `moment`'s binding and count it. A busted slice is gone in that call, so its bones
    /// ring on the game's source; a singe's note ends with its slice, a chime with its flyer.
    pub(crate) fn play(&mut self, ctx: &mut GameContext, moment: Moment) {
        let counts = &mut self.counts;
        let (binding, entity, count) = match moment {
            Moment::Shot(cannon) => (SHOT_BINDING, Some(cannon), &mut counts.shots),
            Moment::Bust(Toasted::SourDoom) => (SOUR_DOOM_BINDING, None, &mut counts.busted),
            Moment::Bust(Toasted::Bagelgeist) => (BAGELGEIST_BINDING, None, &mut counts.busted),
            Moment::Bust(Toasted::RyeIP) => (RYE_IP_BINDING, None, &mut counts.busted),
            Moment::Bust(Toasted::PlainWraith) => (PLAIN_WRAITH_BINDING, None, &mut counts.busted),
            Moment::Singe(slice) => (SINGE_BINDING, Some(slice), &mut counts.singes),
            Moment::Flyer(flyer) => (FLYER_BINDING, Some(flyer), &mut counts.flyers),
            Moment::Bonus => (BONUS_BINDING, None, &mut counts.bonuses),
            Moment::Hit => (HIT_BINDING, None, &mut counts.hits),
            Moment::Clear => (CLEAR_BINDING, None, &mut counts.clears),
        };
        *count += 1;
        let source = if let Some(entity) = entity { ctx.sources.entity(entity) } else { ctx.sources.game() };
        #[cfg(test)]
        self.triggered.push(binding);
        ctx.sources.trigger(ctx.audio, binding, source);
    }

    /// The pause menu's frame: a move up or down jams one unit of the counter-line.
    pub(crate) fn follow_pause(&mut self, ctx: &mut GameContext, paused: bool) {
        if self.game.follow_pause(ctx, paused, MENU_BINDING) {
            #[cfg(test)]
            self.triggered.push(MENU_BINDING);
        }
    }

    /// A running match's frame, after the fleet moved: a bar line dips every live slice's art,
    /// another beat less, and the frames after bring it back up over `BEAT_BOB_SECONDS`. Only
    /// the sprites move: the bodies the landing and the bunkers read are never touched.
    pub(crate) fn bob(&mut self, ctx: &mut GameContext, invaders: &[Invader]) {
        if let Some(pulse) = self.game.beat_pulse(ctx) {
            let peak = if pulse == BeatPulse::Bar { BEAT_BOB_BAR_PIXELS } else { BEAT_BOB_BEAT_PIXELS };
            (self.bob_peak, self.bob_pixels) = (peak, peak);
            #[cfg(test)]
            {
                self.bobbed_at = Some(peak);
            }
        } else {
            self.bob_pixels = (self.bob_pixels - self.bob_peak * ctx.delta_time / BEAT_BOB_SECONDS).max(0.0);
        }
        write_bob(ctx.world, invaders, self.bob_pixels);
    }

    /// Bring every live slice's art back to its spec's offset and forget the dip: a fleet
    /// drawn under the pause menu, a game-over panel or a new match has no beat under it.
    pub(crate) fn rest_bob(&mut self, world: &mut World, invaders: &[Invader]) {
        (self.bob_peak, self.bob_pixels) = (0.0, 0.0);
        write_bob(world, invaders, 0.0);
    }

    /// Every frame, whatever the state: the engine's frame ring, telemetry and refusal
    /// lines, the telemetry ending in the game's counts.
    pub(crate) fn update(&mut self, ctx: &mut GameContext) {
        #[cfg(test)]
        {
            self.triggered.clear();
            self.bobbed_at = None;
        }
        let (counts, jam_units, pulsed) = (self.counts, self.game.jam_units(), self.game.pulsed());
        self.game.update(ctx, || {
            format!(
                "shots={} busted={} singes={} flyers={} bonuses={} hits={} clears={} jam_units={jam_units} pulsed={pulsed}",
                counts.shots, counts.busted, counts.singes, counts.flyers, counts.bonuses, counts.hits, counts.clears
            )
        });
    }
}

/// Draw every live slice `pixels` below its spec's offset; world Y is up, so below is less.
fn write_bob(world: &mut World, invaders: &[Invader], pixels: f32) {
    for invader in invaders {
        if let Some(sprite) = world.get_mut::<Sprite>(invader.entity) {
            sprite.offset.y = invader.kind.spec().sheet.sprite_offset().y - pixels;
        }
    }
}
