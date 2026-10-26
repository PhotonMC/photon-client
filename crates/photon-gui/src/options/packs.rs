//! The Resource Packs screen (`PackSelectionScreen`): the packs that are not in use on the left,
//! the ones in use on the right (the top one wins), a search box over them and Open Folder and
//! Done under.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use photon_assets::{Compatibility, PackEntry, SelectionModel, Side, Title};
use photon_game::text::{Component, Lang, Run, Style};

use crate::layout::{Arena, Grid, Item, Settings, horizontal, place_in_frame};
use crate::widgets::edit_box::EditBox;
use crate::widgets::scroll_area::Place;
use crate::widgets::{Kind, Widget};
use crate::{FontRuns, splitter};
use photon_render::gui::font::Font;

mod draw;
mod input;

/// `layout.setHeaderHeight(4 + 9 + 4 + 9 + 4 + 15 + 4)`, and the footer's default.
const HEADER_HEIGHT: i32 = 49;
const FOOTER_HEIGHT: i32 = 33;
const LIST_WIDTH: i32 = 200;
const SEARCH_HEIGHT: i32 = 15;
/// `TransferableSelectionList`: 36 tall entries, a header of `(int)(9 * 1.5)`, 196 wide rows.
const ENTRY_HEIGHT: i32 = 36;
const HEADER_ENTRY_HEIGHT: i32 = 13;
const ROW_WIDTH: i32 = LIST_WIDTH - 4;
/// `AbstractSelectionList.Entry.CONTENT_PADDING` and the icon.
const PADDING: i32 = 2;
const ICON: i32 = 32;
const TEXT_WIDTH: i32 = 157;

/// What an input asks of the host.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PackInput {
    /// Done: the screen goes (`onClose` commits the model first).
    pub done: bool,
    /// Open Folder.
    pub open_folder: bool,
    /// A pack that does not fit the game was asked for: the host asks first, then calls
    /// [`PackScreen::select_confirmed`].
    pub confirm: Option<(String, Compatibility)>,
}

/// What an entry shows of its pack.
struct PackView {
    id: String,
    title: Vec<Run>,
    /// The description under its source, as the entry draws it.
    description: Vec<Run>,
    /// The text the search looks in.
    haystack: String,
    compatibility: Compatibility,
    required: bool,
    fixed_position: bool,
    /// The pack's icon texture, none for `unknown_pack`.
    icon: Option<String>,
}

/// What has the focus: the search box or a list (0 available, 1 selected).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Search,
    List(usize),
}

/// A selected entry of a list: its heading is one too.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Selected {
    Header,
    Pack(String),
}

pub struct PackScreen {
    arena: Arena,
    title: Widget,
    drop_info: Widget,
    search_rect: usize,
    search: EditBox,
    searched_at: Instant,
    hint: Vec<Run>,
    open_folder: Widget,
    done: Widget,
    model: SelectionModel,
    views: HashMap<String, PackView>,
    headers: [Vec<Run>; 2],
    scroll: [f64; 2],
    /// The list whose bar is held.
    scrolling: Option<usize>,
    focus: Option<Focus>,
    /// The entry each list has selected (`AbstractSelectionList.getSelected`).
    selected: [Option<Selected>; 2],
    folder: PathBuf,
    in_world: bool,
    size: (i32, i32),
}

/// The pack's description under its source (`PackSource.decorate`).
fn extended_description(entry: &PackEntry) -> Component {
    let description = Component::from_json(&entry.description).unwrap_or_else(Component::empty);
    match entry.source.name_key() {
        Some(key) => Component::translatable_with(
            "pack.nameAndSource",
            vec![description, Component::translatable(key)],
        )
        .with_colour("gray"),
        None => description,
    }
}

fn plain(runs: &[Run]) -> String {
    runs.iter().map(|run| run.text.as_str()).collect()
}

