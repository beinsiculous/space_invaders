//! Toast Busters' music through the real game and the engine's harness, whose audio manager
//! plays the committed song on its virtual clock: the sidecar's shape, the pluck line's tiling
//! and the committed meter, the ten moments and the sources they trigger on, the flyer's
//! exit, the order within a frame, the stinger's notes, the pause jam, the fleet's bob, a
//! match start deciding by the song in the slot, and a missing bank leaving the game silent.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use engine_core::prelude::*;
use engine_core::test_support::{GameHarness, InputEvent};
use music_midi::{BackgroundScore, PartMode, PositionSnapshot, Quantize, SongData};

use crate::art::{Toasted, ROSTER};
use crate::gameplay::flyer_spawn_rolls;
use crate::music::{BEAT_BOB_SECONDS, SONG_PATH};
use crate::spawning::invader_home;
use crate::test_support::*;
use crate::types::*;

/// What a trigger waits for before the conductor has drained it: far longer than it takes.
const DRAINED: Duration = Duration::from_secs(2);

fn repository_assets() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("assets")
}

fn committed_song() -> SongData {
    let bytes = std::fs::read(repository_assets().join(SONG_PATH)).expect("the song is committed");
    SongData::from_smf(&bytes).expect("the song parses").0
}

fn conductor(harness: &GameHarness<SpaceInvadersGame>) -> PositionSnapshot {
    harness.audio().song_snapshot().expect("a song is in the slot").conductor
}

fn key(harness: &mut GameHarness<SpaceInvadersGame>, code: KeyCode) {
    harness.step(FRAME, &[InputEvent::KeyPressed(code)]);
    harness.step(FRAME, &[InputEvent::KeyReleased(code)]);
}

fn source_of(harness: &GameHarness<SpaceInvadersGame>, entity: EntityId) -> Option<SourceId> {
    harness.sound_sources().source_of(entity)
}

/// The match stops running while the song plays on: no march, no crumb, no flyer roll, so a
/// test that waits on wall time hears only what it sent.
fn hold_the_match(harness: &mut GameHarness<SpaceInvadersGame>) {
    harness.context(|game, _| game.state = GameState::GameOver { won: false });
}

/// A match with its triggers drained, its lane set to a stretch of frames that rolls no flyer.
fn settled(mode: GameMode, chaos: ChaosMode) -> GameHarness<SpaceInvadersGame> {
    let mut harness = harness(mode, chaos);
    let quiet = (1u32..).find(|&start| (start..start + 1200).all(|frame| !flyer_spawn_rolls(frame, FRAME))).expect("a quiet stretch");
    harness.context(|game, _| game.frame_count = quiet);
    settle(&mut harness);
    harness
}

/// Step quietly until a frame's trace holds a trigger, and return it.
fn next_trace(harness: &mut GameHarness<SpaceInvadersGame>, keep: &[EntityId]) -> Vec<&'static str> {
    for _ in 0..60 {
        step_quietly(harness, 1, keep);
        let trace = harness.game().music.triggered.clone();
        if !trace.is_empty() {
            return trace;
        }
    }
    panic!("no trigger in a second of frames");
}

/// Roll a flyer into the lane through the production spawn, and return it.
fn send_a_flyer(harness: &mut GameHarness<SpaceInvadersGame>) -> EntityId {
    let next = harness.game().frame_count.wrapping_add(1);
    let rolls = (next..).find(|&frame| flyer_spawn_rolls(frame, FRAME)).expect("a frame rolls a flyer");
    harness.context(|game, _| game.frame_count = rolls - 1);
    harness.step(FRAME, &[]);
    assert_eq!(harness.game().music.triggered, ["flyer"], "the lane's own spawn strikes midnight");
    harness.game().flyer.as_ref().expect("a flyer").entity
}

