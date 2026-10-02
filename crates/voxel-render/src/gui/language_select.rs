//! The language screen (`LanguageSelectScreen`): a title over a search box,
//! the languages the packs name in a list, and under it a warning, the font
//! button and Done.

use std::time::Instant;

use voxel_game::language::LanguageInfo;
use voxel_game::text::{Component, Lang, Run, Style};

use crate::gui::edit_box::{Clipboard, EditBox, edit_key};
use crate::gui::font::Font;
use crate::gui::layout::{Arena, Grid, Item, Settings, horizontal, place_in_frame};
use crate::gui::navigation::{Candidate, Direction, Rectangle, next_in_direction};
use crate::gui::scroll_area::{Place, SCROLLBAR_WIDTH};
use crate::gui::widgets::{Kind, Widget};
use crate::gui::{GuiAtlas, GuiFrame};

/// `setHeaderHeight((int)(12 + 9 + 15))` and `setFooterHeight(53)`.
const HEADER_HEIGHT: i32 = 36;
const FOOTER_HEIGHT: i32 = 53;
/// The list's own: an entry is 18 tall and a row 220 + 50 wide.
const ENTRY_HEIGHT: i32 = 18;
const ROW_WIDTH: i32 = 270;
/// `AbstractSelectionList.Entry.CONTENT_PADDING`.
const PADDING: i32 = 2;
/// The search box: 200 by 15.
const SEARCH_WIDTH: i32 = 200;
const SEARCH_HEIGHT: i32 = 15;

/// What has the focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Search,
    /// An entry of the list.
    List,
    Fonts,
    Done,
}

/// `FocusNavigationEvent`: Tab (forward or back) or an arrow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Nav {
    Tab(bool),
    Arrow(Direction),
}

/// Where the focus goes: an entry of the list is named by its place among
/// those shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Target {
    Search,
    Entry(usize),
    Fonts,
    Done,
}

/// What an input asks for.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct LanguageInput {
    /// The screen goes (Done, or Enter on a language).
    pub done: bool,
    /// The language chosen, if it is another than the one in use.
    pub language: Option<String>,
}

pub struct LanguageSelectScreen {
    arena: Arena,
    title: Widget,
    warning: Widget,
    fonts: Widget,
    done: Widget,
    search_rect: usize,
    search: EditBox,
    searched_at: Instant,
    hint: Vec<Run>,
    /// Every language, by code, and how it reads in the list.
    languages: Vec<(String, LanguageInfo)>,
    /// The ones the search leaves, as indexes of `languages`.
    shown: Vec<usize>,
    /// The code of the language the list has selected.
    selected: Option<String>,
    /// The language in use.
    current: String,
    place: Place,
    scroll: f64,
    scrolling: bool,
    focus: Option<Focus>,
    /// Whether an entry of the list has the focus, which the screen's focus
    /// being on the list does not say (a click on the bar gives it so).
    entry_focused: bool,
    dragging: bool,
    size: (i32, i32),
}

impl LanguageSelectScreen {
    /// `init`, with the languages and the one in use.
    pub fn new(
        font: &Font,
        lang: &Lang,
        size: (i32, i32),
        in_world: bool,
        languages: Vec<(String, LanguageInfo)>,
        current: &str,
    ) -> Self {
        let runs = |component: &Component, style: &Style| lang.runs(component, style);
        let mut arena = Arena::default();
        let title_text = runs(
            &Component::translatable("options.language.title"),
            &Style::default(),
        );
        let title = Widget {
            rect: arena.add(font.width_of_runs(&title_text), 9),
            active: false,
            kind: Kind::Text { text: title_text },
        };
        let search_rect = arena.add(SEARCH_WIDTH, SEARCH_HEIGHT);
        let warning_text = runs(
            &Component::translatable("options.languageAccuracyWarning"),
            &Style {
                colour: Some(0xBA_BABA),
                ..Style::default()
            },
        );
        let warning = Widget {
            rect: arena.add(font.width_of_runs(&warning_text), 9),
            active: false,
            kind: Kind::Text { text: warning_text },
        };
        let mut button = |key: &str| Widget {
            rect: arena.add(150, 20),
            active: true,
            kind: Kind::Button {
                label: runs(&Component::translatable(key), &Style::default()),
            },
        };
        let fonts = button("options.font");
        let done = button("gui.done");
        let hint = runs(
            &Component::translatable("gui.language.search"),
            &Style {
                colour: Some(0xAA_AAAA),
                italic: Some(true),
                ..Style::default()
            },
        );
        let selected = languages
            .iter()
            .any(|(code, _)| code == current)
            .then(|| current.to_owned());
        let mut screen = Self {
            arena,
            title,
            warning,
            fonts,
            done,
            search_rect,
            search: EditBox::new(SEARCH_WIDTH - 8, 32),
            searched_at: Instant::now(),
            hint,
            shown: (0..languages.len()).collect(),
            languages,
            selected,
            current: current.to_owned(),
            place: Place {
                y: 33,
                width: size.0,
                height: size.1 - 33 - FOOTER_HEIGHT,
                in_world,
            },
            scroll: 0.0,
            scrolling: false,
            focus: Some(Focus::Search),
            entry_focused: false,
            dragging: false,
            size,
        };
        // `centerScrollOn`, at the height the list was made with.
        if let Some(index) = screen.selected_index() {
            let above = index as i32 * ENTRY_HEIGHT + ENTRY_HEIGHT / 2;
            screen.set_scroll(f64::from(above) - f64::from(screen.place.height) / 2.0);
        }
        screen.arrange();
        screen
    }

