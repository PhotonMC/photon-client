use super::*;

impl OptionsList {
    /// A list with no entries, for a screen `width` wide whose list stands
    /// `y` down and is `height` tall.
    pub fn new(width: i32, (y, height): (i32, i32), in_world: bool) -> Self {
        Self {
            arena: Arena::default(),
            place: Place {
                y,
                width,
                height,
                in_world,
            },
            entries: Vec::new(),
            scroll: 0.0,
            scrolling: false,
            focus: None,
            screen_focused: false,
            dragging: false,
            input: InputType::Mouse,
        }
    }

    /// `addHeader`: a line of text, with room above it unless it is first.
    pub fn add_header(&mut self, font: &Font, lang: &Lang, key: &str) {
        let padding_top = if self.entries.is_empty() { 0 } else { 9 * 2 };
        let text = lang.runs(&Component::translatable(key), &Style::default());
        let rect = self.arena.add(font.width_of_runs(&text), 9);
        self.push(Entry {
            y: 0,
            height: padding_top + 9 + 4,
            content: Content::Header {
                text: Widget {
                    rect,
                    active: false,
                    kind: Kind::Text { text },
                },
                padding_top,
            },
        });
    }

    /// `addBig`: one wide widget.
    pub fn add_big(&mut self, lang: &Lang, tag: u8, build: Build) {
        let rect = self.arena.add(ROW_WIDTH, 20);
        let control = build(lang, rect);
        self.push(Entry {
            y: 0,
            height: ITEM_HEIGHT,
            content: Content::Row(vec![Child { tag, rect, control }]),
        });
    }

    /// `addSmall`: the widgets two to a row, the last row with one if they
    /// are odd.
    pub fn add_small(&mut self, lang: &Lang, builds: Vec<(u8, Build)>) {
        let mut builds = builds.into_iter();
        while let Some((tag, build)) = builds.next() {
            let mut children = Vec::new();
            for (tag, build) in std::iter::once((tag, build)).chain(builds.next()) {
                let rect = self.arena.add(SMALL_WIDTH, 20);
                children.push(Child {
                    tag,
                    rect,
                    control: build(lang, rect),
                });
            }
            self.push(Entry {
                y: 0,
                height: ITEM_HEIGHT,
                content: Content::Row(children),
            });
        }
    }

    pub(super) fn push(&mut self, entry: Entry) {
        self.entries.push(entry);
        self.reposition();
    }

    /// `findOption`: the widget an option was added with.
    pub fn find(&self, tag: u8) -> Option<&dyn Control> {
        self.children()
            .find(|child| child.tag == tag)
            .map(|child| &*child.control)
    }

    pub fn find_mut(&mut self, tag: u8) -> Option<&mut dyn Control> {
        for entry in &mut self.entries {
            if let Content::Row(children) = &mut entry.content {
                for child in children {
                    if child.tag == tag {
                        return Some(&mut *child.control);
                    }
                }
            }
        }
        None
    }

    pub(super) fn children(&self) -> impl Iterator<Item = &Child> {
        self.entries.iter().flat_map(|entry| match &entry.content {
            Content::Row(children) => children.iter(),
            Content::Header { .. } => [].iter(),
        })
    }

    /// `getRowLeft`.
    pub(super) fn row_left(&self) -> i32 {
        self.place.width / 2 - ROW_WIDTH / 2
    }

    /// `contentHeight`.
    pub(super) fn max_scroll(&self) -> i32 {
        self.place.max_scroll(self.content_height())
    }

    /// `contentHeight`.
    pub(super) fn content_height(&self) -> i32 {
        self.entries.iter().map(|entry| entry.height).sum::<i32>() + 4
    }

    pub fn scroll_amount(&self) -> f64 {
        self.scroll
    }

    /// `setScrollAmount`: kept between the top and the end, the entries
    /// moved with it (the widgets with them).
    pub fn set_scroll_amount(&mut self, scroll: f64) {
        self.scroll = scroll.clamp(0.0, f64::from(self.max_scroll()));
        self.reposition();
    }

    /// `repositionEntries`, and the widgets where `extractContent` puts them.
    pub(super) fn reposition(&mut self) {
        let mut y = self.place.y + PADDING - self.scroll as i32;
        let x = self.place.width / 2 - 155;
        for entry in &mut self.entries {
            entry.y = y;
            y += entry.height;
            match &entry.content {
                Content::Header { text, padding_top } => {
                    let rect = &mut self.arena.0[text.rect];
                    (rect.x, rect.y) = (x, entry.y + PADDING + padding_top);
                }
                Content::Row(children) => {
                    for (index, child) in children.iter().enumerate() {
                        let rect = &mut self.arena.0[child.rect];
                        (rect.x, rect.y) = (x + COLUMN_STEP * index as i32, entry.y + PADDING);
                    }
                }
            }
        }
    }

