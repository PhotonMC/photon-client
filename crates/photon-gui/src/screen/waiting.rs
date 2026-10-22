//! The screen shown while the server answers a dialog (`WaitingForResponseScreen`): a title, and
//! a button that appears after a second and wakes after five.

use crate::FontRuns;
use photon_game::text::{Component, Lang, Style};

use crate::layout::{Arena, Item, place_in_frame};
use crate::widgets::{Kind, Widget};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

/// The header's height; the footer has none.
const HEADER: i32 = 33;
const CONTENT_MARGIN: i32 = 30;
const BUTTON_WIDTH: i32 = 200;
/// `BUTTON_VISIBLE_AFTER` and `BUTTON_ACTIVE_AFTER`, in seconds.
const VISIBLE_AFTER: usize = 1;
const ACTIVE_AFTER: usize = 5;

pub struct WaitingScreen {
    arena: Arena,
    size: (i32, i32),
    in_world: bool,
    title: Widget,
    button: Widget,
    visible: bool,
    ticks: i32,
}

impl WaitingScreen {
    /// `WaitingForResponseScreen.init`, in a screen of `size` units.
    pub fn new(font: &Font, lang: &Lang, size: (i32, i32), in_world: bool) -> Self {
        let (width, height) = size;
        let mut arena = Arena::default();
        let text = lang.runs(
            &Component::translatable("gui.waitingForResponse.title"),
            &Style::default(),
        );
        let title = Widget {
            rect: arena.add(font.width_of_runs(&text), 9),
            active: false,
            kind: Kind::Text { text },
        };
        let button = Widget {
            rect: arena.add(BUTTON_WIDTH, 20),
            active: false,
            kind: Kind::Button { label: Vec::new() },
        };
        place_in_frame(
            &mut arena,
            Item::Leaf(title.rect),
            (0, 0, width, HEADER),
            (0.5, 0.5),
        );
        // The contents stand `CONTENT_MARGIN` under the header, or as high as the screen lets.
        let y = (HEADER + CONTENT_MARGIN).min(height - 20);
        place_in_frame(
            &mut arena,
            Item::Leaf(button.rect),
            (0, y, width, 20),
            (0.5, 0.5),
        );
        Self {
            arena,
            size,
            in_world,
            title,
            button,
            visible: false,
            ticks: 0,
        }
    }

    /// The screen again for a screen of another size: `init` runs anew, and the count goes on.
    #[must_use]
    pub fn resized(&self, font: &Font, lang: &Lang, size: (i32, i32)) -> Self {
        let mut screen = Self::new(font, lang, size, self.in_world);
        screen.ticks = self.ticks;
        screen
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    fn label(lang: &Lang, seconds: usize) -> Vec<photon_game::text::Run> {
        let component = match seconds {
            0 => Component::empty(),
            1..=4 => Component::translatable_with(
                "gui.waitingForResponse.button.inactive",
                vec![Component::literal((5 - seconds).to_string())],
            ),
            _ => Component::translatable("gui.back"),
        };
        lang.runs(&component, &Style::default())
    }

    /// `tick`: while the button sleeps, its label counts the seconds down.
    pub fn tick(&mut self, lang: &Lang) {
        if self.button.active {
            return;
        }
        let seconds = (self.ticks / 20) as usize;
        self.ticks += 1;
        self.visible = seconds >= VISIBLE_AFTER;
        self.button.kind = Kind::Button {
            label: Self::label(lang, seconds),
        };
        if seconds == ACTIVE_AFTER {
            self.button.active = true;
        }
    }

    pub fn draw(
        &self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        mouse: (i32, i32),
        blurriness: i32,
    ) {
        let (width, height) = self.size;
        let background = if self.in_world {
            frame.blur_before_this_stratum(blurriness);
            "inworld_menu_background"
        } else {
            "menu_background"
        };
        frame.tile_texture(atlas, background, (32, 32), (0, 0, width, height));
        self.title.draw(frame, atlas, font, &self.arena, false);
        if self.visible {
            let hovered = self.button.is_active() && self.button.contains(&self.arena, mouse);
            self.button.draw(frame, atlas, font, &self.arena, hovered);
        }
    }

    /// A press at `mouse`: whether it closes the screen (the button, once it is awake).
    pub fn clicked(&self, mouse: (i32, i32)) -> bool {
        let hit =
            self.visible && self.button.is_active() && self.button.contains(&self.arena, mouse);
        if hit {
            crate::ui_sound::play_down_sound();
        }
        hit
    }

    /// Escape closes the screen once the button is awake.
    pub fn escape(&self) -> bool {
        self.button.active
    }
}
