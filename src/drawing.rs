use engine_core::prelude::*;
use crate::achievements::DISPLAY_SECTIONS;
use crate::art::SAUCER_BURST_COLOR;
use crate::constants::{HUD_FONT_SIZE, JAMMER_POINTS};
use crate::hud::{self, Slot};
use crate::menu::{achievements_panel, mode_hint, mode_select_panel, title_label, title_panel, TITLE_ITEMS};
use crate::types::*;

impl SpaceInvadersGame {
    fn menu_style(&self) -> MenuStyle {
        MenuStyle::from_theme(&ChaosTheme::for_mode(self.chaos_mode))
    }

    pub(crate) fn draw_ui(&self, ctx: &mut GameContext) {
        match &self.state {
            GameState::TitleScreen { selection } => self.draw_title(ctx, *selection),
            GameState::ModeSelect { selection } => self.draw_mode_select(ctx, *selection),
            GameState::Achievements => self.draw_achievements(ctx),
            _ => self.draw_gameplay(ctx),
        }
    }

    fn draw_title(&self, ctx: &mut GameContext, selection: u8) {
        let style = self.menu_style();
        let panel = title_panel("TOAST BUSTERS", ctx.window_size);
        let mut y = panel.begin(ctx.ui, &style);
        for (i, &item) in TITLE_ITEMS.iter().enumerate() {
            y = panel.item(ctx.ui, y, title_label(item), i as u8 == selection, &style);
        }
        panel.hint(ctx.ui, "W/S or D-Pad navigate - SPACE/ENTER, (A), or click confirm", &style);
    }

    fn draw_mode_select(&self, ctx: &mut GameContext, selection: u8) {
        let style = self.menu_style();
        let panel = mode_select_panel("SELECT CHAOS MODE", ctx.window_size);
        let mut y = panel.begin(ctx.ui, &style);
        for (i, &mode) in ChaosMode::ALL.iter().enumerate() {
            // Each entry glows in its chaos mode's banner color.
            let c = ChaosTheme::for_mode(mode).banner_color;
            y = panel.item_colored(ctx.ui, y, mode.label(), c, i as u8 == selection, &style);
        }
        panel.hint(
            ctx.ui,
            mode_hint(ChaosMode::ALL[selection as usize % ChaosMode::ALL.len()]),
            &style,
        );
    }

    fn draw_achievements(&self, ctx: &mut GameContext) {
        let style = self.menu_style();
        let cx = ctx.window_size.x / 2.0;
        let total = ctx.achievements.total();
        let unlocked = ctx.achievements.unlocked_count();

        // Tall window; the section list draws left-aligned inside it.
        let panel = achievements_panel("ACHIEVEMENTS", ctx.window_size);
        let first_y = panel.begin(ctx.ui, &style);
        let rect = panel.panel_rect();
        ctx.ui.label_centered(
            &format!("{unlocked} / {total} unlocked"),
            Vec2::new(cx, first_y - 8.0),
        );

        let left = rect.x + 28.0;
        let mut y = first_y + 18.0;

        let locked_color = Color::new(0.45, 0.45, 0.5, 1.0);
        let unlocked_color = Color::new(1.0, 0.85, 0.25, 1.0);
        let desc_color = Color::new(0.75, 0.75, 0.8, 1.0);
        let header_color = Color::new(0.6, 0.75, 1.0, 1.0);

        for (section, ids) in DISPLAY_SECTIONS {
            ctx.ui.label_styled(section, Vec2::new(left, y), header_color, 16.0);
            y += 22.0;
            for id in *ids {
                let is_unlocked = ctx.achievements.is_unlocked(id);
                // Registry always has entries for these ids (registered in `register_achievements`).
                let Some(ach) = ctx.achievements.get(id) else { continue };

                let (marker, name_color) = if is_unlocked {
                    ("[X]", unlocked_color)
                } else {
                    ("[ ]", locked_color)
                };

                ctx.ui.label_styled(
                    &format!("{marker} {}", ach.name),
                    Vec2::new(left + 8.0, y),
                    name_color,
                    14.0,
                );
                ctx.ui.label_styled(&ach.description, Vec2::new(left + 52.0, y + 16.0), desc_color, 12.0);
                y += 36.0;
            }
            y += 6.0;
        }

        panel.hint(ctx.ui, "ESC, SPACE, or click to go back", &style);
    }

