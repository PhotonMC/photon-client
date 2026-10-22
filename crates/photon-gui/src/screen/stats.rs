//! The statistics screen (`StatsScreen`): a bar of three tabs (general, items, mobs) over the list
//! of the tab chosen, and Done under it. While the server has not answered, every tab shows
//! "Retrieving statistics..." with dots that go round.

mod general;
mod items;
mod list;
mod mobs;

use std::collections::HashMap;

use crate::layout::{Arena, Item, place_in_frame};
use crate::widgets::scroll_area::Place;
use crate::widgets::{Kind, Widget, draw_label};
use crate::{FontRuns, FrameRuns};
use photon_data::stat::StatKey;
use photon_game::text::{Component, Lang, Run, Style};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

use general::GeneralList;
use items::ItemsList;
use mobs::MobsList;

/// `MenuTabBar.HEIGHT`, the bar's height and where the content starts.
const TAB_BAR_HEIGHT: i32 = 24;
const TAB_BAR_MAX_WIDTH: i32 = 400;
const TAB_BAR_MARGIN: i32 = 14;
/// `HeaderAndFooterLayout.DEFAULT_HEADER_AND_FOOTER_HEIGHT`, the footer's.
const FOOTER_HEIGHT: i32 = 33;
/// `LoadingDotsWidget`: the padding under the dots' text and the layout's under the widget.
const LOADING_PADDING_BOTTOM: i32 = 30;
const TABS: usize = 3;

/// What the screen asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatsAction {
    /// Done: back to the screen it came from.
    Close,
}

/// `LoadingDotsText.FRAMES`, three tenths of a second each.
fn loading_dots(millis: u64) -> &'static str {
    const FRAMES: [&str; 4] = ["O o o", "o O o", "o o O", "o O o"];
    FRAMES[(millis / 300 % 4) as usize]
}

/// `Mth.roundToward(input, 2)` of a positive number: up to the next even one.
fn round_to_even(value: i32) -> i32 {
    (value + 1) / 2 * 2
}

pub struct StatsScreen {
    size: (i32, i32),
    in_world: bool,
    arena: Arena,
    done: Widget,
    tab_titles: [Vec<Run>; TABS],
    pending: Vec<Run>,
    selected: usize,
    /// The tab the bar has focused, after one was clicked.
    focused_tab: Option<usize>,
    /// Whether the screen's focus is on the tab bar.
    bar_focused: bool,
    /// The lists, once the server's statistics are in (`isLoading`).
    lists: Option<Lists>,
    /// What each tab's list has in it, which decides if the tab can be chosen.
    tab_active: [bool; TABS],
    /// `StatsCounter`: what the server last said.
    stats: HashMap<StatKey, i32>,
    /// `init` has not yet asked the server for the statistics.
    request_pending: bool,
}

struct Lists {
    general: GeneralList,
    items: ItemsList,
    mobs: MobsList,
}

fn runs(lang: &Lang, key: &str) -> Vec<Run> {
    lang.runs(&Component::translatable(key), &Style::default())
}

impl StatsScreen {
    /// `StatsScreen` and `init`: loading, with the first tab chosen. The statistics so far are
    /// the counter's; the request for them is the caller's to send.
    pub fn new(
        lang: &Lang,
        size: (i32, i32),
        in_world: bool,
        stats: &HashMap<StatKey, i32>,
    ) -> Self {
        let mut arena = Arena::default();
        let rect = arena.add(200, 20);
        let done = Widget {
            rect,
            active: true,
            kind: Kind::Button {
                label: runs(lang, "gui.done"),
            },
        };
        let mut screen = Self {
            size,
            in_world,
            arena,
            done,
            tab_titles: [
                runs(lang, "stat.generalButton"),
                runs(lang, "stat.itemsButton"),
                runs(lang, "stat.mobsButton"),
            ],
            pending: runs(lang, "multiplayer.downloadingStats"),
            selected: 0,
            focused_tab: None,
            bar_focused: false,
            lists: None,
            tab_active: [true, false, false],
            stats: stats.clone(),
            request_pending: true,
        };
        screen.layout();
        screen
    }

