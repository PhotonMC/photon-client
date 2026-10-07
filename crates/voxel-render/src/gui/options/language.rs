//! The language screen (`LanguageSelectScreen`): a title over a search box,
//! the languages the packs name in a list, and under it a warning, the font
//! button and Done.

use std::time::Instant;

use voxel_game::language::LanguageInfo;
use voxel_game::text::{Component, Lang, Run, Style};

use crate::gui::font::Font;
use crate::gui::layout::{Arena, Grid, Item, Settings, horizontal, place_in_frame};
use crate::gui::widgets::edit_box::{Clipboard, EditBox, edit_key};
use crate::gui::widgets::navigation::{Candidate, Direction, Rectangle, next_in_direction};
use crate::gui::widgets::scroll_area::{Place, SCROLLBAR_WIDTH};
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

mod root;
mod scroll;

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
