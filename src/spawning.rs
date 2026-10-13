//! All entity creation. Every entity gets a `Name` component so the editor hierarchy
//! shows "Sour-Doom r0 c3" instead of "Entity 7".
//!
//! Every art entity is built the same way: the sheet's first cell, the cell-sized
//! transform scale, and the sheet's measured anchor as the sprite offset, so the art
//! lands on the entity rather than at the cell's corner. Entities that change clip under
//! their own rules carry a `ClipStateMachine` whose table names their states and clips.

use engine_core::prelude::*;

use crate::art::*;
use crate::constants::*;
use crate::types::*;

/// The components every art entity carries: the sheet's first cell as its sprite at
/// the sheet's depth, the sheet's clips as its animation, and the anchor offset that
/// puts the measured box on the entity.
pub(crate) fn art_components(sheet: &SpriteSheet, spec: &SheetSpec, depth: f32) -> (Sprite, SpriteAnimation) {
    (sheet.sprite().with_offset(spec.sprite_offset()).with_depth(depth), sheet.animation())
}

/// A Toasted marches; Sour-Doom's first hit plays `hurt` once and marches on.
pub(crate) fn toasted_machine(kind: Toasted) -> ClipStateMachine {
    let mut states = vec![(MARCH.to_string(), ClipState::staying(MARCH))];
    if kind == Toasted::SourDoom {
        states.push((HURT.to_string(), ClipState::new(HURT, OnFinished::Next(MARCH.to_string()))));
    }
    ClipStateMachine::new(MARCH, states)
}

/// A cannon stands `idle`, `walk`s while moving, and plays `fire` out on each shot
/// before it rests again. `hurt` is never the cannon's: a lost life plays it as a
/// detached one-shot.
pub(crate) fn hero_machine() -> ClipStateMachine {
    ClipStateMachine::new(IDLE, vec![
        (IDLE.to_string(), ClipState::staying(IDLE)),
        (WALK.to_string(), ClipState::staying(WALK)),
        (FIRE.to_string(), ClipState::new(FIRE, OnFinished::Next(IDLE.to_string()))),
    ])
}

/// What a detached one-shot draws and where.
pub(crate) struct Effect<'a> {
    pub(crate) name: &'a str,
    pub(crate) spec: &'a SheetSpec,
    pub(crate) sheet: &'a SpriteSheet,
    pub(crate) depth: f32,
    pub(crate) clip: &'a str,
    pub(crate) position: Vec2,
    pub(crate) facing: Facing,
}

/// Spawn a detached one-shot: a burst, a splash, a lost life's hurt. Sprite-only, no
/// collider. With `hold` it stays on its last frame (the final defeat's hurt);
/// otherwise it ends itself. The caller keeps the handle in `transient_visuals`.
pub(crate) fn spawn_effect(world: &mut World, effect: &Effect, hold: bool) -> EntityId {
    let (sprite, animation) = art_components(effect.sheet, effect.spec, effect.depth);
    let machine = if hold { ClipStateMachine::single(effect.clip) } else { ClipStateMachine::one_shot(effect.clip) };
    let entity = world.spawn()
        .with(Name::new(effect.name))
        .with(Transform2D::from_parts(effect.position, 0.0, effect.spec.scale()))
        .with(sprite)
        .with(animation)
        .with(machine)
        .id();
    face(world, entity, effect.spec, effect.facing);
    entity
}

/// Turn `entity`'s art to `facing`: a negative x-scale mirrors the cell, and the
/// sprite offset's x is negated with it, because the engine turns an offset with the
/// entity's rotation but never scales it.
pub(crate) fn face(world: &mut World, entity: EntityId, spec: &SheetSpec, facing: Facing) {
    if let Some(transform) = world.get_mut::<Transform2D>(entity) {
        transform.scale.x = spec.scale().x * facing.sign();
    }
    if let Some(sprite) = world.get_mut::<Sprite>(entity) {
        sprite.offset.x = spec.sprite_offset().x * facing.sign();
    }
}

/// Spawn the countertop: a `Tilemap` of its tile, `ceil(WIN / tile)` cells each way,
/// centred on the window. Tiles are placed in pixel units from the entity's position,
/// which anchors the centre of tile (0, 0) — the top-left cell, with rows growing down.
pub(crate) fn spawn_countertop(world: &mut World, sheet: &SpriteSheet) -> EntityId {
    let columns = (WIN_W / COUNTERTOP_TILE_PX).ceil() as u32;
    let rows = (WIN_H / COUNTERTOP_TILE_PX).ceil() as u32;

    let mut map = Tilemap::new(columns, rows, COUNTERTOP_TILE_PX);
    map.tileset = sheet.texture.id;
    map.tile_uv_size = Vec2::new(1.0 / sheet.grid.cols as f32, 1.0 / sheet.grid.rows as f32);
    map.tiles = vec![1; (columns * rows) as usize];
    map.depth = COUNTERTOP_DEPTH;

    let anchor = Vec2::new(
        -(columns as f32 - 1.0) * COUNTERTOP_TILE_PX / 2.0,
        (rows as f32 - 1.0) * COUNTERTOP_TILE_PX / 2.0,
    );
    world.spawn().with(Name::new("Countertop")).with(Transform2D::new(anchor)).with(map).id()
}

