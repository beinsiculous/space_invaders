# Toast Busters

Space Invaders, re-skinned into the Deion world and built on the sibling
[`insiculous_2d`](../../insiculous_2d) engine. Deion — and Cubert in co-op — hold
the counter against four rows of the **Toasted**: haunted slices of bread marching
over four butter bunkers, while Maxwell's cake saucer and Jammer cross the lane
above them. Everything is pixel art drawn at one art pixel per window pixel.

## Running

```bash
cargo run                     # play
cargo run --features editor   # play inside the engine's scene editor
cargo test                    # 68 headless tests
```

The same build runs in the browser at [beinsiculous.com/playground/invaders/](https://beinsiculous.com/playground/invaders/): the game inside the editor, layout only — the rules are compiled in and nothing you change there persists.

Requires the `insiculous_2d` engine checked out side by side
(`../../insiculous_2d`) — the Cargo dependency is a relative path.

## Controls

| Action | Player 1 | Player 2 | Gamepad |
|---|---|---|---|
| Move | A / D | ← / → arrows | Left stick / D-pad (pad 0 = P1, pad 1 = P2) |
| Fire (hold to auto-fire) | Space or mouse button | Enter | A |
| Move (mouse) | Mouse X (single player only) | — | — |
| Pause | Esc | Esc | Start |
| Menus: navigate / confirm / back | W/S, Space, Esc | ↑/↓, Enter, Esc | D-pad, A, Start |
| Collider debug overlay | F1 | F1 | — |

In **single player** Deion answers to BOTH input slots — WASD, arrows, and either
gamepad all drive him, and the mouse takes over on movement. In **2P co-op** Deion
is player 1 and Cubert player 2, each on their own slot. Bindings persist to
`saves/input_settings.json` (hand-editable JSON).

## Mechanics

- **One formation per match**, four rows of ten: **Sour-Doom** (30 points, takes
  two hits), **Bagelgeist** (20), **Rye.I.P.** (20) and **Plain Wraith** (10), top
  row down. The formation marches as one body: right until the outermost *live*
  column touches the bound, then it drops a step and reverses. Dead columns widen
  the march, and the pace ramps from a crawl to a sprint as the fleet thins. Bust
  the last Toasted to win.
- **Bagelgeist has a hole.** A shot that first touches a bagel inside its ring's
  hole passes through it — and goes on to whatever is above.
- **You lose** when a Toasted's body reaches the top of Deion's head height, or
  when every cannon runs out of lives (3 each). A lost life plays Deion's or
  Cubert's hurt; the cannon reappears at its start a moment later. In co-op a
  cannon out of lives leaves the field and its partner plays on.
- **Trigger discipline:** one shot in flight in Normal mode. Deion fires icicles,
  Cubert ice chips; the Toasted drop burnt crumbs. Shots that cross cancel in a
  flash, and a cancel or a miss resets your streak.
- **Butter bunkers** chip away block by block — from your shots, the crumbs, and
  the Toasted chomping through them — and every block draws its bunker's damage
  state: intact, damaged, critical, melted.
- **The flyby lane:** Maxwell's cake saucer pays a mystery 50/100/150/300 bonus.
  Once in four flybys it is **Jammer** instead, who drops jam globs: a glob on a
  cannon jams it for a moment — no shot, no life lost. Busting Jammer pays 100.
- **Chaos modes:** Insane = the Toasted march 1.8× faster, you get 4 shots in the
  air. Ridiculous = they fire 2.4× as often, you answer with twin volleys.
  Insiculous = both at once, with stacked twin volleys.
- **Achievements** (11): clear and perfect-clear per chaos mode, a 10-bust
  Sharpshooter streak, Last Slice, and Saucer Hunter — persisted to
  `saves/space_invaders_achievements.json`.

## Still to come

Toast Busters' levels — one bread per level, the level cards, Poptergeist and Pain
Perdu, the toast shading by chaos tier, and Stay Pufft's finale — are their own batch
(`space_invaders#3`). The site still lists the game as *Insiculous Invaders*.

The art is quarantined AI-assisted pixel art (`ai_` prefix, synced from
`deion_assets`): it ships in free builds only until a person's cleanup pass.