/// The widgets and texts the screen holds apart from its lists.
struct Parts {
    arena: Arena,
    title: Widget,
    drop_info: Widget,
    search_rect: usize,
    open_folder: Widget,
    done: Widget,
    hint: Vec<Run>,
    headers: [Vec<Run>; 2],
}

fn text_widget(arena: &mut Arena, font: &Font, text: Vec<Run>) -> Widget {
    Widget {
        rect: arena.add(font.width_of_runs(&text), 9),
        active: false,
        kind: Kind::Text { text },
    }
}

fn parts(font: &Font, lang: &Lang) -> Parts {
    let runs = |key: &str, style: &Style| lang.runs(&Component::translatable(key), style);
    let gray = Style {
        colour: Some(0xAA_AAAA),
        ..Style::default()
    };
    let mut arena = Arena::default();
    let title = text_widget(
        &mut arena,
        font,
        runs("resourcePack.title", &Style::default()),
    );
    let drop_info = text_widget(&mut arena, font, runs("pack.dropInfo", &gray));
    let search_rect = arena.add(200, SEARCH_HEIGHT);
    let mut button = |key: &str| Widget {
        rect: arena.add(150, 20),
        active: true,
        kind: Kind::Button {
            label: runs(key, &Style::default()),
        },
    };
    let (open_folder, done) = (button("pack.openFolder"), button("gui.done"));
    let italic = Style {
        italic: Some(true),
        ..gray
    };
    let heading = Style {
        bold: Some(true),
        underlined: Some(true),
        ..Style::default()
    };
    Parts {
        arena,
        title,
        drop_info,
        search_rect,
        open_folder,
        done,
        hint: runs("gui.packSelection.search", &italic),
        headers: [
            runs("pack.available.title", &heading),
            runs("pack.selected.title", &heading),
        ],
    }
}

impl PackScreen {
    pub fn new(
        font: &Font,
        lang: &Lang,
        size: (i32, i32),
        in_world: bool,
        model: SelectionModel,
        folder: PathBuf,
    ) -> Self {
        let parts = parts(font, lang);
        let mut screen = Self {
            arena: parts.arena,
            title: parts.title,
            drop_info: parts.drop_info,
            search_rect: parts.search_rect,
            search: EditBox::new(200 - 8, 32),
            searched_at: Instant::now(),
            hint: parts.hint,
            open_folder: parts.open_folder,
            done: parts.done,
            model,
            views: HashMap::new(),
            headers: parts.headers,
            scroll: [0.0; 2],
            scrolling: None,
            focus: Some(Focus::Search),
            selected: [None, None],
            folder,
            in_world,
            size,
        };
        screen.rebuild_views(lang);
        screen.arrange();
        screen.update_done();
        screen
    }

    /// The views of every pack the model knows, in the language now loaded.
    pub fn rebuild_views(&mut self, lang: &Lang) {
        let old: HashMap<String, Option<String>> = self
            .views
            .drain()
            .map(|(id, view)| (id, view.icon))
            .collect();
        let ids: Vec<String> = [Side::Selected, Side::Unselected]
            .into_iter()
            .flat_map(|side| self.model.list(side).to_vec())
            .collect();
        for id in ids {
            let Some(entry) = self.model.entry(&id) else {
                continue;
            };
            let title = match &entry.title {
                Title::Literal(name) => Component::literal(name.clone()),
                Title::Translatable(key) => Component::translatable(key.clone()),
            };
            let title = lang.runs(&title, &Style::default());
            let gray = Style {
                colour: Some(0x80_8080),
                ..Style::default()
            };
            let description_runs = lang.runs(&extended_description(entry), &gray);
            // The search looks in the description as the pack gave it, without its source.
            let bare = Component::from_json(&entry.description).unwrap_or_else(Component::empty);
            let haystack = format!(
                "{}\n{}\n{}",
                id.to_lowercase(),
                plain(&title).to_lowercase(),
                plain(&lang.runs(&bare, &Style::default())).to_lowercase()
            );
            let view = PackView {
                description: description_runs,
                title,
                haystack,
                compatibility: entry.compatibility,
                required: entry.config.required,
                fixed_position: entry.config.fixed_position,
                icon: old.get(&id).cloned().flatten(),
                id: id.clone(),
            };
            self.views.insert(id, view);
        }
    }

