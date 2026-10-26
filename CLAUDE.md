# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
cargo run                     # play the game
cargo run --features editor   # run the game inside the engine's scene editor
cargo build                   # compile check
cargo test                    # run all 89 tests (see "Tests" below)
cargo test <test_name>        # run a single test
cargo clippy --all-targets -- -D warnings   # must stay clean
```

The game depends on the `insiculous_2d` engine by relative path (`../../insiculous_2d`); both checkouts must sit side by side or nothing builds. Engine crates used: `engine_core` (always) and `editor_integration` (only behind the `editor` feature). The `deion_assets` symlink at the repo root (`-> ../../deion_assets`) makes the same assumption.

## Architecture

This is a single-crate game (`insiculous_space_invaders`), shipped as **Toast Busters** — Space Invaders re-skinned into the Deion world. `SpaceInvadersGame` (in `src/types.rs`) implements the engine's `Game` trait in `src/lib.rs` — `init()` loads the sheets and spawns the countertop, which stays up under the menus; achievements register in `register_achievements()`, which the engine calls before the window opens; `cargo run -- --achievements-manifest <path>` exports the list; the cannons, the fleet, the bunkers and the life icons spawn fresh in `start_game()` (`gameplay/flow.rs`) once the menus pick a mode. `main.rs` and `web_entry.rs` are thin entry points. With `--features editor` the identical game runs inside the engine's scene editor via `editor_integration::run_game_with_editor`, and at `/playground/toast-busters/` in the browser (the same feature, built by the engine's `build_wasm.sh --kind editor` and served from the site); no game code changes between the modes. The bare native window is fixed at 800×600 (`main.rs`): the HUD's text is screen-space and the life icons beside it are world sprites, and another size would part them.

**State machine drives everything.** `GameState` (types.rs) is matched at the top of `update()`: `TitleScreen`/`ModeSelect`/`Achievements` dispatch to handlers in `menu.rs`; `Playing` and `GameOver { won }` fall through to `update_gameplay()` in `gameplay/mod.rs`, which orchestrates the per-frame steps implemented across `gameplay/{players,combat,bunkers,flyers,formation,flow}.rs`. Flow is Title → Mode select (1P / 2P co-op) → Chaos select → Playing → GameOver (Space restarts, Esc to title). **There is no level progression yet** — one formation per match; busting the last Toasted is victory.

**Module layout:**

- `art.rs` — one `SheetSpec` per synced sheet (path, cell, and the opaque box **measured** from the PNG over the frames the subject plays), the clip names the game types, one draw depth per sheet, the burst colours, the `Toasted` roster table (`ToastedSpec`: sheet, depth, points, hits, burst colour) and the `Hero` table (Deion and Cubert: sheet, shot, muzzle, life icon)
- `constants.rs` — the layout (the HUD band's rows, the flyby lane, the rows' and columns' cell centres, the bunkers, the ground) and all tuning (speeds, rates, caps, the bunker damage thresholds and the measured slice table, timers), plus the derived positions (`column_cell_x`, `cannon_y`, `cannon_max_x`, `cannon_spawn_x`)
- `types.rs` — game state: `PlayerState` (hero, the game's own x, facing, shots, lives, streak, jam and respawn timers), `Invader`, `EnemyShot`, `Flyer`, `Notice`, `Bunker`/`BunkerState`/`BunkerBlock`, `Sheets`
- `spawning.rs` — every entity recipe and the clip machines; `hud.rs` — the HUD band's map in board space
- `gameplay/` — `mod.rs` (frame orchestration, the pause gate, the F1 overlay, visibility), `formation.rs` (the march, the body-edge landing, the fleet chomping bunkers), `combat.rs` (the Toasted's fire, Bagelgeist's hole, Sour-Doom's two hits, enemy shots, cancels, culling), `bunkers.rs` (`damage_bunker` and the melt), `flyers.rs` (the saucer and Jammer), `players.rs` (movement, facing, fire, jams, hits, the respawn), `flow.rs` (start, finish, title, what a restart clears)
- `drawing.rs` — all UI; `menu.rs` — menu input; `effects.rs` — particle presets; `achievements.rs`
- `music.rs` — Danse Macabre: the eleven bindings, the moments that trigger them, the game's counts and the fleet's bob, over the engine's `GameMusic` (§ Music below)

**Formation march is an offset, not per-entity AI.** Each `Invader` keeps its fixed `(row, column)` slot; `spawning::invader_home` is where its body stands before any march (the slot's whole-pixel cell centre, minus the sheet's measured offset). The whole fleet shares one `formation_offset` + `march_dir`. Pure `march_step()` bounces the offset off `MARCH_BOUND_X` using the outermost **live** columns' cell centres (dead columns widen the march span) and reports `descended`; each bounce drops the offset by `DESCEND_STEP` (16). `march_speed()` ramps linearly from `MARCH_SPEED_BASE` to `MARCH_SPEED_MAX`. **The landing reads body edges**: the fleet lands when a live Toasted's bottom edge reaches `INVASION_Y` (−235, the top of Deion's envelope), the only landing rule — it lies above every cannon.

**Collision is hybrid — rapier events where they exist, manual AABB where they can't.** Shots are dynamic sensors (gravity 0, a `Lifetime` safety net); the Toasted, the cannons and the flyers are kinematic; bunker blocks are static sensors. Collision events are drained ONCE per frame and shared. Rapier does not report kinematic-vs-static pairs, so `rects_overlap()` covers the fleet chomping bunker blocks; the shot–shot cancel inflates its AABB by half a frame of closing speed. **A player's shot collides as its 2 px centre line** over its drawn height, narrower than the icicle or chip it draws — classic Invaders' thin shot.

**Bagelgeist's hole is decided at first contact.** `combat::filter_passing_contacts` runs on the drained events before anything resolves a hit: a player's shot whose *started* contact with a Bagelgeist finds its centre line inside the ring's hole (`BAGELGEIST_HOLE_COLUMNS`, x 27–35 of the cell) becomes a passing pair; every contact of that pair is dropped until the two part (a `stopped` event) or either goes. A fixed gap would slide sideways under a fast shot crossing a moving bagel; deciding once keeps the hole a hole, needs no compound collider, and lets the shot go on to whatever is above.

**The cannons are Deion and Cubert.** Each collides as its **envelope** — the union of its `idle`, `walk` and `fire` boxes (Deion 46×55, Cubert 28×43) — and stands on `GROUND_Y` (−290); the position is the envelope's centre. It faces the way it last moved: a negative x-scale, the sprite offset's x negated with it (the engine turns an offset but never scales it), the muzzle's x mirrored; the collider never turns. `fire` plays out — movement never cuts it short. **A lost life plays `hurt` as a detached one-shot** at the cannon's spot and facing: on a surviving hit the cannon is hidden and inert for 0.4 s, then reappears at its spawn — authoritatively, the body reset and the game's own `PlayerState.x` set, since movement reads that x, never the transform; on the last life the hurt holds its final frame (kept in `transient_visuals`) until a restart or a quit, and the cannon goes.

**Butter bunkers are blocks with a damage count.** Each bunker is 4×3 blocks of 16 px slicing its 64×48 art; a block spawns only where its `intact` slice draws at least 16 opaque pixels (ten a bunker). Every destruction goes through `damage_bunker`: the state comes from the damage (`intact` 0–1, `damaged` 2–3, `critical` 4–5, `melted` 6+); on a state change every block whose slice of the new state draws too little melts away without adding damage, and every block left draws its slice of the new state (`spawning::block_tex_region`, the cell read from the state's sidecar clip). `melted` keeps only the bottom row, so it shows when a bunker is eaten from above; one shot out from below is carved away as it melts.

**The flyby lane** holds one flyer at a time; each spawn roll sends Maxwell's cake saucer three times in four and Jammer once (`flyer_kind_for`). The saucer pays its mystery bonus and earns Saucer Hunter (`ufo_hunter`); Jammer drops a jam glob every 1.2 s over the field — a glob on a live cannon jams it for 1.5 s (no shot, no life lost) and passes the Toasted — and pays 100. Both count as kills for the streak. The flash or jam notice shows in the HUD band's one transient slot, the newer replacing the older.

**2P co-op conventions:** everything in `gameplay/players.rs` is parameterized by player index so 1P and co-op share one code path. The solo cannon answers to BOTH input slots (mouse takeover is 1P-only); co-op puts Deion at −120 on player 1 and Cubert at +120 on player 2. Per-cannon shot caps are independent (`volley_fits`); a cannon at 0 lives despawns while the survivor plays on — `coop_defeated` ends the match only when every cannon is out. Gameplay reads `ctx.players`, never raw keycodes (F1 debug toggle is the sanctioned exception).

**Editor naming:** every spawned entity gets a `Name` component ("Sour-Doom r0 c3", "Bunker 2 r1 c3", "Icicle", "Maxwell's Saucer", "Deion Life", "Countertop") — keep this when adding entities.

**Coordinate and scale conventions (the main trap):**
- World origin is screen centre; the window is 800×600 (`WIN_W`/`WIN_H`).
- One art pixel per window pixel: a sprite's scale is `SheetSpec::scale()` (its cell over `RENDER_UNIT` = 80), pixel-snapped (`with_pixel_snap(true)`). Positions are chosen so every cell centre is a whole pixel (`SheetSpec::position_for_cell`); a body centre, and so a collider's, may sit on a half pixel. Never scale a sprite to fake a footprint — re-measure the art.
- Collider shapes use **absolute pixels** and IGNORE `Transform2D.scale`. Colliders are sized from the same measured boxes the sprites are anchored by. `F1` in-game (or `C` in the editor) overlays magenta collider outlines.

**Chaos modes** (engine `ChaosMode`) split the fleet buffs deliberately — `player_fire_caps()` in gameplay/combat.rs is the SSOT for the player's answer, `(shots_per_fire, max_live_shots)`: **Normal** one shot in flight; **Insane** the fleet marches 1.8× faster, 4 single shots; **Ridiculous** it fires 2.4× as often, a twin volley at the muzzle ±10 px; **Insiculous** both, stacked twin volleys (cap 6). The regression test `fleet_buffs_split_across_chaos_modes` pins the split. The chaos theme is the accent layer only — the banner, the menus, the particle counts; no art is tinted by it, and the golden toast draws as drawn in every mode.

**Achievements** (achievements.rs): 11 — clear + perfect (no life lost by ANY cannon) per chaos mode, Sharpshooter (10 busts in a row; a miss or a cancel resets the streak, a Sour-Doom's absorbed hit neither counts nor resets it), Last Slice (win with some cannon on its last life), Saucer Hunter. **The ids, the score keys and the save paths are the game's original ones** and stay; the titles and descriptions are Toast Busters'. Persist to `saves/space_invaders_achievements.json`.

**Visuals:** every gameplay sprite is synced art, drawn white with emissive 0. **One draw depth per sheet** (`art.rs`), nested countertop, bunkers, jam globs (behind the formation they fall through), the four Toasted, the saucer, Jammer, Deion, Cubert, the shots, the bursts and splashes, the life icons — the renderer batches a sheet whole and transparent texels write depth, so a sheet split across depths would punch holes. **No lattice draws** — the countertop's grout is the grid, and the engine's `GridBackdrop` over it read as two grids (Jesse, 2026-10-13); kills and hits keep their drawn bursts and particles. Every one-shot — crouton bursts, ice bursts, butter splashes, jam splatters, the cannons' hurts — is kept in `transient_visuals` so a restart or a quit ends it. Particle presets live in `effects.rs`. **The life icons are world sprites** in the HUD band (`ctx.ui`'s image command cannot pick a sheet's cell), with their own visibility rule: shown only in a match, each only while its index is below its player's lives. This game has **no localization** — strings are literals.

**Tests (89)** are fully headless:
- `src/gameplay_tests.rs` (29) — the march and its pacing (8 descents to the bunkers, 13 to the landing), the body-edge landing, the roster, caps and the fleet-buff split, the thin shot, the hole's columns, the cannons' spawn and ground, the bunker states and the two chip sequences, the flyby roll and lane, and physics over the exact recipes (a shot reaching a Toasted, the flyer and a block; a crumb and a glob reaching a cannon; `Lifetime` expiry)
- `src/art_tests.rs` (8) — every measured number re-measured from the synced PNGs: every box is the union of the frames its subject plays, the fire poses never widen an envelope, the muzzles, the hole, the bunker slice table; every clip the game types is in its sidecar and every one-shot plays once; the depth ladder
- `src/flow_tests.rs` (13) and `src/combat_tests.rs` (8) — the match through the engine's `GameHarness`: facing, muzzle and body in both facings, the twin volley, `fire` playing out, the three hurt paths, the life icons, every HUD string in its box in the real font, restart and quit leaving nothing; the hole at the fastest march both ways with both shots and a hole shot going on to strike the Sour-Doom above, a ring shot busting the bagel, Sour-Doom's absorbed hit keeping the streak, a bunker chipped from below and redrawn, a glob passing the formation and jamming a cannon, Jammer's globs, the two flyers' payouts
- `src/music_tests.rs` (16) — the song through the harness's virtual clock: the sidecar's eleven bindings and modes, the committed meter, the loop's end and the pluck line tiling it, the song after `init`, each moment's trigger and source (a fire on the cannon's, a bust on the game's with each kind's binding, a singe on the slice's and ended by its bust, a flyer's on its own and ended by its exit off the screen, the bonus, the cannon's last life), the frame's order, the stinger's notes held and released, the pause jam, the bob (on the beat's own frame, back within its seconds, the bodies untouched, at rest on a pause, a restart and a loss), the match start's identity rule, the missing bank
- `src/title_screen_tests.rs` (2) — the title art and its menu
- `src/hud.rs` (3), `src/spawning.rs` (4), `src/achievements.rs` (5), `src/constants.rs` (1 — the startup cards)

`src/test_support.rs` holds the shared fixtures; `step_quietly` sweeps the formation's pseudo-random crumbs and flyers after each frame unless the test placed them, `shoot_into` (or `spawn_test_shot` and `aim_shot`, for two shots landing on one frame) puts an icicle under a slice or the flyer on the step that decides first contact, and `settle` and `poll` step with wall time between frames for the song's virtual clock.

**Paths:** assets and saves resolve through `engine_core::game_root!()` in main.rs (exe directory if it contains `assets/`, else `CARGO_MANIFEST_DIR`), so `cargo run` works from any cwd. Input bindings persist to `saves/input_settings.json`.

## Toast Busters (landed 2026-10-13)

The fifth of the Phase G Deion re-skins (`coordination/art-revamp/plan.md` § Batch 10 at the working-set root; Jesse's rulings R12–R16, the decisions D58–D69). In-game and in the window it is **TOAST BUSTERS**; the crate and `SpaceInvadersGame` keep their names, and the site names it **Toast Busters** at `/games/toast-busters/`, the former `/games/invaders/` redirecting there, and its web save keys are `beinsiculous.games.toast-busters.*`, with the former `beinsiculous.games.invaders.*` keys read at boot.

- **Art enters only through the sync.** `assets/sprites/sync.list` pins the `deion_assets` commit and lists the nineteen sheets beside the branding three; `python3 deion_assets/scripts/sync_sprites.py .` copies each PNG and its `.sheet.ron` in, and `--check` hashes the copies against the pinned blobs. Never hand-copy or hand-edit a file there — fix the master in `deion_assets` and re-sync. The working set's `scripts/check-sprite-sync.sh` runs that check over every game.
- **The art is free-tier only until Jesse's cleanup pass**: the synced copies keep their `ai_` prefix, so `deion_assets/scripts/check_no_ai_assets.sh assets` fails on a paid build, as it must.
- **Next: Toast Busters' levels** (`space_invaders#3`) — one bread per level, the level cards, Poptergeist's pop-up and Pain Perdu, the toast shading by chaos tier, and Stay Pufft's finale; its own plan revision.
- Style SSOT: `deion_assets/DEION_STYLE.md`. Sheet clip names (the `.sheet.ron` sidecars) are the stable API between art and code; `art.rs` spells each one once.

