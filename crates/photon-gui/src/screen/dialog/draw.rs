//! Drawing the dialog screen.

use super::*;

impl DialogScreen {
    /// The body's text and items, scrolled, with the focus outline on the part that has it.
    fn draw_parts(
        &self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        item: &mut ItemDrawer<'_>,
    ) {
        let c = self.container;
        for (index, part) in self.parts.iter().enumerate() {
            let rect = self.part_rect(part);
            if let Some(lines) = &part.lines {
                let middle = rect.x + rect.width / 2;
                for (line, runs) in lines.iter().enumerate() {
                    let x = middle - font.width_of_runs(runs) / 2;
                    let y = rect.y + PADDING + line as i32 * 9;
                    frame.styled_text(atlas, font, runs, x, y, 0xFFFF_FFFF, true);
                }
            }
            // `ItemDisplayWidget`: the item at the widget's corner. The items are not cut by the
            // box, so one it only partly shows is left out.
            if let Some(shown) = &part.item
                && rect.y >= c.y
                && rect.y + 16 <= c.y + c.height
            {
                let stack = photon_world::ClientItemStackState::new(shown.count, shown.id);
                if shown.decorations {
                    frame.item_with_count(
                        atlas,
                        font,
                        item,
                        &stack,
                        shown.count,
                        None,
                        rect.x,
                        rect.y,
                    );
                } else {
                    frame.item(item, &stack, rect.x, rect.y);
                }
            }
            if self.focus == Some(Focus::Part(index)) {
                outline(frame, atlas, rect);
            }
        }
    }

    /// The background, the title, the body in its box, the buttons, the warning button, and the
    /// tooltip of the one the mouse is on (or the keyboard has).
    pub fn draw(
        &self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        item: &mut ItemDrawer<'_>,
        (mouse, blurriness): ((i32, i32), i32),
    ) {
        let (width, height) = self.size;
        let background = menu_background(frame, self.in_world, blurriness);
        frame.tile_texture(atlas, background, (32, 32), (0, 0, width, height));
        self.title.draw(frame, atlas, font, &self.arena, false);
        self.draw_box(frame, (atlas, font), item, mouse);
        for (index, (widget, _)) in self.buttons.iter().enumerate().skip(self.listed) {
            let highlighted =
                self.button_hit(index, mouse) || self.focus == Some(Focus::Button(index));
            widget.draw(frame, atlas, font, &self.arena, highlighted);
        }
        let warned = self.warning.contains(&self.arena, mouse);
        let focused = self.focus == Some(Focus::Warning);
        let sprite = if warned || focused {
            "dialog/warning_button_highlighted"
        } else {
            "dialog/warning_button"
        };
        let rect = self.arena.0[self.warning.rect];
        frame.sprite(atlas, sprite, rect.x, rect.y, rect.width, rect.height);
        if let Some(pending) = self.tooltip(frame.millis, mouse) {
            pending.draw(frame, atlas, font, mouse, self.size);
        }
    }

    /// The body in its box: the parts, the controls and the buttons it scrolls, cut to the box,
    /// and the scrollbar.
    fn draw_box(
        &self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        item: &mut ItemDrawer<'_>,
        mouse: (i32, i32),
    ) {
        let c = self.container;
        let mark = frame.mark();
        self.draw_parts(frame, (atlas, font), item);
        for (index, control) in self.controls.iter().enumerate() {
            let focused = self.focus == Some(Focus::Input(index));
            control.draw(frame, (atlas, font), self.control_rect(index), focused);
        }
        // The body's buttons are drawn where the scrolling puts them, inside the box.
        let mut shifted = self.arena.clone();
        for (widget, _) in &self.buttons[..self.listed] {
            shifted.0[widget.rect].y -= self.scroll as i32;
        }
        for index in 0..self.listed {
            let highlighted =
                self.button_hit(index, mouse) || self.focus == Some(Focus::Button(index));
            self.buttons[index]
                .0
                .draw(frame, atlas, font, &shifted, highlighted);
        }
        // The scissor stack starts as the screen, so a box wider than it is cut there too.
        let (width, height) = self.size;
        frame.clip_from(
            mark,
            (
                c.x.max(0),
                c.y.max(0),
                (c.x + c.width).min(width),
                (c.y + c.height).min(height),
            ),
        );
        self.place().draw_scrollbar(
            frame,
            atlas,
            c.x + c.width - SCROLLBAR_WIDTH,
            self.scroll,
            self.content_height,
        );
    }

    /// The tooltip to show: a focused one wins over one the mouse is on.
    fn tooltip(&self, millis: u64, mouse: (i32, i32)) -> Option<Pending> {
        let mut pending: Option<Pending> = None;
        let mut ask = |tooltip: &Holder, rectangle: Rectangle, hovered: bool, focused: bool| {
            if let Some(asked) =
                tooltip.refresh(millis, (hovered, focused), self.input.get(), rectangle)
                && (pending.is_none() || focused)
            {
                pending = Some(asked);
            }
        };
        for (index, (_, tooltip)) in self.buttons.iter().enumerate() {
            let hovered = self.button_hit(index, mouse);
            let focused = self.focus == Some(Focus::Button(index));
            ask(tooltip, self.button_rect(index), hovered, focused);
        }
        let rect = self.arena.0[self.warning.rect];
        let rectangle = Rectangle {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        };
        let warned = self.warning.contains(&self.arena, mouse);
        ask(
            &self.warning_tooltip,
            rectangle,
            warned,
            self.focus == Some(Focus::Warning),
        );
        pending
    }
}
