# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
cargo run                     # play the game
cargo run --features editor   # run the game inside the engine's scene editor
cargo build                   # compile check
cargo test                    # run tests (36: gameplay_tests.rs 25, spawning.rs 6, achievements.rs 5)
cargo test <test_name>        # run a single test
```

The game depends on the `insiculous_2d` engine by relative path (`../../insiculous_2d`); both checkouts must sit side by side or nothing builds. Engine crates used: `engine_core` (always) and `editor_integration` (only behind the `editor` feature).

## Architecture

This is a single-crate game (`insiculous_space_invaders`) built on the in-house `insiculous_2d` ECS engine. `SpaceInvadersGame` (in `src/types.rs`) implements the engine's `Game` trait in `src/main.rs` — `init()` loads the font, registers achievements, and spawns only the background + deforming grid; the cannons, fleet, and barriers all spawn fresh in `start_game()` (`gameplay/flow.rs`) once the menus pick a mode. With `--features editor` the identical game runs inside the engine's scene editor via `editor_integration::run_game_with_editor`; no game code changes between the two modes.

**State machine drives everything.** `GameState` (types.rs) is matched at the top of `update()` in main.rs: `TitleScreen`/`ModeSelect`/`Achievements` dispatch to handlers in `menu.rs`; `Playing` and `GameOver { won }` fall through to `update_gameplay()` in `gameplay/mod.rs`, which orchestrates the per-frame steps implemented across `gameplay/{players,combat,formation,ufo,flow}.rs`. Flow is Title → Mode select (1P / 2P co-op) → Chaos select → Playing → GameOver (Space restarts, Esc to title). **There is no wave/level progression** — one formation per match; killing the last invader is victory, and every start rebuilds the full fleet, barriers, and roster from scratch.

**Formation march is an offset, not per-entity AI.** Each `Invader` keeps its immutable `(row, col)` home slot (`invader_home_x/y` in spawning.rs); the whole fleet shares one `formation_offset` + `march_dir`. Pure `march_step()` bounces the offset off `MARCH_BOUND_X` using the outermost **live** invaders' home X (dead columns widen the march span) and reports `descended`; each bounce drops the offset by `DESCEND_STEP`. `march_speed()` ramps linearly from `MARCH_SPEED_BASE` (full fleet) to `MARCH_SPEED_MAX` (last invader). Every frame each invader's kinematic body is retargeted to home + offset via `set_kinematic_target`. Defeat when any invader crosses `INVASION_Y` or touches a live cannon (`fleet_has_landed`).

**Collision is hybrid — rapier events where they exist, manual AABB where they can't.** Bullets are dynamic sensors (gravity 0, `Lifetime` safety net); invaders/cannons/UFO are kinematic; barrier blocks are static sensors. Rapier `take_collision_events()` (drained ONCE per frame into a shared `Vec`) drives bullet-vs-invader, bullet-vs-UFO, bullet-vs-cannon, and bullet-vs-barrier. Rapier does **not** report kinematic-vs-static or kinematic-vs-kinematic pairs, so `rects_overlap()` (gameplay/mod.rs) covers: marching invaders chomping barrier blocks, invader-touches-cannon defeat, and bullet-vs-bullet cancels (those pairs rapier would report, but closing speeds can tunnel a whole frame — the AABB test inflates by half a frame of closing speed instead).

**Bullets are owned lists, not queries.** Each `PlayerState` owns its `bullets: Vec<EntityId>` (per-cannon fire cooldown, live-bullet cap, score, lives, sharpshooter streak); the fleet shares one `invader_bullets` Vec. Invader return fire and UFO spawns are deterministic pseudo-random draws — `hash_f32/hash_u32` over `frame_count` with distinct salts (ufo.rs) so streams don't correlate. The UFO is a purely optional bonus target crossing the top lane (`ufo_entry`/`ufo_offscreen`/`ufo_bonus`, classic 50/100/150/300 table).

**2P co-op conventions:** everything in `gameplay/players.rs` is parameterized by player index so 1P and co-op share one code path. Single player fields one center cannon answering to BOTH input slots (WASD+arrows+either pad; mouse takeover is 1P-only and P1-only); co-op splits two cannons to ∓`CANNON_COOP_OFFSET`, each on its own `PlayerId`. P1 wears the chaos theme's accent color, P2 the fixed `PLAYER2_COLOR`. Per-cannon bullet caps are independent (`volley_fits`); the fleet's return fire tests every live cannon; a cannon at 0 lives despawns (and stops being an invasion target) while the survivor plays on — `coop_defeated` ends the match only when every cannon is out. Gameplay reads `ctx.players` (`move_x`, `GameAction::Action1`, `just_activated_any`), never raw keycodes (F1 debug toggle is the sanctioned exception).

**Editor naming:** every spawned entity gets a `Name` component ("Invader r0 c3", "Barrier 2 r1 c4", "Player Bullet", "UFO") so the editor hierarchy is readable — keep this when adding entities.

**Coordinate and scale conventions (the main trap):**
- World origin is screen center; window is 800×600 (`WIN_W`/`WIN_H`).
- The renderer multiplies `Transform2D.scale` by `RENDER_UNIT = 80.0` to get pixel size — that's why sprite scales are `size / 80.0` (see `PLAYER_SCALE`, `INVADER_SCALE`, etc.).
- Collider shapes use **absolute pixels** and IGNORE `Transform2D.scale` entirely. Sprites and colliders are sized through different paths, so they can silently diverge. `F1` in-game (or `C` in the editor) overlays magenta collider outlines to check.

**All tuning lives in `src/constants.rs`** (sizes, speeds, formation layout, fire rates, chaos multipliers, colors, emissives) and all entity creation lives in `src/spawning.rs` + the `spawn_ufo`/`spawn_bullet` recipes, spawned from those constants. Values tuned live in the editor inspector must be copied back into constants.rs to persist.

**Chaos modes** (engine `ChaosMode`) split the fleet buffs deliberately — `player_fire_caps()` in gameplay/combat.rs is the SSOT for the player's answer, returning `(shots_per_fire, max_live_bullets)`:
- **Normal** — the classic: 1 bullet in flight.
- **Insane** — fleet marches 1.8× faster (`INSANE_MARCH_MULT`); player gets 4 single shots on screen.
- **Ridiculous** — fleet fires 2.4× as often (`RIDICULOUS_FIRE_MULT`); player gets twin cannons (2-shot volleys, one volley in flight).
- **Insiculous** — both buffs; twin cannons AND stacked volleys (cap 6).
The regression test `fleet_buffs_split_across_chaos_modes` pins this split — Insane must NOT fire faster, Ridiculous must NOT march faster. `ChaosTheme::for_mode` drives colors; `apply_theme()` (flow.rs) re-tints live entities; the menu mirrors the pick into `ctx.chaos_mode`.

**Achievements** (achievements.rs): 11 total — clear + perfect (no life lost by ANY cannon) per chaos mode, plus Sharpshooter (10 consecutive kill shots — a bullet cancel or miss resets the streak, tracked per cannon in `finish_player_shot`), Last Stand (win with some cannon on its last life), UFO Hunter. Persist to `saves/space_invaders_achievements.json`.

**Visuals:** the Geometry-Wars look is neon rects off one white 1×1 texture — `Sprite::with_emissive` feeds the engine bloom (bullets 2.5, UFO 1.8, cannon 1.5, invaders 0.9, barriers 0.6), particle presets live in `effects.rs`, and kills/hits kick radial impulses into the spring-mass grid (`ripple_grid`). Gameplay sprites are hidden on menu screens via `update_entity_visibility`. This game has **no localization** (unlike Pong) — strings are literals.

**Tests (36)** are fully headless: pure math (march, caps, UFO draws, co-op rules) plus physics simulations that step `PhysicsSystem` with the **exact spawn recipes** the game uses (bullet-vs-invader/UFO/cannon/barrier contact registration, Lifetime expiry). They live in `src/gameplay_tests.rs` (25, `#[cfg(test)]` module wired in main.rs), `src/spawning.rs` (6, layout invariants like "invasion line sits below the barriers"), and `src/achievements.rs` (5, registry/display parity).