    /// The place of the selected language among those shown.
    fn selected_index(&self) -> Option<usize> {
        let code = self.selected.as_ref()?;
        self.shown
            .iter()
            .position(|&index| &self.languages[index].0 == code)
    }

    fn content_height(&self) -> i32 {
        self.shown.len() as i32 * ENTRY_HEIGHT + 4
    }

    fn set_scroll(&mut self, scroll: f64) {
        self.scroll = scroll.clamp(0.0, f64::from(self.place.max_scroll(self.content_height())));
    }

    /// Where the entry at `index` starts (`repositionEntries`).
    fn entry_y(&self, index: usize) -> i32 {
        self.place.y + PADDING - self.scroll as i32 + index as i32 * ENTRY_HEIGHT
    }

    fn row_left(&self) -> i32 {
        self.place.width / 2 - ROW_WIDTH / 2
    }

    fn scrollbar_x(&self) -> i32 {
        self.row_left() + ROW_WIDTH + SCROLLBAR_WIDTH + 2
    }

    /// `repositionElements`: the header, the footer and the list again.
    fn arrange(&mut self) {
        let (width, height) = self.size;
        let centred = Settings::default().align_horizontally(0.5);
        let mut header = Grid::new();
        header.row_spacing = 4;
        header.add(Item::Leaf(self.title.rect), (0, 0), (1, 1), centred);
        header.add(Item::Leaf(self.search_rect), (1, 0), (1, 1), centred);
        header.arrange(&mut self.arena);
        let frame = (
            0,
            0,
            width.max(header.width),
            HEADER_HEIGHT.max(header.height),
        );
        place_in_frame(
            &mut self.arena,
            Item::Grid(Box::new(header)),
            frame,
            (0.5, 0.5),
        );

        let buttons = horizontal(&self.arena, &[self.fonts.rect, self.done.rect], 8);
        let mut footer = Grid::new();
        footer.row_spacing = 8;
        footer.add(Item::Leaf(self.warning.rect), (0, 0), (1, 1), centred);
        footer.add(Item::Grid(Box::new(buttons)), (1, 0), (1, 1), centred);
        footer.arrange(&mut self.arena);
        let frame = (
            0,
            height - FOOTER_HEIGHT,
            width.max(footer.width),
            FOOTER_HEIGHT.max(footer.height),
        );
        place_in_frame(
            &mut self.arena,
            Item::Grid(Box::new(footer)),
            frame,
            (0.5, 0.5),
        );

        // `updateSizeAndPosition`.
        self.place = Place {
            y: HEADER_HEIGHT,
            width,
            height: height - HEADER_HEIGHT - FOOTER_HEIGHT,
            ..self.place
        };
        if let Some(index) = self.selected_index() {
            self.scroll_to_entry(index);
        }
        self.set_scroll(self.scroll);
    }

