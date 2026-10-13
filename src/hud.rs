//! Where the HUD sits: the band's map, in **board space** — the 800 x 600 window's own
//! rectangle, its top-left at the origin — translated to the screen by
//! [`board_origin`], so a window that is not the board's size (the editor's) crops the
//! HUD the same way it crops the art, and the life icons, which are world sprites,
//! never part from the score beside them.
//!
//! Row one holds each score with its life icons, and the chaos banner centred between
//! them; row two each streak under its score, and the one centred transient. Every
//! slot is a box, so a test can hold the map: inside the band, clear of the flyby lane,
//! and no two labels that can show at once overlapping.

use engine_core::prelude::*;

use crate::constants::*;

/// A text slot in the band.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Slot {
    Score { player: usize },
    Streak { player: usize },
    Banner,
    Transient,
}

/// The board's top-left corner in screen space: the window centred on it.
pub(crate) fn board_origin(window_size: Vec2) -> Vec2 {
    (window_size - Vec2::new(WIN_W, WIN_H)) / 2.0
}

/// Player `player`'s side of the band: player one's on the left, player two's on the
/// right.
fn is_right_side(player: usize) -> bool {
    player > 0
}

fn score_column_x(player: usize) -> f32 {
    if is_right_side(player) {
        WIN_W - HUD_TEXT_MARGIN - HUD_SCORE_WIDTH
    } else {
        HUD_TEXT_MARGIN
    }
}

fn row_rect(x: f32, width: f32, row: (f32, f32)) -> Rect {
    Rect::new(x, row.0, width, row.1 - row.0)
}

/// A slot's box in board space.
pub(crate) fn slot_box(slot: Slot) -> Rect {
    match slot {
        Slot::Score { player } => row_rect(score_column_x(player), HUD_SCORE_WIDTH, HUD_ROW_ONE),
        Slot::Streak { player } => row_rect(score_column_x(player), HUD_SCORE_WIDTH, HUD_ROW_TWO),
        Slot::Banner => row_rect((WIN_W - HUD_BANNER_WIDTH) / 2.0, HUD_BANNER_WIDTH, HUD_ROW_ONE),
        Slot::Transient => row_rect((WIN_W - HUD_TRANSIENT_WIDTH) / 2.0, HUD_TRANSIENT_WIDTH, HUD_ROW_TWO),
    }
}

/// How a slot's text aligns in its box: a right-side score reads toward the window's
/// edge.
pub(crate) fn slot_alignment(slot: Slot) -> TextAlign {
    match slot {
        Slot::Score { player } | Slot::Streak { player } if is_right_side(player) => TextAlign::Right,
        Slot::Score { .. } | Slot::Streak { .. } => TextAlign::Left,
        Slot::Banner | Slot::Transient => TextAlign::Center,
    }
}

/// A slot's box in screen space.
pub(crate) fn slot_on_screen(window_size: Vec2, slot: Slot) -> Rect {
    slot_box(slot).translate(board_origin(window_size))
}

const LIFE_ICON_SIZE: f32 = 16.0;

/// Life icon `index`'s box in board space: beside its player's score, outward from the
/// banner — after player one's score, before player two's.
pub(crate) fn life_icon_box(player: usize, index: u32) -> Rect {
    let score = slot_box(Slot::Score { player });
    let step = LIFE_ICON_SIZE + LIFE_ICON_GAP;
    let x = if is_right_side(player) {
        score.x - LIFE_ICON_GAP - LIFE_ICON_SIZE - index as f32 * step
    } else {
        score.x + score.width + LIFE_ICON_GAP + index as f32 * step
    };
    row_rect(x, LIFE_ICON_SIZE, HUD_ROW_ONE)
}

/// Where life icon `index` of `player` stands in the world: its box's centre, on a
/// whole pixel.
pub(crate) fn life_icon_position(player: usize, index: u32) -> Vec2 {
    let centre = life_icon_box(player, index).center();
    Vec2::new(centre.x - WIN_W / 2.0, WIN_H / 2.0 - centre.y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::art::{SAUCER, half_size};
    use crate::types::GameMode;

    /// The band above the flyby lane: screen y 0–45.
    const HUD_BAND_HEIGHT: f32 = 45.0;

    /// Every slot `mode` can show, solo or co-op.
    fn slots(mode: GameMode) -> Vec<Slot> {
        let mut slots = Vec::new();
        for player in 0..mode.player_count() {
            slots.push(Slot::Score { player });
            slots.push(Slot::Streak { player });
        }
        slots.push(Slot::Banner);
        slots.push(Slot::Transient);
        slots
    }

    fn overlaps(first: Rect, second: Rect) -> bool {
        first.left() < second.right()
            && second.left() < first.right()
            && first.top() < second.bottom()
            && second.top() < first.bottom()
    }

    fn every_box(mode: GameMode) -> Vec<(String, Rect)> {
        let mut boxes: Vec<(String, Rect)> =
            slots(mode).into_iter().map(|slot| (format!("{slot:?}"), slot_box(slot))).collect();
        for player in 0..mode.player_count() {
            for index in 0..STARTING_LIVES {
                boxes.push((format!("life {player} {index}"), life_icon_box(player, index)));
            }
        }
        boxes
    }

    #[test]
    fn every_hud_box_sits_inside_the_band_and_none_overlaps_another_solo_or_coop() {
        for mode in [GameMode::SinglePlayer, GameMode::TwoPlayerCoop] {
            let boxes = every_box(mode);
            for (name, bounds) in &boxes {
                assert!(bounds.top() >= 0.0 && bounds.bottom() <= HUD_BAND_HEIGHT, "{mode:?} {name} {bounds:?}");
                assert!(bounds.left() >= 0.0 && bounds.right() <= WIN_W, "{mode:?} {name} {bounds:?}");
            }
            for (index, (name, bounds)) in boxes.iter().enumerate() {
                for (other_name, other) in &boxes[index + 1..] {
                    assert!(!overlaps(*bounds, *other), "{mode:?}: {name} overlaps {other_name}");
                }
            }
        }
    }

    #[test]
    fn the_band_ends_above_the_flyby_lane() {
        let saucer_top = SAUCER.position_for_cell(Vec2::new(0.0, FLYBY_CELL_Y)).y + half_size(&SAUCER).y;
        assert!(saucer_top <= WIN_H / 2.0 - HUD_BAND_HEIGHT, "the saucer reaches {saucer_top}");
    }

    #[test]
    fn a_life_icon_stands_on_a_whole_pixel() {
        for player in 0..2 {
            for index in 0..STARTING_LIVES {
                let position = life_icon_position(player, index);
                assert_eq!(position, position.round(), "player {player} life {index}");
            }
        }
    }
}
