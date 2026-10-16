//! The sign editing screens (`AbstractSignEditScreen`, `SignEditScreen`, `HangingSignEditScreen`):
//! the sign's board blown up in the middle, its four lines in the text's colour with the cursor
//! on the line being edited, and Done under it.

use crate::gui::{FontRuns, FrameRuns};
use photon_game::text::{Component, Lang, Run, Style};

use crate::block_entity::sign::{SignKind, dark_color};
use crate::gui::font::Font;
use crate::gui::layout::Arena;
use crate::gui::widgets::edit_box::{Clipboard, edit_key};
use crate::gui::widgets::text_field::TextField;
use crate::gui::widgets::{Kind, Widget};
use crate::gui::{GuiAtlas, GuiFrame};

/// Which board the screen shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignBoard {
    /// A standing sign (`SignEditScreen`), whose post shows.
    Standing,
    /// A sign on a wall, its board without the post.
    Wall,
    /// A hanging sign on a ceiling or a wall (`HangingSignEditScreen`).
    Hanging,
}

impl SignBoard {
    fn kind(self) -> SignKind {
        match self {
            Self::Hanging => SignKind::Hanging,
            _ => SignKind::Standing,
        }
    }
}

/// `SignEditScreen.MAGIC_TEXT_SCALE`.
const STANDING_TEXT_SCALE: f32 = 0.976_562_8;

/// `TextCursorUtils.isCursorVisible`: on for 300 ms, off for 300.
fn cursor_visible(millis: u64) -> bool {
    (millis / 300).is_multiple_of(2)
}

/// `AbstractSignEditScreen` with the face being edited.
pub struct SignEditScreen {
    board: SignBoard,
    wood: String,
    messages: [String; 4],
    line: usize,
    field: TextField,
    /// The lines' colour: a glowing dye's own, a dark shade of another.
    colour: u32,
    title: Vec<Run>,
    arena: Arena,
    done: Widget,
    done_focused: bool,
    size: (i32, i32),
    /// `cursorBlinkStartTime`, on the frame's clock: the first frame's when not given.
    opened_at: Option<u64>,
}

impl SignEditScreen {
    /// `init` for a sign of a `wood` (`WoodType.name`) whose face reads `messages`, its dye (an
    /// index into `DYES`) and glow, opened at `now` on the frame's clock (or when first drawn).
    pub fn new(
        lang: &Lang,
        (board, wood): (SignBoard, &str),
        messages: [String; 4],
        (color, glowing): (usize, bool),
        size: (i32, i32),
        now: Option<u64>,
    ) -> Self {
        let mut arena = Arena::default();
        let rect = arena.add(200, 20);
        let runs = |key: &str| lang.runs(&Component::translatable(key), &Style::default());
        let title = runs(if board == SignBoard::Hanging {
            "hanging_sign.edit"
        } else {
            "sign.edit"
        });
        let colour = if glowing {
            photon_game::sign::DYE_TEXT_COLORS[color]
        } else {
            dark_color(color, false)
        };
        let mut screen = Self {
            board,
            wood: wood.to_owned(),
            field: TextField::new(&messages[0]),
            messages,
            line: 0,
            colour,
            title,
            arena,
            done: Widget {
                rect,
                active: true,
                kind: Kind::Button {
                    label: runs("gui.done"),
                },
            },
            done_focused: false,
            size,
            opened_at: now,
        };
        screen.resize(size);
        screen
    }