    /// `updateSizeAndPosition`: a screen of another size.
    pub fn resize(&mut self, width: i32, (y, height): (i32, i32)) {
        self.place.width = width;
        self.place.y = y;
        self.place.height = height;
        self.reposition();
        if let Some((entry, _)) = self.focus {
            self.scroll_to_entry(entry);
        }
        self.set_scroll_amount(self.scroll);
    }

    /// `scrollToEntry`.
    pub(super) fn scroll_to_entry(&mut self, entry: usize) {
        let top_delta = self.entries[entry].y - self.place.y - PADDING;
        if top_delta < 0 {
            self.set_scroll_amount(self.scroll + f64::from(top_delta));
        }
        let bottom_delta =
            self.place.bottom() - self.entries[entry].y - self.entries[entry].height - PADDING;
        if bottom_delta < 0 {
            self.set_scroll_amount(self.scroll - f64::from(bottom_delta));
        }
    }

    /// `scrollBarX`.
    pub(super) fn scrollbar_x(&self) -> i32 {
        self.row_left() + ROW_WIDTH + SCROLLBAR_WIDTH + 2
    }

    /// The menu list's background, scrolled with the list, the widgets in
    /// it cut to the list, the lines above and below, then the scroll bar.
    /// The tooltip a widget asks for, to draw over everything.
    pub fn draw(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        mouse: (i32, i32),
    ) -> Option<Pending> {
        let place = self.place;
        place.draw_background(frame, atlas, self.scroll);
        // The hover of a widget needs the mouse inside the cut.
        let inside = place.contains((f64::from(mouse.0), f64::from(mouse.1)));
        let mouse = if inside { mouse } else { (i32::MIN, i32::MIN) };
        let mark = frame.mark();
        let mut pending: Option<Pending> = None;
        for (entry_index, entry) in self.entries.iter().enumerate() {
            if entry.y + entry.height < place.y || entry.y > place.bottom() {
                continue;
            }
            match &entry.content {
                Content::Header { text, .. } => text.draw(frame, atlas, font, &self.arena, false),
                Content::Row(children) => {
                    for (child_index, child) in children.iter().enumerate() {
                        child.control.draw(frame, atlas, font, &self.arena, mouse);
                        let Some(holder) = child.control.tooltip() else {
                            continue;
                        };
                        let rect = self.arena.0[child.rect];
                        let hovered = inside
                            && mouse.0 >= rect.x
                            && mouse.0 < rect.x + rect.width
                            && mouse.1 >= rect.y
                            && mouse.1 < rect.y + rect.height;
                        let focused = self.focus == Some((entry_index, child_index));
                        let widget = Rectangle {
                            x: rect.x,
                            y: rect.y,
                            width: rect.width,
                            height: rect.height,
                        };
                        if let Some(asked) =
                            holder.refresh(frame.millis, (hovered, focused), self.input, widget)
                            && (pending.is_none() || focused)
                        {
                            pending = Some(asked);
                        }
                    }
                }
            }
        }
        frame.clip_from(mark, (0, place.y, place.width, place.bottom()));
        place.draw_edges(
            frame,
            atlas,
            (self.scroll, self.content_height()),
            self.scrollbar_x(),
        );
        pending
    }

    /// `getEntryAtPosition`.
    pub(super) fn entry_at(&self, (x, y): (f64, f64)) -> Option<usize> {
        let (x, y) = (x as i32, y as i32);
        let left = self.row_left();
        self.entries.iter().position(|entry| {
            x >= left && x < left + ROW_WIDTH && y >= entry.y && y < entry.y + entry.height
        })
    }

    /// `ContainerEventHandler.setFocused` down the list: the widget that had
    /// the focus loses it, and an entry that sticks out of the list is
    /// scrolled into it (`setSelected`).
    pub(super) fn focus_on(&mut self, focus: Option<Focus>, input: InputType) {
        let old = self.focus;
        if old == focus {
            return;
        }
        if let Some((entry, child)) = old
            && let Content::Row(children) = &mut self.entries[entry].content
        {
            children[child].control.set_focused(false, input);
        }
        self.focus = focus;
        if let Some((entry, child)) = focus {
            if let Content::Row(children) = &mut self.entries[entry].content {
                children[child].control.set_focused(true, input);
            }
            let changed_entry = !matches!(old, Some((before, _)) if before == entry);
            let top = self.entries[entry].y + PADDING;
            let bottom = self.entries[entry].y + self.entries[entry].height - PADDING;
            let keyboard = matches!(input, InputType::KeyboardArrow | InputType::KeyboardTab);
            if changed_entry && (keyboard || top < self.place.y || bottom > self.place.bottom()) {
                self.scroll_to_entry(entry);
            }
        }
    }

    /// Whether the screen's focus is on the list.
    pub fn screen_focused(&self) -> bool {
        self.screen_focused
    }

