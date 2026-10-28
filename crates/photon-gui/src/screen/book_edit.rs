//! The book and quill's screen (`BookEditScreen`): the page's text box over the book, the page
//! buttons, the page's number, and Sign and Done under the book.

use crate::{FontRuns, FrameRuns};
use photon_game::text::{Component, Lang, Run, Style};

use super::book::page_style;
use crate::layout::Arena;
use crate::widgets::edit_box::Clipboard;
use crate::widgets::text_area::MultiLineEditBox;
use crate::widgets::{Kind, Widget};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

/// `IMAGE_WIDTH`: the book is 192 square.
const IMAGE: i32 = 192;
/// `TEXT_WIDTH` and `TEXT_HEIGHT`.
const TEXT_WIDTH: i32 = 114;
const TEXT_HEIGHT: i32 = 126;
/// A book holds at most a hundred pages of 1024 characters.
const MAX_PAGES: usize = 100;
const PAGE_CHARACTERS: usize = 1024;
/// A page button is 23 by 13.
const PAGE_BUTTON: (i32, i32) = (23, 13);
/// The book's black, for the text and the cursor.
const BLACK: u32 = 0xFF00_0000;

/// What the screen asks of the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookEditAction {
    /// Done: the screen closes and the pages are saved (`saveChanges`).
    Done,
    /// Escape: the screen closes and nothing is saved.
    Close,
    /// Sign: the signing screen opens.
    Sign,
}

/// The screen's widgets in their tab order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Page,
    Back,
    Forward,
    Sign,
    Done,
}

const ORDER: [Focus; 5] = [
    Focus::Page,
    Focus::Back,
    Focus::Forward,
    Focus::Sign,
    Focus::Done,
];

/// `BookEditScreen`.
pub struct BookEditScreen {
    size: (i32, i32),
    pages: Vec<String>,
    current: usize,
    page: MultiLineEditBox,
    arena: Arena,
    sign: Widget,
    done: Widget,
    focus: Option<Focus>,
    indicator: Vec<Run>,
}