    /// The window was resized: Done moves with it.
    pub fn resize(&mut self, (width, height): (i32, i32)) {
        self.size = (width, height);
        let rect = &mut self.arena.0[self.done.rect];
        rect.x = width / 2 - 100;
        rect.y = height / 4 + 144;
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    /// The lines as they stand, which `removed` sends (`ServerboundSignUpdatePacket`).
    pub fn messages(&self) -> &[String; 4] {
        &self.messages
    }

    fn line_height(&self) -> i32 {
        self.board.kind().line_height()
    }

    /// `keyPressed`: the arrows and Enter go from line to line, the line's keys edit it, Tab
    /// focuses Done. Whether the screen is done (Escape, or Space on Done).
    pub fn key_pressed(
        &mut self,
        font: &Font,
        key: glfw::Key,
        (control, shift): (bool, bool),
        clipboard: &mut dyn Clipboard,
    ) -> bool {
        use glfw::Key;
        let step = match key {
            Key::Up => Some(3),
            Key::Down | Key::Enter | Key::KpEnter => Some(1),
            _ => None,
        };
        if let Some(step) = step {
            self.line = (self.line + step) & 3;
            self.field
                .set_cursor_to_end(&self.messages[self.line], false);
            return false;
        }
        let max_width = self.board.kind().max_line_width();
        let valid = |text: &str| font.width(text) <= max_width;
        if let Some(edit) = edit_key(key)
            && self.field.key_pressed(
                &mut self.messages[self.line],
                edit,
                (control, shift),
                clipboard,
                &valid,
            )
        {
            return false;
        }
        match key {
            Key::Escape => true,
            // `Screen.keyPressed`'s Tab: the only widget takes the focus.
            Key::Tab => {
                self.done_focused = true;
                false
            }
            Key::Space => self.done_focused,
            _ => false,
        }
    }

    /// `charTyped`: into the line, while it fits the sign.
    pub fn char_typed(&mut self, font: &Font, character: char) {
        let max_width = self.board.kind().max_line_width();
        let valid = |text: &str| font.width(text) <= max_width;
        self.field
            .char_typed(&mut self.messages[self.line], character, &valid);
    }

    /// `mouseClicked`: whether Done was pressed.
    pub fn clicked(&self, mouse: (i32, i32)) -> bool {
        self.done.contains(&self.arena, mouse)
    }

    /// `extractBackground` and `extractRenderState`: the world dimmed, Done, the title, then the
    /// sign.
    pub fn draw(&mut self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        self.opened_at.get_or_insert(frame.millis);
        let (width, height) = self.size;
        frame.fill_gradient(atlas, (0, 0, width, height), 0xC010_1010, 0xD010_1010);
        let highlighted = self.done_focused || self.done.contains(&self.arena, mouse);
        self.done.draw(frame, atlas, font, &self.arena, highlighted);
        let title_width = font.width_of_runs(&self.title);
        frame.styled_text(
            atlas,
            font,
            &self.title,
            width / 2 - title_width / 2,
            40,
            0xFFFF_FFFF,
            true,
        );
        self.draw_sign(frame, atlas, font);
    }

    /// `extractSign`: the board, then the text, each in its pose about the sign's middle.
    fn draw_sign(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font) {
        let middle = self.size.0 as f32 / 2.0;
        let mark = frame.mark();
        let (offset, text_scale) = match self.board {
            SignBoard::Hanging => {
                let texture = format!("hanging_signs/{}", self.wood);
                frame.blit_texture(atlas, &texture, (16, 16), (0, 0), -8, -8, 16, 16);
                frame.transform_from(mark, [4.5, 0.0, 0.0, 4.5, middle, 125.0 - 13.0]);
                (125.0, 1.0)
            }
            board => {
                let shown = if board == SignBoard::Wall { 12 } else { 26 };
                let texture = format!("signs/{}", self.wood);
                frame.blit_texture(atlas, &texture, (24, 26), (0, 0), -12, -13, 24, shown);
                frame.transform_from(mark, [3.9, 0.0, 0.0, 3.9, middle, 90.0 + 27.0]);
                (90.0, STANDING_TEXT_SCALE)
            }
        };
        let mark = frame.mark();
        self.draw_text(frame, atlas, font);
        frame.transform_from(mark, [text_scale, 0.0, 0.0, text_scale, middle, offset]);
    }

    /// `extractSignText`: the four lines centred, the cursor on the edited one (`_` after its
    /// end, a bar inside it) while it blinks on, and the selection over the text.
    fn draw_text(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font) {
        let colour = self.colour;
        let opened_at = self.opened_at.unwrap_or(frame.millis);
        let show_cursor = cursor_visible(frame.millis.saturating_sub(opened_at));
        let height = self.line_height();
        let middle = 4 * height / 2;
        let cursor_y = self.line as i32 * height - middle;
        let (cursor, selection) = (self.field.cursor(), self.field.selection());
        let head_width = |line: &[char], end: usize| {
            font.width(&line[..end.min(line.len())].iter().collect::<String>())
        };
        for (index, line) in self.messages.iter().enumerate() {
            let left = -font.width(line) / 2;
            frame.text(
                atlas,
                font,
                line,
                left,
                index as i32 * height - middle,
                colour,
            );
            let chars: Vec<char> = line.chars().collect();
            if index == self.line && show_cursor && cursor >= chars.len() {
                let cursor_x = head_width(&chars, cursor) - font.width(line) / 2;
                frame.text(atlas, font, "_", cursor_x, cursor_y, colour);
            }
        }
        let line = &self.messages[self.line];
        let chars: Vec<char> = line.chars().collect();
        let half = font.width(line) / 2;
        let cursor_x = head_width(&chars, cursor) - half;
        if cursor < chars.len() && show_cursor {
            let opaque = colour | 0xFF00_0000;
            frame.fill(
                atlas,
                (cursor_x, cursor_y - 1, cursor_x + 1, cursor_y + height),
                opaque,
            );
        }
        if selection != cursor {
            let start = head_width(&chars, cursor.min(selection)) - half;
            let end = head_width(&chars, cursor.max(selection)) - half;
            frame.text_highlight(
                atlas,
                (start.min(end), cursor_y, start.max(end), cursor_y + height),
            );
        }
    }
}