    /// How many widgets an entry can give the focus to (a header's one is
    /// a text, which never takes it).
    pub(super) fn widgets_in(&self, entry: usize) -> usize {
        match &self.entries[entry].content {
            Content::Row(children) => children.len(),
            Content::Header { .. } => 0,
        }
    }

    pub(super) fn is_active(&self, (entry, child): Focus) -> bool {
        matches!(&self.entries[entry].content,
            Content::Row(children) if children[child].control.is_active())
    }

    /// The first active widget of an entry among `order`.
    pub(super) fn first_active(
        &self,
        entry: usize,
        order: impl Iterator<Item = usize>,
    ) -> Option<Focus> {
        order
            .map(|child| (entry, child))
            .find(|&at| self.is_active(at))
    }

    /// `ContainerObjectSelectionList.nextFocusPath`: where the focus would
    /// go, given the widget that has it (if any).
    pub fn next_focus(&self, nav: Nav) -> Option<Focus> {
        if self.entries.is_empty() {
            return None;
        }
        match nav {
            Nav::Tab(forward) => self.tab_focus(forward),
            Nav::Arrow(direction) => self.arrow_focus(direction),
        }
    }

    /// Tab: on along the entry's widgets, then the entries after (or
    /// before) it, a header having none.
    pub(super) fn tab_focus(&self, forward: bool) -> Option<Focus> {
        if let Some((entry, child)) = self.focus {
            let count = self.widgets_in(entry);
            let found = if forward {
                self.first_active(entry, child + 1..count)
            } else {
                self.first_active(entry, (0..child).rev())
            };
            if found.is_some() {
                return found;
            }
        }
        let entries: Vec<usize> = match (self.focus, forward) {
            (Some((entry, _)), true) => (entry + 1..self.entries.len()).collect(),
            (Some((entry, _)), false) => (0..entry).rev().collect(),
            (None, true) => (0..self.entries.len()).collect(),
            (None, false) => (0..self.entries.len()).rev().collect(),
        };
        entries.into_iter().find_map(|entry| {
            let count = self.widgets_in(entry);
            if forward {
                self.first_active(entry, 0..count)
            } else {
                self.first_active(entry, (0..count).rev())
            }
        })
    }

    /// An arrow: left and right along the entry; up and down to the entry
    /// above or below, at the same place in it (or the last one it has).
    pub(super) fn arrow_focus(&self, direction: Direction) -> Option<Focus> {
        let horizontal = matches!(direction, Direction::Left | Direction::Right);
        if let (true, Some((entry, child))) = (horizontal, self.focus) {
            let count = self.widgets_in(entry);
            return if direction == Direction::Right {
                self.first_active(entry, child + 1..count)
            } else {
                self.first_active(entry, (0..child).rev())
            };
        }
        // With no focus left enters at the end of the first entry, right at
        // its start.
        let (index, direction) = match (self.focus, direction) {
            (Some((_, child)), direction) => (child, direction),
            (None, Direction::Left) => (usize::MAX, Direction::Down),
            (None, Direction::Right) => (0, Direction::Down),
            (None, direction) => (0, direction),
        };
        let down = direction == Direction::Down;
        let mut entry = self.focus.map(|(entry, _)| entry);
        loop {
            entry = match (entry, down) {
                (None, true) => Some(0),
                (None, false) => self.entries.len().checked_sub(1),
                (Some(entry), true) => Some(entry + 1),
                (Some(entry), false) => entry.checked_sub(1),
            }
            .filter(|entry| *entry < self.entries.len());
            let entry = entry?;
            let count = self.widgets_in(entry);
            if count > 0 {
                let at = (entry, index.min(count - 1));
                if self.is_active(at) {
                    return Some(at);
                }
            }
        }
    }

    /// A key for the widget that has the focus: whether it took it, and the
    /// tag of the widget whose option changed.
    pub fn key_pressed(&mut self, lang: &Lang, key: SliderKey, shift: bool) -> (bool, Option<u8>) {
        let Some((entry, child)) = self.focus else {
            return (false, None);
        };
        let Content::Row(children) = &mut self.entries[entry].content else {
            return (false, None);
        };
        let child = &mut children[child];
        let (taken, changed) = child.control.key_pressed(lang, &self.arena, key, shift);
        (taken, changed.then_some(child.tag))
    }

    /// The focus moves by a key: the widget before loses it, the new one
    /// takes it and the list shows it.
    pub fn move_focus(&mut self, focus: Option<Focus>, input: InputType) {
        self.focus_on(None, input);
        self.focus_on(focus, input);
        self.screen_focused = focus.is_some();
        self.input = input;
    }

    /// The list's own rectangle, which arrow navigation looks at.
    pub fn rectangle(&self) -> Rectangle {
        Rectangle {
            x: 0,
            y: self.place.y,
            width: self.place.width,
            height: self.place.height,
        }
    }
}
