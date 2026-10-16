//! The text the draw layer takes: runs of text with the look they are drawn in. The screens turn a
//! text component's runs into these; the renderer reads nothing of a component but this.

/// One stretch of text and its look (`Style`, as far as drawing reads it).
#[derive(Debug, Clone, PartialEq)]
pub struct TextRun {
    pub text: String,
    /// `0xRRGGBB`, when the style names a colour.
    pub colour: Option<u32>,
    pub bold: bool,
    pub italic: bool,
    /// `0xAARRGGBB` of the shadow, when the style names one (zero alpha: none).
    pub shadow: Option<u32>,
    /// Whether the style equals the previous run's whole style, so the two are one stretch.
    pub joins_previous: bool,
}

impl TextRun {
    /// Text in no style of its own.
    pub fn plain(text: &str) -> Self {
        Self {
            text: text.to_owned(),
            colour: None,
            bold: false,
            italic: false,
            shadow: None,
            joins_previous: false,
        }
    }
}

/// A component's runs as the draw layer takes them.
pub fn draw_runs(runs: &[photon_game::text::Run]) -> Vec<TextRun> {
    runs.iter()
        .enumerate()
        .map(|(index, run)| TextRun {
            text: run.text.clone(),
            colour: run.style.colour,
            bold: run.style.bold == Some(true),
            italic: run.style.italic == Some(true),
            shadow: run.style.shadow,
            joins_previous: index > 0 && runs[index - 1].style == run.style,
        })
        .collect()
}

/// Measuring text components' runs; the screens' view of [`crate::gui::font::Font`].
pub trait FontRuns {
    /// `Font.width` of styled text: bold glyphs advance a unit more.
    fn width_of_runs(&self, runs: &[photon_game::text::Run]) -> i32;
}

impl FontRuns for crate::gui::font::Font {
    fn width_of_runs(&self, runs: &[photon_game::text::Run]) -> i32 {
        self.width_of_text_runs(&draw_runs(runs))
    }
}

/// Drawing text components' runs; the screens' view of [`crate::gui::GuiFrame`].
pub trait FrameRuns {
    #[expect(
        clippy::too_many_arguments,
        reason = "mirrors the parameters of the reference's `GuiGraphics` call"
    )]
    fn styled_text(
        &mut self,
        atlas: &crate::gui::GuiAtlas,
        font: &crate::gui::font::Font,
        runs: &[photon_game::text::Run],
        x: i32,
        y: i32,
        default_colour: u32,
        shadow: bool,
    );
    fn tooltip(
        &mut self,
        atlas: &crate::gui::GuiAtlas,
        font: &crate::gui::font::Font,
        lines: &[Vec<photon_game::text::Run>],
        mouse: (i32, i32),
        screen: (i32, i32),
    );
    fn tooltip_with(
        &mut self,
        atlas: &crate::gui::GuiAtlas,
        font: &crate::gui::font::Font,
        lines: &[Vec<photon_game::text::Run>],
        positioner: &crate::gui::Positioner,
        mouse: (i32, i32),
        screen: (i32, i32),
    );
}

impl FrameRuns for crate::gui::GuiFrame {
    fn styled_text(
        &mut self,
        atlas: &crate::gui::GuiAtlas,
        font: &crate::gui::font::Font,
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
        atlas: &crate::gui::GuiAtlas,
        font: &crate::gui::font::Font,
        lines: &[Vec<photon_game::text::Run>],
        mouse: (i32, i32),
        screen: (i32, i32),
    ) {
        self.tooltip_runs(atlas, font, &draw_lines(lines), mouse, screen);
    }

    fn tooltip_with(
        &mut self,
        atlas: &crate::gui::GuiAtlas,
        font: &crate::gui::font::Font,
        lines: &[Vec<photon_game::text::Run>],
        positioner: &crate::gui::Positioner,
        mouse: (i32, i32),
        screen: (i32, i32),
    ) {
        self.tooltip_with_runs(atlas, font, &draw_lines(lines), positioner, mouse, screen);
    }
}

/// The lines of a tooltip as the draw layer takes them.
pub fn draw_lines(lines: &[Vec<photon_game::text::Run>]) -> Vec<Vec<TextRun>> {
    lines.iter().map(|line| draw_runs(line)).collect()
}

/// Items with the decorations the game's rules give them (a durability bar, a cooldown, a count);
/// the screens' view of [`crate::gui::GuiFrame`].
pub trait ItemSlots {
    /// `itemBar`: the durability (or a bundle's fullness) under the item, on a black strip.
    fn item_bar(
        &mut self,
        atlas: &crate::gui::GuiAtlas,
        stack: &photon_world::ClientItemStackState,
        x: i32,
        y: i32,
    );

    /// `itemCooldown`: a white veil over the item, as tall as what is left of its group's cooldown.
    fn item_cooldown(
        &mut self,
        atlas: &crate::gui::GuiAtlas,
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
        atlas: &crate::gui::GuiAtlas,
        font: &crate::gui::font::Font,
        draw: &mut crate::gui::ItemDrawer<'_>,
        stack: &photon_world::ClientItemStackState,
        count: i32,
        text: Option<(&str, u32)>,
        x: i32,
        y: i32,
    );
}

impl ItemSlots for crate::gui::GuiFrame {
    fn item_bar(
        &mut self,
        atlas: &crate::gui::GuiAtlas,
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
        atlas: &crate::gui::GuiAtlas,
        stack: &photon_world::ClientItemStackState,
        x: i32,
        y: i32,
    ) {
        self.item_cooldown_group(atlas, (x, y), &photon_game::item::cooldowns::group(stack));
    }

    fn item_with_count(
        &mut self,
        atlas: &crate::gui::GuiAtlas,
        font: &crate::gui::font::Font,
        draw: &mut crate::gui::ItemDrawer<'_>,
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
