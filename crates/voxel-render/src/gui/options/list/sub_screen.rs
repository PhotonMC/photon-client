use super::*;

impl SubScreen {
    /// `init` of a screen `size` big with the title `key` and an empty
    /// list, which `fill` adds the options to.
    pub fn new(
        font: &Font,
        lang: &Lang,
        size: (i32, i32),
        key: &str,
        in_world: bool,
        fill: impl FnOnce(&mut OptionsList),
    ) -> Self {
        let (width, height) = size;
        let mut arena = Arena::default();
        let text = lang.runs(&Component::translatable(key), &Style::default());
        let title = Widget {
            rect: arena.add(font.width_of_runs(&text), 9),
            active: false,
            kind: Kind::Text { text },
        };
        let done_label: Vec<Run> =
            lang.runs(&Component::translatable("gui.done"), &Style::default());
        let done = Widget {
            rect: arena.add(200, 20),
            active: true,
            kind: Kind::Button { label: done_label },
        };
        let mut screen = Self {
            arena,
            title,
            warning: None,
            done,
            list: OptionsList::new(
                width,
                (HEADER_AND_FOOTER, height - 2 * HEADER_AND_FOOTER),
                in_world,
            ),
            done_focused: false,
            size,
            in_world,
        };
        fill(&mut screen.list);
        screen.arrange();
        screen
    }

    /// `repositionElements`: the title in the header, Done in the footer,
    /// the list between.
    pub(super) fn arrange(&mut self) {
        let (width, height) = self.size;
        // The header: the title, and the line under it if there is one.
        let centred = Settings::default().align_horizontally(0.5);
        let mut header = Grid::new();
        header.row_spacing = 2;
        header.add(Item::Leaf(self.title.rect), (0, 0), (1, 1), centred);
        if let Some((warning, _)) = &self.warning {
            header.add(Item::Leaf(warning.rect), (1, 0), (1, 1), centred);
        }
        header.arrange(&mut self.arena);
        let frame = (
            0,
            0,
            width.max(header.width),
            HEADER_AND_FOOTER.max(header.height),
        );
        place_in_frame(
            &mut self.arena,
            Item::Grid(Box::new(header)),
            frame,
            (0.5, 0.5),
        );
        place_in_frame(
            &mut self.arena,
            Item::Leaf(self.done.rect),
            (0, height - HEADER_AND_FOOTER, width, HEADER_AND_FOOTER),
            (0.5, 0.5),
        );
        self.list
            .resize(width, (HEADER_AND_FOOTER, height - 2 * HEADER_AND_FOOTER));
    }

    /// A line of words in the header under the title, `line` being the
    /// runs and how wide they are, shown or hidden (`restartWarning`).
    /// Hidden, it still has its place.
    pub fn set_header_line(&mut self, (runs, width): (&[Run], i32), show: bool) {
        match &mut self.warning {
            Some((_, shown)) if *shown == show => return,
            Some((_, shown)) => *shown = show,
            None if show => {
                let rect = self.arena.add(width, 9);
                let widget = Widget {
                    rect,
                    active: false,
                    kind: Kind::Text {
                        text: runs.to_vec(),
                    },
                };
                self.warning = Some((widget, true));
            }
            None => return,
        }
        self.arrange();
    }

