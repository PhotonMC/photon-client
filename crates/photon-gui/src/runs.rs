//! The game's text runs and items as the draw layer takes them: the screens keep their `Run`s and
//! `Stack`s, and these turn them into what `GuiFrame` and `Font` draw and measure.

use photon_render::gui::TextRun;

/// A component's runs as the draw layer takes them.
pub fn draw_runs(runs: &[photon_game::text::Run]) -> Vec<TextRun<'_>> {
    runs.iter()
        .enumerate()
        .map(|(index, run)| TextRun {
            text: run.text.as_str().into(),
            colour: run.style.colour,
            bold: run.style.bold == Some(true),
            italic: run.style.italic == Some(true),
            shadow: run.style.shadow,
            joins_previous: index > 0 && runs[index - 1].style == run.style,
        })
        .collect()
}

/// Measuring text components' runs; the screens' view of [`photon_render::gui::font::Font`].
pub trait FontRuns {
    /// `Font.width` of styled text: bold glyphs advance a unit more.
    fn width_of_runs(&self, runs: &[photon_game::text::Run]) -> i32;
}

impl FontRuns for photon_render::gui::font::Font {
    fn width_of_runs(&self, runs: &[photon_game::text::Run]) -> i32 {
        self.width_of_text_runs(&draw_runs(runs))
    }
}

/// Drawing text components' runs; the screens' view of [`photon_render::gui::GuiFrame`].
pub trait FrameRuns {
    #[expect(
        clippy::too_many_arguments,
        reason = "mirrors the parameters of the reference's `GuiGraphics` call"
    )]
    fn styled_text(
        &mut self,
        atlas: &photon_render::gui::GuiAtlas,
        font: &photon_render::gui::font::Font,
        runs: &[photon_game::text::Run],
        x: i32,
        y: i32,
        default_colour: u32,
        shadow: bool,
    );
    fn tooltip(
        &mut self,
        atlas: &photon_render::gui::GuiAtlas,
        font: &photon_render::gui::font::Font,
        lines: &[Vec<photon_game::text::Run>],
        mouse: (i32, i32),
        screen: (i32, i32),
    );
    fn tooltip_with(
        &mut self,
        atlas: &photon_render::gui::GuiAtlas,
        font: &photon_render::gui::font::Font,
        lines: &[Vec<photon_game::text::Run>],
        positioner: &photon_render::gui::Positioner,
        mouse: (i32, i32),
        screen: (i32, i32),
    );
}

impl FrameRuns for photon_render::gui::GuiFrame {
    fn styled_text(
        &mut self,
        atlas: &photon_render::gui::GuiAtlas,
        font: &photon_render::gui::font::Font,
        runs: &[photon_game::text::Run],
        x: i32,
        y: i32,
        default_colour: u32,
        shadow: bool,
    ) {
        self.styled_text_runs(
            atlas,
            font,
            &draw_runs(runs),
            (x, y),
            default_colour,
            shadow,
        );
    }

    fn tooltip(
        &mut self,
        atlas: &photon_render::gui::GuiAtlas,
        font: &photon_render::gui::font::Font,
        lines: &[Vec<photon_game::text::Run>],
        mouse: (i32, i32),
        screen: (i32, i32),
    ) {
        self.tooltip_runs(atlas, font, &draw_lines(lines), mouse, screen);
    }

    fn tooltip_with(
        &mut self,
        atlas: &photon_render::gui::GuiAtlas,
        font: &photon_render::gui::font::Font,
        lines: &[Vec<photon_game::text::Run>],
        positioner: &photon_render::gui::Positioner,
        mouse: (i32, i32),
        screen: (i32, i32),
    ) {
        self.tooltip_with_runs(atlas, font, &draw_lines(lines), positioner, mouse, screen);
    }
}

/// The lines of a tooltip as the draw layer takes them.
pub fn draw_lines(lines: &[Vec<photon_game::text::Run>]) -> Vec<Vec<TextRun<'_>>> {
    lines.iter().map(|line| draw_runs(line)).collect()
}

/// Items with the decorations the game's rules give them (a durability bar, a cooldown, a count);
/// the screens' view of [`photon_render::gui::GuiFrame`].
pub trait ItemSlots {
    /// `itemBar`: the durability (or a bundle's fullness) under the item, on a black strip.
    fn item_bar(
        &mut self,
        atlas: &photon_render::gui::GuiAtlas,
        stack: &photon_world::ClientItemStackState,
        x: i32,
        y: i32,
    );

    /// `itemCooldown`: a white veil over the item, as tall as what is left of its group's cooldown.
    fn item_cooldown(
        &mut self,
        atlas: &photon_render::gui::GuiAtlas,
        stack: &photon_world::ClientItemStackState,
        x: i32,
        y: i32,
    );

    /// `GuiGraphics.item` and `itemDecorations`: an item in the slot at `(x, y)`, then its count
    /// (or the given text) in the given colour when it is not one.
    #[expect(
        clippy::too_many_arguments,
        reason = "mirrors the parameters of the reference's `GuiGraphics` call"
    )]
    fn item_with_count(
        &mut self,
        atlas: &photon_render::gui::GuiAtlas,
        font: &photon_render::gui::font::Font,
        draw: &mut photon_render::gui::ItemDrawer<'_>,
        stack: &photon_world::ClientItemStackState,
        count: i32,
        text: Option<(&str, u32)>,
        x: i32,
        y: i32,
    );
}

impl ItemSlots for photon_render::gui::GuiFrame {
    fn item_bar(
        &mut self,
        atlas: &photon_render::gui::GuiAtlas,
        stack: &photon_world::ClientItemStackState,
        x: i32,
        y: i32,
    ) {
        if let Some(bar) = photon_game::menus::container::item_bar(stack) {
            self.item_bar_strip(atlas, (x, y), bar);
        }
    }

    fn item_cooldown(
        &mut self,
        atlas: &photon_render::gui::GuiAtlas,
        stack: &photon_world::ClientItemStackState,
        x: i32,
        y: i32,
    ) {
        self.item_cooldown_group(atlas, (x, y), &photon_game::item::cooldowns::group(stack));
    }

    fn item_with_count(
        &mut self,
        atlas: &photon_render::gui::GuiAtlas,
        font: &photon_render::gui::font::Font,
        draw: &mut photon_render::gui::ItemDrawer<'_>,
        stack: &photon_world::ClientItemStackState,
        count: i32,
        text: Option<(&str, u32)>,
        x: i32,
        y: i32,
    ) {
        self.item(draw, stack, x, y);
        self.item_bar(atlas, stack, x, y);
        self.item_cooldown(atlas, stack, x, y);
        let count_text = count.to_string();
        let shown = match text {
            Some(text) => Some(text),
            None if count != 1 => Some((count_text.as_str(), 0xFFFF_FFFF)),
            None => None,
        };
        if let Some((text, colour)) = shown {
            self.text_with_shadow(
                atlas,
                font,
                text,
                x + 19 - 2 - font.width(text),
                y + 6 + 3,
                colour,
            );
        }
    }
}
