//! The server list (`JoinMultiplayerScreen`, `ServerSelectionList`): the saved servers, each
//! still being asked about (`ServerData.State.PINGING`), the local network's header below them,
//! and the buttons under the list.

use photon_game::text::{Component, Lang, Run, Style};

use crate::layout::{Grid, Item, Settings, horizontal, place_in_frame};
use crate::widgets::button_set::ButtonSet;
use crate::widgets::scroll_area::{Place, SCROLLBAR_WIDTH};
use crate::widgets::{Kind, Widget};
use crate::{FontRuns, FrameRuns};
use photon_render::gui::{GuiAtlas, GuiFrame, font::Font};

/// `HeaderAndFooterLayout(this, 33, 60)`.
mod ping;
use ping::PingView;

const HEADER_HEIGHT: i32 = 33;
const FOOTER_HEIGHT: i32 = 60;
/// The list's entries and `getRowWidth`.
const ENTRY_HEIGHT: i32 = 36;
const ROW_WIDTH: i32 = 305;
/// A server's icon.
const ICON: i32 = 32;
/// `-8355712`: the grey of a server's second line and status.
const GREY: u32 = 0xFF80_8080;
/// `LoadingDotsText.FRAMES`, three tenths of a second each.
const DOTS: [&str; 4] = ["O o o", "o O o", "o o O", "o O o"];

/// What a button or a click on the list asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServersAction {
    /// Join the saved server at this place.
    Join(usize),
    Direct,
    Add,
    Edit(usize),
    Delete(usize),
    Refresh,
    Back,
    /// The saved servers at these two places changed places (saved at once).
    Swap(usize, usize),
}

/// The footer's buttons, in the order they are added.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Button {
    Select,
    Direct,
    Add,
    Edit,
    Delete,
    Refresh,
    Back,
}

/// The list's entry with the selection: a saved server or the local network's header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Selected {
    Online(usize),
    LanHeader,
}

pub struct JoinMultiplayerScreen {
    buttons: ButtonSet<Button>,
    title: Widget,
    /// The saved servers' names, in the list's order, and what their pings found.
    names: Vec<String>,
    pings: Vec<Option<PingView>>,
    /// The texture each server's icon is drawn from (`FaviconTexture.textureLocation`), when it
    /// has one in the atlas.
    icons: Vec<Option<String>>,
    selected: Option<Selected>,
    /// Whether the list has the focus (a click on it gives it), which whitens the selection.
    list_focused: bool,
    scanning: Vec<Run>,
    pinging: Vec<Run>,
    place: Place,
    scroll: f64,
}

impl JoinMultiplayerScreen {
    /// `init` for the saved servers `names`, in a screen `size` big.
    pub fn new(font: &Font, lang: &Lang, size: (i32, i32), names: Vec<String>) -> Self {
        let runs = |key: &str| lang.runs(&Component::translatable(key), &Style::default());
        let mut buttons = ButtonSet::new(size);
        let title_text = runs("multiplayer.title");
        let title_rect = buttons.arena.add(font.width_of_runs(&title_text), 9);
        let title = Widget {
            rect: title_rect,
            active: false,
            kind: Kind::Text { text: title_text },
        };
        let mut add = |key: &str, width: i32, button: Button| {
            buttons.add(
                (0, 0, width, 20),
                (Kind::Button { label: runs(key) }, true),
                button,
                None,
            )
        };
        let top = [
            add("selectServer.select", 100, Button::Select),
            add("selectServer.direct", 100, Button::Direct),
            add("selectServer.add", 100, Button::Add),
        ];
        let bottom = [
            add("selectServer.edit", 74, Button::Edit),
            add("selectServer.delete", 74, Button::Delete),
            add("selectServer.refresh", 74, Button::Refresh),
            add("gui.back", 74, Button::Back),
        ];
        let mut screen = Self {
            buttons,
            title,
            pings: vec![None; names.len()],
            icons: vec![None; names.len()],
            names,
            selected: None,
            list_focused: false,
            scanning: runs("lanServer.scanning"),
            pinging: runs("multiplayer.status.pinging"),
            place: Place {
                y: HEADER_HEIGHT,
                width: size.0,
                height: size.1 - HEADER_HEIGHT - FOOTER_HEIGHT,
                in_world: false,
            },
            scroll: 0.0,
        };
        screen.arrange(size, &top, &bottom);
        screen.selection_changed();
        screen
    }

