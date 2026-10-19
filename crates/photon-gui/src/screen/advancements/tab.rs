//! One tab of the screen (`AdvancementTab`, `AdvancementTabType`): the tab itself on the window's
//! edge, and the tree it shows.

use std::collections::HashMap;

use super::widget::AdvancementWidget;
use photon_core::ids::ItemId;
use photon_game::advancements::{ClientAdvancements, DisplayInfo};
use photon_game::text::{Lang, Run, Style};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame, ItemDrawer};

/// Where a tab sits on the window: above, below, left or right of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TabType {
    Above,
    Below,
    Left,
    Right,
}

impl TabType {
    const ALL: [Self; 4] = [Self::Above, Self::Below, Self::Left, Self::Right];

    fn width(self) -> i32 {
        match self {
            Self::Above | Self::Below => 28,
            Self::Left | Self::Right => 32,
        }
    }

    fn height(self) -> i32 {
        match self {
            Self::Above | Self::Below => 32,
            Self::Left | Self::Right => 28,
        }
    }

    /// How many tabs go there.
    fn max(self) -> usize {
        match self {
            Self::Above | Self::Below => 8,
            Self::Left | Self::Right => 5,
        }
    }

    fn x(self, index: usize) -> i32 {
        match self {
            Self::Above | Self::Below => (self.width() + 4) * index as i32,
            Self::Left => -self.width() + 4,
            Self::Right => 248,
        }
    }

    fn y(self, index: usize) -> i32 {
        match self {
            Self::Above => -self.height() + 4,
            Self::Below => 136,
            Self::Left | Self::Right => self.height() * index as i32,
        }
    }

    /// The sprite of the tab at `index`: the first, the last or one between.
    fn sprite(self, index: usize, selected: bool) -> String {
        let (side, edges) = match self {
            Self::Above => ("above", ["left", "middle", "right"]),
            Self::Below => ("below", ["left", "middle", "right"]),
            Self::Left => ("left", ["top", "middle", "bottom"]),
            Self::Right => ("right", ["top", "middle", "bottom"]),
        };
        let edge = if index == 0 {
            edges[0]
        } else if index == self.max() - 1 {
            edges[2]
        } else {
            edges[1]
        };
        let suffix = if selected { "_selected" } else { "" };
        format!("advancements/tab_{side}_{edge}{suffix}")
    }

    /// Where the icon goes on the tab at `index`, from the window's corner.
    fn icon(self, index: usize) -> (i32, i32) {
        let (dx, dy) = match self {
            Self::Above => (6, 9),
            Self::Below => (6, 6),
            Self::Left => (10, 5),
            Self::Right => (6, 5),
        };
        (self.x(index) + dx, self.y(index) + dy)
    }
}

pub(super) struct AdvancementTab {
    pub(super) root: String,
    kind: TabType,
    index: usize,
    pub(super) title: Vec<Run>,
    icon: ItemId,
    /// The root's background texture, as a texture name under `textures/gui` (`None` draws none).
    pub(super) background: Option<String>,
    pub(super) widgets: Vec<AdvancementWidget>,
    by_id: HashMap<String, usize>,
    scroll: (f64, f64),
    min: (i32, i32),
    max: (i32, i32),
    pub(super) fade: f32,
    centered: bool,
    hovered: Option<usize>,
}

impl AdvancementTab {
    /// `AdvancementTab.create`: the tab for the `index`th root, if it has a display and there is
    /// room for it.
    pub(super) fn create(
        index: usize,
        root: &str,
        advancements: &ClientAdvancements,
        (font, lang): (&Font, &Lang),
    ) -> Option<Self> {
        let advancement = advancements.get(root)?;
        let display = advancement.display.as_ref()?;
        let mut left = index;
        let kind = TabType::ALL.into_iter().find(|kind| {
            if left < kind.max() {
                true
            } else {
                left -= kind.max();
                false
            }
        })?;
        let mut tab = Self {
            root: root.to_owned(),
            kind,
            index,
            title: lang.runs(&display.title, &Style::default()),
            icon: display.icon.0,
            background: display.background.as_deref().map(texture_name),
            widgets: Vec::new(),
            by_id: HashMap::new(),
            scroll: (0.0, 0.0),
            min: (i32::MAX, i32::MAX),
            max: (i32::MIN, i32::MIN),
            fade: 0.0,
            centered: false,
            hovered: None,
        };
        tab.add_advancement(advancement.id.as_str(), display, advancements, (font, lang));
        Some(tab)
    }

