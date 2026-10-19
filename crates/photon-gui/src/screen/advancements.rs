//! The advancements screen (`AdvancementsScreen`): the window with its tabs, the tree of the tab
//! chosen that can be dragged, and Done.

mod tab;
mod widget;

use crate::layout::{Arena, Item, place_in_frame};
use crate::widgets::{Kind, Widget};
use crate::{FontRuns, FrameRuns};
use photon_game::advancements::{ClientAdvancements, Event};
use photon_game::text::{Component, Lang, Run, Style};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame, ItemDrawer};

use tab::AdvancementTab;

const WINDOW_WIDTH: i32 = 252;
const WINDOW_HEIGHT: i32 = 140;
/// The footer's and header's height (`HeaderAndFooterLayout`).
const BAR_HEIGHT: i32 = 33;

/// What the screen asks for.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AdvancementsOut {
    /// `ServerboundSeenAdvancementsPacket.openedTab`: the tab that was shown.
    pub opened_tab: Option<String>,
    /// Done: back to the screen it came from.
    pub close: bool,
}

pub struct AdvancementsScreen {
    size: (i32, i32),
    in_world: bool,
    arena: Arena,
    title: Widget,
    done: Widget,
    title_runs: Vec<Run>,
    empty_label: Vec<Run>,
    sad_label: Vec<Run>,
    left: i32,
    top: i32,
    tabs: Vec<AdvancementTab>,
    selected: Option<usize>,
    scrolling: bool,
}

fn runs(lang: &Lang, key: &str) -> Vec<Run> {
    lang.runs(&Component::translatable(key), &Style::default())
}