    /// `Done` is active while a pack is selected.
    fn update_done(&mut self) {
        self.done.active = !self.model.list(Side::Selected).is_empty();
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    pub fn resize(&mut self, size: (i32, i32)) {
        self.size = size;
        self.arrange();
    }

    pub fn model(&self) -> &SelectionModel {
        &self.model
    }

    pub fn folder(&self) -> &std::path::Path {
        &self.folder
    }

    /// `onClose`'s `commit`: the model with the lists as they are left.
    pub fn commit(&mut self) -> &photon_assets::Repository {
        self.model.commit()
    }

    /// A pack's icon texture, once the host has put it where the GUI draws from.
    pub fn set_icon(&mut self, id: &str, texture: Option<String>) {
        if let Some(view) = self.views.get_mut(id) {
            view.icon = texture;
        }
    }

    /// The ids of the packs with no icon texture yet, with their `pack.png` bytes.
    pub fn icons_wanted(&self) -> Vec<(String, Vec<u8>)> {
        self.views
            .values()
            .filter(|view| view.icon.is_none())
            .filter_map(|view| {
                let bytes = self.model.entry(&view.id)?.icon.clone()?;
                Some((view.id.clone(), bytes))
            })
            .collect()
    }

    /// `findNewPacks` and the lists again, after the folder changed.
    pub fn reload(&mut self, lang: &Lang, entries: Vec<PackEntry>) {
        self.model.find_new_packs(entries);
        self.rebuild_views(lang);
        self.update_done();
        self.selected = [None, None];
        for side in 0..2 {
            self.set_scroll(side, self.scroll[side]);
        }
    }

    /// `repositionElements`: the header, the footer and the two lists.
    fn arrange(&mut self) {
        let (width, height) = self.size;
        let centred = Settings::default().align_horizontally(0.5);
        let mut header = Grid::new();
        header.row_spacing = 4;
        header.add(Item::Leaf(self.title.rect), (0, 0), (1, 1), centred);
        header.add(Item::Leaf(self.drop_info.rect), (1, 0), (1, 1), centred);
        header.add(Item::Leaf(self.search_rect), (2, 0), (1, 1), centred);
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

        let mut footer = horizontal(&self.arena, &[self.open_folder.rect, self.done.rect], 8);
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
        for side in 0..2 {
            self.set_scroll(side, self.scroll[side]);
        }
    }

    /// The left edge of a list: 0 available, 1 selected.
    fn list_x(&self, list: usize) -> i32 {
        if list == 0 {
            self.size.0 / 2 - 15 - LIST_WIDTH
        } else {
            self.size.0 / 2 + 15
        }
    }

    fn place(&self) -> Place {
        Place {
            y: HEADER_HEIGHT,
            width: LIST_WIDTH,
            height: self.size.1 - HEADER_HEIGHT - FOOTER_HEIGHT,
            in_world: self.in_world,
        }
    }

    fn side(list: usize) -> Side {
        if list == 0 {
            Side::Unselected
        } else {
            Side::Selected
        }
    }

    /// The packs a list shows after the search (`filterEntries`).
    fn shown(&self, list: usize) -> Vec<&PackView> {
        let text = self.search.value().to_lowercase();
        self.model
            .list(Self::side(list))
            .iter()
            .filter_map(|id| self.views.get(id))
            .filter(|view| text.trim().is_empty() || view.haystack.contains(&text))
            .collect()
    }

    fn content_height(&self, list: usize) -> i32 {
        HEADER_ENTRY_HEIGHT + self.shown(list).len() as i32 * ENTRY_HEIGHT + 4
    }

    fn scrollable(&self, list: usize) -> bool {
        self.place().max_scroll(self.content_height(list)) > 0
    }

    fn set_scroll(&mut self, list: usize, scroll: f64) {
        let max = self.place().max_scroll(self.content_height(list));
        self.scroll[list] = scroll.clamp(0.0, f64::from(max));
    }

    /// The top of the entry `index` of a list (the header being 0).
    fn entry_y(&self, list: usize, index: usize) -> i32 {
        let above = if index == 0 {
            0
        } else {
            HEADER_ENTRY_HEIGHT + (index as i32 - 1) * ENTRY_HEIGHT
        };
        HEADER_HEIGHT + PADDING - self.scroll[list] as i32 + above
    }

    fn bar_x(&self, list: usize) -> i32 {
        self.list_x(list) + LIST_WIDTH - crate::widgets::scroll_area::SCROLLBAR_WIDTH
    }

    /// What has the focus, for the oracle: `search`, `available`, `selected` or `none`.
    pub fn focus_name(&self) -> &'static str {
        match self.focus {
            Some(Focus::Search) => "search",
            Some(Focus::List(0)) => "available",
            Some(Focus::List(_)) => "selected",
            None => "none",
        }
    }

