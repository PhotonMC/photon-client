//! The forms of the server list: adding or editing a server (`ManageServerScreen`: its name, its
//! address and whether its resource pack is taken) and joining one by its address
//! (`DirectJoinServerScreen`).

use std::time::Instant;

use voxel_game::servers::{PackStatus, is_valid_address};
use voxel_game::text::{Component, Lang, Run, Style};

use crate::gui::font::Font;
use crate::gui::layout::Arena;
use crate::gui::widgets::cycle::CycleButton;
use crate::gui::widgets::edit_box::{Clipboard, EditBox, edit_key};
use crate::gui::widgets::{Kind, Widget};
use crate::gui::{GuiAtlas, GuiFrame};

/// `-6250336`: the labels' grey.
const LABEL_GREY: u32 = 0xFFA0_A0A0;
/// `EditBox.textColor`.
const TEXT_COLOUR: u32 = 0xFFE0_E0E0;

/// Which form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormKind {
    Add,
    Edit,
    Direct,
}

/// What the form was left with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormResult {
    /// Done (Add, Edit) or Join Server (Direct): the name (the default one for an empty box), the
    /// address and the resource pack choice.
    Accepted {
        name: String,
        ip: String,
        pack: PackStatus,
    },
    /// Cancel or Escape.
    Cancelled,
}

/// What can have the focus, in the order Tab goes through it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Name,
    Ip,
    Pack,
    Done,
    Cancel,
}

/// An edit box with its border at `(x, y)`, 200 by 20.
struct Field {
    edit: EditBox,
    at: (i32, i32),
}

pub struct ServerFormScreen {
    kind: FormKind,
    size: (i32, i32),
    title: Vec<Run>,
    title_y: i32,
    labels: Vec<(Vec<Run>, i32)>,
    name: Option<Field>,
    ip: Field,
    hint: Vec<Run>,
    default_name: String,
    arena: Arena,
    pack: Option<CycleButton<PackStatus>>,
    done: Widget,
    cancel: Widget,
    focus: Option<Focus>,
    focused_at: Instant,
}

fn pack_name(status: &PackStatus) -> Component {
    Component::translatable(match status {
        PackStatus::Enabled => "manageServer.resourcePack.enabled",
        PackStatus::Disabled => "manageServer.resourcePack.disabled",
        PackStatus::Prompt => "manageServer.resourcePack.prompt",
    })
}

impl ServerFormScreen {
    /// `init` of the form for a server named `name` at `ip` (the Direct form takes only `ip`, the
    /// last address joined), in a screen `size` big. The first box has the focus.
    pub fn new(
        font: &Font,
        lang: &Lang,
        kind: FormKind,
        size: (i32, i32),
        (name, ip, pack): (&str, &str, PackStatus),
    ) -> Self {
        let runs = |key: &str| lang.runs(&Component::translatable(key), &Style::default());
        let (width, height) = size;
        let left = width / 2 - 100;
        let field = |text: &str, y: i32, max: usize| {
            let mut edit = EditBox::new(192, max);
            edit.set_value(text, font);
            Field {
                edit,
                at: (left, y),
            }
        };
        let mut arena = Arena::default();
        let mut button = |key: &str, y: i32| {
            let rect = arena.add(200, 20);
            arena.0[rect].x = left;
            arena.0[rect].y = y;
            Widget {
                rect,
                active: true,
                kind: Kind::Button { label: runs(key) },
            }
        };
        let screen = match kind {
            FormKind::Add | FormKind::Edit => {
                let done = button("gui.done", height / 4 + 96 + 18);
                let cancel = button("gui.cancel", height / 4 + 120 + 18);
                let rect = arena.add(200, 20);
                arena.0[rect].x = left;
                arena.0[rect].y = height / 4 + 72;
                let pack = CycleButton::new(
                    lang,
                    rect,
                    Component::translatable("manageServer.resourcePack"),
                    vec![
                        PackStatus::Enabled,
                        PackStatus::Disabled,
                        PackStatus::Prompt,
                    ],
                    pack_name,
                    &pack,
                );
                Self {
                    kind,
                    size,
                    title: runs(if kind == FormKind::Add {
                        "manageServer.add.title"
                    } else {
                        "manageServer.edit.title"
                    }),
                    title_y: 17,
                    labels: vec![
                        (runs("manageServer.enterName"), 53),
                        (runs("manageServer.enterIp"), 94),
                    ],
                    name: Some(field(name, 66, 32)),
                    ip: field(ip, 106, 128),
                    // `EditBox.setHint`: a hint with no style of its own is dark grey.
                    hint: lang.runs(
                        &Component::translatable("selectServer.defaultName"),
                        &Style {
                            colour: Some(0x55_5555),
                            ..Style::default()
                        },
                    ),
                    default_name: lang.text("selectServer.defaultName"),
                    arena,
                    pack: Some(pack),
                    done,
                    cancel,
                    focus: Some(Focus::Name),
                    focused_at: Instant::now(),
                }
            }
            FormKind::Direct => {
                let done = button("selectServer.select", height / 4 + 96 + 12);
                let cancel = button("gui.cancel", height / 4 + 120 + 12);
                Self {
                    kind,
                    size,
                    title: runs("selectServer.direct"),
                    title_y: 20,
                    labels: vec![(runs("manageServer.enterIp"), 100)],
                    name: None,
                    ip: field(ip, 116, 128),
                    hint: Vec::new(),
                    default_name: lang.text("selectServer.defaultName"),
                    arena,
                    pack: None,
                    done,
                    cancel,
                    focus: Some(Focus::Ip),
                    focused_at: Instant::now(),
                }
            }
        };
        let mut screen = screen;
        screen.address_changed();
        screen
    }

