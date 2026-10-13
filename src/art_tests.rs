//! The art contract: every number `art.rs` and `constants.rs` measured from the synced
//! sheets is re-measured here from the PNGs themselves, and every clip the game types is
//! held against its sidecar. A redrawn sheet that moves a box, a muzzle, the hole or a
//! bunker slice fails here, not on the screen.

use std::collections::BTreeSet;

use engine_core::prelude::*;
use image::RgbaImage;

use crate::art::*;
use crate::constants::*;
use crate::test_support::{asset_base, sidecar};
use crate::types::BunkerState;

/// The synced PNG behind `spec`, decoded.
fn pixels(spec: &SheetSpec) -> RgbaImage {
    let path = asset_base().join(spec.path);
    image::open(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display())).to_rgba8()
}

/// The frames `clips` play on `spec`'s sheet, from its sidecar.
fn frames(spec: &SheetSpec, clips: &[&str]) -> Vec<u32> {
    let sheet = sidecar(spec).sheet;
    clips.iter()
        .flat_map(|name| {
            let (_, clip) = sheet.clips.iter().find(|(clip_name, _)| clip_name == name)
                .unwrap_or_else(|| panic!("{} has no clip {name}", spec.path));
            clip.frame_indices.clone()
        })
        .collect()
}

/// The opaque box of `frame`, in cell pixels, bottom-right one past the last pixel.
fn frame_box(image: &RgbaImage, spec: &SheetSpec, frame: u32) -> Option<(Vec2, Vec2)> {
    let columns = image.width() / spec.cell.x as u32;
    let origin = (frame % columns * spec.cell.x as u32, frame / columns * spec.cell.y as u32);
    let mut found: Option<(Vec2, Vec2)> = None;
    for y in 0..spec.cell.y as u32 {
        for x in 0..spec.cell.x as u32 {
            if image.get_pixel(origin.0 + x, origin.1 + y)[3] == 0 {
                continue;
            }
            let (low, high) = (Vec2::new(x as f32, y as f32), Vec2::new(x as f32 + 1.0, y as f32 + 1.0));
            found = Some(found.map_or((low, high), |(min, max)| (min.min(low), max.max(high))));
        }
    }
    found
}

/// The union of the opaque boxes over `clips`.
fn measured(spec: &SheetSpec, clips: &[&str]) -> (Vec2, Vec2) {
    let image = pixels(spec);
    frames(spec, clips).into_iter()
        .filter_map(|frame| frame_box(&image, spec, frame))
        .reduce(|(min, max), (low, high)| (min.min(low), max.max(high)))
        .unwrap_or_else(|| panic!("{} draws nothing in {clips:?}", spec.path))
}

/// Every sheet the game draws, with the clips it types and which of them play once.
fn every_sheet() -> Vec<(&'static SheetSpec, Vec<&'static str>, Vec<&'static str>)> {
    vec![
        (&SOUR_DOOM, vec![MARCH, HURT], vec![HURT]),
        (&BAGELGEIST, vec![MARCH], vec![]),
        (&RYE_I_P, vec![MARCH], vec![]),
        (&PLAIN_WRAITH, vec![MARCH], vec![]),
        (&SAUCER, vec![FLY], vec![]),
        (&JAMMER, vec![MARCH], vec![]),
        (&DEION, vec![IDLE, WALK, FIRE, HURT], vec![FIRE, HURT]),
        (&CUBERT, vec![IDLE, WALK, FIRE, HURT], vec![FIRE, HURT]),
        (&ICICLE, vec![FLY], vec![]),
        (&ICE_CHIP, vec![FLY], vec![]),
        (&BURNT_CRUMB, vec![FLY], vec![]),
        (&JAM_GLOB, vec![FLY], vec![]),
        (&CROUTON_BURST, vec![BURST_DIE], vec![BURST_DIE]),
        (&ICE_BURST, vec![BURST_HIT], vec![BURST_HIT]),
        (&BUTTER_SPLASH, vec![SPLAT], vec![SPLAT]),
        (&JAM_SPLATTER, vec![SPLAT], vec![SPLAT]),
        (&BUTTER_BUNKER, BUNKER_STATE_CLIPS.to_vec(), vec![]),
        (&COUNTERTOP, vec![], vec![]),
        (&LIFE_ICON, vec![LIFE_ICON_DEION, LIFE_ICON_CUBERT], vec![]),
    ]
}

#[test]
fn every_clip_the_game_types_is_declared_and_every_one_shot_plays_once() {
    for (spec, clips, one_shots) in every_sheet() {
        let sheet = sidecar(spec).sheet;
        for name in clips {
            let (_, clip) = sheet.clips.iter().find(|(clip_name, _)| clip_name == name)
                .unwrap_or_else(|| panic!("{} has no clip {name}", spec.path));
            assert_eq!(!clip.looping, one_shots.contains(&name), "{} {name}'s looping", spec.path);
        }
    }
}

#[test]
fn every_sheet_is_measured_inside_its_own_cell_and_cut_by_it() {
    for (spec, _, _) in every_sheet() {
        assert!(spec.is_within_cell(), "{}", spec.path);
        let image = pixels(spec);
        let grid = sidecar(spec).sheet.grid;
        assert_eq!(
            (image.width(), image.height()),
            (grid.cols * spec.cell.x as u32, grid.rows * spec.cell.y as u32),
            "{}'s cell", spec.path,
        );
    }
}