#[test]
fn the_committed_sidecar_binds_eleven_moments_on_seven_unquantized_parts() {
    let text = std::fs::read_to_string(repository_assets().join(format!("{SONG_PATH}.ron"))).expect("the sidecar");
    let file = music_midi::parse_song_file("ai_toast_busters.mid.ron", &text).expect("the sidecar parses");
    let binding = |action: &str| file.bindings.iter().find(|binding| binding.action == action).expect(action).clone();
    let mode_of = |action: &str| file.parts.iter().find(|part| part.name == binding(action).part).expect(action).mode;
    let mut parts: Vec<&str> = file.parts.iter().map(|part| part.name.as_str()).collect();
    parts.sort_unstable();
    assert_eq!(parts, ["bones", "bonus", "chime", "clear", "hurt", "jam", "pluck"], "the background tracks select no part");
    assert_eq!(file.bindings.len(), 11);
    for (action, instrument) in [("plain_wraith", "xylophone"), ("rye_ip", "clarinet"), ("bagelgeist", "bells"),
        ("sour_doom", "bass"), ("singe", "bass")] {
        assert_eq!((binding(action).part.as_str(), binding(action).instrument.as_str()), ("bones", instrument), "{action}");
    }
    assert_eq!(mode_of("shot"), PartMode::AtSongPosition, "a shot plucks the bass of its moment");
    for action in ["flyer", "bonus", "hit", "clear", "menu", "plain_wraith"] {
        assert_eq!(mode_of(action), PartMode::Loop, "{action}");
    }
    assert!(file.parts.iter().all(|part| part.quantize == Quantize::Off), "no trigger is parked on the loop seam");
    assert!(file.provenance.is_some());
    assert_eq!(file.soundfont, "soundfonts/toast_busters.sf2");
}

#[test]
fn the_song_is_in_three_four_for_thirty_two_bars_and_its_pluck_line_tiles_the_loop() {
    let song = committed_song();
    let meter = song.time_signatures().first().expect("a meter is written, not the 4/4 default");
    assert_eq!((meter.tick, meter.numerator, meter.denominator), (0, 3, 4), "else the bob's bars drift off the waltz");
    let loop_end = BackgroundScore::new(&song, &Default::default()).end_tick();
    assert_eq!(loop_end, 32 * 3 * u64::from(song.ticks_per_quarter()), "the waltz bass's held note ends it on a downbeat");
    let pluck = song.tracks().iter().find(|track| track.name() == Some("pluck")).expect("the pluck track");
    let units = pluck.units();
    assert_eq!(units[0].tick, 0, "it starts on the first tick");
    let end_of = |unit: &music_midi::MusicalUnit| unit.tick + unit.notes.iter().map(|note| note.duration_ticks).max().unwrap_or(0);
    for pair in units.windows(2) {
        assert_eq!(end_of(&pair[0]), pair[1].tick, "a unit ends where the next begins, at {}", pair[1].tick);
    }
    assert_eq!(end_of(units.last().expect("a unit")), loop_end, "the last ends on the loop's last tick, so no shot lands in a rest");
}

#[test]
fn the_song_plays_after_init() {
    let mut harness = title_harness();
    assert!(!harness.game().music.game.song_failed());
    let before = conductor(&harness).blocks_rendered;
    settle(&mut harness);
    assert!(conductor(&harness).blocks_rendered > before, "the virtual clock renders the song");
    let own = format!("{}/{SONG_PATH}", repository_assets().display());
    assert_eq!(harness.audio().song_label(), Some(own.as_str()));
}

#[test]
fn a_fire_plucks_once_on_the_cannons_source_and_a_twin_volley_is_one_fire() {
    let mut harness = settled(GameMode::SinglePlayer, ChaosMode::Ridiculous);
    let cannon = harness.game().players[0].entity.expect("the cannon");
    let before = conductor(&harness);
    step_with(&mut harness, &[press(KeyCode::Space)], &[]);
    assert_eq!(harness.game().music.triggered, ["shot"]);
    assert_eq!(harness.game().players[0].shots.len(), 2, "two icicles left the muzzle");
    step_with(&mut harness, &[release(KeyCode::Space)], &[]);
    assert_eq!(harness.game().music.counts.shots, 1);
    assert!(source_of(&harness, cannon).is_some(), "the cannon's own activation");
    hold_the_match(&mut harness);
    poll(&mut harness, DRAINED, "one accepted pluck", |harness| conductor(harness).accepted_triggers == before.accepted_triggers + 1);
    assert_eq!(conductor(&harness).silent_triggers, before.silent_triggers, "the bass line had a note sounding");
}