    pub fn kind(&self) -> FormKind {
        self.kind
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    /// What the address box holds.
    pub fn address(&self) -> String {
        self.ip.edit.value()
    }

    /// `updateAddButtonStatus`: Done wakes for an address the resolver can read.
    fn address_changed(&mut self) {
        self.done.active = is_valid_address(&self.ip.edit.value());
    }

    pub fn draw(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        let (width, height) = self.size;
        frame.blit_texture_stretched(
            atlas,
            "title/background/panorama_overlay",
            (16, 128),
            (0, 0, 16, 128),
            (0, 0, width, height),
        );
        frame.tile_texture(atlas, "menu_background", (32, 32), (0, 0, width, height));
        if let Some(pack) = &self.pack {
            pack.draw(frame, atlas, font, &self.arena, mouse);
        }
        for (widget, which) in [(&self.done, Focus::Done), (&self.cancel, Focus::Cancel)] {
            let highlighted = widget.contains(&self.arena, mouse) || self.focus == Some(which);
            widget.draw(frame, atlas, font, &self.arena, highlighted);
        }
        let title_width = font.width_of_runs(&self.title);
        frame.styled_text(
            atlas,
            font,
            &self.title,
            width / 2 - title_width / 2,
            self.title_y,
            0xFFFF_FFFF,
            true,
        );
        for (label, y) in &self.labels {
            frame.styled_text(atlas, font, label, width / 2 - 99, *y, LABEL_GREY, true);
        }
        if let Some(name) = &self.name {
            let focused = self.focus == Some(Focus::Name);
            self.draw_field(frame, atlas, font, name, focused, Some(&self.hint));
        }
        let focused = self.focus == Some(Focus::Ip);
        self.draw_field(frame, atlas, font, &self.ip, focused, None);
    }

    /// `EditBox.extractWidgetRenderState` with its border: the field's sprite, the text and the
    /// cursor, or the hint in an empty box without the focus.
    fn draw_field(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        field: &Field,
        focused: bool,
        hint: Option<&[Run]>,
    ) {
        let (x, y) = field.at;
        let sprite = if focused {
            "widget/text_field_highlighted"
        } else {
            "widget/text_field"
        };
        frame.nine_slice(atlas, sprite, (200, 20), 1, false, x, y, 200, 20);
        let text_at = (x + 4, y + (20 - 8) / 2);
        field.edit.draw_formatted(
            frame,
            atlas,
            font,
            text_at,
            focused.then_some(self.focused_at),
            None,
        );
        if let Some(hint) = hint.filter(|_| field.edit.is_empty() && !focused) {
            frame.styled_text(atlas, font, hint, text_at.0, text_at.1, TEXT_COLOUR, true);
        }
    }

    /// The focus moves to `focus`; a box that takes it blinks its cursor from now.
    fn set_focus(&mut self, focus: Option<Focus>) {
        if focus != self.focus && matches!(focus, Some(Focus::Name | Focus::Ip)) {
            self.focused_at = Instant::now();
        }
        if let Some(pack) = &mut self.pack {
            pack.set_focused(focus == Some(Focus::Pack));
        }
        self.focus = focus;
    }

    /// The focusable widgets in Tab's order.
    fn order(&self) -> Vec<Focus> {
        let mut order = Vec::new();
        if self.name.is_some() {
            order.push(Focus::Name);
        }
        order.push(Focus::Ip);
        if self.pack.is_some() {
            order.push(Focus::Pack);
        }
        if self.done.active {
            order.push(Focus::Done);
        }
        order.push(Focus::Cancel);
        order
    }

    fn tab(&mut self, forward: bool) {
        let order = self.order();
        let place = self
            .focus
            .and_then(|focus| order.iter().position(|candidate| *candidate == focus));
        let count = order.len();
        let next = match (place, forward) {
            (None, true) => 0,
            (None, false) => count - 1,
            (Some(index), true) => (index + 1) % count,
            (Some(index), false) => (index + count - 1) % count,
        };
        self.set_focus(Some(order[next]));
    }

    fn field_mut(&mut self, focus: Focus) -> Option<&mut Field> {
        match focus {
            Focus::Name => self.name.as_mut(),
            Focus::Ip => Some(&mut self.ip),
            _ => None,
        }
    }

    /// `onAdd` / `onSelect`.
    fn accept(&self) -> FormResult {
        let name = self
            .name
            .as_ref()
            .map(|field| field.edit.value())
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| self.default_name.clone());
        FormResult::Accepted {
            name,
            ip: self.ip.edit.value(),
            pack: self
                .pack
                .as_ref()
                .map_or(PackStatus::Prompt, |pack| *pack.value()),
        }
    }