/// The backdrop grid's colour for a chaos theme: its grid colour at the low alpha that
/// lets the lattice read over the countertop without veiling it.
pub(crate) fn backdrop_color(theme: &ChaosTheme) -> Vec4 {
    let grid = theme.grid_color;
    Vec4::new(grid.x, grid.y, grid.z, BACKDROP_ALPHA)
}

/// Spawn the deforming grid drawn over the art — the countertop is an opaque map, so a
/// lattice under the sprites would never show. The engine simulates and draws it;
/// gameplay events ripple it.
pub(crate) fn spawn_backdrop(world: &mut World, theme: &ChaosTheme) -> EntityId {
    world.spawn()
        .with(Name::new("Grid Backdrop"))
        .with(Transform2D::new(Vec2::ZERO))
        .with(GridBackdrop {
            draw_order: GridDrawOrder::OverSprites,
            color: backdrop_color(theme),
            ..GridBackdrop::default()
        })
        .id()
}

/// The cell centre of formation slot (`row`, `column`) before any march.
pub(crate) fn invader_home_cell(row: usize, column: usize) -> Vec2 {
    Vec2::new(column_cell_x(column), ROW_CELL_Y[row.min(ROW_CELL_Y.len() - 1)])
}

/// Where the Toasted in slot (`row`, `column`) stands before any march: its body's
/// centre, with its cell centred on the slot's whole pixel.
pub(crate) fn invader_home(row: usize, column: usize) -> Vec2 {
    Toasted::for_row(row).spec().sheet.position_for_cell(invader_home_cell(row, column))
}

/// Spawn the full formation, the roster by row: kinematic bodies colliding as their
/// measured body boxes, each marching on its own sheet. The formation moves as one —
/// each frame gameplay retargets every body to its home plus the shared offset.
pub(crate) fn spawn_invaders(world: &mut World, sheets: &Sheets) -> Vec<Invader> {
    let mut invaders = Vec::with_capacity(ROSTER.len() * FORMATION_COLUMNS);
    for (row, &kind) in ROSTER.iter().enumerate() {
        let spec = kind.spec();
        let size = spec.sheet.bounds.1 - spec.sheet.bounds.0;
        for column in 0..FORMATION_COLUMNS {
            let (sprite, animation) = art_components(sheets.toasted(kind), spec.sheet, spec.depth);
            let entity = world.spawn()
                .with(Name::new(format!("{} r{row} c{column}", spec.name)))
                .with(Transform2D::from_parts(invader_home(row, column), 0.0, spec.sheet.scale()))
                .with(sprite)
                .with(animation)
                .with(toasted_machine(kind))
                .with(RigidBody::new_kinematic().with_rotation_locked(true))
                .with(Collider::box_collider(size.x, size.y).with_friction(0.0))
                .id();
            invaders.push(Invader { entity, row, column, kind, hits_left: spec.hits });
        }
    }
    invaders
}

/// The centre of block (`row`, `column`) of bunker `bunker`, row 0 the top.
pub(crate) fn bunker_block_position(bunker: usize, row: usize, column: usize) -> Vec2 {
    let left = BUNKER_XS[bunker] - BUNKER_CELL.x / 2.0;
    let top = BUNKER_CELL_Y + BUNKER_CELL.y / 2.0;
    Vec2::new(
        left + (column as f32 + 0.5) * BUNKER_BLOCK_PX,
        top - (row as f32 + 0.5) * BUNKER_BLOCK_PX,
    )
}

/// Whether a block at (`row`, `column`) stands in `state`: its slice draws enough.
pub(crate) fn block_stands(state: BunkerState, row: usize, column: usize) -> bool {
    BUNKER_SLICE_OPAQUE[state.index()][row][column] >= BLOCK_MIN_OPAQUE_PIXELS
}