    /// `HeaderAndFooterLayout.arrangeElements`: the title in the header, the two rows of buttons
    /// in the footer, the list between.
    fn arrange(&mut self, (width, height): (i32, i32), top: &[usize], bottom: &[usize]) {
        let arena = &mut self.buttons.arena;
        let centred = Settings::default().align_horizontally(0.5);
        let header = (
            0,
            0,
            width,
            HEADER_HEIGHT.max(arena.0[self.title.rect].height),
        );
        place_in_frame(arena, Item::Leaf(self.title.rect), header, (0.5, 0.5));
        let mut footer = Grid::new();
        footer.row_spacing = 4;
        footer.add(
            Item::Grid(Box::new(horizontal(arena, top, 4))),
            (0, 0),
            (1, 1),
            centred,
        );
        footer.add(
            Item::Grid(Box::new(horizontal(arena, bottom, 4))),
            (1, 0),
            (1, 1),
            centred,
        );
        footer.arrange(arena);
        let frame = (
            0,
            height - FOOTER_HEIGHT,
            width.max(footer.width),
            FOOTER_HEIGHT.max(footer.height),
        );
        place_in_frame(arena, Item::Grid(Box::new(footer)), frame, (0.5, 0.5));
    }

    pub fn size(&self) -> (i32, i32) {
        self.buttons.size()
    }

    /// The saved servers and the local network's header.
    fn entries(&self) -> usize {
        self.names.len() + 1
    }

    fn content_height(&self) -> i32 {
        self.entries() as i32 * ENTRY_HEIGHT + 4
    }

    fn row_left(&self) -> i32 {
        self.place.width / 2 - ROW_WIDTH / 2
    }

    fn entry_y(&self, index: usize) -> i32 {
        self.place.y + 2 - self.scroll as i32 + index as i32 * ENTRY_HEIGHT
    }

    /// `getEntryAtPosition` while the mouse is on the list.
    fn entry_at(&self, (x, y): (i32, i32)) -> Option<usize> {
        if !self.place.contains((f64::from(x), f64::from(y))) {
            return None;
        }
        let left = self.row_left();
        (0..self.entries()).find(|&index| {
            let top = self.entry_y(index);
            x >= left && x < left + ROW_WIDTH && y >= top && y < top + ENTRY_HEIGHT
        })
    }

    /// `onSelectedChange`: Join wakes for an entry that is not the header, Edit and Delete for a
    /// saved server.
    fn selection_changed(&mut self) {
        let online = matches!(self.selected, Some(Selected::Online(_)));
        for button in [Button::Select, Button::Edit, Button::Delete] {
            self.buttons.set_active(button as usize, online);
        }
    }

    pub fn draw(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        let (width, height) = self.size();
        // `extractBackground` with no level: the panorama's overlay, then the menu's background.
        frame.blit_texture_stretched(
            atlas,
            "title/background/panorama_overlay",
            (16, 128),
            (0, 0, 16, 128),
            (0, 0, width, height),
        );
        frame.tile_texture(atlas, "menu_background", (32, 32), (0, 0, width, height));
        self.title
            .draw(frame, atlas, font, &self.buttons.arena, false);
        let tooltip = self.draw_list(frame, atlas, font, mouse);
        self.buttons.draw(frame, atlas, font, mouse);
        if let Some(lines) = tooltip {
            frame.tooltip(atlas, font, &lines, mouse, (width, height));
        }
    }