impl AdvancementsScreen {
    /// `AdvancementsScreen` and `init`: the screen listens to the advancements, and a tab is shown
    /// (the first if none was). What the server is to be told comes back.
    pub fn new(
        advancements: &mut ClientAdvancements,
        (font, lang): (&Font, &Lang),
        size: (i32, i32),
        in_world: bool,
    ) -> (Self, AdvancementsOut) {
        let mut arena = Arena::default();
        let title_runs = runs(lang, "gui.advancements");
        let title_rect = arena.add(font.width_of_runs(&title_runs), 9);
        let done_rect = arena.add(200, 20);
        let mut screen = Self {
            size,
            in_world,
            arena,
            title: Widget {
                rect: title_rect,
                active: true,
                kind: Kind::Text {
                    text: title_runs.clone(),
                },
            },
            done: Widget {
                rect: done_rect,
                active: true,
                kind: Kind::Button {
                    label: runs(lang, "gui.done"),
                },
            },
            title_runs,
            empty_label: runs(lang, "advancements.empty"),
            sad_label: runs(lang, "advancements.sad_label"),
            left: 0,
            top: 0,
            tabs: Vec::new(),
            selected: None,
            scrolling: false,
        };
        advancements.listen();
        screen.sync(advancements, (font, lang));
        let tab = screen
            .selected
            .or_else(|| (!screen.tabs.is_empty()).then_some(0))
            .map(|index| screen.tabs[index].root.clone());
        let out = AdvancementsOut {
            opened_tab: tab.clone(),
            close: false,
        };
        advancements.select_tab(tab.as_deref());
        screen.sync(advancements, (font, lang));
        screen.layout();
        (screen, out)
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    /// `repositionElements`: the window in the middle, the title over it and Done under it.
    fn layout(&mut self) {
        let (width, height) = self.size;
        self.left = (width - WINDOW_WIDTH) / 2;
        self.top = (height - WINDOW_HEIGHT) / 2;
        place_in_frame(
            &mut self.arena,
            Item::Leaf(self.title.rect),
            (0, 0, width, BAR_HEIGHT),
            (0.5, 0.5),
        );
        place_in_frame(
            &mut self.arena,
            Item::Leaf(self.done.rect),
            (0, height - BAR_HEIGHT, width, BAR_HEIGHT),
            (0.5, 0.5),
        );
    }

    /// `Screen.init` again at a new size: only the elements are put where they go.
    pub fn resize(&mut self, size: (i32, i32)) {
        self.size = size;
        self.layout();
    }

    /// The listener's callbacks: what the advancements told since last asked.
    pub fn sync(&mut self, advancements: &mut ClientAdvancements, fonts: (&Font, &Lang)) {
        for event in advancements.take_events() {
            match event {
                Event::AddRoot(id) => {
                    if let Some(tab) =
                        AdvancementTab::create(self.tabs.len(), &id, advancements, fonts)
                    {
                        self.tabs.push(tab);
                    }
                }
                Event::AddTask(id) => self.add_task(&id, advancements, fonts),
                Event::Progress(id) => {
                    if let Some(tab) = self.tab_of(&id, advancements) {
                        let progress = advancements.progress(&id);
                        self.tabs[tab].set_progress(&id, progress);
                    }
                }
                Event::SelectedTab(root) => {
                    self.selected =
                        root.and_then(|root| self.tabs.iter().position(|tab| tab.root == root));
                }
                Event::Cleared => {
                    self.tabs.clear();
                    self.selected = None;
                }
            }
        }
    }

    /// `getTab(node)`: the tab of the root above an advancement.
    fn tab_of(&self, id: &str, advancements: &ClientAdvancements) -> Option<usize> {
        let root = advancements.root_of(id)?;
        self.tabs.iter().position(|tab| tab.root == root.id)
    }

    /// `onAddAdvancementTask`: the advancement goes in its tab's tree if it shows anything.
    fn add_task(&mut self, id: &str, advancements: &ClientAdvancements, fonts: (&Font, &Lang)) {
        let Some(tab) = self.tab_of(id, advancements) else {
            return;
        };
        if let Some(display) = advancements
            .get(id)
            .and_then(|advancement| advancement.display.as_ref())
        {
            self.tabs[tab].add_advancement(id, display, advancements, fonts);
        }
    }

    /// `tick`: the tab shown follows the pointer.
    pub fn tick(&mut self, mouse: (i32, i32)) {
        if let Some(selected) = self.selected {
            self.tabs[selected].tick((mouse.0 - self.left - 9, mouse.1 - self.top - 18));
        }
    }

    /// A press of the primary button: the tab it is on is chosen (and the server told), and Done
    /// is pressed if it was on it.
    pub fn clicked(
        &mut self,
        advancements: &mut ClientAdvancements,
        fonts: (&Font, &Lang),
        mouse: (i32, i32),
    ) -> AdvancementsOut {
        let mut out = AdvancementsOut::default();
        let window = (self.left, self.top);
        if let Some(tab) = self
            .tabs
            .iter()
            .find(|tab| tab.is_mouse_over(window, mouse))
        {
            let root = tab.root.clone();
            advancements.select_tab(Some(&root));
            out.opened_tab = Some(root);
            self.sync(advancements, fonts);
        }
        out.close = self.done.contains(&self.arena, mouse);
        out
    }

    /// A drag with the primary button: the tree moves (the first event only starts it).
    pub fn dragged(&mut self, delta: (f64, f64)) {
        if !self.scrolling {
            self.scrolling = true;
        } else if let Some(selected) = self.selected {
            self.tabs[selected].scroll(delta);
        }
    }

    /// The button let go.
    pub fn released(&mut self) {
        self.scrolling = false;
    }

    /// The wheel: the tree moves by sixteen a notch.
    pub fn scrolled(&mut self, (x, y): (f64, f64)) -> bool {
        match self.selected {
            Some(selected) => {
                self.tabs[selected].scroll((x * 16.0, y * 16.0));
                true
            }
            None => false,
        }
    }

    /// `removed`: the screen stops listening.
    pub fn close(&mut self, advancements: &mut ClientAdvancements) {
        advancements.unlisten();
    }

    /// The window over the menu background, the tree inside it, the tabs and the title, then the
    /// boxes of what is hovered.
    pub fn draw(
        &mut self,
        frame: &mut GuiFrame,
        (atlas, font, lang): (&GuiAtlas, &Font, &Lang),
        mouse: (i32, i32),
        item: &mut ItemDrawer<'_>,
    ) {
        let (width, height) = self.size;
        let background = if self.in_world {
            "inworld_menu_background"
        } else {
            "menu_background"
        };
        frame.tile_texture(atlas, background, (32, 32), (0, 0, width, height));
        let highlighted = self.done.contains(&self.arena, mouse);
        self.title.draw(frame, atlas, font, &self.arena, false);
        self.done.draw(frame, atlas, font, &self.arena, highlighted);
        self.draw_inside(frame, (atlas, font), item);
        self.draw_window(frame, (atlas, font), item);
        self.draw_tooltips(frame, (atlas, font, lang), mouse);
    }

    /// `extractInside`: the tree of the tab shown, or the empty state.
    fn draw_inside(
        &mut self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        item: &mut ItemDrawer<'_>,
    ) {
        let origin = (self.left + 9, self.top + 18);
        let Some(selected) = self.selected else {
            frame.fill(
                atlas,
                (origin.0, origin.1, origin.0 + 234, origin.1 + 113),
                0xFF00_0000,
            );
            let middle = origin.0 + 117;
            for (label, y) in [
                (&self.empty_label, origin.1 + 56 - 9 / 2),
                (&self.sad_label, origin.1 + 113 - 9),
            ] {
                let x = middle - font.width_of_runs(label) / 2;
                frame.styled_text(atlas, font, label, x, y, 0xFFFF_FFFF, true);
            }
            return;
        };
        let tab = &mut self.tabs[selected];
        let background = tab.background.clone();
        tab.draw_contents(
            frame,
            (atlas, item),
            (origin, background.as_deref(), self.size),
        );
    }

    /// `extractWindow`: the window's picture, the tabs when there are more than one, and the title.
    fn draw_window(
        &self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        item: &mut ItemDrawer<'_>,
    ) {
        frame.blit_texture(
            atlas,
            "advancements/window",
            (256, 256),
            (0, 0),
            self.left,
            self.top,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
        );
        let window = (self.left, self.top);
        if self.tabs.len() > 1 {
            for (index, tab) in self.tabs.iter().enumerate() {
                tab.draw_tab(frame, atlas, window, self.selected == Some(index));
            }
            for tab in &self.tabs {
                tab.draw_icon(frame, item, window);
            }
        }
        let title = self
            .selected
            .map_or(&self.title_runs, |selected| &self.tabs[selected].title);
        frame.styled_text(
            atlas,
            font,
            title,
            self.left + 8,
            self.top + 6,
            0xFF40_4040,
            false,
        );
    }

    /// `extractTooltips`: the veil and the hovered box of the tab shown, and the title of a tab the
    /// pointer is over.
    fn draw_tooltips(
        &self,
        frame: &mut GuiFrame,
        (atlas, font, lang): (&GuiAtlas, &Font, &Lang),
        mouse: (i32, i32),
    ) {
        let origin = (self.left + 9, self.top + 18);
        if let Some(selected) = self.selected {
            self.tabs[selected].draw_tooltips(frame, (atlas, font, lang), (origin, self.size.0));
        }
        if self.tabs.len() > 1 {
            let window = (self.left, self.top);
            for tab in self
                .tabs
                .iter()
                .filter(|tab| tab.is_mouse_over(window, mouse))
            {
                frame.tooltip(
                    atlas,
                    font,
                    std::slice::from_ref(&tab.title),
                    mouse,
                    self.size,
                );
            }
        }
    }
}