/// The part of the bunker sheet a block at (`row`, `column`) draws in `state`: its 16 px
/// slice of the cell the state's clip shows, in texture coordinates. A sheet without the
/// clip — the placeholder before `init` — falls back to the state's place in the order.
pub(crate) fn block_tex_region(sheet: &SpriteSheet, state: BunkerState, row: usize, column: usize) -> [f32; 4] {
    let columns = sheet.grid.cols.max(1);
    let rows = sheet.grid.rows.max(1);
    let cell = sheet.clips.iter()
        .find(|(name, _)| name == state.clip())
        .and_then(|(_, clip)| clip.frame_indices.first().copied())
        .unwrap_or(state.index() as u32);
    let sheet_size = Vec2::new(columns as f32 * BUNKER_CELL.x, rows as f32 * BUNKER_CELL.y);
    let x = (cell % columns) as f32 * BUNKER_CELL.x + column as f32 * BUNKER_BLOCK_PX;
    let y = (cell / columns) as f32 * BUNKER_CELL.y + row as f32 * BUNKER_BLOCK_PX;
    [x / sheet_size.x, y / sheet_size.y, BUNKER_BLOCK_PX / sheet_size.x, BUNKER_BLOCK_PX / sheet_size.y]
}

/// Spawn every bunker's blocks, sliced from its `intact` art: static sensors a shot
/// reports and game code removes — nothing bounces off butter. A block whose slice
/// draws too little never spawns.
pub(crate) fn spawn_bunkers(world: &mut World, sheet: &SpriteSheet) -> Vec<BunkerBlock> {
    let mut blocks = Vec::new();
    let block_scale = Vec2::splat(BUNKER_BLOCK_PX / RENDER_UNIT);
    for bunker in 0..BUNKER_COUNT {
        for row in 0..BUNKER_BLOCK_ROWS {
            for column in 0..BUNKER_BLOCK_COLUMNS {
                if !block_stands(BunkerState::Intact, row, column) {
                    continue;
                }
                let [x, y, width, height] = block_tex_region(sheet, BunkerState::Intact, row, column);
                let entity = world.spawn()
                    .with(Name::new(format!("Bunker {bunker} r{row} c{column}")))
                    .with(Transform2D::from_parts(bunker_block_position(bunker, row, column), 0.0, block_scale))
                    .with(sheet.sprite().with_tex_region(x, y, width, height).with_depth(BUNKER_DEPTH))
                    .with(RigidBody::new_static())
                    .with(Collider::box_collider(BUNKER_BLOCK_PX, BUNKER_BLOCK_PX).as_sensor())
                    .id();
                blocks.push(BunkerBlock { entity, bunker, row, column });
            }
        }
    }
    blocks
}

/// Spawn `hero`'s cannon at `x`: kinematic, colliding as its envelope — the union of
/// its idle, walk and fire boxes — so no pose it can draw pokes out of its collider.
/// The collider never turns; only the art faces.
pub(crate) fn spawn_cannon(world: &mut World, sheet: &SpriteSheet, hero: Hero, x: f32) -> EntityId {
    let spec = hero.spec();
    let size = hero.half_envelope() * 2.0;
    let (sprite, animation) = art_components(sheet, spec.sheet, spec.depth);
    world.spawn()
        .with(Name::new(spec.name))
        .with(Transform2D::from_parts(Vec2::new(x, cannon_y(hero)), 0.0, spec.sheet.scale()))
        .with(sprite)
        .with(animation)
        .with(hero_machine())
        .with(RigidBody::new_kinematic().with_rotation_locked(true))
        .with(Collider::box_collider(size.x, size.y).with_friction(0.0))
        .id()
}

/// Spawn one HUD life icon: a world sprite in the HUD band, because the UI's image
/// command cannot pick a sheet's cell.
pub(crate) fn spawn_life_icon(world: &mut World, sheet: &SpriteSheet, hero: Hero, position: Vec2) -> EntityId {
    let (sprite, animation) = art_components(sheet, &LIFE_ICON, LIFE_ICON_DEPTH);
    world.spawn()
        .with(Name::new(format!("{} Life", hero.spec().name)))
        .with(Transform2D::from_parts(position, 0.0, LIFE_ICON.scale()))
        .with(sprite)
        .with(animation)
        .with(ClipStateMachine::single(hero.spec().life_icon_clip))
        .id()
}

/// What a shot is drawn as and how it collides.
pub(crate) struct ShotRecipe<'a> {
    pub(crate) name: &'a str,
    pub(crate) spec: &'a SheetSpec,
    pub(crate) sheet: &'a SpriteSheet,
    pub(crate) depth: f32,
    pub(crate) half_collider: Vec2,
}