    /// `extractWidgetRenderState` of the list; the tooltip a status icon asks for.
    fn draw_list(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        mouse: (i32, i32),
    ) -> Option<Vec<Vec<Run>>> {
        let place = self.place;
        let hovered = self.entry_at(mouse);
        place.draw_background(frame, atlas, self.scroll);
        let mark = frame.mark();
        let left = self.row_left();
        let mut tooltip = None;
        for index in 0..self.entries() {
            let y = self.entry_y(index);
            if y + ENTRY_HEIGHT < place.y || y > place.bottom() {
                continue;
            }
            let entry = if index < self.names.len() {
                Selected::Online(index)
            } else {
                Selected::LanHeader
            };
            if self.selected == Some(entry) {
                let outline = if self.list_focused { 0xFFFF_FFFF } else { GREY };
                let (x1, y1) = (left + ROW_WIDTH, y + ENTRY_HEIGHT);
                frame.fill(atlas, (left, y, x1, y1), outline);
                frame.fill(atlas, (left + 1, y + 1, x1 - 1, y1 - 1), 0xFF00_0000);
            }
            let content = (left + 2, y + 2);
            match entry {
                Selected::Online(index) => {
                    let asked = self.draw_server(
                        frame,
                        atlas,
                        font,
                        (index, content),
                        (mouse, hovered == Some(index)),
                    );
                    tooltip = tooltip.or(asked);
                }
                Selected::LanHeader => self.draw_lan_header(frame, atlas, font, content),
            }
        }
        frame.clip_from(mark, (0, place.y, place.width, place.bottom()));
        place.draw_edges(
            frame,
            atlas,
            (self.scroll, self.content_height()),
            left + ROW_WIDTH + SCROLLBAR_WIDTH + 2,
        );
        tooltip
    }

    /// `OnlineServerEntry.extractContent` of a server still being asked about: its name, the
    /// unknown icon, the turning pinging bars, and on the mouse the join and move arrows.
    fn draw_server(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        (index, (x, y)): (usize, (i32, i32)),
        (mouse, hovered): ((i32, i32), bool),
    ) -> Option<Vec<Vec<Run>>> {
        let name = [Run {
            text: self.names[index].clone(),
            style: Style::default(),
        }];
        frame.styled_text(atlas, font, &name, x + ICON + 3, y + 1, 0xFFFF_FFFF, true);
        frame.blit_texture(
            atlas,
            self.icons[index]
                .as_deref()
                .unwrap_or("textures/misc/unknown_server"),
            (ICON, ICON),
            (0, 0),
            (x, y, ICON, ICON),
        );
        let status_x = x + ROW_WIDTH - 4 - 10 - 5;
        let ping = self.pings[index].as_ref();
        let sprite = if let Some(view) = ping {
            view.icon.to_owned()
        } else {
            let mut bars = (frame.millis / 100 + index as u64 * 2) & 7;
            if bars > 4 {
                bars = 8 - bars;
            }
            format!("server_list/pinging_{}", bars + 1)
        };
        if let Some(view) = ping {
            for (line, text) in (0..).zip(&view.motd) {
                frame.styled_text(
                    atlas,
                    font,
                    text,
                    x + ICON + 3,
                    y + 12 + 9 * line,
                    GREY,
                    true,
                );
            }
        }
        frame.sprite(atlas, &sprite, status_x, y, 10, 8);
        let status = ping.map_or(&[][..], |view| view.status.as_slice());
        let status_width = font.width_of_runs(status);
        let text_x = status_x - status_width - 5;
        frame.styled_text(atlas, font, status, text_x, y + 1, GREY, true);
        let mut tooltip = None;
        if mouse.0 >= status_x && mouse.0 <= status_x + 10 && mouse.1 >= y && mouse.1 <= y + 8 {
            tooltip = Some(vec![ping.map_or_else(
                || self.pinging.clone(),
                |view| view.icon_tooltip.clone(),
            )]);
        } else if let Some(players) = ping.and_then(|view| view.players.as_ref())
            && mouse.0 >= text_x
            && mouse.0 <= text_x + status_width
            && mouse.1 >= y
            && mouse.1 <= y - 1 + 9
        {
            tooltip = Some(players.clone());
        }
        if hovered {
            frame.fill(atlas, (x, y, x + ICON, y + ICON), 0xA090_9090);
            let at = (mouse.0 - x, mouse.1 - y);
            let sprite = |name: &str, highlighted: bool| {
                if highlighted {
                    format!("server_list/{name}_highlighted")
                } else {
                    format!("server_list/{name}")
                }
            };
            frame.sprite(
                atlas,
                &sprite("join", over_right_half(at)),
                x,
                y,
                ICON,
                ICON,
            );
            if index > 0 {
                frame.sprite(
                    atlas,
                    &sprite("move_up", over_top_left(at)),
                    x,
                    y,
                    ICON,
                    ICON,
                );
            }
            if index + 1 < self.names.len() {
                let over = over_bottom_left(at);
                frame.sprite(atlas, &sprite("move_down", over), x, y, ICON, ICON);
            }
        }
        tooltip
    }