impl BookEditScreen {
    /// The screen over a book whose pages read `pages` (one empty page when it has none), the
    /// page focused at `now` on the frame's clock (or from the first frame).
    pub fn new(
        (font, lang): (&Font, &Lang),
        pages: Vec<String>,
        size: (i32, i32),
        now: Option<u64>,
    ) -> Self {
        let runs = |key: &str| lang.runs(&Component::translatable(key), &Style::default());
        let mut arena = Arena::default();
        let button = |arena: &mut Arena, key: &str| Widget {
            rect: arena.add(98, 20),
            active: true,
            kind: Kind::Button { label: runs(key) },
        };
        let sign = button(&mut arena, "book.signButton");
        let done = button(&mut arena, "gui.done");
        let mut page = MultiLineEditBox::new(font, (0, 28, 122, 134), (BLACK, BLACK), false);
        page.field.set_character_limit(PAGE_CHARACTERS);
        page.field.set_line_limit((TEXT_HEIGHT / 9) as usize);
        let mut screen = Self {
            size,
            pages: if pages.is_empty() {
                vec![String::new()]
            } else {
                pages
            },
            current: 0,
            page,
            arena,
            sign,
            done,
            focus: Some(Focus::Page),
            indicator: Vec::new(),
        };
        screen.page.set_focused(true, now);
        screen.resize(size);
        screen.update_page(font, lang);
        screen
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    /// The pages as they stand.
    pub fn pages(&self) -> &[String] {
        &self.pages
    }

    /// `eraseEmptyTrailingPages`: the pages `saveChanges` sends.
    pub fn saved_pages(&self) -> Vec<String> {
        let mut pages = self.pages.clone();
        while pages.last().is_some_and(String::is_empty) {
            pages.pop();
        }
        pages
    }

    pub fn current(&self) -> usize {
        self.current
    }

    fn left(&self) -> i32 {
        (self.size.0 - IMAGE) / 2
    }

    /// The window was resized: everything moves with it.
    pub fn resize(&mut self, (width, height): (i32, i32)) {
        self.size = (width, height);
        self.page.rect.0 = (width - TEXT_WIDTH) / 2 - 8;
        for (widget, x) in [
            (&self.sign, width / 2 - 98 - 2),
            (&self.done, width / 2 + 2),
        ] {
            let rect = &mut self.arena.0[widget.rect];
            rect.x = x;
            rect.y = 2 + IMAGE + 2;
        }
    }

    fn page_button(&self, forward: bool) -> (i32, i32) {
        (self.left() + if forward { 116 } else { 43 }, 2 + 157)
    }

    /// `updateButtonVisibility`: the back button shows past the first page.
    fn back_visible(&self) -> bool {
        self.current > 0
    }

    /// `updatePageContent`: the box takes the page, past its line limit if need be.
    fn update_page(&mut self, font: &Font, lang: &Lang) {
        self.page.set_value(font, &self.pages[self.current], true);
        let indicator = Component::translatable_with(
            "book.pageIndicator",
            vec![
                Component::literal((self.current + 1).to_string()),
                Component::literal(self.pages.len().to_string()),
            ],
        );
        self.indicator = lang.runs(&indicator, &page_style());
    }

    /// The box's text back into the page (`setValueListener`).
    fn store_page(&mut self) {
        self.pages[self.current] = self.page.field.value();
    }

    /// Shown again after the signing screen (`rebuildWidgets`): the page's box takes the page
    /// and the focus again.
    pub fn reopened(&mut self, font: &Font, lang: &Lang) {
        self.update_page(font, lang);
        self.set_focus(Some(Focus::Page), None);
    }

    /// `pageBack`.
    fn page_back(&mut self, font: &Font, lang: &Lang) {
        if self.current > 0 {
            self.current -= 1;
            self.update_page(font, lang);
        }
    }

    /// `pageForward`: the next page, or a new one at the end while there are fewer than a
    /// hundred.
    fn page_forward(&mut self, font: &Font, lang: &Lang) {
        if self.current + 1 >= self.pages.len() && self.pages.len() < MAX_PAGES {
            self.pages.push(String::new());
        }
        if self.current + 1 < self.pages.len() {
            self.current += 1;
        }
        self.update_page(font, lang);
    }

    /// `setFocused` of a widget, the box told whether it has the focus.
    fn set_focus(&mut self, focus: Option<Focus>, now: Option<u64>) {
        self.focus = focus;
        self.page.set_focused(focus == Some(Focus::Page), now);
    }

    fn focusable(&self, focus: Focus) -> bool {
        focus != Focus::Back || self.back_visible()
    }

    /// `Screen`'s Tab: the next widget that may take the focus, round to the first.
    fn tab(&mut self, forward: bool) {
        let usable: Vec<Focus> = ORDER.into_iter().filter(|&f| self.focusable(f)).collect();
        let at = self
            .focus
            .and_then(|focus| usable.iter().position(|&f| f == focus));
        let next = match (at, forward) {
            (Some(at), true) => usable.get(at + 1).copied(),
            (Some(at), false) => at.checked_sub(1).map(|at| usable[at]),
            (None, _) => None,
        }
        .or(if forward {
            usable.first().copied()
        } else {
            usable.last().copied()
        });
        self.set_focus(next, None);
    }

    /// A widget pressed (`onPress`).
    fn press(&mut self, focus: Focus, font: &Font, lang: &Lang) -> Option<BookEditAction> {
        match focus {
            Focus::Back | Focus::Forward => {
                // `PageButton.playDownSound`: the page turn replaces the click.
                crate::ui_sound::play_ui("minecraft:item.book.page_turn");
                if focus == Focus::Back {
                    self.page_back(font, lang);
                } else {
                    self.page_forward(font, lang);
                }
            }
            Focus::Sign => {
                crate::ui_sound::play_down_sound();
                return Some(BookEditAction::Sign);
            }
            Focus::Done => {
                crate::ui_sound::play_down_sound();
                return Some(BookEditAction::Done);
            }
            Focus::Page => {}
        }
        None
    }

    /// `keyPressed`: Page Up and Page Down turn the page, Escape closes, the focused widget takes
    /// the rest, then Tab moves the focus.
    pub fn key_pressed(
        &mut self,
        (font, lang): (&Font, &Lang),
        key: glfw::Key,
        (control, shift): (bool, bool),
        clipboard: &mut dyn Clipboard,
    ) -> Option<BookEditAction> {
        use glfw::Key;
        match key {
            // `backButton.onPress`: called directly, so it makes no sound.
            Key::PageUp => {
                self.page_back(font, lang);
                return None;
            }
            Key::PageDown => {
                self.page_forward(font, lang);
                return None;
            }
            Key::Escape => return Some(BookEditAction::Close),
            _ => {}
        }
        match self.focus {
            Some(Focus::Page) => {
                if self
                    .page
                    .key_pressed(font, key, (control, shift), clipboard)
                {
                    self.store_page();
                    return None;
                }
            }
            Some(focus) if matches!(key, Key::Enter | Key::KpEnter | Key::Space) => {
                return self.press(focus, font, lang);
            }
            _ => {}
        }
        if key == Key::Tab {
            self.tab(!shift);
        }
        None
    }

    /// `charTyped`: into the page while it has the focus.
    pub fn char_typed(&mut self, font: &Font, character: char) {
        if self.focus == Some(Focus::Page) && self.page.char_typed(font, character) {
            self.store_page();
        }
    }

    fn over(point: (f64, f64), (x, y): (i32, i32), (w, h): (i32, i32)) -> bool {
        point.0 >= f64::from(x)
            && point.1 >= f64::from(y)
            && point.0 < f64::from(x + w)
            && point.1 < f64::from(y + h)
    }

    /// The widget under the mouse, in the order the screen asks them.
    fn under(&self, at: (f64, f64)) -> Option<Focus> {
        let point = (at.0 as i32, at.1 as i32);
        ORDER.into_iter().find(|&focus| match focus {
            Focus::Page => self.page.contains(at),
            Focus::Back => {
                self.back_visible() && Self::over(at, self.page_button(false), PAGE_BUTTON)
            }
            Focus::Forward => Self::over(at, self.page_button(true), PAGE_BUTTON),
            Focus::Sign => self.sign.contains(&self.arena, point),
            Focus::Done => self.done.contains(&self.arena, point),
        })
    }

    /// `mouseClicked`: the widget under the mouse takes the click, and the focus unless it is a
    /// page button (`PageButton.shouldTakeFocusAfterInteraction`); the page puts its cursor
    /// there. `now` is the frame's clock.
    pub fn clicked(
        &mut self,
        (font, lang): (&Font, &Lang),
        at: (f64, f64),
        (shift, double): (bool, bool),
        now: Option<u64>,
    ) -> Option<BookEditAction> {
        let focus = self.under(at)?;
        if !matches!(focus, Focus::Back | Focus::Forward) {
            self.set_focus(Some(focus), now);
        }
        if focus == Focus::Page {
            self.page.clicked(font, at, shift, double);
            return None;
        }
        self.press(focus, font, lang)
    }

    /// `mouseDragged` over the page that has the focus.
    pub fn dragged(&mut self, font: &Font, at: (f64, f64), shift: bool) {
        if self.focus == Some(Focus::Page) {
            self.page.dragged(font, at, shift);
        }
    }

    /// `extractBackground` and `extractRenderState`: the world dimmed and the book, the page,
    /// the buttons, then the page's number right-aligned.
    pub fn draw(&mut self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        let (width, height) = self.size;
        frame.fill_gradient(atlas, (0, 0, width, height), 0xC010_1010, 0xD010_1010);
        let left = self.left();
        frame.blit_texture(atlas, "book", (256, 256), (0, 0), (left, 2, IMAGE, IMAGE));
        self.page.draw(frame, atlas, font);
        let point = (f64::from(mouse.0), f64::from(mouse.1));
        for (forward, focus) in [(false, Focus::Back), (true, Focus::Forward)] {
            if !self.focusable(focus) {
                continue;
            }
            let (x, y) = self.page_button(forward);
            let lit = self.focus == Some(focus) || Self::over(point, (x, y), PAGE_BUTTON);
            let sprite = match (forward, lit) {
                (true, true) => "widget/page_forward_highlighted",
                (true, false) => "widget/page_forward",
                (false, true) => "widget/page_backward_highlighted",
                (false, false) => "widget/page_backward",
            };
            frame.sprite(atlas, sprite, x, y, PAGE_BUTTON.0, PAGE_BUTTON.1);
        }
        for (widget, focus) in [(&self.sign, Focus::Sign), (&self.done, Focus::Done)] {
            let lit = self.focus == Some(focus) || widget.contains(&self.arena, mouse);
            widget.draw(frame, atlas, font, &self.arena, lit);
        }
        let indicator_width = font.width_of_runs(&self.indicator);
        frame.styled_text(
            atlas,
            font,
            &self.indicator,
            left + 148 - indicator_width,
            2 + 16,
            0xFFFF_FFFF,
            true,
        );
    }
}

#[cfg(test)]
mod click_tests {
    use super::*;

    struct NoClip;
    impl Clipboard for NoClip {
        fn get(&mut self) -> String {
            String::new()
        }
        fn set(&mut self, _: &str) {}
    }

    #[test]
    fn page_keys_are_silent_and_the_buttons_click() {
        let (font, lang) = (Font::default(), Lang::default());
        let mut screen = BookEditScreen::new((&font, &lang), vec![String::new()], (320, 240), None);
        crate::ui_sound::take();
        screen.key_pressed(
            (&font, &lang),
            glfw::Key::PageDown,
            (false, false),
            &mut NoClip,
        );
        assert!(crate::ui_sound::take().is_empty());
        screen.press(Focus::Forward, &font, &lang);
        let sounds = crate::ui_sound::take();
        assert_eq!(sounds.len(), 1);
        assert_eq!(sounds[0].event, "minecraft:item.book.page_turn");
    }
}
