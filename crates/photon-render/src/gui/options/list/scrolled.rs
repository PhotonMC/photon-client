use super::*;

impl OptionsList {
    /// A press of the primary button at `mouse`: whether the list took it
    /// (it landed on a widget or the scroll bar), and the tag of the widget
    /// whose option changed.
    pub fn clicked(&mut self, lang: &Lang, mouse: (f64, f64), shift: bool) -> (bool, Option<u8>) {
        if !self.place.contains(mouse) {
            return (false, None);
        }
        self.scrolling = self.max_scroll() > 0
            && mouse.0 >= f64::from(self.scrollbar_x())
            && mouse.0 <= f64::from(self.scrollbar_x() + SCROLLBAR_WIDTH)
            && mouse.1 >= f64::from(self.place.y)
            && mouse.1 < f64::from(self.place.bottom());
        let hit = self.entry_at(mouse).and_then(|entry| {
            let Content::Row(children) = &self.entries[entry].content else {
                return None;
            };
            children
                .iter()
                .position(|child| child.control.is_mouse_over(&self.arena, mouse))
                .map(|child| (entry, child))
        });
        let Some((entry, child)) = hit else {
            // An entry (or the bar) takes the click, so the list takes the
            // screen's focus, though no widget of it has taken any.
            let took = self.scrolling || self.entry_at(mouse).is_some();
            self.screen_focused |= took;
            return (took, None);
        };
        self.screen_focused = true;
        let mut changed = None;
        if let Content::Row(children) = &mut self.entries[entry].content {
            let child = &mut children[child];
            if child.control.pressed(lang, &self.arena, mouse, shift) {
                changed = Some(child.tag);
            }
        }
        self.input = InputType::Mouse;
        self.focus_on(Some((entry, child)), InputType::Mouse);
        self.dragging = true;
        (true, changed)
    }

    /// The mouse moved to `(x, y)`, by `dy` down, with the primary button
    /// down.
    pub fn dragged(&mut self, lang: &Lang, (x, y): (f64, f64), dy: f64) -> Option<u8> {
        if self.scrolling {
            let to = self
                .place
                .dragged_scroll(self.scroll, y, dy, self.content_height());
            self.set_scroll_amount(to);
        }
        if self.dragging
            && let Some((entry, child)) = self.focus
            && let Content::Row(children) = &mut self.entries[entry].content
        {
            let child = &mut children[child];
            if child.control.dragged(lang, &self.arena, x) {
                return Some(child.tag);
            }
        }
        None
    }

    /// The primary button let go.
    pub fn released(&mut self, lang: &Lang) {
        self.scrolling = false;
        if self.dragging {
            self.dragging = false;
            if let Some((entry, child)) = self.focus
                && let Content::Row(children) = &mut self.entries[entry].content
            {
                children[child].control.released(lang);
            }
        }
    }

    /// `AbstractScrollArea.mouseScrolled`: the wheel moves the list the
    /// mouse is over.
    pub fn scrolled(&mut self, mouse: (f64, f64), scroll_y: f64) -> bool {
        if !self.place.contains(mouse) {
            return false;
        }
        self.set_scroll_amount(self.scroll - scroll_y * SCROLL_RATE);
        true
    }
}