    /// `LANHeader`: "Scanning for games on your local network" and the beating dots below.
    fn draw_lan_header(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        (x, y): (i32, i32),
    ) {
        let width = font.width_of_runs(&self.scanning);
        let middle = x + (ROW_WIDTH - 4) / 2;
        let left = middle - width / 2;
        let centre = left + width / 2;
        frame.styled_text(
            atlas,
            font,
            &self.scanning,
            centre - width / 2,
            y + 2,
            0xFFFF_FFFF,
            true,
        );
        let dots = DOTS[(frame.millis / 300 % 4) as usize];
        frame.text_with_shadow(
            atlas,
            font,
            dots,
            centre - font.width(dots) / 2,
            y + 2 + 9 + 6 + 9 + 2 - 9 - 2,
            GREY,
        );
    }

    /// A press of the primary button at `mouse`; `double` for the second of a double click.
    pub fn clicked(&mut self, mouse: (i32, i32), double: bool) -> Option<ServersAction> {
        self.buttons.mouse_input();
        if let Some(button) = self.buttons.clicked(mouse) {
            return self.press(button);
        }
        let index = self.entry_at(mouse)?;
        self.list_focused = true;
        if index == self.names.len() {
            self.selected = Some(Selected::LanHeader);
            self.selection_changed();
            return None;
        }
        // `OnlineServerEntry.mouseClicked`, then the list selects the entry clicked.
        let at = (
            mouse.0 - self.row_left() - 2,
            mouse.1 - self.entry_y(index) - 2,
        );
        let action = if over_right_half(at) {
            Some(ServersAction::Join(index))
        } else if index > 0 && over_top_left(at) {
            Some(ServersAction::Swap(index, index - 1))
        } else if index + 1 < self.names.len() && over_bottom_left(at) {
            Some(ServersAction::Swap(index, index + 1))
        } else if double {
            Some(ServersAction::Join(index))
        } else {
            None
        };
        let selected = match action {
            Some(ServersAction::Swap(from, to)) => {
                self.names.swap(from, to);
                self.pings.swap(from, to);
                self.icons.swap(from, to);
                to
            }
            _ => index,
        };
        self.selected = Some(Selected::Online(selected));
        self.selection_changed();
        action
    }

    /// Enter: the selected server joined; Escape: back.
    pub fn key_pressed(&mut self, enter: bool) -> Option<ServersAction> {
        if !enter {
            return Some(ServersAction::Back);
        }
        match self.selected {
            Some(Selected::Online(index)) => Some(ServersAction::Join(index)),
            _ => self.buttons.pressed().and_then(|button| self.press(button)),
        }
    }

    fn press(&self, button: Button) -> Option<ServersAction> {
        let online = match self.selected {
            Some(Selected::Online(index)) => Some(index),
            _ => None,
        };
        match button {
            Button::Select => online.map(ServersAction::Join),
            Button::Direct => Some(ServersAction::Direct),
            Button::Add => Some(ServersAction::Add),
            Button::Edit => online.map(ServersAction::Edit),
            Button::Delete => online.map(ServersAction::Delete),
            Button::Refresh => Some(ServersAction::Refresh),
            Button::Back => Some(ServersAction::Back),
        }
    }

    /// `AbstractScrollArea.mouseScrolled` on the list: half an entry a notch.
    pub fn scrolled(&mut self, mouse: (f64, f64), notches: f64) {
        if self.place.contains(mouse) {
            let max = f64::from(self.place.max_scroll(self.content_height()));
            self.scroll = (self.scroll - notches * f64::from(ENTRY_HEIGHT / 2)).clamp(0.0, max);
        }
    }
}

/// `SelectableEntry.mouseOverRightHalf` of the icon, the mouse at `(x, y)` from its corner.
fn over_right_half((x, y): (i32, i32)) -> bool {
    (ICON / 2..ICON).contains(&x) && (0..ICON).contains(&y)
}

/// `mouseOverTopLeftQuarter`.
fn over_top_left((x, y): (i32, i32)) -> bool {
    (0..ICON / 2).contains(&x) && (0..ICON / 2).contains(&y)
}

/// `mouseOverBottomLeftQuarter`.
fn over_bottom_left((x, y): (i32, i32)) -> bool {
    (0..ICON / 2).contains(&x) && (ICON / 2..ICON).contains(&y)
}