    /// The pack each list has selected (the heading has an empty id), `none` for no entry.
    pub fn selected_ids(&self) -> [String; 2] {
        let name = |entry: &Option<Selected>| match entry {
            None => "none".to_owned(),
            Some(Selected::Header) => String::new(),
            Some(Selected::Pack(id)) => id.clone(),
        };
        [name(&self.selected[0]), name(&self.selected[1])]
    }

    /// The available list's scroll.
    pub fn scroll_amount(&self) -> f64 {
        self.scroll[0]
    }

    fn search_focused(&self) -> bool {
        self.focus == Some(Focus::Search)
    }

    pub fn search_text(&self) -> String {
        self.search.value()
    }
}

/// The description in lines of `width`, two at most; a longer text ends in an ellipsis
/// (`MultiLineLabel.create` with a row limit).
fn description_lines(font: &Font, runs: &[Run], width: i32) -> Vec<Vec<Run>> {
    let split = splitter::split_lines(runs, width, font, &Style::default());
    let more = split.len() > 2;
    let mut lines: Vec<Vec<Run>> = split.into_iter().take(2).map(|(line, _)| line).collect();
    if more && let Some(last) = lines.last_mut() {
        let own = font.width_of_runs(last);
        ellipsize(font, last, own);
    }
    lines
}