    /// Whether the request for the statistics is yet to be sent (`init`'s): once.
    pub fn take_request(&mut self) -> bool {
        std::mem::take(&mut self.request_pending)
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    /// `onStatsUpdated`: the first time, the lists are built from what the counter holds.
    pub fn on_stats_updated(&mut self, lang: &Lang, stats: &HashMap<StatKey, i32>) {
        self.stats = stats.clone();
        if self.lists.is_some() {
            return;
        }
        let general = GeneralList::new(lang);
        let items = ItemsList::new(lang, stats);
        let mobs = MobsList::new(lang, stats);
        // `setTabActiveStateAndTooltip`: a tab with nothing in its list cannot be chosen.
        self.tab_active = [true, !items.is_empty(), !mobs.is_empty()];
        self.lists = Some(Lists {
            general,
            items,
            mobs,
        });
        self.selected = 0;
        // `setFocused(tabNavigationBar)`: the bar has the focus, no tab of it.
        self.bar_focused = true;
        self.focused_tab = None;
        self.layout();
    }

    /// The counter changed again: what the lists show follows it.
    pub fn set_stats(&mut self, stats: &HashMap<StatKey, i32>) {
        self.stats = stats.clone();
    }

    /// `Screen.init(width, height)` on a screen that was initialized: `repositionElements`, which
    /// lays the same lists out again.
    pub fn resize(&mut self, size: (i32, i32)) {
        self.size = size;
        self.layout();
    }

    /// `repositionElements`: the footer's Done, and the list to the room between the bar and it.
    fn layout(&mut self) {
        let (width, height) = self.size;
        place_in_frame(
            &mut self.arena,
            Item::Leaf(self.done.rect),
            (0, height - FOOTER_HEIGHT, width, FOOTER_HEIGHT),
            (0.5, 0.5),
        );
        let place = self.list_place();
        if let Some(lists) = &mut self.lists {
            lists.general.place(place);
            lists.items.place(place);
            lists.mobs.place(place);
        }
    }

    /// `StatisticsTab.doLayout`: from under the bar to over the footer.
    fn list_place(&self) -> Place {
        Place {
            y: TAB_BAR_HEIGHT,
            width: self.size.0,
            height: self.size.1 - TAB_BAR_HEIGHT - FOOTER_HEIGHT,
            in_world: self.in_world,
        }
    }

    /// `MenuTabBar.arrangeElements`: the width of a tab and where the first one starts.
    fn tab_geometry(&self) -> (i32, i32) {
        let tabs_width = TAB_BAR_MAX_WIDTH.min(self.size.0) - 2 * TAB_BAR_MARGIN;
        let tab_width = round_to_even((tabs_width as f32 / TABS as f32) as i32);
        let x = round_to_even((self.size.0 - tabs_width) / 2);
        (x, tab_width)
    }

    /// The background, the tab bar, the content of the tab and Done over the footer's line.
    pub fn draw(&self, draw: &mut super::ScreenDraw<'_, '_>, mouse: (i32, i32), millis: u64) {
        let (atlas, font) = (draw.atlas, draw.font);
        let frame = &mut *draw.frame;
        let (width, height) = self.size;
        // `extractMenuBackground`: the tab header's picture, then the menu's from under it.
        frame.tile_texture(
            atlas,
            "tab_header_background",
            (16, 16),
            (0, 0, width, TAB_BAR_HEIGHT),
        );
        let background = if self.in_world {
            "inworld_menu_background"
        } else {
            "menu_background"
        };
        frame.tile_texture(
            atlas,
            background,
            (32, 32),
            (0, TAB_BAR_HEIGHT, width, height),
        );
        self.draw_tab_bar(frame, (atlas, font), mouse);
        match &self.lists {
            None => self.draw_loading(frame, (atlas, font), millis),
            Some(lists) => match self.selected {
                0 => lists.general.draw(frame, (atlas, font), &self.stats),
                1 => lists
                    .items
                    .draw(frame, (atlas, font), (&self.stats, &mut *draw.item), mouse),
                2 => lists.mobs.draw(frame, (atlas, font)),
                _ => {}
            },
        }
        frame.blit_texture(
            atlas,
            "footer_separator",
            (32, 2),
            (0, 0),
            0,
            height - FOOTER_HEIGHT,
            width,
            2,
        );
        let highlighted = self.done.contains(&self.arena, mouse);
        self.done.draw(frame, atlas, font, &self.arena, highlighted);
        // Widget tooltips are drawn last (`GuiGraphicsExtractor.setTooltipForNextFrame`).
        if self.selected == 1
            && let Some(lists) = &self.lists
            && let Some((stack, at)) = lists.items.hovered_stack(mouse)
        {
            let layout = super::container::ContainerLayout::new(width, height, (0, 0));
            let lines = (draw.tooltip)(&stack, false);
            let at = (f64::from(at.0), f64::from(at.1));
            super::bundle_tooltip::stack_tooltip(&layout, draw, &stack, &lines, at);
        }
    }

    /// `MenuTabBar.extractWidgetRenderState`: the header's line either side of the tabs, then each
    /// tab.
    fn draw_tab_bar(
        &self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        mouse: (i32, i32),
    ) {
        let (first, tab_width) = self.tab_geometry();
        let line_y = TAB_BAR_HEIGHT - 2;
        frame.blit_texture(
            atlas,
            "header_separator",
            (32, 2),
            (0, 0),
            0,
            line_y,
            first,
            2,
        );
        let after_last = first + tab_width * TABS as i32;
        frame.blit_texture(
            atlas,
            "header_separator",
            (32, 2),
            (0, 0),
            after_last,
            line_y,
            self.size.0,
            2,
        );
        for tab in 0..TABS {
            let x = first + tab as i32 * tab_width;
            let over =
                mouse.0 >= x && mouse.0 < x + tab_width && mouse.1 >= 0 && mouse.1 < TAB_BAR_HEIGHT;
            let hovered = over || self.focused_tab == Some(tab);
            self.draw_tab(frame, (atlas, font), tab, (x, tab_width), hovered);
        }
    }

    /// `MenuTabButton.extractWidgetRenderState`.
    fn draw_tab(
        &self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        tab: usize,
        (x, width): (i32, i32),
        hovered: bool,
    ) {
        let selected = tab == self.selected;
        let sprite = match (selected, hovered) {
            (true, false) => "widget/tab_selected",
            (true, true) => "widget/tab_selected_highlighted",
            (false, false) => "widget/tab",
            (false, true) => "widget/tab_highlighted",
        };
        frame.sprite(atlas, sprite, x, 0, width, TAB_BAR_HEIGHT);
        let active = self.tab_active[tab];
        let title = &self.tab_titles[tab];
        if selected {
            frame.tile_texture(
                atlas,
                "menu_background",
                (32, 32),
                (x + 2, 2, width - 4, TAB_BAR_HEIGHT - 2),
            );
            let colour = if active { 0xFFFF_FFFF } else { 0xFFA0_A0A0 };
            let line = font.width_of_runs(title).min(width - 4);
            let left = x + (width - line) / 2;
            frame.fill(
                atlas,
                (left, TAB_BAR_HEIGHT - 2, left + line, TAB_BAR_HEIGHT - 1),
                colour,
            );
        }
        let top = if selected { 0 } else { 3 };
        draw_label(
            frame,
            atlas,
            font,
            (title, active),
            (x + 1, x + width - 1),
            (top, TAB_BAR_HEIGHT),
        );
    }

    /// `LoadingTab`: the text and the dots under it, centred in the room under the bar.
    fn draw_loading(&self, frame: &mut GuiFrame, (atlas, font): (&GuiAtlas, &Font), millis: u64) {
        let area_height = self.size.1 - TAB_BAR_HEIGHT - FOOTER_HEIGHT;
        let text_width = font.width_of_runs(&self.pending);
        // The widget is as tall as the area, in a layout with 30 more under it, centred on the area.
        let top = TAB_BAR_HEIGHT + (area_height - (area_height + LOADING_PADDING_BOTTOM)) / 2;
        let left = (self.size.0 - text_width) / 2;
        let centre = left + text_width / 2;
        frame.styled_text(
            atlas,
            font,
            &self.pending,
            centre - text_width / 2,
            top + 2,
            0xFFFF_FFFF,
            true,
        );
        let dots = loading_dots(millis);
        let bottom = top + area_height;
        frame.text_with_shadow(
            atlas,
            font,
            dots,
            centre - font.width(dots) / 2,
            bottom - 9 - 2,
            0xFF80_8080,
        );
    }

    /// The tab the pointer is over, active or not.
    fn tab_at(&self, mouse: (i32, i32)) -> Option<usize> {
        let (first, tab_width) = self.tab_geometry();
        let in_bar = mouse.1 >= 0
            && mouse.1 < TAB_BAR_HEIGHT
            && mouse.0 >= first
            && mouse.0 < first + TABS as i32 * tab_width;
        in_bar.then(|| ((mouse.0 - first) / tab_width) as usize)
    }

    /// A press of the primary button at `mouse`: of the children (Done, the tab bar, the list) the
    /// first one the pointer is over gets it, and the focus goes to it if it took it. An inactive
    /// tab takes nothing.
    pub fn clicked(&mut self, mouse: (i32, i32)) -> Option<StatsAction> {
        if self.done.contains(&self.arena, mouse) {
            crate::ui_sound::play_down_sound();
            return Some(StatsAction::Close);
        }
        if let Some(tab) = self.tab_at(mouse) {
            if self.tab_active[tab] {
                self.press_tab(tab);
            }
        } else if self.press_list(mouse) {
            // The list has the focus now, and the bar lets go of its tab.
            self.bar_focused = false;
            self.focused_tab = None;
        }
        None
    }

    /// A tab chosen: it is focused in the bar, and the screen then gives the bar its focus, which
    /// lets go of the tab if the bar had not had it.
    fn press_tab(&mut self, tab: usize) {
        self.selected = tab;
        self.focused_tab = Some(tab);
        if !self.bar_focused {
            self.bar_focused = true;
            self.focused_tab = None;
            self.unfocus_lists();
        }
    }

    /// A press in the list of the tab chosen: whether it took it.
    fn press_list(&mut self, mouse: (i32, i32)) -> bool {
        let place = self.list_place();
        let inside = place.contains((f64::from(mouse.0), f64::from(mouse.1)));
        match &mut self.lists {
            Some(lists) if inside => match self.selected {
                0 => lists.general.clicked(mouse),
                1 => lists.items.clicked(mouse, &self.stats),
                _ => lists.mobs.clicked(mouse),
            },
            _ => false,
        }
    }

    /// The tab bar took the focus: no list has it.
    fn unfocus_lists(&mut self) {
        if let Some(lists) = &mut self.lists {
            lists.general.unfocus();
            lists.items.unfocus();
            lists.mobs.unfocus();
        }
    }

    /// A turn of the wheel over the screen: the list scrolls.
    pub fn scrolled(&mut self, mouse: (i32, i32), delta: f64) {
        let place = self.list_place();
        let selected = self.selected;
        if let Some(lists) = &mut self.lists
            && place.contains((f64::from(mouse.0), f64::from(mouse.1)))
        {
            match selected {
                0 => lists.general.scroll_by(delta),
                1 => lists.items.scroll_by(delta),
                2 => lists.mobs.scroll_by(delta),
                _ => {}
            }
        }
    }
}
