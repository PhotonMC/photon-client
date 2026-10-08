//! The signing of a book and quill (`BookSignScreen`): the title typed in the middle of the book,
//! the author under it, the warning that signing is final, and Sign and Close and Cancel under
//! the book.

use photon_game::text::{Component, Lang, Run, Style};

use crate::gui::chat::split;
use crate::gui::font::Font;
use crate::gui::layout::Arena;
use crate::gui::widgets::edit_box::{Clipboard, EditBox, edit_key};
use crate::gui::widgets::{Kind, Widget};
use crate::gui::{GuiAtlas, GuiFrame};

/// The book is 192 square, its text 114 wide.
const IMAGE: i32 = 192;
const TEXT_WIDTH: i32 = 114;
/// A title is at most fifteen characters.
const TITLE_LENGTH: usize = 15;
const BLACK: u32 = 0xFF00_0000;

/// What the screen asks of the game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BookSignAction {
    /// The book is signed with this title (trimmed) and the screen closes (`saveChanges`).
    Finalize(String),
    /// Back to the book's pages, the title kept as typed.
    Cancel(String),
    /// Escape: the screen closes and nothing is sent.
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Title,
    Finalize,
    Cancel,
}

/// `String.trim`: what is at most a space goes from both ends.
fn java_trim(text: &str) -> String {
    text.trim_matches(|c: char| c <= ' ').to_owned()
}

/// `BookSignScreen`.
pub struct BookSignScreen {
    size: (i32, i32),
    title: EditBox,
    arena: Arena,
    finalize: Widget,
    cancel: Widget,
    focus: Option<Focus>,
    /// When the title took the focus.
    focused_at: std::time::Instant,
    label: Vec<Run>,
    author: Vec<Run>,
    warning: Vec<Run>,
}