    /// `scrollToEntry`.
    fn scroll_to_entry(&mut self, index: usize) {
        let top_delta = self.entry_y(index) - self.place.y - PADDING;
        if top_delta < 0 {
            self.set_scroll(self.scroll + f64::from(top_delta));
        }
        let bottom_delta = self.place.bottom() - self.entry_y(index) - ENTRY_HEIGHT - PADDING;
        if bottom_delta < 0 {
            self.set_scroll(self.scroll - f64::from(bottom_delta));
        }
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    pub fn resize(&mut self, size: (i32, i32)) {
        self.size = size;
        self.arrange();
    }

    pub fn scroll_amount(&self) -> f64 {
        self.scroll
    }

    /// The text of the search box.
    pub fn search_text(&self) -> String {
        self.search.value()
    }

    /// The language the list has selected.
    pub fn selected(&self) -> Option<&str> {
        self.selected.as_deref()
    }

    pub fn draw(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        let (width, height) = self.size;
        let background = if self.place.in_world {
            "inworld_menu_background"
        } else {
            "menu_background"
        };
        frame.tile_texture(atlas, background, (32, 32), (0, 0, width, height));
        self.title.draw(frame, atlas, font, &self.arena, false);
        // The search box: its sprite, its text, and the hint when it is
        // empty and not the one with the focus.
        let rect = self.arena.0[self.search_rect];
        let focused = self.focus == Some(Focus::Search);
        frame.nine_slice(
            atlas,
            if focused {
                "widget/text_field_highlighted"
            } else {
                "widget/text_field"
            },
            (200, 20),
            1,
            false,
            rect.x,
            rect.y,
            rect.width,
            rect.height,
        );
        let text_at = (rect.x + 4, rect.y + (rect.height - 8) / 2);
        self.search.draw_formatted(
            frame,
            atlas,
            font,
            text_at,
            focused.then_some(self.searched_at),
            None,
        );
        if self.search.is_empty() && !focused {
            frame.styled_text(
                atlas,
                font,
                &self.hint,
                text_at.0,
                text_at.1,
                0xFFE0_E0E0,
                true,
            );
        }
        self.draw_list(frame, atlas, font);
        self.warning.draw(frame, atlas, font, &self.arena, false);
        for (widget, which) in [(&self.fonts, Focus::Fonts), (&self.done, Focus::Done)] {
            let highlighted = widget.contains(&self.arena, mouse) || self.focus == Some(which);
            widget.draw(frame, atlas, font, &self.arena, highlighted);
        }
    }

    fn draw_list(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font) {
        let place = self.place;
        place.draw_background(frame, atlas, self.scroll);
        let mark = frame.mark();
        let left = self.row_left();
        for (position, &index) in self.shown.iter().enumerate() {
            let y = self.entry_y(position);
            if y + ENTRY_HEIGHT < place.y || y > place.bottom() {
                continue;
            }
            let (code, info) = &self.languages[index];
            if self.selected.as_ref() == Some(code) {
                // `extractSelection`: an outline round black.
                let outline = if self.entry_focused {
                    0xFFFF_FFFF
                } else {
                    0xFF80_8080
                };
                let (x1, y1) = (left + ROW_WIDTH, y + ENTRY_HEIGHT);
                frame.fill(atlas, (left, y, x1, y1), outline);
                frame.fill(atlas, (left + 1, y + 1, x1 - 1, y1 - 1), 0xFF00_0000);
            }
            let text = vec![Run {
                text: info.label(),
                style: Style::default(),
            }];
            let x = place.width / 2 - font.width_of_runs(&text) / 2;
            frame.styled_text(
                atlas,
                font,
                &text,
                x,
                y + PADDING + (ENTRY_HEIGHT - 4) / 2 - 4,
                0xFFFF_FFFF,
                true,
            );
        }
        frame.clip_from(mark, (0, place.y, place.width, place.bottom()));
        place.draw_edges(
            frame,
            atlas,
            (self.scroll, self.content_height()),
            self.scrollbar_x(),
        );
    }

    /// `setSelected`: the language is the list's, and the list scrolls to
    /// show it if it sticks out.
    fn select(&mut self, position: usize) {
        self.selected = Some(self.languages[self.shown[position]].0.clone());
        let top = self.entry_y(position) + PADDING;
        let bottom = self.entry_y(position) + ENTRY_HEIGHT - PADDING;
        if top < self.place.y || bottom > self.place.bottom() {
            self.scroll_to_entry(position);
        }
    }

    /// `onDone`: the selected language, if it is another, and the screen
    /// goes.
    fn finish(&self) -> LanguageInput {
        let language = self.selected.clone().filter(|code| *code != self.current);
        LanguageInput {
            done: true,
            language,
        }
    }

    /// A press of the primary button at `mouse`; `double` for the second of
    /// a double click.
    pub fn clicked(&mut self, font: &Font, mouse: (f64, f64), double: bool) -> LanguageInput {
        let at = (mouse.0 as i32, mouse.1 as i32);
        let search = self.arena.0[self.search_rect];
        if at.0 >= search.x
            && at.0 < search.x + search.width
            && at.1 >= search.y
            && at.1 < search.y + search.height
        {
            self.set_focus(Some(Focus::Search));
            self.search.click_at(at.0 - (search.x + 4), font);
            self.dragging = true;
            return LanguageInput::default();
        }
        if self.place.contains(mouse) {
            self.scrolling = self.place.max_scroll(self.content_height()) > 0
                && mouse.0 >= f64::from(self.scrollbar_x())
                && mouse.0 <= f64::from(self.scrollbar_x() + SCROLLBAR_WIDTH);
            let left = self.row_left();
            let position = self.shown.iter().enumerate().position(|(position, _)| {
                let y = self.entry_y(position);
                at.0 >= left && at.0 < left + ROW_WIDTH && at.1 >= y && at.1 < y + ENTRY_HEIGHT
            });
            if let Some(position) = position {
                self.select(position);
                self.set_focus(Some(Focus::List));
                self.entry_focused = true;
                self.dragging = true;
                if double {
                    return self.finish();
                }
            } else if self.scrolling {
                self.set_focus(Some(Focus::List));
                self.dragging = true;
            }
            return LanguageInput::default();
        }
        let hit = [(&self.fonts, Focus::Fonts), (&self.done, Focus::Done)]
            .into_iter()
            .find(|(widget, _)| widget.is_active() && widget.contains(&self.arena, at))
            .map(|(_, which)| which);
        if let Some(which) = hit {
            self.set_focus(Some(which));
            self.dragging = true;
            if which == Focus::Done {
                return self.finish();
            }
        }
        LanguageInput::default()
    }

    /// The mouse moved to `mouse`, by `dy` down, with the primary button
    /// down.
    pub fn dragged(&mut self, mouse: (f64, f64), dy: f64) {
        if self.scrolling {
            let to = self
                .place
                .dragged_scroll(self.scroll, mouse.1, dy, self.content_height());
            self.set_scroll(to);
        }
    }

    pub fn released(&mut self) {
        self.scrolling = false;
        self.dragging = false;
    }

    /// The wheel over the list.
    pub fn scrolled(&mut self, mouse: (f64, f64), scroll_y: f64) {
        if self.place.contains(mouse) {
            self.set_scroll(self.scroll - scroll_y * f64::from(ENTRY_HEIGHT / 2));
        }
    }

    /// The screen's focus moves to a widget: the one before loses it (the
    /// list drops its entry's), and a search box blinks from now.
    fn set_focus(&mut self, focus: Option<Focus>) {
        if focus != Some(Focus::List) {
            self.entry_focused = false;
        }
        if focus == Some(Focus::Search) && self.focus != focus {
            self.searched_at = Instant::now();
        }
        self.focus = focus;
    }

    /// `nextEntry`: the entry above or below one (or the first or last with
    /// none), which a vertical direction names.
    fn next_entry(&self, direction: Direction, start: Option<usize>) -> Option<usize> {
        let delta: isize = match direction {
            Direction::Up => -1,
            Direction::Down => 1,
            Direction::Left | Direction::Right => return None,
        };
        let index = match start {
            Some(start) => start as isize + delta,
            None if delta > 0 => 0,
            None => self.shown.len() as isize - 1,
        };
        (index >= 0 && (index as usize) < self.shown.len()).then_some(index as usize)
    }

    /// `ObjectSelectionList.nextFocusPath`: the entry the focus would go to.
    /// With an entry focused an arrow that finds no next one drops both the
    /// focus and the selection, as the reference does while asking.
    fn list_next(&mut self, nav: Nav, change: bool) -> Option<usize> {
        if self.shown.is_empty() {
            return None;
        }
        if self.entry_focused {
            let Nav::Arrow(direction) = nav else {
                return None;
            };
            let next = self.next_entry(direction, self.selected_index());
            if next.is_none() && change {
                self.entry_focused = false;
                self.selected = None;
            }
            return next;
        }
        let vertical = match nav {
            Nav::Tab(forward) => {
                if forward {
                    Direction::Down
                } else {
                    Direction::Up
                }
            }
            Nav::Arrow(direction @ (Direction::Up | Direction::Down)) => direction,
            Nav::Arrow(_) => Direction::Down,
        };
        self.selected_index()
            .or_else(|| self.next_entry(vertical, None))
    }

    /// What the focused widget itself offers (`focus.nextFocusPath`).
    fn focused_next(&mut self, nav: Nav) -> Option<Target> {
        match self.focus {
            Some(Focus::List) => self.list_next(nav, true).map(Target::Entry),
            _ => None,
        }
    }

    /// `handleTabNavigation` of the screen: the title, search box, list, the
    /// warning, the font button and Done in order; the texts take nothing.
    fn root_tab(&mut self, forward: bool) -> Option<Target> {
        let order = [Focus::Search, Focus::List, Focus::Fonts, Focus::Done];
        let at = self
            .focus
            .and_then(|focus| order.iter().position(|candidate| *candidate == focus));
        let start = match (at, forward) {
            (Some(at), true) => at + 1,
            (Some(at), false) => at,
            (None, true) => 0,
            (None, false) => order.len(),
        };
        let visit: Vec<Focus> = if forward {
            order[start..].to_vec()
        } else {
            order[..start].iter().rev().copied().collect()
        };
        for focus in visit {
            let found = match focus {
                Focus::Search => Some(Target::Search),
                Focus::List => self.list_next(Nav::Tab(forward), true).map(Target::Entry),
                Focus::Fonts => self.fonts.is_active().then_some(Target::Fonts),
                Focus::Done => self.done.is_active().then_some(Target::Done),
            };
            if found.is_some() {
                return found;
            }
        }
        None
    }

    /// `handleArrowNavigation` of the screen: the widget that lies next to
    /// the focused one in the direction.
    fn root_arrow(&mut self, direction: Direction) -> Option<Target> {
        let list_entry = if self.focus == Some(Focus::List) {
            None
        } else {
            self.list_next(Nav::Arrow(direction), false)
        };
        let rectangle = |rect: usize| {
            let rect = self.arena.0[rect];
            Rectangle {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
            }
        };
        let children = [
            (rectangle(self.title.rect), false),
            (rectangle(self.search_rect), true),
            (
                Rectangle {
                    x: 0,
                    y: self.place.y,
                    width: self.place.width,
                    height: self.place.height,
                },
                list_entry.is_some(),
            ),
            (rectangle(self.warning.rect), false),
            (rectangle(self.fonts.rect), self.fonts.is_active()),
            (rectangle(self.done.rect), self.done.is_active()),
        ]
        .map(|(rectangle, focusable)| Candidate {
            rectangle,
            focusable,
        });
        let focused = self.focus.map(|focus| match focus {
            Focus::Search => 1,
            Focus::List => 2,
            Focus::Fonts => 4,
            Focus::Done => 5,
        });
        let container = Rectangle {
            x: 0,
            y: 0,
            width: self.size.0,
            height: self.size.1,
        };
        Some(
            match next_in_direction(&children, focused, direction, container)? {
                1 => Target::Search,
                2 => Target::Entry(list_entry?),
                4 => Target::Fonts,
                _ => Target::Done,
            },
        )
    }

    /// `changeFocus`: what had the focus lets go, and the target takes it
    /// (an entry becomes the selected one, and the list shows it).
    fn apply(&mut self, target: Target) {
        match target {
            Target::Search => self.set_focus(Some(Focus::Search)),
            Target::Fonts => self.set_focus(Some(Focus::Fonts)),
            Target::Done => self.set_focus(Some(Focus::Done)),
            Target::Entry(position) => {
                self.set_focus(Some(Focus::List));
                self.entry_focused = true;
                self.selected = Some(self.languages[self.shown[position]].0.clone());
                // A keyboard always shows the entry (`setSelected`).
                self.scroll_to_entry(position);
            }
        }
    }

    /// `Screen.keyPressed`'s navigation: Tab and the arrows.
    fn navigate(&mut self, nav: Nav) {
        let mut target = self.focused_next(nav);
        if target.is_none() {
            target = match nav {
                Nav::Tab(forward) => self.root_tab(forward),
                Nav::Arrow(direction) => self.root_arrow(direction),
            };
        }
        if target.is_none() && matches!(nav, Nav::Tab(_)) {
            // Past the last widget the focus is cleared and goes round.
            self.entry_focused = false;
            self.focus = None;
            if let Nav::Tab(forward) = nav {
                target = self.root_tab(forward);
            }
        }
        if let Some(target) = target {
            self.apply(target);
        }
    }

    /// A character typed: the search box takes it, and the list shows what
    /// matches.
    pub fn char_typed(&mut self, font: &Font, character: char) {
        if self.focus == Some(Focus::Search) && self.search.char_typed(character, font) {
            self.filter();
        }
    }

    /// A key: the focused widget takes it (the search box edits, an entry of
    /// the list is chosen by Enter, Space), or Tab and the arrows move the
    /// focus.
    pub fn key_pressed(
        &mut self,
        font: &Font,
        key: glfw::Key,
        (control, shift): (bool, bool),
        clipboard: &mut dyn Clipboard,
    ) -> LanguageInput {
        use glfw::Key;
        let selection = matches!(key, Key::Enter | Key::KpEnter | Key::Space);
        match (self.focus, key) {
            (Some(Focus::Search), key) if edit_key(key).is_some() => {
                let before = self.search.value();
                if let Some(edit) = edit_key(key) {
                    self.search
                        .key_pressed(edit, control, shift, font, clipboard);
                }
                if self.search.value() != before {
                    self.filter();
                }
                return LanguageInput::default();
            }
            (Some(Focus::List), _) if selection && self.entry_focused => return self.finish(),
            (Some(Focus::Done), _) if selection => return self.finish(),
            _ => {}
        }
        match key {
            Key::Tab => self.navigate(Nav::Tab(!shift)),
            Key::Left => self.navigate(Nav::Arrow(Direction::Left)),
            Key::Right => self.navigate(Nav::Arrow(Direction::Right)),
            Key::Up => self.navigate(Nav::Arrow(Direction::Up)),
            Key::Down => self.navigate(Nav::Arrow(Direction::Down)),
            _ => {}
        }
        LanguageInput::default()
    }

    /// `filterEntries`: the languages whose name or region holds the text,
    /// in any case; nothing is selected after it.
    fn filter(&mut self) {
        let text = self.search.value().to_lowercase();
        self.shown = self
            .languages
            .iter()
            .enumerate()
            .filter(|(_, (_, info))| {
                text.is_empty()
                    || info.name.to_lowercase().contains(&text)
                    || info.region.to_lowercase().contains(&text)
            })
            .map(|(index, _)| index)
            .collect();
        self.selected = None;
        self.set_scroll(self.scroll);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assets() -> Option<(Font, Lang)> {
        let root = std::path::PathBuf::from(std::env::var("VOXEL_ASSETS").ok()?);
        Some((Font::load(&root)?, Lang::load(&root)))
    }

    fn info(name: &str, region: &str) -> LanguageInfo {
        LanguageInfo {
            region: region.to_owned(),
            name: name.to_owned(),
            bidirectional: false,
        }
    }

    #[test]
    fn done_gives_a_language_only_when_it_is_another() {
        let Some((font, lang)) = assets() else { return };
        let languages = vec![
            ("de_de".to_owned(), info("Deutsch", "Deutschland")),
            ("en_us".to_owned(), info("English", "US")),
        ];
        let mut screen =
            LanguageSelectScreen::new(&font, &lang, (427, 240), true, languages, "en_us");
        // Done at once keeps the language in use.
        let done = screen.clicked(&font, (300.0, 225.0), false);
        assert_eq!((done.done, done.language), (true, None));
        // A double click on another one takes it.
        let first_row_y = 36.0 + 2.0 + 5.0;
        let chosen = screen.clicked(&font, (213.0, first_row_y), true);
        assert_eq!(chosen.language.as_deref(), Some("de_de"));
        // Searching leaves nothing selected, so Done changes nothing.
        let mut screen = LanguageSelectScreen::new(
            &font,
            &lang,
            (427, 240),
            true,
            vec![("de_de".to_owned(), info("Deutsch", "Deutschland"))],
            "en_us",
        );
        screen.char_typed(&font, 'd');
        assert_eq!(screen.selected(), None);
        assert_eq!(screen.search_text(), "d");
    }
}
