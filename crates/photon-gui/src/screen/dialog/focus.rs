//! The mouse, keys and focus of the dialog screen.

use super::*;

/// Whether `mouse` is inside `rect`.
fn contains(rect: Rectangle, (x, y): (i32, i32)) -> bool {
    x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
}

/// The place after (or before) `at` in a ring of `len` places; none yet starts at an end.
fn next_in(len: usize, at: Option<usize>, forward: bool) -> usize {
    match (at, forward) {
        (None, true) => 0,
        (None, false) => len - 1,
        (Some(at), true) => (at + 1) % len,
        (Some(at), false) => (at + len - 1) % len,
    }
}

impl DialogScreen {
    /// A press of the primary button: the scrollbar taken hold of, a part of the body or a button
    /// (which takes the focus), and for the buttons what they do.
    pub fn clicked(&mut self, mouse: (i32, i32)) -> Option<Press> {
        self.input.set(InputType::Mouse);
        let c = self.container;
        if self.in_container(mouse) {
            self.dragging = self.max_scroll() > 0 && mouse.0 >= c.x + c.width - SCROLLBAR_WIDTH;
            if self.clicked_body(mouse) {
                return None;
            }
        }
        if let Some(index) = (0..self.buttons.len()).find(|&index| self.button_hit(index, mouse)) {
            self.focus = Some(Focus::Button(index));
            return Some(Press::Button(index));
        }
        if self.in_container(mouse) {
            return None;
        }
        if self.warning.contains(&self.arena, mouse) {
            self.focus = Some(Focus::Warning);
            return Some(Press::Warning);
        }
        None
    }

    /// A click on a part or a control of the body, which takes the focus (and a control changes).
    fn clicked_body(&mut self, mouse: (i32, i32)) -> bool {
        if let Some(index) = self
            .parts
            .iter()
            .position(|part| contains(self.part_rect(part), mouse))
        {
            self.focus = Some(Focus::Part(index));
            return true;
        }
        let control =
            (0..self.controls.len()).find(|&index| contains(self.control_rect(index), mouse));
        if let Some(index) = control {
            self.focus = Some(Focus::Input(index));
            self.controls[index].press();
            return true;
        }
        false
    }

    /// The primary button is let go.
    pub fn released(&mut self) {
        self.dragging = false;
    }

    /// The mouse moved with the scrollbar held: `y` is where it is, `dy` how far it went.
    pub fn dragged(&mut self, y: f64, dy: f64) {
        if self.dragging {
            let scroll = self
                .place()
                .dragged_scroll(self.scroll, y, dy, self.content_height);
            self.set_scroll(scroll);
        }
    }

    /// The wheel over the box (`AbstractScrollArea.mouseScrolled`).
    pub fn scrolled(&mut self, mouse: (i32, i32), amount: f64) {
        if self.in_container(mouse) {
            self.set_scroll(self.scroll - amount * SCROLL_RATE);
        }
    }

    fn set_scroll(&mut self, scroll: f64) {
        self.scroll = scroll.clamp(0.0, f64::from(self.max_scroll()));
    }

    fn order(&self) -> Vec<Focus> {
        let mut order = vec![Focus::Warning];
        order.extend((0..self.parts.len()).map(Focus::Part));
        order.extend((0..self.controls.len()).map(Focus::Input));
        order.extend((0..self.buttons.len()).map(Focus::Button));
        order
    }

    /// Tab (and Shift+Tab): the focus goes to the next part; at the end it starts over. A part
    /// the box hides is scrolled into view (`ScrollableLayout.Container.setFocused`).
    pub fn tab(&mut self, forward: bool) {
        self.input.set(InputType::KeyboardTab);
        let order = self.order();
        let at = self
            .focus
            .and_then(|focus| order.iter().position(|&other| other == focus));
        self.focus = Some(order[next_in(order.len(), at, forward)]);
        self.scroll_focus_into_view();
    }

    /// A focused part the box hides is scrolled into view.
    fn scroll_focus_into_view(&mut self) {
        let shown = match self.focus {
            Some(Focus::Part(index)) => Some(self.part_rect(&self.parts[index])),
            Some(Focus::Input(index)) => Some(self.control_rect(index)),
            Some(Focus::Button(index)) if index < self.listed => Some(self.button_rect(index)),
            _ => None,
        };
        let Some(rect) = shown else {
            return;
        };
        let c = self.container;
        let (top, bottom) = (rect.y - c.y, rect.y + rect.height - (c.y + c.height));
        if top < 0 {
            self.set_scroll(self.scroll + f64::from(top) - SCROLL_RATE);
        } else if bottom > 0 {
            self.set_scroll(self.scroll + f64::from(bottom) + SCROLL_RATE);
        }
    }

    /// Enter or Space: the focused button, or a control changed.
    pub fn pressed(&mut self) -> Option<Press> {
        match self.focus? {
            Focus::Button(index) => Some(Press::Button(index)),
            Focus::Warning => Some(Press::Warning),
            Focus::Part(_) => None,
            Focus::Input(index) => {
                self.controls[index].press();
                None
            }
        }
    }

    /// What the controls hold now, for the action a button runs (`Action.createAction`).
    pub fn values(&self) -> Vec<photon_game::dialog::InputValue> {
        self.controls.iter().map(Control::value).collect()
    }

    /// The controls of the screen this one replaces on a resize keep what they held.
    pub fn keep_inputs_of(&mut self, old: &Self) {
        for (control, before) in self.controls.iter_mut().zip(&old.controls) {
            control.keep(before);
        }
    }

    /// Escape, when the dialog lets it close.
    pub fn escape(&self) -> Option<Press> {
        self.can_close.then_some(Press::Close)
    }
}