**Paths:** assets and saves resolve through `engine_core::game_root!()` in main.rs (exe directory if it contains `assets/`, else `CARGO_MANIFEST_DIR`), so `cargo run` works from any cwd. Input bindings persist to `saves/input_settings.json`.

## The Deion Re-skin (Phase G): Burger Invaders

Planned identity — the game still ships the neon look today. Invaders is FOURTH in the Phase G re-skin order (Pong → Frogger → Breakout → Snake → **Invaders** → Asteroids).

- **New title: Burger Invaders.** The player cannon becomes **side-scroller Deion** firing **icicles UP from his mohawk** — his universal projectile language across the games. P2's cannon becomes **Cubert** firing ice chips (smaller, scrappier projectiles).
- **Levels BUILD A BURGER bottom-up:** L1 = single patty (bun + patty), L2 = + cheese, L3 = + lettuce, then tomato / pickles / onions… (exact ingredient order and level count still open). NOTE: today's code has NO level progression (one formation per match) — the burger-stack structure implies adding one, which is new engineering, not just re-skinning.
- **Enemy ranks reflect the layer being fought:** bun/bread layers = **In-Bread Yokels** (canon rank-and-file: toast slices with fried-egg faces, march-wiggle); patty layer = **little brown angry meatball guys** (shared cross-game character — the rocks in Meatieroids, a hazard in Hot Dog!); cheese layer = **cheese-wedge guys**; lettuce = leafy guys; etc. New minor characters need Jesse's design pass.
- **Bunkers = burger buns** with bites taken as they degrade (kept from existing canon — today's per-block chomping maps naturally onto bite marks). **UFO flyby = Dr. Maxwell on his cake saucer** (kept).
- **Style SSOT:** `deion_assets/DEION_STYLE.md` via the root symlink (the symlink assumes the standard side-by-side checkout — the same requirement the Cargo path dep already imposes). Settled metrics: 16px base cell, nearest filtering, 5× integer scale to RENDER_UNIT=80, one art cell = one world unit; never fake a footprint via `Transform2D.scale`. Deion's chaos forms apply (Insane steam / Ridiculous ice / Insiculous 32×32 giant — the 2×2-cell footprint means an honest bigger collider, since colliders are absolute pixels and ignore scale).
- **Runtime assets arrive ONLY via the deion_assets sync copy into `assets/sprites/`** (F2, not yet built) — never symlink or hand-copy art in. AI art is quarantined (`ai_` prefix, `deion_assets/ai/` only) — tiered ship rule (DEION_STYLE.md §6, Aug 19 2026): may ship in FREE web builds, never in paid/marketplace builds; `deion_assets/scripts/check_no_ai_assets.sh` must pass on any paid release's asset tree. Sheet clip names are the stable API.

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
