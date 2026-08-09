# Insiculous Space Invaders

A neon Geometry-Wars-styled take on Space Invaders, built on the sibling
[`insiculous_2d`](../../insiculous_2d) engine (game 3 of the 20 Games
Challenge). Everything on screen is a glowing rectangle: a 5×10 alien fleet
marches over four destructible bunkers while one or two cannons hold the
line, with bloom, particle bursts, and a spring-mass background grid that
ripples on every kill.

## Running

```bash
cargo run                     # play
cargo run --features editor   # play inside the engine's scene editor
cargo test                    # 36 headless tests
```

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

In **single player** the lone cannon answers to BOTH input slots — WASD,
arrows, and either gamepad all drive it, and the mouse takes over on
movement. In **2P co-op** each cannon binds to its own player slot.
Bindings persist to `saves/input_settings.json` (hand-editable JSON).

## Mechanics

- **One formation per match.** The 5×10 fleet marches as a single body:
  right until the outermost *live* invader touches the bound, then it drops
  a step and reverses. Dead columns widen the march span, and the march
  speed ramps from a crawl (full fleet) to a sprint (last invader
  standing). Kill the last invader to win — there is no second wave today.
- **You lose** when any invader reaches the invasion line or touches a live
  cannon, or when every cannon runs out of lives (3 each). In co-op, a
  cannon at zero lives despawns and the survivor plays on.
- **Trigger discipline:** classic one-bullet-in-flight rule in Normal mode
  (chaos modes relax it — see below). Bullets that cross an enemy bullet
  cancel in a flash; a cancel or a miss resets your kill streak.
- **Bunkers** are grids of small blocks chipped away by bullets from both
  sides — and chomped directly by low-marching invaders.
- **UFO:** a mystery ship occasionally crosses the top lane; shooting it
  pays a classic 50/100/150/300 bonus.
- **Scoring:** 30/20/20/10/10 points by row, top row pays most. Co-op keeps
  separate scores and lives per cannon (shared fleet, shared fate).
- **Chaos modes:** Insane = the fleet marches 1.8× faster, you get 4 shots
  in the air. Ridiculous = the fleet fires 2.4× as often, you answer with
  twin cannons. Insiculous = both at once, with stacked twin-cannon
  volleys.
- **Achievements** (11): clear and perfect-clear per chaos mode, a
  10-kill-shot Sharpshooter streak, Last Stand, and UFO Hunter — persisted
  to `saves/space_invaders_achievements.json`.

## The Deion Pivot: Burger Invaders

The planned Phase G re-skin (the game is still neon today): the invasion
becomes **Burger Invaders**, where every level is one layer of a burger
being assembled bottom-up, and clearing a layer stacks it onto the bun.

- **The defenders:** side-scroller **Deion** firing icicles UP from his
  mohawk (his universal projectile language); P2 is **Cubert** firing ice
  chips — smaller, scrappier.
- **The burger builds as you win:**

```
        ... ???  (top bun? Maxwell?)         L?
        ~ ~ ~  onions                        L6?
        o o o  pickles                       L5?
        ●●●●●  tomato                        L4?
        ~~~~~  lettuce   → leafy guys        L3
        ▲▲▲▲▲  cheese    → cheese-wedge guys L2
        █████  patty     → angry meatballs   L1
       (~~~~~) bottom bun → In-Bread Yokels
```

- **Enemy ranks reflect the layer being fought:** bun/bread layers are the
  **In-Bread Yokels** (canon rank-and-file — toast slices with fried-egg
  faces and a march-wiggle); the patty layer is the **little brown angry
  meatball guys** (the same cross-game characters as the rocks in
  Meatieroids and a hazard in Hot Dog!); cheese = cheese-wedge guys;
  lettuce = leafy guys; onward per ingredient.
- **Bunkers become burger buns** with bites taken out as they degrade
  (kept from existing canon — the block-chomping already behaves like
  bites). The **UFO flyby becomes Dr. Maxwell on his cake saucer** (kept).
- Style source of truth: `deion_assets/DEION_STYLE.md` (root symlink to the
  shared art repo). 16px cells, nearest filtering, 5× integer scale.

### Open questions

- Full ingredient/level order and total level count (tomato, pickles,
  onions… in what order, ending where?).
- Per-layer enemy designs beyond Yokels / meatballs / cheese wedges — the
  new minor characters need Jesse's design pass.
- Does per-level enemy variety need a roster system, versus today's single
  hardcoded formation (row colors/values only)? Today the game has NO level
  progression at all — the burger stack implies building one.
- Boss burger finale? Is Dr. Maxwell waiting at the top of the stack?

Answered questions move up into the theme spec above and get DELETED from
this list (live-docs convention).
