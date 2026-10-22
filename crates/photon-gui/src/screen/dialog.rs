//! The screen of a notice, confirmation or multi-action dialog (`SimpleDialogScreen`,
//! `MultiButtonDialogScreen`): the title with the warning button beside it, the body (and for a
//! button list its grid of buttons) in a scrolling box, and the dialog's own buttons in a row at
//! the bottom, over the menu background.

use crate::ItemSlots;
use crate::{FontRuns, FrameRuns};
use std::cell::Cell;

use super::dialog_body::{
    BAR, PADDING, Part, body_layout, centred, middle, outline, pack_into_columns,
};
mod draw;
mod focus;

use super::dialog_input::Control;

use photon_game::dialog::{ActionButton, Dialog};
use photon_game::text::{Component, Lang, Style};

use crate::layout::{Arena, Grid, Item, horizontal, place_in_frame};
use photon_render::gui::font::Font;

use crate::tooltip::{Holder, Pending};
use crate::widgets::navigation::Rectangle;
use crate::widgets::scroll_area::{Place, SCROLLBAR_WIDTH};
use crate::widgets::slider::InputType;
use crate::widgets::{Kind, Widget};
use photon_render::gui::{GuiAtlas, GuiFrame, ItemDrawer};

/// `HeaderAndFooterLayout`'s content margin.
/// `ButtonListDialogScreen.FOOTER_MARGIN`: the footer of a button list with no exit button.
const SHORT_FOOTER: i32 = 5;
pub(super) const CONTENT_MARGIN: i32 = 30;
/// `ScrollableLayout`'s space either side of the content: the bar and its gap.
pub(super) const RESERVE: i32 = 4 + SCROLLBAR_WIDTH;
pub(super) const SCROLL_RATE: f64 = 10.0;
const WARNING_SIZE: i32 = 20;

/// What the player did on the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Press {
    /// One of the dialog's own buttons, by its place in `main_actions`.
    Button(usize),
    /// The warning button: the screen about custom screens opens.
    Warning,
    /// The screen is closed (`Screen.onClose`).
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Warning,
    Part(usize),
    Input(usize),
    Button(usize),
}

/// `ScrollableLayout.Container`: where the box stands.
#[derive(Debug, Clone, Copy)]
pub(super) struct Container {
    pub(super) x: i32,
    pub(super) y: i32,
    pub(super) width: i32,
    pub(super) height: i32,
}

pub struct DialogScreen {
    arena: Arena,
    size: (i32, i32),
    in_world: bool,
    can_close: bool,
    title: Widget,
    warning: Widget,
    warning_tooltip: Holder,
    parts: Vec<Part>,
    controls: Vec<Control>,
    /// The dialog's buttons in `main_actions` order: the first `listed` stand in the body (and
    /// scroll with it), the rest in the footer.
    buttons: Vec<(Widget, Holder)>,
    listed: usize,
    container: Container,
    content_height: i32,
    scroll: f64,
    dragging: bool,
    focus: Option<Focus>,
    input: Cell<InputType>,
}

/// The header: the title and the warning button, put in the middle of the bar.
fn place_header(arena: &mut Arena, (title, warning): (&Widget, &Widget), width: i32) {
    let mut header = Grid::new();
    header.column_spacing = 10;
    header.add(Item::Leaf(title.rect), (0, 0), (1, 1), middle());
    header.add(Item::Leaf(warning.rect), (0, 1), (1, 1), middle());
    header.arrange(arena);
    let header_width = width.max(header.width);
    let header_height = BAR.max(header.height);
    place_in_frame(
        arena,
        Item::Grid(Box::new(header)),
        (0, 0, header_width, header_height),
        (0.5, 0.5),
    );
}