#[test]
fn each_kind_busted_rattles_the_bones_on_its_own_binding_and_a_singe_ends_with_its_sour_doom() {
    let mut harness = settled(GameMode::SinglePlayer, ChaosMode::Normal);
    clear_field(&mut harness, |invader| invader.column == 4 || (invader.column == 0 && invader.row == 3));
    let (before, mut traces) = (conductor(&harness), Vec::new());
    for (row, &kind) in ROSTER.iter().enumerate() {
        let target = slice(&harness, row, 4);
        while harness.game().invaders.iter().any(|invader| invader.entity == target) {
            shoot_into(&mut harness, 0, Target::Invader(target), aim_for(kind));
            traces.extend(next_trace(&mut harness, &[]));
            if traces == ["singe"] {
                assert!(source_of(&harness, target).is_some(), "a singe plays on the Sour-Doom's own source");
            }
        }
        assert_eq!(source_of(&harness, target), None, "a bust's despawn ends whatever source the slice had");
    }
    assert_eq!(traces, ["singe", "sour_doom", "bagelgeist", "rye_ip", "plain_wraith"], "each kind to its own binding, top row down");
    hold_the_match(&mut harness);
    let rung = |harness: &GameHarness<SpaceInvadersGame>| {
        let now = conductor(harness);
        now.accepted_triggers == before.accepted_triggers + 5 && now.live_sources == before.live_sources + 1
    };
    poll(&mut harness, DRAINED, "five accepted, the singe's source ended and the game's live", rung);
    settle(&mut harness);
    assert!(rung(&harness), "the busts ring on the game's source, which no despawn ends");
}

#[test]
fn a_flyer_strikes_midnight_on_its_own_source_and_leaving_the_screen_ends_it() {
    let mut harness = settled(GameMode::SinglePlayer, ChaosMode::Normal);
    let flyer = send_a_flyer(&mut harness);
    assert!(source_of(&harness, flyer).is_some(), "the flyer's own activation");
    for _ in 0..900 {
        step_quietly(&mut harness, 1, &[flyer]);
        if harness.game().flyer.as_ref().is_none_or(|lane| lane.entity != flyer) {
            break;
        }
        assert!(source_of(&harness, flyer).is_some(), "kept while it crosses");
    }
    assert!(!harness.world().entities().contains(&flyer), "it flew off the far edge");
    assert_eq!(source_of(&harness, flyer), None, "the despawn is the visibility exit: its source ended in that tail");
}

#[test]
fn a_flyer_busted_by_an_icicle_rings_the_bonus_on_the_games_source() {
    let mut harness = settled(GameMode::SinglePlayer, ChaosMode::Normal);
    let before = conductor(&harness);
    let flyer = send_a_flyer(&mut harness);
    let cell = crate::spawning::flyer_art(harness.game().flyer.as_ref().expect("a flyer").kind).0.cell;
    shoot_into(&mut harness, 0, Target::Flyer(flyer), cell.x / 2.0);
    assert_eq!(next_trace(&mut harness, &[flyer]), ["bonus"]);
    assert_eq!(source_of(&harness, flyer), None, "the flyer is gone, its chime's source with it");
    hold_the_match(&mut harness);
    poll(&mut harness, DRAINED, "the chime and the bonus accepted", |harness| conductor(harness).accepted_triggers == before.accepted_triggers + 2);
}

#[test]
fn the_cannons_last_life_ends_its_source() {
    let mut harness = settled(GameMode::SinglePlayer, ChaosMode::Normal);
    let cannon = harness.game().players[0].entity.expect("the cannon");
    step_with(&mut harness, &[press(KeyCode::Space)], &[]);
    step_with(&mut harness, &[release(KeyCode::Space)], &[]);
    harness.context(|game, ctx| {
        game.destroy_all_shots(ctx.world);
        game.players[0].lives = 1;
    });
    assert!(source_of(&harness, cannon).is_some());
    let crumb = drop_on_cannon(&mut harness, 0, EnemyShotKind::Crumb);
    assert_eq!(next_trace(&mut harness, &[crumb]), ["hit"]);
    assert_eq!(harness.game().state, GameState::GameOver { won: false });
    assert_eq!(source_of(&harness, cannon), None, "the cannon went with its last life, its source with it");
}

#[test]
fn a_fire_on_the_frame_an_earlier_icicle_lands_plucks_before_the_bust() {
    let mut harness = settled(GameMode::SinglePlayer, ChaosMode::Insane);
    clear_field(&mut harness, |invader| invader.row == 3 && (invader.column == 4 || invader.column == 0));
    let wraith = slice(&harness, 3, 4);
    shoot_into(&mut harness, 0, Target::Invader(wraith), aim_for(Toasted::PlainWraith));
    step_with(&mut harness, &[press(KeyCode::Space)], &[]);
    assert_eq!(harness.game().music.triggered, ["shot", "plain_wraith"], "firing runs before the shots resolve");
}