    fn draw_gameplay(&self, ctx: &mut GameContext) {
        let centre = ctx.window_size / 2.0;
        self.draw_hud(ctx);

        if let GameState::GameOver { won } = &self.state {
            let message = if *won { "ALL RIGHT, THIS SLICE IS TOAST!" } else { "TOASTED" };
            let style = self.menu_style();
            let panel = MenuPanel::new(message, centre, 400.0, 2);
            let mut y = panel.begin(ctx.ui, &style);
            y = panel.line(ctx.ui, y, &self.final_score_line(), &style);
            panel.line(ctx.ui, y, "SPACE or ENTER to play again", &style);
            panel.hint(ctx.ui, "ESC for title screen", &style);
        }

        if self.pause.is_active() {
            let style = self.menu_style();
            self.pause.draw(ctx.ui, ctx.window_size, &style);
        }
    }

    /// The HUD band, from `hud`'s map: each score and its streak, the chaos banner, and
    /// the one transient. The life icons beside each score are world sprites.
    fn draw_hud(&self, ctx: &mut GameContext) {
        for (player, state) in self.players.iter().enumerate() {
            let score = match self.mode {
                GameMode::SinglePlayer => format!("SCORE {}", state.score),
                GameMode::TwoPlayerCoop => format!("P{} {}", player + 1, state.score),
            };
            draw_in_slot(ctx, Slot::Score { player }, &score, HUD_TEXT_COLOR);
            if state.shot_streak >= 3 {
                draw_in_slot(ctx, Slot::Streak { player }, &format!("STREAK x{}", state.shot_streak), HUD_TEXT_COLOR);
            }
        }

        let theme = ChaosTheme::for_mode(self.chaos_mode);
        if let Some(banner) = theme.banner_text {
            draw_in_slot(ctx, Slot::Banner, banner, Color::from(theme.banner_color));
        }
        if let Some((notice, _)) = self.notice {
            draw_in_slot(ctx, Slot::Transient, &notice_text(notice, self.mode), notice_color(notice));
        }
    }

    /// Game-over final-score line: one score in single player, both in co-op.
    fn final_score_line(&self) -> String {
        match self.mode {
            GameMode::SinglePlayer => {
                format!("Final score: {}", self.players[0].score)
            }
            GameMode::TwoPlayerCoop => {
                let p1 = self.players.first().map(|p| p.score).unwrap_or(0);
                let p2 = self.players.get(1).map(|p| p.score).unwrap_or(0);
                format!("P1 {p1}   P2 {p2}")
            }
        }
    }
}

const HUD_TEXT_COLOR: Color = Color::new(0.96, 0.92, 0.84, 1.0);

/// Draw `text` in `slot`'s box, aligned as the slot reads.
fn draw_in_slot(ctx: &mut GameContext, slot: Slot, text: &str, color: Color) {
    let bounds = hud::slot_on_screen(ctx.window_size, slot);
    ctx.ui.label_in_bounds_styled(text, bounds, hud::slot_alignment(slot), color, HUD_FONT_SIZE, 0.0);
}

/// What the transient slot says. A jam reads as the jammed player's own complaint solo,
/// and names the player in co-op.
pub(crate) fn notice_text(notice: Notice, mode: GameMode) -> String {
    match notice {
        Notice::SaucerBonus(bonus) => format!("SAUCER +{bonus}"),
        Notice::JammerBusted => format!("JAMMER +{JAMMER_POINTS}"),
        Notice::Jammed { player } => match mode {
            GameMode::SinglePlayer => "HE JAMMED ME!".to_string(),
            GameMode::TwoPlayerCoop => format!("P{} JAMMED!", player + 1),
        },
    }
}

fn notice_color(notice: Notice) -> Color {
    match notice {
        Notice::SaucerBonus(_) => Color::from(SAUCER_BURST_COLOR),
        Notice::JammerBusted | Notice::Jammed { .. } => Color::from(JAM_TEXT_COLOR),
    }
}

/// Grape jam, lightened to read on the dark band.
const JAM_TEXT_COLOR: Vec4 = Vec4::new(0.78, 0.6, 0.95, 1.0);