/// The controls of the dialog, in the body after its elements.
fn add_controls(
    arena: &mut Arena,
    (font, lang): (&Font, &Lang),
    body: &mut Grid,
    common: &photon_game::dialog::CommonData,
) -> Vec<Control> {
    // A kind that is not built yet is left out; the dialog still shows.
    let controls: Vec<Control> = common
        .inputs
        .iter()
        .filter_map(|input| Control::new(arena, (font, lang), input))
        .collect();
    for (row, control) in controls.iter().enumerate() {
        body.add(
            Item::Leaf(control.rect),
            (common.body.len() + row, 0),
            (1, 1),
            centred(),
        );
    }
    controls
}

/// The buttons that stand in the footer, in a row in the middle of it.
fn place_footer(arena: &mut Arena, buttons: &[(Widget, Holder)], size: (i32, i32), height: i32) {
    let leaves: Vec<usize> = buttons.iter().map(|(widget, _)| widget.rect).collect();
    let mut footer = horizontal(arena, &leaves, 8);
    footer.arrange(arena);
    let footer_width = size.0.max(footer.width);
    place_in_frame(
        arena,
        Item::Grid(Box::new(footer)),
        (0, size.1 - height, footer_width, height),
        (0.5, 0.5),
    );
}

/// `makeSureWarningButtonIsInBounds`.
fn keep_warning_in_bounds(arena: &mut Arena, warning: usize, (width, height): (i32, i32)) {
    let place = &mut arena.0[warning];
    if place.x < 0 || place.y < 0 || place.x > width - 20 || place.y > height - 20 {
        place.x = (width - 40).max(0);
        place.y = 5.min(height);
    }
}

/// `ScrollableLayout`: the box is as tall as the body, to what the bars leave, and is put at the
/// top of the content frame.
pub(super) fn container_for(
    (width, height): (i32, i32),
    footer_height: i32,
    (body_width, body_height): (i32, i32),
) -> Container {
    let box_width = body_width + 2 * RESERVE;
    let box_height = body_height.min(height - BAR - footer_height);
    let frame_width = width.max(box_width);
    Container {
        x: (0.5 * (frame_width - box_width) as f32) as i32,
        y: (BAR + CONTENT_MARGIN).min(height - footer_height - box_height),
        width: box_width,
        height: box_height,
    }
}

/// The texture a menu background is tiled with; over a game, the frame is blurred first.
pub(super) fn menu_background(
    frame: &mut GuiFrame,
    in_world: bool,
    blurriness: i32,
) -> &'static str {
    if !in_world {
        return "menu_background";
    }
    frame.blur_before_this_stratum(blurriness);
    "inworld_menu_background"
}

impl DialogScreen {
    /// `SimpleDialogScreen.init` and `ButtonListDialogScreen.init`: the screen of a dialog, in a
    /// screen of `size` units. A type without a screen yet gives none.
    pub fn new(
        font: &Font,
        lang: &Lang,
        size: (i32, i32),
        dialog: &Dialog,
        in_world: bool,
    ) -> Option<Self> {
        let common = dialog.common();
        if matches!(dialog, Dialog::Other(_) | Dialog::List { .. }) {
            return None;
        }
        let (width, height) = size;
        let mut arena = Arena::default();
        let runs = |component: &Component| lang.runs(component, &Style::default());
        let title_text = runs(&common.title);
        let title = Widget {
            rect: arena.add(font.width_of_runs(&title_text), 9),
            active: false,
            kind: Kind::Text { text: title_text },
        };
        let warning = Widget {
            rect: arena.add(WARNING_SIZE, WARNING_SIZE),
            active: true,
            kind: Kind::Icon {
                sprite: String::new(),
                size: (0, 0),
            },
        };
        let (mut body, parts) = body_layout(&mut arena, (font, lang), &common.body);
        let controls = add_controls(&mut arena, (font, lang), &mut body, common);
        let buttons: Vec<(Widget, Holder)> = dialog
            .main_actions()
            .into_iter()
            .map(|action| Self::button(&mut arena, lang, action))
            .collect();
        let (listed, columns) = dialog.button_list().unwrap_or((0, 1));
        if listed > 0 {
            let leaves: Vec<usize> = buttons[..listed]
                .iter()
                .map(|(widget, _)| widget.rect)
                .collect();
            let grid = pack_into_columns(&arena, &leaves, columns as usize);
            body.add(
                Item::Grid(Box::new(grid)),
                (common.body.len() + controls.len(), 0),
                (1, 1),
                centred(),
            );
        }
        // A button list with no exit button leaves a short footer.
        let footer_height = if listed > 0 && buttons.len() == listed {
            SHORT_FOOTER
        } else {
            BAR
        };
        body.arrange(&mut arena);
        let (body_width, body_height) = (body.width, body.height);
        let container = container_for((width, height), footer_height, (body_width, body_height));
        let mut body = Item::Grid(Box::new(body));
        body.set_x(&mut arena, container.x + RESERVE);
        body.set_y(&mut arena, container.y);
        place_header(&mut arena, (&title, &warning), width);
        place_footer(&mut arena, &buttons[listed..], size, footer_height);
        keep_warning_in_bounds(&mut arena, warning.rect, size);
        Some(Self {
            size,
            in_world,
            can_close: common.can_close_with_escape,
            title,
            warning,
            warning_tooltip: Holder::of(
                lang,
                &Component::translatable("menu.custom_screen_info.tooltip"),
            ),
            parts,
            controls,
            buttons,
            listed,
            container,
            content_height: body_height,
            scroll: 0.0,
            dragging: false,
            focus: None,
            input: Cell::new(InputType::Mouse),
            arena,
        })
    }