#[test]
fn two_icicles_landing_in_one_frame_trigger_in_the_cannons_shot_order() {
    let mut harness = settled(GameMode::SinglePlayer, ChaosMode::Insane);
    clear_field(&mut harness, |invader| (invader.row == 0 && invader.column == 4) || invader.row == 3);
    let (sour_doom, wraith) = (slice(&harness, 0, 4), slice(&harness, 3, 0));
    harness.context(|game, _| game.invaders.iter_mut().for_each(|invader| invader.hits_left = 1));
    let shots = [spawn_test_shot(&mut harness, 0), spawn_test_shot(&mut harness, 0)];
    step_quietly(&mut harness, 1, &[]);
    aim_shot(&mut harness, 0, shots[0], Target::Invader(sour_doom), aim_for(Toasted::SourDoom));
    aim_shot(&mut harness, 0, shots[1], Target::Invader(wraith), aim_for(Toasted::PlainWraith));
    assert_eq!(next_trace(&mut harness, &[]), ["sour_doom", "plain_wraith"]);
}

#[test]
fn the_last_slice_busts_then_clears_on_one_frame_and_the_last_chord_is_held_and_then_released() {
    let mut harness = settled(GameMode::SinglePlayer, ChaosMode::Normal);
    clear_field(&mut harness, |invader| invader.row == 3 && invader.column == 4);
    let wraith = slice(&harness, 3, 4);
    let before = conductor(&harness);
    assert_eq!(before.pending_events, 0, "nothing else has triggered");
    shoot_into(&mut harness, 0, Target::Invader(wraith), aim_for(Toasted::PlainWraith));
    assert_eq!(next_trace(&mut harness, &[]), ["plain_wraith", "clear"]);
    assert_eq!(harness.game().state, GameState::GameOver { won: true });
    poll(&mut harness, DRAINED, "the bust and the stinger accepted, the chord held", |harness| {
        let now = conductor(harness);
        now.accepted_triggers == before.accepted_triggers + 2 && now.pending_events > 0
    });
    assert_eq!(conductor(&harness).silent_triggers, before.silent_triggers, "the unit had notes to play");
    let song = committed_song();
    let stinger = song.tracks().iter().find(|track| track.name() == Some("stinger")).expect("the stinger track");
    let tempo = song.tempo_map();
    let longest = stinger.units().iter().flat_map(|unit| unit.notes.iter().map(move |note| (unit.tick, note.duration_ticks)))
        .map(|(tick, duration)| tempo.seconds_at_tick(tick + duration) - tempo.seconds_at_tick(tick))
        .fold(0.0, f64::max);
    poll(&mut harness, Duration::from_secs_f64(longest + 2.0), "every held note-off sent", |harness| conductor(harness).pending_events == 0);
}

#[test]
fn the_pause_menu_jams_one_unit_a_move_and_nothing_on_open_confirm_the_pointer_or_a_reopen() {
    let mut harness = settled(GameMode::SinglePlayer, ChaosMode::Normal);
    let jams = |harness: &GameHarness<SpaceInvadersGame>| harness.game().music.game.jam_units();
    harness.step(FRAME, &[InputEvent::KeyPressed(KeyCode::Escape), InputEvent::KeyPressed(KeyCode::KeyW)]);
    harness.step(FRAME, &[InputEvent::KeyReleased(KeyCode::Escape), InputEvent::KeyReleased(KeyCode::KeyW)]);
    assert!(harness.game().pause.is_active());
    assert_eq!(jams(&harness), 0, "the frame it opens jams nothing, a move on it included");
    let accepted = conductor(&harness).accepted_triggers;
    key(&mut harness, KeyCode::KeyS);
    assert_eq!(jams(&harness), 1, "one move, one unit");
    settle(&mut harness);
    assert_eq!(conductor(&harness).accepted_triggers, accepted + 1, "the song plays on under the pause menu");
    let (resume, restart) = (drawn_at(&harness, "Resume"), drawn_at(&harness, "Restart"));
    for at in [restart, resume, restart, resume] {
        harness.step(FRAME, &[InputEvent::MouseMoved(at.x, at.y)]);
    }
    assert_eq!(jams(&harness), 1, "a pointer swept across the rows jams nothing");
    harness.step(FRAME, &[InputEvent::MouseMoved(resume.x, resume.y)]);
    key(&mut harness, KeyCode::Enter);
    assert!(!harness.game().pause.is_active(), "confirm on Resume resumes");
    assert_eq!(jams(&harness), 1, "and confirm jams nothing");
    harness.step(FRAME, &[InputEvent::KeyPressed(KeyCode::Escape), InputEvent::KeyPressed(KeyCode::KeyS)]);
    harness.step(FRAME, &[InputEvent::KeyReleased(KeyCode::Escape), InputEvent::KeyReleased(KeyCode::KeyS)]);
    assert!(harness.game().pause.is_active());
    assert_eq!(jams(&harness), 1, "a reopen jams nothing on its opening frame");
    assert_eq!(harness.game().music.triggered, Vec::<&str>::new());
}