## Music (Danse Macabre, 2026-10)

Toast Busters plays Camille Saint-Saëns's *Danse Macabre* through the engine's conductor
(`training.md` § Conductor Pattern in the engine). The game's part is `src/music.rs` — its
constants, its moments, its counts and its bob — standing on the engine's `GameMusic`
(`engine_core::game_music`), which carries the request, the match start's identity rule, the
jam's latch, the beat and the telemetry. Its tests are `src/music_tests.rs`.

- **The song** is `assets/ai/music/ai_toast_busters.mid` and its sidecar: thirty-two bars of 3/4 in
  G minor at 144 BPM, 40 s, looping on a phrase's downbeat. An agent sequenced it from memory
  through the composer's command API (`song new`, `song edit …`, `song save
  ai/music/ai_toast_busters.mid`), so it sits under `ai/music` with the `ai_` prefix until a
  person's pass promotes it; the composer's Save wrote the sidecar and the stanza in
  `assets/ai/music/LICENSE`. Never hand-edit the sidecar — open the song in the composer (`cargo
  run --features editor`, File → Open Song) and save it again. The meter is written, not assumed:
  a song without one is 4/4, and the bob's bars would drift across the waltz's downbeats. The loop
  ends where its last written note ends, so the waltz bass's last note is held to the bar line.
- **The background** — the clarinet's waltz tune (the solo violin's, on track `tune`) over the
  acoustic bass's oom-pah-pah (`waltz_bass`) — plays by itself from `init` on, through the menus,
  under the pause menu and under the game-over panel. `song new` makes a track `melody` on channel 1
  with program 0, and the composer changes no track's program; the bank has no preset 0, so a note
  there would sound on its lowest preset, the xylophone. That track stays empty.
- **Eleven bindings, four source kinds**, every part `quantize: Off` (a quantized trigger in the
  loop's last beat would be parked on the seam and dropped):

  | binding | moment | part (mode) | instrument | source |
  |---|---|---|---|---|
  | `plain_wraith` | a Plain Wraith busted (`strike_invader`) | `bones` (Loop) | xylophone | the game's — the slice is gone in that call and its note rings on |
  | `rye_ip` | a Rye-I.P. busted | `bones` (Loop) | clarinet | the game's |
  | `bagelgeist` | a Bagelgeist busted | `bones` (Loop) | bells | the game's |
  | `sour_doom` | a Sour-Doom busted | `bones` (Loop) | bass | the game's |
  | `singe` | a Sour-Doom's first hit, which it survives | `bones` (Loop) | bass | the slice's own — the icicle that busts it releases the note |
  | `shot` | a cannon fires (`update_one_cannon_fire`), once a fire, a twin volley included | `pluck` (AtSongPosition) | bass | the cannon's own |
  | `flyer` | a flyer enters the lane (`update_flyer`) | `chime` (Loop, one low stroke) | bells | the flyer's own |
  | `bonus` | the flyer busted (`check_flyer_hit`) | `bonus` (Loop, one high chord) | bells | the game's |
  | `hit` | a cannon hit (`player_hit`) | `hurt` (Loop, one low held note) | clarinet | the game's |
  | `clear` | the last slice busted (`check_win_condition`) | `clear` (Loop, the dance's last chord) | clarinet | the game's |
  | `menu` | a move in the pause menu | `jam` (Loop, the counter-line) | clarinet | the menu's |

  The four kinds walk **one** tune on four instruments, and a Sour-Doom's singe and bust share the
  bass, so the loaf has one timbre. A shot plucks the bass note written for that moment of the
  waltz, always in the chord; the pluck line has no rest from tick 0 to the loop's end, which a test
  holds. **No binding for a Toasted's crumb**: no player act is behind it, forty slices firing at
  random would be a storm of triggers with no tune in it, and a crumb that lands is the `hit`. Bunker
  chips, shot cancels, Jammer's glob and the fleet's descent have none either.
- **The sources' lifecycle**: a flyer leaving the screen is despawned by `update_flyer`, which is
  its visibility exit — the engine ends its source in that frame's tail, with no call from the
  game. A cannon's last life despawns it in `player_hit`, ending its source; a surviving hit keeps
  the entity and the source. `start_game` and `reset_to_title` despawn what the match spawned, so
  the next match's cannon activates on its first fire.
- **The order within a frame** is `update_gameplay`'s: the shots (`update_firing`), the flyer
  entering, the busts and singes (each cannon's shots in order), the bonus, the hits, the clear.
  `Music.triggered`, built only in tests, records the frame's calls.
- **A match start decides by the song in the slot** (`GameMusic::restart`, from `start_game`, which
  every match start takes): none → requested again; the game's own → back to its top; another song
  (the composer's audition in the editor) → left alone.
- **The bob** — there is no grid here, the countertop's grout is the grid: on each bar line every
  live slice's art dips `BEAT_BOB_BAR_PIXELS` (4) below its spec's offset, on any other beat
  `BEAT_BOB_BEAT_PIXELS` (2; zero is bars only), and comes back up over `BEAT_BOB_SECONDS` (0.12 s).
  World Y is up, so a dip is a smaller `Sprite.offset.y`. **Only the sprites move**: the bodies that
  the landing and the bunkers read are never touched, so a dip never lands the fleet or chips a
  bunker. It is written after `update_gameplay`'s steps, while Playing; the fleet is put back at rest
  when the pause menu is up, on a restart, at the match's end and on the way to the title, so no dip
  is ever drawn without a beat under it.
- **The pause menu keeps the engine's labels**, and its hint says nothing of the jam: a hint that
  does overflows the engine's 300 px pause panel (insiculous_2d#202), and the text waits on that.
  Only the pause menu jams.
- **No recorded fallback.** A song that cannot play is logged once at `error` ("toast busters: the
  song could not play: …"), every trigger is refused and counted, and the game is as silent as it
  was before it had music; each match start asks again.
- **Telemetry**: every five seconds a `conductor=on …` line with the game's `shots`, `busted`,
  `singes`, `flyers`, `bonuses`, `hits`, `clears`, `jam_units`, `pulsed` and `refused`, at `info`
  when asked — `?telemetry=1` on the web page, `TOAST_BUSTERS_TELEMETRY=1` natively. `init` ends
  with `toast busters: init complete`, the line the boot measurement stops on.
- **The bank** is `assets/soundfonts/toast_busters.sf2`, FluidR3 GM's xylophone (0:13), clarinet
  (0:71), tubular bells (0:14) and acoustic bass (0:32), 3.43 MiB, cut by
  `scripts/prepare_soundfont.sh` — a wrapper over the engine's `scripts/prepare_game_soundfont.sh`
  with `--bank <path>` for a FluidR3_GM.sf2 got any other way; the source's and the output's
  sha256s are in the `LICENSE` beside it. No violin fits: FluidR3's is 3 MiB alone, and any string
  preset beside the xylophone is over the cap. Budget: **the bank under 4 MiB** (the engine's script
  refuses to write one over it), the song under 256 KiB. On the web every asset, the bank
  included, is fetched before the first frame.
- The browser bundles moved to **v7** (game) and **v6** (editor) with the music
  (`src/web_entry.rs` names the four places each version lives).

## Work tracking

Open work lives on the **Studio Board** (https://github.com/orgs/beinsiculous/projects/1)
as issues in this repo. **Always pass `-R beinsiculous/space_invaders`** — a bare `gh` command
resolves against the session's working directory, which is often the working-set root, so
it lists and files against the wrong repository.

```sh
gh issue list -R beinsiculous/space_invaders
gh api repos/beinsiculous/space_invaders/milestones --jq '.[] | "\(.title): \(.description)"'
```

Issues are grouped into **sprint milestones**; each description records the batch's
internal order and its gates. Take the next unblocked issue in a sprint, not an arbitrary
one. Claim by assigning yourself; close with `fixes beinsiculous/space_invaders#N` in the commit.

**Unfinished work becomes an issue.** Anything you don't finish — work you deferred, debt
you created, a follow-up you spotted — is filed before you report done. Never buried in a
doc, never left as a bare `TODO:`, never dropped. The `file-issue` skill carries the shape;
`sprint-planning` groups issues into shippable batches.

## Review workflow

The adversarial-review skill lives in `.claude/skills/`. Approved plans go to `review/plan.md` and are reviewed via `scripts/request-review.sh plan review/plan.md --reviewer=kimi` BEFORE implementation. Commits over 100 changed lines are gated by `scripts/commit-review-hook.sh` — the `ADV_REVIEWED=1` prefix is used only after a code-mode review adjudicated with the user, or when the user explicitly skipped review. `review/` is gitignored transients. NOTE: `scripts/request-review.sh` and `scripts/commit-review-hook.sh` are copies — the canonical ones live in the working-set root, not in `insiculous_2d`. Never edit a copy: fix the root's and re-copy, and `scripts/check-skill-parity.sh` there reports any repo that drifted.