    /// The focused widget pressed (Enter, Space, or a click).
    fn press(&mut self, focus: Focus, lang: &Lang, shift: bool) -> Option<FormResult> {
        match focus {
            Focus::Done if self.done.active => Some(self.accept()),
            Focus::Cancel => Some(FormResult::Cancelled),
            Focus::Pack => {
                if let Some(pack) = &mut self.pack {
                    pack.press(lang, shift);
                }
                None
            }
            _ => None,
        }
    }

    /// A character typed into the focused box.
    pub fn char_typed(&mut self, font: &Font, character: char) {
        let Some(focus) = self.focus else {
            return;
        };
        if let Some(field) = self.field_mut(focus) {
            field.edit.char_typed(character, font);
        }
        self.address_changed();
    }

    /// A key: the focused box edits, Tab moves the focus, Enter or Space presses the focused
    /// button (and Enter in the Direct form's box joins, as `DirectJoinServerScreen.keyPressed`
    /// does), Escape cancels.
    pub fn key_pressed(
        &mut self,
        (font, lang): (&Font, &Lang),
        key: glfw::Key,
        (control, shift): (bool, bool),
        clipboard: &mut dyn Clipboard,
    ) -> Option<FormResult> {
        use glfw::Key;
        let confirm = matches!(key, Key::Enter | Key::KpEnter);
        match key {
            Key::Escape => return Some(FormResult::Cancelled),
            Key::Tab => {
                self.tab(!shift);
                return None;
            }
            _ => {}
        }
        let focus = self.focus?;
        if self.kind == FormKind::Direct && focus == Focus::Ip && confirm && self.done.active {
            return Some(self.accept());
        }
        if let (Some(field), Some(edit)) = (self.field_mut(focus), edit_key(key)) {
            field
                .edit
                .key_pressed(edit, control, shift, font, clipboard);
            self.address_changed();
            return None;
        }
        if confirm || key == Key::Space {
            return self.press(focus, lang, shift);
        }
        None
    }

    /// A press of the primary button at `mouse`: a box takes the focus and the cursor, a button
    /// is pressed.
    pub fn clicked(
        &mut self,
        (font, lang): (&Font, &Lang),
        mouse: (i32, i32),
        shift: bool,
    ) -> Option<FormResult> {
        let inside = |(x, y): (i32, i32)| {
            mouse.0 >= x && mouse.1 >= y && mouse.0 < x + 200 && mouse.1 < y + 20
        };
        for focus in [Focus::Name, Focus::Ip] {
            let Some(at) = self.field_mut(focus).map(|field| field.at) else {
                continue;
            };
            if inside(at) {
                self.set_focus(Some(focus));
                if let Some(field) = self.field_mut(focus) {
                    field.edit.click_at(mouse.0 - at.0 - 4, font);
                }
                return None;
            }
        }
        let pack_hit = self.pack.as_ref().is_some_and(|pack| {
            pack.is_mouse_over(&self.arena, (f64::from(mouse.0), f64::from(mouse.1)))
        });
        let hit = if pack_hit {
            Some(Focus::Pack)
        } else if self.done.active && self.done.contains(&self.arena, mouse) {
            Some(Focus::Done)
        } else if self.cancel.contains(&self.arena, mouse) {
            Some(Focus::Cancel)
        } else {
            None
        };
        let focus = hit?;
        self.set_focus(Some(focus));
        self.press(focus, lang, shift)
    }
}
