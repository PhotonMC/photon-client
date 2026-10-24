//! The credits and attribution screen (`CreditsAndAttributionScreen`): the title screen's
//! copyright line and the options' last button open it. A title, three wide buttons and Done.

use crate::FontRuns;
use photon_game::text::{Component, Lang, Style};

use crate::layout::{Item, place_in_frame};
use crate::widgets::button_set::ButtonSet;
use crate::widgets::navigation::Direction;
use crate::widgets::{Kind, Widget};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

/// `HeaderAndFooterLayout(this)`: the default header and footer heights, and the contents' margin.
const HEADER_AND_FOOTER: i32 = 33;
const CONTENT_MARGIN_TOP: i32 = 30;
const BUTTON_WIDTH: i32 = 210;
const BUTTON_SPACING: i32 = 8;
const DONE_WIDTH: i32 = 200;

/// What a press asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreditsAction {
    /// The credits roll, without the poem.
    Credits,
    /// `CommonLinks.ATTRIBUTION`, to be confirmed.
    Attribution,
    /// `CommonLinks.LICENSES`, to be confirmed.
    Licenses,
    /// Done, or Escape: back to the screen before.
    Done,
}

pub struct CreditsAttributionScreen {
    buttons: ButtonSet<CreditsAction>,
    title: Widget,
    in_world: bool,
}

fn runs(lang: &Lang, key: &str) -> Vec<photon_game::text::Run> {
    lang.runs(&Component::translatable(key), &Style::default())
}

impl CreditsAttributionScreen {
    /// `init` in a screen of `size` units; `in_world` for one opened over a level.
    pub fn new(font: &Font, lang: &Lang, size: (i32, i32), in_world: bool) -> Self {
        let (width, height) = size;
        let mut buttons = ButtonSet::new(size);
        let text = runs(lang, "credits_and_attribution.screen.title");
        let title = Widget {
            rect: buttons.arena.add(font.width_of_runs(&text), 9),
            active: false,
            kind: Kind::Text { text },
        };
        place_in_frame(
            &mut buttons.arena,
            Item::Leaf(title.rect),
            (0, 0, width, HEADER_AND_FOOTER),
            (0.5, 0.5),
        );
        let contents_height = 3 * 20 + 2 * BUTTON_SPACING;
        let top = (HEADER_AND_FOOTER + CONTENT_MARGIN_TOP)
            .min(height - HEADER_AND_FOOTER - contents_height);
        let wide = [
            (
                "credits_and_attribution.button.credits",
                CreditsAction::Credits,
            ),
            (
                "credits_and_attribution.button.attribution",
                CreditsAction::Attribution,
            ),
            (
                "credits_and_attribution.button.licenses",
                CreditsAction::Licenses,
            ),
        ];
        for (row, (key, action)) in (0..).zip(wide) {
            let label = Kind::Button {
                label: runs(lang, key),
            };
            let x = (width - BUTTON_WIDTH) / 2;
            let y = top + row * (20 + BUTTON_SPACING);
            buttons.add((x, y, BUTTON_WIDTH, 20), (label, true), action, None);
        }
        let done = Kind::Button {
            label: runs(lang, "gui.done"),
        };
        let y = height - HEADER_AND_FOOTER + (HEADER_AND_FOOTER - 20) / 2;
        buttons.add(
            ((width - DONE_WIDTH) / 2, y, DONE_WIDTH, 20),
            (done, true),
            CreditsAction::Done,
            None,
        );
        Self {
            buttons,
            title,
            in_world,
        }
    }

    pub fn size(&self) -> (i32, i32) {
        self.buttons.size()
    }

    /// The screen again for a screen of another size: `init` runs anew.
    pub fn resize(&mut self, font: &Font, lang: &Lang, size: (i32, i32)) {
        *self = Self::new(font, lang, size, self.in_world);
    }

    pub fn draw(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        let (width, height) = self.size();
        let background = if self.in_world {
            "inworld_menu_background"
        } else {
            "menu_background"
        };
        frame.tile_texture(atlas, background, (32, 32), (0, 0, width, height));
        self.title
            .draw(frame, atlas, font, &self.buttons.arena, false);
        self.buttons.draw(frame, atlas, font, mouse);
    }

    /// A press of the primary button at `mouse`.
    pub fn clicked(&self, mouse: (i32, i32)) -> Option<CreditsAction> {
        self.buttons.clicked(mouse)
    }

    /// Tab (and Shift+Tab).
    pub fn tab(&mut self, forward: bool) {
        self.buttons.tab(forward);
    }

    /// An arrow key.
    pub fn arrow(&mut self, direction: Direction) {
        self.buttons.arrow(direction);
    }

    /// Enter or Space on the focused button.
    pub fn pressed(&self) -> Option<CreditsAction> {
        self.buttons.pressed()
    }
}