impl BookSignScreen {
    /// `init` with the title typed so far, the title focused (`setInitialFocus`, which a screen
    /// shown again runs too).
    pub fn new((font, lang): (&Font, &Lang), title: &str, owner: &str, size: (i32, i32)) -> Self {
        let runs = |component: &Component| lang.runs(component, &Style::default());
        let mut arena = Arena::default();
        let button = |arena: &mut Arena, key: &str| Widget {
            rect: arena.add(98, 20),
            active: true,
            kind: Kind::Button {
                label: runs(&Component::translatable(key)),
            },
        };
        let finalize = button(&mut arena, "book.finalizeButton");
        let cancel = button(&mut arena, "gui.cancel");
        let mut edit = EditBox::new(TEXT_WIDTH, TITLE_LENGTH)
            .with_colour(BLACK)
            .without_shadow();
        edit.set_value(title, font);
        let mut author = Component::translatable_with(
            "book.byAuthor",
            vec![Component::literal(owner.to_owned())],
        );
        author.style.colour = photon_game::text::named_colour("dark_gray");
        let mut screen = Self {
            size,
            title: edit,
            arena,
            finalize,
            cancel,
            focus: Some(Focus::Title),
            focused_at: std::time::Instant::now(),
            label: runs(&Component::translatable("book.editTitle")),
            author: runs(&author),
            warning: runs(&Component::translatable("book.finalizeWarning")),
        };
        screen.update_finalize();
        screen.resize(size);
        screen
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    /// The title as typed.
    pub fn title(&self) -> String {
        self.title.value()
    }

    /// The window was resized: the buttons move with it.
    pub fn resize(&mut self, (width, height): (i32, i32)) {
        self.size = (width, height);
        for (widget, x) in [
            (&self.finalize, width / 2 - 100),
            (&self.cancel, width / 2 + 2),
        ] {
            let rect = &mut self.arena.0[widget.rect];
            rect.x = x;
            rect.y = 196;
        }
    }

    fn title_left(&self) -> i32 {
        (self.size.0 - TEXT_WIDTH) / 2 - 3
    }

    /// The responder: Sign and Close wakes while the title is not blank.
    fn update_finalize(&mut self) {
        self.finalize.active = !self.title.value().chars().all(char::is_whitespace);
    }

    fn focus_title(&mut self) {
        if self.focus != Some(Focus::Title) {
            self.focused_at = std::time::Instant::now();
        }
        self.focus = Some(Focus::Title);
    }

    fn press(&self, focus: Focus) -> Option<BookSignAction> {
        match focus {
            Focus::Finalize if self.finalize.active => {
                Some(BookSignAction::Finalize(java_trim(&self.title.value())))
            }
            Focus::Cancel => Some(BookSignAction::Cancel(self.title.value())),
            _ => None,
        }
    }

    /// `Screen`'s Tab over the title and the buttons that may take the focus.
    fn tab(&mut self, forward: bool) {
        let mut usable = vec![Focus::Title];
        if self.finalize.active {
            usable.push(Focus::Finalize);
        }
        usable.push(Focus::Cancel);
        let at = self
            .focus
            .and_then(|focus| usable.iter().position(|&f| f == focus));
        let next = match (at, forward) {
            (Some(at), true) => usable.get(at + 1).copied(),
            (Some(at), false) => at.checked_sub(1).map(|at| usable[at]),
            (None, _) => None,
        }
        .unwrap_or(if forward {
            usable[0]
        } else {
            usable[usable.len() - 1]
        });
        if next == Focus::Title {
            self.focus_title();
        } else {
            self.focus = Some(next);
        }
    }

    /// `keyPressed`: Enter signs a title that is not empty, Escape closes, the focused widget
    /// takes the rest, then Tab moves the focus.
    pub fn key_pressed(
        &mut self,
        font: &Font,
        key: glfw::Key,
        (control, shift): (bool, bool),
        clipboard: &mut dyn Clipboard,
    ) -> Option<BookSignAction> {
        use glfw::Key;
        let confirm = matches!(key, Key::Enter | Key::KpEnter);
        if self.focus == Some(Focus::Title) && !self.title.is_empty() && confirm {
            return Some(BookSignAction::Finalize(java_trim(&self.title.value())));
        }
        if key == Key::Escape {
            return Some(BookSignAction::Close);
        }
        match self.focus {
            Some(Focus::Title) => {
                if let Some(edit) = edit_key(key)
                    && self
                        .title
                        .key_pressed(edit, control, shift, font, clipboard)
                {
                    self.update_finalize();
                    return None;
                }
            }
            Some(focus) if confirm || key == Key::Space => return self.press(focus),
            _ => {}
        }
        if key == Key::Tab {
            self.tab(!shift);
        }
        None
    }

    /// `charTyped`: into the title while it has the focus.
    pub fn char_typed(&mut self, font: &Font, character: char) {
        if self.focus == Some(Focus::Title) && self.title.char_typed(character, font) {
            self.update_finalize();
        }
    }

    /// Where the title's text starts: centred in its box.
    fn text_left(&self, font: &Font) -> i32 {
        self.title_left() + (TEXT_WIDTH - font.width(&self.title.shown(font).text)) / 2
    }

    /// `mouseClicked`: the title takes the cursor where it is clicked; a button is pressed.
    pub fn clicked(&mut self, font: &Font, (x, y): (i32, i32)) -> Option<BookSignAction> {
        let left = self.title_left();
        if (left..left + TEXT_WIDTH).contains(&x) && (50..70).contains(&y) {
            self.focus_title();
            let offset = x - self.text_left(font);
            self.title.click_at(offset, font);
            return None;
        }
        for (widget, focus) in [
            (&self.finalize, Focus::Finalize),
            (&self.cancel, Focus::Cancel),
        ] {
            if widget.is_active() && widget.contains(&self.arena, (x, y)) {
                return self.press(focus);
            }
        }
        None
    }

    /// `extractBackground` and `extractRenderState`: the world dimmed and the book, the title and
    /// the buttons, then the label, the author and the warning in black.
    pub fn draw(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        let (width, height) = self.size;
        frame.fill_gradient(atlas, (0, 0, width, height), 0xC010_1010, 0xD010_1010);
        let left = (width - IMAGE) / 2;
        frame.blit_texture(atlas, "book", (256, 256), (0, 0), left, 2, IMAGE, IMAGE);
        let focused = (self.focus == Some(Focus::Title)).then_some(self.focused_at);
        self.title.draw_formatted(
            frame,
            atlas,
            font,
            (self.text_left(font), 50),
            focused,
            None,
        );
        for (widget, focus) in [
            (&self.finalize, Focus::Finalize),
            (&self.cancel, Focus::Cancel),
        ] {
            let lit = self.focus == Some(focus) || widget.contains(&self.arena, mouse);
            widget.draw(frame, atlas, font, &self.arena, lit);
        }
        let centred = |runs: &[Run]| left + 36 + (TEXT_WIDTH - font.width_of_runs(runs)) / 2;
        frame.styled_text(
            atlas,
            font,
            &self.label,
            centred(&self.label),
            34,
            BLACK,
            false,
        );
        frame.styled_text(
            atlas,
            font,
            &self.author,
            centred(&self.author),
            60,
            BLACK,
            false,
        );
        for (index, line) in split(&self.warning, TEXT_WIDTH, font).iter().enumerate() {
            frame.styled_text(
                atlas,
                font,
                line,
                left + 36,
                82 + index as i32 * 9,
                BLACK,
                false,
            );
        }
    }
}