    /// `addAdvancement` for one with a display (and `addWidget`): the widget, the extent of the tree
    /// and every widget's parent.
    pub(super) fn add_advancement(
        &mut self,
        id: &str,
        display: &DisplayInfo,
        advancements: &ClientAdvancements,
        fonts: (&Font, &Lang),
    ) {
        let requirements = advancements.get(id).map_or(0, |a| a.requirements.len());
        let widget = AdvancementWidget::new(id, display, requirements, fonts);
        let (x0, y0) = (widget.x, widget.y);
        if let Some(index) = self.by_id.get(id) {
            self.widgets[*index] = widget;
        } else {
            self.widgets.push(widget);
            self.by_id.insert(id.to_owned(), self.widgets.len() - 1);
        }
        self.min = (self.min.0.min(x0), self.min.1.min(y0));
        self.max = (self.max.0.max(x0 + 28), self.max.1.max(y0 + 27));
        for widget in 0..self.widgets.len() {
            self.attach_to_parent(widget, advancements);
        }
    }

    /// `attachToParent`: the nearest ancestor with a display that has a widget here.
    fn attach_to_parent(&mut self, widget: usize, advancements: &ClientAdvancements) {
        if self.widgets[widget].parent.is_some() {
            return;
        }
        let id = self.widgets[widget].id.clone();
        let Some(mut parent) = advancements.parent_of(&id) else {
            return;
        };
        // `getFirstVisibleParent`: past the ancestors that show nothing.
        let found = loop {
            if advancements
                .get(parent)
                .is_some_and(|a| a.display.is_some())
            {
                break self.by_id.get(parent).copied();
            }
            match advancements.parent_of(parent) {
                Some(next) => parent = next,
                None => break None,
            }
        };
        if let Some(found) = found {
            self.widgets[widget].parent = Some(found);
            self.widgets[found].children.push(widget);
        }
    }

    pub(super) fn set_progress(
        &mut self,
        id: &str,
        progress: Option<&photon_game::advancements::AdvancementProgress>,
    ) {
        if let Some(index) = self.by_id.get(id) {
            self.widgets[*index].progress = progress.cloned();
        }
    }

    /// `tick`: the widget under the pointer (relative to the tree's window), and the fade that
    /// follows it.
    pub(super) fn tick(&mut self, (mx, my): (i32, i32)) {
        let mut hovering = false;
        if mx > 0 && mx < 234 && my > 0 && my < 113 {
            let scroll = self.scroll_ints();
            if let Some(index) = self
                .widgets
                .iter()
                .position(|w| w.is_mouse_over(scroll, (mx, my)))
            {
                hovering = true;
                self.hovered = Some(index);
            }
        }
        if hovering {
            self.fade = (self.fade + 0.06).clamp(0.0, 0.3);
        } else {
            self.fade = (self.fade - 0.12).clamp(0.0, 1.0);
            self.hovered = None;
        }
    }

    pub(super) fn can_scroll_horizontally(&self) -> bool {
        self.max.0 - self.min.0 > 234
    }

    pub(super) fn can_scroll_vertically(&self) -> bool {
        self.max.1 - self.min.1 > 113
    }

    /// `scroll`.
    pub(super) fn scroll(&mut self, (x, y): (f64, f64)) {
        if self.can_scroll_horizontally() {
            self.scroll.0 = (self.scroll.0 + x).clamp(-f64::from(self.max.0 - 234), 0.0);
        }
        if self.can_scroll_vertically() {
            self.scroll.1 = (self.scroll.1 + y).clamp(-f64::from(self.max.1 - 113), 0.0);
        }
    }

    fn scroll_ints(&self) -> (i32, i32) {
        (self.scroll.0.floor() as i32, self.scroll.1.floor() as i32)
    }

    /// `isMouseOver` of the tab on the window at `window`.
    pub(super) fn is_mouse_over(&self, window: (i32, i32), (mx, my): (i32, i32)) -> bool {
        let x = window.0 + self.kind.x(self.index);
        let y = window.1 + self.kind.y(self.index);
        mx > x && mx < x + self.kind.width() && my > y && my < y + self.kind.height()
    }

    /// `extractTab`: the tab's sprite on the window.
    pub(super) fn draw_tab(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        window: (i32, i32),
        selected: bool,
    ) {
        let x = window.0 + self.kind.x(self.index);
        let y = window.1 + self.kind.y(self.index);
        frame.sprite(
            atlas,
            &self.kind.sprite(self.index, selected),
            x,
            y,
            self.kind.width(),
            self.kind.height(),
        );
    }

