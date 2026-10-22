//! The mouse, keys and focus of the dialog screen.

use super::*;

/// Whether `mouse` is inside `rect`.
pub(super) fn contains(rect: Rectangle, (x, y): (i32, i32)) -> bool {
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
    pub fn clicked(&mut self, mouse: (i32, i32), font: &Font) -> Option<Press> {
        self.input.set(InputType::Mouse);
        let c = self.container;
        if self.in_container(mouse) {
            self.dragging = self.max_scroll() > 0 && mouse.0 >= c.x + c.width - SCROLLBAR_WIDTH;
            if self.clicked_body(mouse, font) {
                return None;
            }
        }
        if let Some(index) = (0..self.buttons.len()).find(|&index| self.button_hit(index, mouse)) {
            self.set_focus(Some(Focus::Button(index)));
            crate::ui_sound::play_down_sound();
            return Some(Press::Button(index));
        }
        if self.in_container(mouse) {
            return None;
        }
        if self.warning.contains(&self.arena, mouse) {
            self.set_focus(Some(Focus::Warning));
            crate::ui_sound::play_down_sound();
            return Some(Press::Warning);
        }
        None
    }

    /// A click on a part or a control of the body, which takes the focus (and a control changes).
    fn clicked_body(&mut self, mouse: (i32, i32), font: &Font) -> bool {
        if let Some(index) = self
            .parts
            .iter()
            .position(|part| contains(self.part_rect(part), mouse))
        {
            self.set_focus(Some(Focus::Part(index)));
            return true;
        }
        let control = (0..self.controls.len()).find(|&index| contains(self.hit_rect(index), mouse));
        if let Some(index) = control {
            self.set_focus(Some(Focus::Input(index)));
            let rect = self.control_rect(index);
            self.controls[index].clicked(mouse, rect, (self.shift.get(), font));
            return true;
        }
        false
    }

    /// `setFocused`: the control that loses the focus and the one that gains it are told.
    fn set_focus(&mut self, focus: Option<Focus>) {
        if self.focus == focus {
            return;
        }
        if let Some(Focus::Input(index)) = self.focus {
            self.controls[index].focus(false);
        }
        if let Some(Focus::Input(index)) = focus {
            self.controls[index].focus(true);
        }
        self.focus = focus;
    }

    /// Where the mouse and the focus find control `index`, scrolled.
    fn hit_rect(&self, index: usize) -> Rectangle {
        self.controls[index].hit_rect(self.control_rect(index))
    }

    /// A key the focused text box edits with: it takes the key before the screen does.
    pub fn edit_key(
        &mut self,
        font: &Font,
        key: glfw::Key,
        modifiers: (bool, bool),
        clipboard: &mut dyn Clipboard,
    ) -> bool {
        match self.focus {
            Some(Focus::Input(index)) => self.controls[index].edit(font, key, modifiers, clipboard),
            _ => false,
        }
    }

    /// A character typed into the focused text box. Whether it took it.
    pub fn char_typed(&mut self, font: &Font, character: char) -> bool {
        match self.focus {
            Some(Focus::Input(index)) => self.controls[index].char_typed(font, character),
            _ => false,
        }
    }

    /// Whether Shift is down for the next key or click.
    pub fn set_shift(&self, shift: bool) {
        self.shift.set(shift);
    }

    /// An arrow key: the focused control moves, if it takes it.
    pub fn arrow(&mut self, right: bool) -> bool {
        let Some(Focus::Input(index)) = self.focus else {
            return false;
        };
        let width = self.arena.0[self.controls[index].rect].width;
        self.controls[index].arrow(right, width)
    }

    /// The primary button is let go.
    pub fn released(&mut self) {
        self.dragging = false;
        for control in &mut self.controls {
            control.released();
        }
    }

    /// The mouse moved with the primary button down: `x` and `y` are where it is, `dy` how far it
    /// went up or down.
    pub fn dragged(&mut self, (x, y): (f64, f64), dy: f64, font: &Font) {
        for index in 0..self.controls.len() {
            let rect = self.control_rect(index);
            self.controls[index].dragged((x as i32, y as i32), rect, (font, self.shift.get()));
        }
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
        self.set_focus(Some(order[next_in(order.len(), at, forward)]));
        self.scroll_focus_into_view();
    }

    /// A focused part the box hides is scrolled into view.
    fn scroll_focus_into_view(&mut self) {
        let shown = match self.focus {
            Some(Focus::Part(index)) => Some(self.part_rect(&self.parts[index])),
            Some(Focus::Input(index)) => Some(self.hit_rect(index)),
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
            Focus::Button(index) => {
                crate::ui_sound::play_down_sound();
                Some(Press::Button(index))
            }
            Focus::Warning => {
                crate::ui_sound::play_down_sound();
                Some(Press::Warning)
            }
            Focus::Part(_) => None,
            Focus::Input(index) => {
                self.controls[index].press(self.shift.get());
                None
            }
        }
    }

    /// What the controls hold now, for the action a button runs (`Action.createAction`).
    pub fn values(&self) -> Vec<photon_game::dialog::InputValue> {
        self.controls.iter().map(Control::value).collect()
    }

    /// The widgets of the screen this one replaces on a resize live on (`repositionElements` only
    /// arranges them again): the controls keep what they held, the focus stays where it was and
    /// the scroll is clamped to the new box.
    pub fn keep_inputs_of(&mut self, old: &Self) {
        for (control, before) in self.controls.iter_mut().zip(&old.controls) {
            control.keep(before);
        }
        self.focus = old.focus;
        self.input.set(old.input.get());
        self.set_scroll(old.scroll);
    }

    /// Escape, when the dialog lets it close.
    pub fn escape(&self) -> Option<Press> {
        self.can_close.then_some(Press::Close)
    }
}