    /// The window changed its GUI size: the layout is made again and the
    /// list keeps what it holds.
    pub fn resize(&mut self, size: (i32, i32)) {
        self.size = size;
        self.arrange();
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    pub fn draw(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        let (width, height) = self.size;
        let background = if self.in_world {
            "inworld_menu_background"
        } else {
            "menu_background"
        };
        frame.tile_texture(atlas, background, (32, 32), (0, 0, width, height));
        self.title.draw(frame, atlas, font, &self.arena, false);
        if let Some((warning, true)) = &self.warning {
            warning.draw(frame, atlas, font, &self.arena, false);
        }
        let pending = self.list.draw(frame, atlas, font, mouse);
        let highlighted = self.done.contains(&self.arena, mouse) || self.done_focused;
        self.done.draw(frame, atlas, font, &self.arena, highlighted);
        if let Some(pending) = pending {
            pending.draw(frame, atlas, font, mouse, self.size);
        }
    }

    /// A press of the primary button: the list takes it if it is on it,
    /// else Done.
    pub fn clicked(&mut self, lang: &Lang, mouse: (f64, f64), shift: bool) -> SubScreenInput {
        let (took, changed) = self.list.clicked(lang, mouse, shift);
        if took {
            self.done_focused = false;
            return SubScreenInput {
                done: false,
                changed,
            };
        }
        let at = (mouse.0 as i32, mouse.1 as i32);
        let done = self.done.is_active() && self.done.contains(&self.arena, at);
        if done {
            self.apply(Target::Done, InputType::Mouse);
        }
        SubScreenInput {
            done,
            changed: None,
        }
    }

    pub fn dragged(&mut self, lang: &Lang, mouse: (f64, f64), dy: f64) -> Option<u8> {
        self.list.dragged(lang, mouse, dy)
    }

    /// The focus moves to `target`; what had it lets go.
    pub(super) fn apply(&mut self, target: Target, input: InputType) {
        self.done_focused = false;
        match target {
            Target::List(focus) => self.list.move_focus(Some(focus), input),
            Target::Done => {
                self.list.move_focus(None, input);
                self.done_focused = true;
            }
        }
    }

    pub(super) fn clear_focus(&mut self) {
        self.done_focused = false;
        self.list.move_focus(None, InputType::None);
    }

    /// `handleTabNavigation` of the screen, whose widgets in order are the
    /// title, the list and Done: the focused widget's own next first.
    pub(super) fn tab_target(&self, forward: bool) -> Option<Target> {
        if self.list.screen_focused()
            && let Some(focus) = self.list.next_focus(Nav::Tab(forward))
        {
            return Some(Target::List(focus));
        }
        let at = if self.list.screen_focused() {
            Some(0)
        } else {
            self.done_focused.then_some(1)
        };
        let start = match (at, forward) {
            (Some(at), true) => at + 1,
            (Some(at), false) => at,
            (None, true) => 0,
            (None, false) => 2,
        };
        let order: Vec<usize> = if forward {
            (start..2).collect()
        } else {
            (0..start).rev().collect()
        };
        order.into_iter().find_map(|child| match child {
            0 => self.list.next_focus(Nav::Tab(forward)).map(Target::List),
            _ => (self.done.is_active() && !self.done_focused).then_some(Target::Done),
        })
    }

    /// Tab (and Shift+Tab): the focus moves on, and past the last widget
    /// round to the first.
    pub fn tab(&mut self, forward: bool) {
        let mut target = self.tab_target(forward);
        if target.is_none() {
            self.clear_focus();
            target = self.tab_target(forward);
        }
        if let Some(target) = target {
            self.apply(target, InputType::KeyboardTab);
        }
    }

    /// `handleArrowNavigation` of the screen: the list's own step, else the
    /// widget that lies next to the focused one.
    pub(super) fn arrow(&mut self, direction: Direction) {
        let list_step = self.list.next_focus(Nav::Arrow(direction));
        if self.list.screen_focused()
            && let Some(focus) = list_step
        {
            self.apply(Target::List(focus), InputType::KeyboardArrow);
            return;
        }
        let rectangle = |rect: usize| {
            let rect = self.arena.0[rect];
            Rectangle {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
            }
        };
        // The title and the line under it never take the focus.
        let mut children = vec![Candidate {
            rectangle: rectangle(self.title.rect),
            focusable: false,
        }];
        if let Some((warning, _)) = &self.warning {
            children.push(Candidate {
                rectangle: rectangle(warning.rect),
                focusable: false,
            });
        }
        let list_at = children.len();
        children.push(Candidate {
            rectangle: self.list.rectangle(),
            focusable: list_step.is_some(),
        });
        children.push(Candidate {
            rectangle: rectangle(self.done.rect),
            focusable: self.done.is_active(),
        });
        let focused = if self.list.screen_focused() {
            Some(list_at)
        } else {
            self.done_focused.then_some(list_at + 1)
        };
        let container = Rectangle {
            x: 0,
            y: 0,
            width: self.size.0,
            height: self.size.1,
        };
        match next_in_direction(&children, focused, direction, container) {
            Some(at) if at == list_at => {
                if let Some(focus) = list_step {
                    self.apply(Target::List(focus), InputType::KeyboardArrow);
                }
            }
            Some(_) => self.apply(Target::Done, InputType::KeyboardArrow),
            None => {}
        }
    }

    /// `Screen.keyPressed` for a key other than Tab and Escape: the focused
    /// widget takes it, or an arrow moves the focus.
    pub fn key_pressed(&mut self, lang: &Lang, key: SliderKey, shift: bool) -> SubScreenInput {
        if self.list.screen_focused() {
            let (taken, changed) = self.list.key_pressed(lang, key, shift);
            if taken {
                return SubScreenInput {
                    done: false,
                    changed,
                };
            }
        } else if self.done_focused && key == SliderKey::Select && self.done.is_active() {
            return SubScreenInput {
                done: true,
                changed: None,
            };
        }
        match key {
            SliderKey::Left => self.arrow(Direction::Left),
            SliderKey::Right => self.arrow(Direction::Right),
            SliderKey::Up => self.arrow(Direction::Up),
            SliderKey::Down => self.arrow(Direction::Down),
            SliderKey::Select => {}
        }
        SubScreenInput::default()
    }

    pub fn released(&mut self, lang: &Lang) {
        self.list.released(lang);
    }
}