/// Every live slice's drawn offset below its spec's, in pixels: zero is at rest.
fn dips(harness: &GameHarness<SpaceInvadersGame>) -> Vec<f32> {
    let drawn = |entity: EntityId| harness.world().get::<Sprite>(entity).expect("a slice's sprite").offset.y;
    harness.game().invaders.iter().map(|invader| invader.kind.spec().sheet.sprite_offset().y - drawn(invader.entity)).collect()
}

/// Step quietly, with wall time between frames, to a beat's frame and check it: every slice
/// drawn the beat's peak below its spec, every body where the march put it. Returns the peak.
fn wait_for_a_dip(harness: &mut GameHarness<SpaceInvadersGame>) -> f32 {
    let started = Instant::now();
    loop {
        assert!(started.elapsed() < DRAINED, "a beat within two seconds (one every 0.42 s)");
        let marched = harness.game().formation_offset;
        std::thread::sleep(Duration::from_millis(3));
        step_quietly(harness, 1, &[]);
        let Some(peak) = harness.game().music.bobbed_at else { continue };
        assert!(dips(harness).iter().all(|&dip| dip == peak), "read on the beat's own frame, every slice dips {peak} px");
        for invader in &harness.game().invaders {
            let body = position_of(harness, invader.entity);
            let target = invader_home(invader.row, invader.column) + marched;
            assert!((body - target).length() < 0.01, "the body stands where the march put it, untouched by the bob");
        }
        return peak;
    }
}

#[test]
fn a_beat_dips_the_fleets_art_below_its_spec_and_it_is_back_within_the_bobs_seconds() {
    let mut harness = settled(GameMode::SinglePlayer, ChaosMode::Normal);
    let frames_back = (BEAT_BOB_SECONDS / FRAME).ceil() as usize;
    let started = Instant::now();
    loop {
        assert!(started.elapsed() < Duration::from_secs(10), "a dip with no beat in the frames after it");
        assert!(wait_for_a_dip(&mut harness) > 0.0, "a dip is down, toward the countertop");
        let mut beat_again = false;
        for _ in 0..frames_back {
            step_quietly(&mut harness, 1, &[]);
            beat_again |= harness.game().music.bobbed_at.is_some();
        }
        if !beat_again {
            assert!(dips(&harness).iter().all(|&dip| dip == 0.0), "back at the spec's offset in {BEAT_BOB_SECONDS} s");
            break;
        }
    }
}

/// Beats the song has crossed, counted from its top: three to a bar of 3/4.
fn beats_into_the_song(harness: &GameHarness<SpaceInvadersGame>) -> u64 {
    let position = harness.audio().song_position().expect("playing");
    position.bar * 3 + position.beat
}