/// A text cut at the end to fit `limit` with an ellipsis (`StringWidget.setMaxWidth`).
fn ellipsize(font: &Font, runs: &mut Vec<Run>, limit: i32) {
    let style = runs.last().map(|run| run.style.clone()).unwrap_or_default();
    while font.width_of_runs(runs) + font.width("...") > limit && !runs.is_empty() {
        let emptied = runs.last_mut().is_some_and(|run| {
            run.text.pop();
            run.text.is_empty()
        });
        if emptied {
            runs.pop();
        }
    }
    runs.push(Run {
        text: "...".to_owned(),
        style,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use photon_assets::{PackSource, Position, Repository, SelectionConfig, vanilla_entry};

    fn assets() -> Option<(Font, Lang)> {
        let root = PathBuf::from(std::env::var("PHOTON_ASSETS").ok()?);
        Some((Font::load(&root)?, Lang::load(&root)))
    }

    fn folder_pack(name: &str, compatibility: Compatibility) -> PackEntry {
        PackEntry {
            id: format!("file/{name}"),
            title: Title::Literal(name.to_owned()),
            description: photon_core::json::parse("\"a pack\"").unwrap(),
            compatibility,
            source: PackSource::Default,
            config: SelectionConfig {
                required: false,
                fixed_position: false,
                default_position: Position::Top,
            },
            icon: None,
        }
    }

    fn screen(font: &Font, lang: &Lang, packs: Vec<PackEntry>) -> PackScreen {
        let mut entries = vec![vanilla_entry(None)];
        entries.extend(packs);
        let model = SelectionModel::new(Repository::new(entries, &[]));
        PackScreen::new(font, lang, (427, 240), true, model, PathBuf::new())
    }

    /// Where the icon of the first pack of a list is (the heading is 13 tall).
    fn first_icon(screen: &PackScreen, list: usize) -> (f64, f64) {
        let x = screen.list_x(list) + 2 * PADDING + 4;
        let y = HEADER_HEIGHT + PADDING + HEADER_ENTRY_HEIGHT + PADDING + 4;
        (f64::from(x), f64::from(y))
    }

    #[test]
    fn pressing_an_icon_moves_a_pack_between_the_lists() {
        let Some((font, lang)) = assets() else { return };
        let mut screen = screen(
            &font,
            &lang,
            vec![folder_pack("a", Compatibility::Compatible)],
        );
        assert_eq!(screen.model().list(Side::Unselected), ["file/a"]);
        let input = screen.clicked(&font, first_icon(&screen, 0));
        assert_eq!(input, PackInput::default());
        assert_eq!(screen.model().list(Side::Selected), ["file/a", "vanilla"]);
        // The top of the selected list is now under the pointer: its left half unselects it.
        screen.clicked(&font, first_icon(&screen, 1));
        assert_eq!(screen.model().list(Side::Selected), ["vanilla"]);
    }

    #[test]
    fn the_pressed_list_keeps_the_focus_and_the_moved_pack_is_selected() {
        let Some((font, lang)) = assets() else { return };
        let mut screen = screen(
            &font,
            &lang,
            vec![folder_pack("a", Compatibility::Compatible)],
        );
        screen.clicked(&font, first_icon(&screen, 0));
        // The press was in the left list: it keeps the focus, the pack is selected in the right.
        assert_eq!(screen.focus, Some(Focus::List(0)));
        assert_eq!(screen.selected[1], Some(Selected::Pack("file/a".into())));
        // A press on the text of the right list's first row moves the focus there.
        let (x, y) = first_icon(&screen, 1);
        screen.clicked(&font, (x + 40.0, y));
        assert_eq!(screen.focus, Some(Focus::List(1)));
        assert_eq!(screen.selected[1], Some(Selected::Pack("file/a".into())));
        screen.refilter();
        assert_eq!(screen.selected, [None, None]);
    }

    #[test]
    fn a_pack_that_does_not_fit_is_asked_about_first() {
        let Some((font, lang)) = assets() else { return };
        let mut screen = screen(
            &font,
            &lang,
            vec![folder_pack("old", Compatibility::TooOld)],
        );
        let input = screen.clicked(&font, first_icon(&screen, 0));
        assert_eq!(
            input.confirm,
            Some(("file/old".to_owned(), Compatibility::TooOld))
        );
        assert_eq!(screen.model().list(Side::Unselected), ["file/old"]);
        screen.select_confirmed("file/old");
        assert_eq!(screen.model().list(Side::Selected), ["file/old", "vanilla"]);
    }

    #[test]
    fn the_search_keeps_the_packs_that_hold_the_text() {
        let Some((font, lang)) = assets() else { return };
        let mut screen = screen(
            &font,
            &lang,
            vec![
                folder_pack("Alpha", Compatibility::Compatible),
                folder_pack("Beta", Compatibility::Compatible),
            ],
        );
        assert_eq!(screen.shown(0).len(), 2);
        for letter in "alp".chars() {
            screen.char_typed(&font, letter);
        }
        let shown: Vec<&str> = screen
            .shown(0)
            .iter()
            .map(|view| view.id.as_str())
            .collect();
        assert_eq!(shown, ["file/Alpha"]);
    }

    #[test]
    fn done_waits_for_a_selected_pack() {
        let Some((font, lang)) = assets() else { return };
        let screen = screen(&font, &lang, Vec::new());
        // The game's own pack is always selected.
        assert!(screen.done.active);
    }
}