    /// `extractIcon`.
    pub(super) fn draw_icon(
        &self,
        frame: &mut GuiFrame,
        item: &mut ItemDrawer<'_>,
        window: (i32, i32),
    ) {
        let (dx, dy) = self.kind.icon(self.index);
        let stack = photon_world::ClientItemStackState::new(1, self.icon);
        frame.item(item, &stack, window.0 + dx, window.1 + dy);
    }

    /// `extractContents`: the tiled background, the lines between the advancements and the
    /// advancements, in the 234 by 113 window at `origin`. The tree centres itself the first time.
    pub(super) fn draw_contents(
        &mut self,
        frame: &mut GuiFrame,
        (atlas, item): (&GuiAtlas, &mut ItemDrawer<'_>),
        (origin, background, screen): ((i32, i32), Option<&str>, (i32, i32)),
    ) {
        if !self.centered {
            self.scroll.0 = f64::from(117 - (self.max.0 + self.min.0) / 2);
            self.scroll.1 = f64::from(56 - (self.max.1 + self.min.1) / 2);
            self.centered = true;
        }
        let mark = frame.mark();
        let (scroll_x, scroll_y) = self.scroll_ints();
        let at = (origin.0 + scroll_x, origin.1 + scroll_y);
        if let Some(name) = background {
            let (left, top) = (scroll_x % 16, scroll_y % 16);
            for x in -1..=15 {
                for y in -1..=8 {
                    frame.blit_texture(
                        atlas,
                        name,
                        (16, 16),
                        (0, 0),
                        origin.0 + left + 16 * x,
                        origin.1 + top + 16 * y,
                        16,
                        16,
                    );
                }
            }
        }
        for background in [true, false] {
            self.draw_connections(frame, atlas, 0, (at, background));
        }
        self.draw_widgets(frame, (atlas, item), 0, at);
        // What is off the screen is dropped as well: the scissor is cut to it.
        let (left, top) = (origin.0.max(0), origin.1.max(0));
        let (right, bottom) = (
            (origin.0 + 234).min(screen.0),
            (origin.1 + 113).min(screen.1),
        );
        frame.clip_from(mark, (left, top, right, bottom));
    }

    /// `extractConnectivity` down the tree from a widget.
    fn draw_connections(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        widget: usize,
        (at, background): ((i32, i32), bool),
    ) {
        let this = &self.widgets[widget];
        if let Some(parent) = this.parent {
            let parent = &self.widgets[parent];
            this.draw_connection(frame, atlas, (parent.x, parent.y), (at, background));
        }
        for child in &this.children {
            self.draw_connections(frame, atlas, *child, (at, background));
        }
    }

    /// `extractRenderState` down the tree from a widget.
    fn draw_widgets(
        &self,
        frame: &mut GuiFrame,
        (atlas, item): (&GuiAtlas, &mut ItemDrawer<'_>),
        widget: usize,
        at: (i32, i32),
    ) {
        let this = &self.widgets[widget];
        this.draw_frame(frame, atlas, at);
        if let Some((x, y)) = this.icon_place(at) {
            let stack = photon_world::ClientItemStackState::new(1, this.display.icon.0);
            frame.item(item, &stack, x, y);
        }
        for child in &this.children {
            self.draw_widgets(frame, (atlas, item), *child, at);
        }
    }

    /// `extractTooltips`: the dark veil over the tree that follows the fade, and the box of the
    /// widget under the pointer.
    pub(super) fn draw_tooltips(
        &self,
        frame: &mut GuiFrame,
        (atlas, font, lang): (&GuiAtlas, &Font, &Lang),
        (origin, screen_width): ((i32, i32), i32),
    ) {
        let alpha = (self.fade * 255.0).floor() as u32;
        frame.fill(
            atlas,
            (origin.0, origin.1, origin.0 + 234, origin.1 + 113),
            alpha << 24,
        );
        if let Some(index) = self.hovered {
            self.widgets[index].draw_hover(
                frame,
                (atlas, font, lang),
                (self.scroll_ints(), origin, screen_width),
            );
        }
    }
}

/// The atlas name of a background such as `minecraft:gui/advancements/backgrounds/stone` (the
/// texture's folder is `textures/`, and the atlas packs the ones under `textures/gui`).
fn texture_name(background: &str) -> String {
    let path = background.strip_prefix("minecraft:").unwrap_or(background);
    path.strip_prefix("gui/").unwrap_or(path).to_owned()
}