/// Spawn a shot at `position` (its drawn box's centre) flying at `velocity`: a dynamic
/// sensor with a `Lifetime` safety net, playing `fly`.
pub(crate) fn spawn_shot(world: &mut World, physics: &mut PhysicsSystem, recipe: &ShotRecipe, position: Vec2, velocity: Vec2) -> EntityId {
    let (sprite, animation) = art_components(recipe.sheet, recipe.spec, recipe.depth);
    let size = recipe.half_collider * 2.0;
    let entity = world.spawn()
        .with(Name::new(recipe.name))
        .with(Transform2D::from_parts(position, 0.0, recipe.spec.scale()))
        .with(sprite)
        .with(animation)
        .with(ClipStateMachine::single(FLY))
        .with(RigidBody::new_dynamic()
            .with_gravity_scale(0.0)
            .with_rotation_locked(true)
            .with_linear_damping(0.0)
            .with_angular_damping(0.0))
        .with(Collider::box_collider(size.x, size.y).as_sensor())
        .with(Lifetime::new(SHOT_LIFETIME))
        .id();
    // Buffered-safe on the spawn frame; applied once the body syncs.
    physics.set_velocity(entity, velocity, 0.0);
    entity
}

/// The sheet and clip a flyer crosses the lane on.
pub(crate) fn flyer_art(kind: FlyerKind) -> (&'static SheetSpec, f32, &'static str, &'static str) {
    match kind {
        FlyerKind::Saucer => (&SAUCER, SAUCER_DEPTH, FLY, "Maxwell's Saucer"),
        FlyerKind::Jammer => (&JAMMER, JAMMER_DEPTH, MARCH, "Jammer"),
    }
}

/// Spawn a flyer with its cell centred on (`x`, the lane): kinematic, colliding as
/// its measured body.
pub(crate) fn spawn_flyer(world: &mut World, sheet: &SpriteSheet, kind: FlyerKind, x: f32) -> EntityId {
    let (spec, depth, clip, name) = flyer_art(kind);
    let size = spec.bounds.1 - spec.bounds.0;
    let (sprite, animation) = art_components(sheet, spec, depth);
    world.spawn()
        .with(Name::new(name))
        .with(Transform2D::from_parts(spec.position_for_cell(Vec2::new(x, FLYBY_CELL_Y)), 0.0, spec.scale()))
        .with(sprite)
        .with(animation)
        .with(ClipStateMachine::single(clip))
        .with(RigidBody::new_kinematic().with_rotation_locked(true))
        .with(Collider::box_collider(size.x, size.y).with_friction(0.0))
        .id()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_formation_is_four_rows_of_ten_by_the_roster() {
        let mut world = World::new();
        let invaders = spawn_invaders(&mut world, &Sheets::default());
        assert_eq!(invaders.len(), 40);
        for invader in &invaders {
            assert_eq!(invader.kind, ROSTER[invader.row]);
            assert_eq!(invader.hits_left, invader.kind.spec().hits);
        }
    }

    #[test]
    fn every_formation_cell_centre_is_a_whole_pixel_and_the_layout_is_symmetric() {
        for row in 0..ROSTER.len() {
            for column in 0..FORMATION_COLUMNS {
                let cell = invader_home(row, column) + Toasted::for_row(row).spec().sheet.sprite_offset();
                assert_eq!(cell, cell.round(), "row {row} column {column}");
            }
        }
        assert_eq!(column_cell_x(0), -234.0);
        assert_eq!(column_cell_x(FORMATION_COLUMNS - 1), 234.0);
    }

    #[test]
    fn a_bunker_spawns_ten_blocks_and_the_two_corner_slivers_never() {
        let mut world = World::new();
        let blocks = spawn_bunkers(&mut world, &Sheets::default().butter_bunker);
        assert_eq!(blocks.len(), BUNKER_COUNT * 10);
        for bunker in 0..BUNKER_COUNT {
            for corner in [0, BUNKER_BLOCK_COLUMNS - 1] {
                assert!(!blocks.iter().any(|b| b.bunker == bunker && b.row == 0 && b.column == corner));
            }
        }
    }

    #[test]
    fn a_bunker_block_draws_its_own_slice_of_its_state() {
        let sheet = SpriteSheet {
            texture: TextureHandle { id: 0 },
            grid: SheetGrid::new(4, 1),
            clips: Vec::new(),
            path: String::new(),
        };
        // The sheet is the four states side by side: 256 x 48.
        assert_eq!(block_tex_region(&sheet, BunkerState::Intact, 0, 0), [0.0, 0.0, 16.0 / 256.0, 16.0 / 48.0]);
        assert_eq!(
            block_tex_region(&sheet, BunkerState::Critical, 2, 3),
            [(128.0 + 48.0) / 256.0, 32.0 / 48.0, 16.0 / 256.0, 16.0 / 48.0],
        );
    }
}
