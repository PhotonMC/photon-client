use super::*;
use crate::{FontRuns, FrameRuns};

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
    pub(super) fn selected_index(&self) -> Option<usize> {
        let code = self.selected.as_ref()?;
        self.shown
            .iter()
            .position(|&index| &self.languages[index].0 == code)
    }

    pub(super) fn content_height(&self) -> i32 {
        self.shown.len() as i32 * ENTRY_HEIGHT + 4
    }

    pub(super) fn set_scroll(&mut self, scroll: f64) {
        self.scroll = scroll.clamp(0.0, f64::from(self.place.max_scroll(self.content_height())));
    }

    /// Where the entry at `index` starts (`repositionEntries`).
    pub(super) fn entry_y(&self, index: usize) -> i32 {
        self.place.y + PADDING - self.scroll as i32 + index as i32 * ENTRY_HEIGHT
    }

    pub(super) fn row_left(&self) -> i32 {
        self.place.width / 2 - ROW_WIDTH / 2
    }

    pub(super) fn scrollbar_x(&self) -> i32 {
        self.row_left() + ROW_WIDTH + SCROLLBAR_WIDTH + 2
    }

    /// `repositionElements`: the header, the footer and the list again.
    pub(super) fn arrange(&mut self) {
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
    pub(super) fn scroll_to_entry(&mut self, index: usize) {
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

    pub(super) fn draw_list(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font) {
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
    pub(super) fn select(&mut self, position: usize) {
        self.selected = Some(self.languages[self.shown[position]].0.clone());
        let top = self.entry_y(position) + PADDING;
        let bottom = self.entry_y(position) + ENTRY_HEIGHT - PADDING;
        if top < self.place.y || bottom > self.place.bottom() {
            self.scroll_to_entry(position);
        }
    }

    /// `onDone`: the selected language, if it is another, and the screen
    /// goes.
    pub(super) fn finish(&self) -> LanguageInput {
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
            crate::ui_sound::play_down_sound();
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
    pub(super) fn set_focus(&mut self, focus: Option<Focus>) {
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
    pub(super) fn next_entry(&self, direction: Direction, start: Option<usize>) -> Option<usize> {
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
}