#[test]
fn every_measured_box_is_the_union_of_the_frames_its_subject_plays() {
    for (spec, clips) in [
        (&SOUR_DOOM, &[MARCH][..]),
        (&BAGELGEIST, &[MARCH]),
        (&RYE_I_P, &[MARCH]),
        (&PLAIN_WRAITH, &[MARCH]),
        (&SAUCER, &[FLY]),
        (&JAMMER, &[MARCH]),
        (&ICICLE, &[FLY]),
        (&ICE_CHIP, &[FLY]),
        (&BURNT_CRUMB, &[FLY]),
        (&JAM_GLOB, &[FLY]),
        (&DEION, &[IDLE, WALK, FIRE]),
        (&CUBERT, &[IDLE, WALK, FIRE]),
    ] {
        assert_eq!(measured(spec, clips), spec.bounds, "{} over {clips:?}", spec.path);
    }
}

#[test]
fn a_cannons_fire_poses_never_widen_its_envelope() {
    for spec in [&DEION, &CUBERT] {
        assert_eq!(measured(spec, &[IDLE, WALK]), spec.bounds, "{}: fire stays inside idle and walk", spec.path);
    }
}

#[test]
fn each_muzzle_is_the_crests_top_row_in_every_fire_frame_and_the_first_idle_frame() {
    for hero in [Hero::Deion, Hero::Cubert] {
        let spec = hero.spec();
        let image = pixels(spec.sheet);
        // A shot leaves while `fire` plays; the idle bobs, so only its first frame agrees.
        let mut checked = frames(spec.sheet, &[FIRE]);
        checked.push(frames(spec.sheet, &[IDLE])[0]);
        for frame in checked {
            let (top_left, _) = frame_box(&image, spec.sheet, frame).expect("the frame draws");
            assert_eq!(top_left.y, spec.muzzle_in_cell.y, "{hero:?} frame {frame}'s top row");
            let columns = image.width() / spec.sheet.cell.x as u32;
            let origin_x = frame % columns * spec.sheet.cell.x as u32;
            let origin_y = frame / columns * spec.sheet.cell.y as u32;
            let crest = image.get_pixel(origin_x + spec.muzzle_in_cell.x as u32, origin_y + top_left.y as u32);
            assert_ne!(crest[3], 0, "{hero:?} frame {frame}: the muzzle is on the crest");
        }
    }
}

#[test]
fn bagelgeists_hole_spans_columns_27_to_35_in_every_march_frame() {
    let image = pixels(&BAGELGEIST);
    let (first, past_last) = BAGELGEIST_HOLE_COLUMNS;
    for frame in frames(&BAGELGEIST, &[MARCH]) {
        let origin_x = frame % (image.width() / 64) * 64;
        let origin_y = frame / (image.width() / 64) * 64;
        let opaque = |x: u32, y: u32| image.get_pixel(origin_x + x, origin_y + y)[3] != 0;
        let widest = (0..64).any(|y| {
            (first as u32..past_last as u32).all(|x| !opaque(x, y))
                && opaque(first as u32 - 1, y)
                && opaque(past_last as u32, y)
        });
        assert!(widest, "frame {frame} has no ring row with its hole exactly at x {first}–{}", past_last - 1.0);
    }
}

#[test]
fn the_bunker_slice_table_is_the_sheets_own_pixels() {
    let image = pixels(&BUTTER_BUNKER);
    for state in [BunkerState::Intact, BunkerState::Damaged, BunkerState::Critical, BunkerState::Melted] {
        let frame = frames(&BUTTER_BUNKER, &[state.clip()])[0];
        let origin_x = frame * BUNKER_CELL.x as u32;
        for (row, expected_row) in BUNKER_SLICE_OPAQUE[state.index()].iter().enumerate() {
            for (column, &expected) in expected_row.iter().enumerate() {
                let count = (0..16)
                    .flat_map(|y| (0..16).map(move |x| (x, y)))
                    .filter(|&(x, y)| {
                        image.get_pixel(origin_x + column as u32 * 16 + x, row as u32 * 16 + y)[3] != 0
                    })
                    .count() as u32;
                assert_eq!(count, expected, "{state:?} r{row} c{column}");
            }
        }
    }
}

#[test]
fn each_sheet_draws_at_one_depth_nested_from_the_counter_up_under_the_particles() {
    let ladder = [
        COUNTERTOP_DEPTH, BUNKER_DEPTH, JAM_GLOB_DEPTH, PLAIN_WRAITH_DEPTH, RYE_I_P_DEPTH,
        BAGELGEIST_DEPTH, SOUR_DOOM_DEPTH, SAUCER_DEPTH, JAMMER_DEPTH, DEION_DEPTH, CUBERT_DEPTH,
        ICICLE_DEPTH, ICE_CHIP_DEPTH, BURNT_CRUMB_DEPTH, CROUTON_BURST_DEPTH, ICE_BURST_DEPTH,
        BUTTER_SPLASH_DEPTH, JAM_SPLATTER_DEPTH, LIFE_ICON_DEPTH,
    ];
    assert!(ladder.windows(2).all(|pair| pair[0] < pair[1]), "the ladder rises, one depth per sheet");
    let distinct: BTreeSet<i32> = ladder.iter().map(|depth| (depth * 100.0) as i32).collect();
    assert_eq!(distinct.len(), ladder.len());
    assert!(ladder.iter().all(|&depth| depth < 0.5), "every sheet under the engine's particles");
    for kind in ROSTER {
        assert!(JAM_GLOB_DEPTH < kind.spec().depth, "a glob falls behind the formation");
    }
}