    /// `DialogControlSet.createActionButton`: a button of the width the dialog asks for, with its
    /// tooltip.
    fn button(arena: &mut Arena, lang: &Lang, action: &ActionButton) -> (Widget, Holder) {
        let widget = Widget {
            rect: arena.add(action.width, 20),
            active: true,
            kind: Kind::Button {
                label: lang.runs(&action.label, &Style::default()),
            },
        };
        let tooltip = action
            .tooltip
            .as_ref()
            .map_or_else(Holder::default, |text| Holder::of(lang, text));
        (widget, tooltip)
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    fn max_scroll(&self) -> i32 {
        (self.content_height - self.container.height).max(0)
    }

    fn place(&self) -> Place {
        Place {
            y: self.container.y,
            width: self.size.0,
            height: self.container.height,
            in_world: self.in_world,
        }
    }

    /// Where a part of the body is now, scrolled.
    fn part_rect(&self, part: &Part) -> Rectangle {
        let rect = self.arena.0[part.rect];
        Rectangle {
            x: rect.x,
            y: rect.y - self.scroll as i32,
            width: rect.width,
            height: rect.height,
        }
    }

    /// Where control `index` is now, scrolled.
    fn control_rect(&self, index: usize) -> Rectangle {
        let rect = self.arena.0[self.controls[index].rect];
        Rectangle {
            x: rect.x,
            y: rect.y - self.scroll as i32,
            width: rect.width,
            height: rect.height,
        }
    }

    fn in_container(&self, (x, y): (i32, i32)) -> bool {
        let c = self.container;
        x >= c.x && y >= c.y && x < c.x + c.width && y < c.y + c.height
    }

    /// Where button `index` is now: the ones in the body scroll with it.
    fn button_rect(&self, index: usize) -> Rectangle {
        let rect = self.arena.0[self.buttons[index].0.rect];
        let shift = if index < self.listed {
            self.scroll as i32
        } else {
            0
        };
        Rectangle {
            x: rect.x,
            y: rect.y - shift,
            width: rect.width,
            height: rect.height,
        }
    }

    /// Whether the mouse is on button `index`; the box cuts the ones in the body.
    fn button_hit(&self, index: usize, mouse: (i32, i32)) -> bool {
        let rect = self.button_rect(index);
        let (x, y) = mouse;
        x >= rect.x
            && x < rect.x + rect.width
            && y >= rect.y
            && y < rect.y + rect.height
            && (index >= self.listed || self.in_container(mouse))
    }
}
