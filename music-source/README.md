# Midnight Toast

Original AI-authored C-minor spooky synth-disco for Toast Busters, CC0-1.0. No
borrowed melody, recordings or sample bank. A spectral call leaps from C to G,
answers through Eb and Bb, and lands over a staccato octave bass and four-on-floor
kick. Bars 1–8 introduce the four-bar hook, 9–16 expand its register, 17–24 strip
back alternate lead bars and rhythm, 25–32 reprise; snare fills mark each return.
This is a distinct composition, not a transposed earlier game. All pitched backing
and cues use C/Eb/G/Bb, avoiding semitone/tritone collisions across every section.

144 BPM, explicit 4/4, 480 PPQ, 32 bars. Integer MIDI tempo 416667 us/quarter yields
53.333376 seconds. The last actual Nightwalk bass note ends at tick 61440; no cue
extends it. Background names Nightwalk/Spectre/Answers/Kick/Snare/Hat select no part.
Production BeatPulse still owns the art bob; only the obsolete test meter changed.

`midnight_toast.evil` and the ten original patch JSON files are editable MIDI Evil
sources. **Running compose.py replaces hand-edited project and patch sources.**
Do so only when intentionally regenerating from the composition recipe.

The eleven actions and their source ownership/order are unchanged. Four legacy
instrument names remain stable map identifiers: xylophone13 is glass bones,
clarinet71 is filtered spectral reed, bells14 is halo triangles, bass32 is dry
filtered toast thud. Four enemy types walk the shared bones motif on these distinct
timbres. Sour-Doom bust and singe intentionally retain the same timbre/motif;
their game versus slice sources distinguish lifetime. Shot is bass32 at velocity54,
a dry positional pluck; its 120-tick units tile the entire loop without rests.
Bones velocity118 and high-register three/four-note bonus/clear chords are stronger
than the shot. Hurt is a low two-note reed stab. Flyer and pause-menu cues remain.
Selectors bones/pluck/chime/bonus/hurt/stinger/jam, pluck AtSongPosition and all
others Loop, all quantize Off; no gameplay or routing rewrite.

Render recipe: seed11700 plus instrument index, 32000 Hz, velocity127, mastergain0.8,
720-tick gates. All 50 WAVs are finite exact-key samples, 21600 frames/0.675 seconds,
with no resampling or loop points. SF2 zones declare 50ms release and longest gate
0.42s (actual maximum480ticks =0.416667s). Every root and render path matches exactly.
Four triggered slots (<8), one layer/key (<6), 2,168,820-byte bank (<4MiB), 12,003-byte
MIDI (<256KiB). Peak backing is seven voices; the declared slots load independently.

## Rebuild and install

From games/space_invaders in the normal sibling layout:

```sh
python3 music-source/compose.py
python3 music-source/build.py /tmp/invaders-build-a
python3 music-source/build.py /tmp/invaders-build-b
cmp /tmp/invaders-build-a/ai_toast_busters.mid /tmp/invaders-build-b/ai_toast_busters.mid
cmp /tmp/invaders-build-a/ai_toast_busters.sf2 /tmp/invaders-build-b/ai_toast_busters.sf2
cp /tmp/invaders-build-a/ai_toast_busters.mid assets/ai/music/ai_toast_busters.mid
cp /tmp/invaders-build-a/ai_toast_busters.sf2 assets/soundfonts/ai_toast_busters.sf2
cargo run --manifest-path ../../insiculous_2d/Cargo.toml -p editor --example remap_soundfont -- \
  assets ai/music/ai_toast_busters.mid soundfonts/ai_toast_busters.sf2 \
  'Original AI composition and MIDI Evil synthesis for Be Insiculous, 2026' CC0-1.0
```

Retain the old .mid.ron through export: native SongDocument remapping owns the map,
fingerprint and provenance save. It verifies every declared preset/key and loads
Conductor before installation is accepted. Delete the old bank only after this
closure passes. The new bank LICENSE records original CC0 synthesis and its hash.
`build.py` and `scripts/prepare_soundfont.sh` delegate to shared build_music.py;
default output is ignored music-source/rendered/, never runtime assets.

Shared builder **7c8fed0**, engine HEAD **d6fe15caee2dd1ae9af1fa988d2db8944bfc9fad**
(the followup only prevents test caches). Engine packer/remap/audition baseline
3623e73c04906f6a4d601908c366c1bc5b6b6f5d. MIDI Evil exporter baseline46ae908,
approved revision **3d061afafc64c71266e643271cb25ee88f4e84e8**; actual checkout used
**fc2713a6c18986b68f53c96305d3ea455f8dbd4a**. Python3.12.3, Rust1.94.0,
x86_64 Linux; existing Cargo.lock. Two fresh builds match all53 outputs byte-for-byte.
No cross-platform floating-point identity claim. Runtime uses committed MIDI/map/bank.

Historical editor scores need their original bank and LICENSE restored together,
not a retarget to new synthesis. Pre-migration a29cb45f2674b0a638ae19226f432d06129e5dee
retains that closure. Restore it separately, for example:

```sh
git worktree add --detach /tmp/toast-busters-historical a29cb45f2674b0a638ae19226f432d06129e5dee
```

Open that worktree's assets with the engine editor/audition; building the old game
also requires its normal relative engine layout. AI prefixes/quarantine remain.

## Auditions

```sh
python3 music-source/preview.py
```

Authored idle/sparse and all eleven action TSVs are editable, read-only inputs to
preview.py. It generates only ignored previews/dense.tsv. Each action demo must
have nonempty, finite event timestamps in inclusive0..7.9s; all demos are validated
before any rendering or onset analysis. First100ms onset windows use actual TSV
timestamps, including edits. Empty idle.tsv is intentional and exempt.

The real engine conductor and production limiter at unity buses render idle,
sparse and dense WAVs for108seconds, two full wraps. Dense includes two-player
20Hz rapidfire bursts with recovery gaps plus every other action. Eight-second
demo-*.wav and isolated-*.wav cover all eleven actions at final original gain.
Isolation subtracts identical idle PCM only when no limiting occurred in either
render (at most one PCM16 quantization step); it is not a replacement synthesizer.
These are synthetic event mixes, not captured gameplay or human listening approval.
Logs retain peak/RMS, 100ms cue/backing onset levels and engine counters.

Voice estimate at the dense event rates: two sources firing20Hz each, 104.17ms
maximum positional gate plus50ms release need at most eight bass voices; overlapping
bones/singe add two, comfortably below24. Reed has four clear voices plus two hurt,
one enemy and one menu voice; halo has three bonus, one flyer and one enemy voice.
Glass needs one. Background channels are independent. Clipping, limiting, nonfinite,
rejected, late, stale or silent triggers must all stay zero. The closure test also
renders every action in a fresh Conductor with music0/sfx1 and4096 stereo frames,
asserting finite unclipped PCM and positive energy, not merely accepted triggers.

Local browser bundles use the constants verified in src/web_entry.rs:
`toast-busters --kind games --version v7` and `invaders --kind editor --version v6`.
No deployment or version change is part of this batch.