#[test]
fn a_pause_a_restart_or_a_loss_on_a_dipped_frame_leaves_the_fleet_at_rest() {
    let mut harness = settled(GameMode::SinglePlayer, ChaosMode::Normal);
    let at_rest = |harness: &GameHarness<SpaceInvadersGame>| dips(harness).iter().all(|&dip| dip == 0.0);
    wait_for_a_dip(&mut harness);
    harness.step(FRAME, &[InputEvent::KeyPressed(KeyCode::Escape)]);
    assert!(harness.game().pause.is_active() && at_rest(&harness), "at rest on the menu's first frame");
    let (pulsed, beat) = (harness.game().music.game.pulsed(), beats_into_the_song(&harness));
    harness.step(FRAME, &[InputEvent::KeyReleased(KeyCode::Escape)]);
    poll(&mut harness, Duration::from_secs(3), "two beats crossed under the pause menu", |harness| beats_into_the_song(harness) >= beat + 2);
    assert!(harness.game().music.game.pulsed() == pulsed && at_rest(&harness), "no beat bobs a frozen fleet");
    key(&mut harness, KeyCode::Escape);
    wait_for_a_dip(&mut harness);
    harness.context(|game, ctx| game.start_game(ctx));
    assert_eq!((harness.game().music.bob_pixels, harness.game().music.bob_peak), (0.0, 0.0), "a restart forgets the dip");
    step_quietly(&mut harness, 1, &[]);
    assert!(harness.game().music.bobbed_at.is_some() || at_rest(&harness), "no old dip decays over the new fleet");
    harness.context(|game, _| game.players[0].lives = 1);
    wait_for_a_dip(&mut harness);
    let crumb = harness.context(|game, ctx| {
        let player = &game.players[0];
        let top = crate::constants::cannon_y(player.hero) + player.hero.half_envelope().y;
        let above = top + crate::art::half_size(&crate::art::BURNT_CRUMB).y + 1.0;
        game.spawn_enemy_shot(ctx.world, EnemyShotKind::Crumb, Vec2::new(player.x, above), crate::constants::CRUMB_SPEED);
        game.enemy_shots.last().map(|shot| shot.entity).expect("the crumb")
    });
    let mut dipped_when_lost = 0.0;
    while harness.game().state == GameState::Playing {
        assert!(harness.game().music.bob_pixels > 0.0, "the last life goes while the fleet is still dipped");
        dipped_when_lost = harness.game().music.bob_pixels;
        step_with(&mut harness, &[], &[crumb]);
    }
    assert!(dipped_when_lost > 0.0 && !harness.game().invaders.is_empty(), "a loss with slices dipped and alive");
    assert!(at_rest(&harness), "drawn at rest behind the game-over panel");
}

#[test]
fn a_match_start_requests_the_song_into_an_empty_slot_and_takes_its_own_back_to_the_top() {
    let mut harness = title_harness();
    let own = format!("{}/{SONG_PATH}", repository_assets().display());
    harness.audio_mut().stop_song();
    harness.context(|game, ctx| game.start_game(ctx));
    assert_eq!(harness.audio().song_label(), Some(own.as_str()), "an empty slot is requested into");
    hold_the_match(&mut harness);
    poll(&mut harness, DRAINED, "the song past its first beat", |harness| beats_into_the_song(harness) >= 1);
    harness.context(|game, ctx| game.start_game(ctx));
    hold_the_match(&mut harness);
    settle(&mut harness);
    let position = harness.audio().song_position().expect("playing");
    assert_eq!((position.bar, position.beat), (0, 0), "a match begins at the top of the waltz");
}

/// An asset folder holding every directory the game reads but `soundfonts/`, each a link to
/// the repository's own, removed however the test ends.
#[cfg(unix)]
struct AssetsWithoutBank {
    root: PathBuf,
}

#[cfg(unix)]
impl AssetsWithoutBank {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("toast-busters-missing-bank-{}", std::process::id()));
        std::fs::remove_dir_all(&root).ok();
        let assets = root.join("assets");
        std::fs::create_dir_all(assets.join("ai")).expect("a scratch asset folder");
        for directory in ["fonts", "sprites", "ai/music"] {
            std::os::unix::fs::symlink(repository_assets().join(directory), assets.join(directory)).expect("linked");
        }
        Self { root }
    }
}

#[cfg(unix)]
impl Drop for AssetsWithoutBank {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.root).ok();
    }
}

#[cfg(unix)]
#[test]
fn without_its_bank_the_game_runs_silent_and_counts_every_trigger_it_refuses() {
    let scratch = AssetsWithoutBank::new();
    let assets = scratch.root.join("assets");
    let config = crate::game_config(assets.to_str().expect("UTF-8"));
    let mut harness = GameHarness::new(SpaceInvadersGame::default(), config);
    harness.step(FRAME, &[]);
    harness.context(|game, ctx| game.start_game(ctx));
    assert!(harness.game().music.game.song_failed(), "the song failed at init");
    assert_eq!(harness.game().music.game.failures(), 2, "and again at the match start");
    assert_eq!(harness.audio().song_label(), None);
    let refused = harness.sound_sources().refused_triggers();
    step_with(&mut harness, &[press(KeyCode::Space)], &[]);
    assert_eq!(harness.game().music.triggered, ["shot"]);
    assert_eq!(harness.sound_sources().refused_triggers(), refused + 1, "refused, and counted");
    assert_eq!(harness.game().state, GameState::Playing, "and the game runs on");
}
