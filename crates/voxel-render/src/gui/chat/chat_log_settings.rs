use super::*;

impl ChatLog {
    /// `ChatComponent.tick`: the clock, and the deletions that have waited
    /// long enough. `marker` is the deleted message's text.
    pub fn tick(&mut self, font: &Font, marker: &[Run]) {
        self.ticks += 1;
        if self.deletions.is_empty() {
            return;
        }
        let now = self.ticks;
        let waiting = std::mem::take(&mut self.deletions);
        for (signature, after) in waiting {
            if now >= after {
                if let Some(delayed) = self.delete_or_delay(&signature, font, marker) {
                    self.deletions.push(delayed);
                }
            } else {
                self.deletions.push((signature, after));
            }
        }
    }

    pub fn ticks(&self) -> u64 {
        self.ticks
    }

    pub fn settings(&self) -> ChatSettings {
        self.settings
    }

    /// A chat with no messages that wraps for these settings.
    pub fn with_settings(settings: ChatSettings) -> Self {
        Self {
            settings,
            ..Self::default()
        }
    }

    /// The options changed: `rescaleChat` takes the chat back to the bottom
    /// and wraps its messages again for the new width and scale.
    pub fn set_settings(&mut self, settings: ChatSettings, font: &Font) {
        if settings != self.settings {
            self.settings = settings;
            self.rescale(font);
        }
    }

    /// `rescaleChat`.
    pub fn rescale(&mut self, font: &Font) {
        self.reset_scroll();
        self.refresh(font);
    }

    /// `getLineHeight`: a message's line and the space between lines.
    pub(super) fn entry_height(&self) -> i32 {
        (f64::from(MESSAGE_HEIGHT) * (self.settings.line_spacing + 1.0)) as i32
    }

    /// `getLinesPerPage`: the lines a chat shows, an open one more than a
    /// closed one.
    pub fn lines_per_page(&self, focused: bool) -> usize {
        let share = if focused {
            self.settings.height_focused
        } else {
            self.settings.height_unfocused
        };
        (height_of(share) / self.entry_height()).max(0) as usize
    }

    /// `ChatComponent.addMessage`: the message kept and wrapped, newest
    /// first. A chat that is scrolled back keeps what it shows.
    pub fn add(&mut self, runs: &[Run], font: &Font, focused: bool) {
        self.add_message(runs, None, None, font, focused);
    }

    /// `addMessage` of a component, which is worded in `lang` now and again
    /// when the language changes.
    pub fn add_component(
        &mut self,
        component: &Component,
        lang: &Lang,
        (signature, tag): (Option<Vec<u8>>, Option<Tag>),
        font: &Font,
        focused: bool,
    ) {
        self.add_message(
            &lang.runs(component, &Style::default()),
            signature,
            tag,
            font,
            focused,
        );
        if let Some(message) = self.messages.front_mut() {
            message.source = Some(component.clone());
        }
    }

    /// The language changed: the messages made of a component are worded
    /// again, and the lines wrapped anew (`rescaleChat` on a reload).
    pub fn set_lang(&mut self, lang: &Lang, font: &Font) {
        for message in &mut self.messages {
            if let Some(source) = &message.source {
                message.runs = lang.runs(source, &Style::default());
            }
        }
        self.rescale(font);
    }

    /// `addMessage` with a signature and a tag.
    pub fn add_message(
        &mut self,
        runs: &[Run],
        signature: Option<Vec<u8>>,
        tag: Option<Tag>,
        font: &Font,
        focused: bool,
    ) {
        let message = Message {
            runs: runs.to_vec(),
            source: None,
            added: self.ticks,
            signature,
            tag,
        };
        self.add_to_display(&message, font, focused);
        self.messages.push_front(message);
        self.messages.truncate(MAX_LINES);
    }

    /// `addMessageToDisplayQueue`: the message's lines, the width short of a
    /// tag's icon.
    pub(super) fn add_to_display(&mut self, message: &Message, font: &Font, focused: bool) {
        let mut width =
            (f64::from(width_of(self.settings.width)) / self.settings.scale).floor() as i32;
        if message.tag.as_ref().is_some_and(Tag::has_icon) {
            width -= 9 + 4 + 2;
        }
        for line in wrap(&message.runs, width, font) {
            self.lines.push_front(ChatLine {
                runs: line,
                added: message.added,
                tag: message.tag.clone(),
            });
            if focused && self.scroll > 0 {
                self.new_message = true;
                self.scroll_by(1, true);
            }
        }
        self.lines.truncate(MAX_LINES);
    }

    /// `ChatComponent.deleteMessage`: a signed message is replaced by the
    /// marker, once it is 60 ticks old.
    pub fn delete(&mut self, signature: &[u8], font: &Font, marker: &[Run]) {
        if let Some(delayed) = self.delete_or_delay(signature, font, marker) {
            self.deletions.push(delayed);
        }
    }

    pub(super) fn delete_or_delay(
        &mut self,
        signature: &[u8],
        font: &Font,
        marker: &[Run],
    ) -> Option<(Vec<u8>, u64)> {
        let place = self
            .messages
            .iter()
            .position(|message| message.signature.as_deref() == Some(signature))?;
        let after = self.messages[place].added + TIME_BEFORE_DELETION;
        if self.ticks >= after {
            self.messages[place] = Message {
                runs: marker.to_vec(),
                source: None,
                added: self.messages[place].added,
                signature: None,
                tag: Some(Tag::System),
            };
            self.refresh(font);
            return None;
        }
        Some((signature.to_vec(), after))
    }

    /// `refreshTrimmedMessages`: the lines made again from the messages,
    /// oldest first.
    pub(super) fn refresh(&mut self, font: &Font) {
        self.lines.clear();
        let messages: Vec<Message> = self.messages.iter().rev().cloned().collect();
        for message in &messages {
            self.add_to_display(message, font, false);
        }
    }

    /// `scrollChat`, in lines.
    pub fn scroll_by(&mut self, amount: i32, focused: bool) {
        let limit = self
            .lines
            .len()
            .saturating_sub(self.lines_per_page(focused));
        self.scroll = (self.scroll as i32 + amount).clamp(0, limit as i32) as usize;
        if self.scroll == 0 {
            self.new_message = false;
        }
    }

    /// `resetChatScroll`.
    pub fn reset_scroll(&mut self) {
        self.scroll = 0;
        self.new_message = false;
    }

    /// `ChatComponent.extractRenderState`: the lines over the hotbar. An open
    /// chat with the mouse over a tag gives back the tag's tooltip.
    pub fn draw(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        gui: (i32, i32),
        focused: bool,
    ) {
        self.draw_with(frame, atlas, font, gui, focused, None, &Lang::default());
    }

    /// [`Self::draw`] with the mouse, for the tooltip of a tag it is over.
    /// Everything is drawn in the chat's own units, under a pose that scales
    /// by the option and moves 4 right.
    #[expect(
        clippy::too_many_arguments,
        reason = "mirrors the reference's render call"
    )]
    pub fn draw_with(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        (gui_width, gui_height): (i32, i32),
        focused: bool,
        mouse: Option<(i32, i32)>,
        lang: &Lang,
    ) -> Option<Vec<Vec<Run>>> {
        let first = (frame.vertices.len(), frame.log.as_ref().map_or(0, Vec::len));
        let scale = self.settings.scale as f32;
        let max_width = (width_of(self.settings.width) as f32 / scale).ceil() as i32;
        let chat_bottom = ((gui_height - BOTTOM_MARGIN) as f32 / scale).floor() as i32;
        let text_opacity = self.settings.opacity as f32 * 0.9 + 0.1;
        let background_opacity = self.settings.background_opacity as f32;
        let spacing = self.settings.line_spacing;
        let entry_height = self.entry_height();
        // `Math.round`.
        let entry_bottom_to_message_y =
            (8.0 * (spacing + 1.0) - 4.0 * spacing + 0.5).floor() as i32;
        let per_page = self.lines_per_page(focused);
        let shown = self.lines.len().saturating_sub(self.scroll).min(per_page);
        // `forEachLine`: from the oldest of the page to the newest, the ones
        // not faded out; the backgrounds first, then each line's text and tag.
        let visible: Vec<(usize, f32)> = (0..shown)
            .rev()
            .filter_map(|i| {
                let line = &self.lines[i + self.scroll];
                let alpha = if focused {
                    1.0
                } else {
                    fade(self.ticks - line.added)
                };
                (alpha > 1.0e-5).then_some((i, alpha))
            })
            .collect();
        for &(i, alpha) in &visible {
            let entry_bottom = chat_bottom - i as i32 * entry_height;
            let background = ((alpha * background_opacity * 255.0) as u32) << 24;
            frame.fill(
                atlas,
                (
                    -4,
                    entry_bottom - entry_height,
                    max_width + 4 + 4,
                    entry_bottom,
                ),
                background,
            );
        }
        let local = mouse.map(|mouse| pose_undone(scale, mouse));
        let over = |local: Option<(f32, f32)>, (x0, y0, x1, y1): (i32, i32, i32, i32)| {
            local.is_some_and(|(x, y)| {
                x >= x0 as f32 && x < x1 as f32 && y >= y0 as f32 && y < y1 as f32
            })
        };
        let mut tooltip = None;
        // The style with a hover event that the mouse is over, the last one
        // drawn (`hoveredTextStyle`).
        let mut hovered: Option<&Style> = None;
        for &(i, alpha) in &visible {
            let entry_bottom = chat_bottom - i as i32 * entry_height;
            let entry_top = entry_bottom - entry_height;
            let text_top = entry_bottom - entry_bottom_to_message_y;
            let opacity = alpha * text_opacity;
            let text = ((opacity * 255.0) as u32) << 24 | 0x00FF_FFFF;
            let line = &self.lines[i + self.scroll];
            frame.styled_text(atlas, font, &line.runs, 0, text_top, text, true);
            if let Some(point) = local
                && focused
                && let Some(style) =
                    crate::gui::text_hit::style_under(font, &line.runs, (0, text_top), point)
                && style.hover.is_some()
            {
                hovered = Some(style);
            }
            let Some(tag) = &line.tag else {
                continue;
            };
            let bar = ((opacity * 255.0) as u32) << 24 | tag.colour();
            frame.fill(atlas, (-4, entry_top, -2, entry_bottom), bar);
            if focused && over(local, (-4, entry_top, -2, entry_bottom)) {
                tooltip = Some(tag.tooltip(lang));
            }
            if tag.has_icon() && focused {
                // `handleTagIcon`: shown while the mouse is over it.
                let left = font.width_of_runs(&line.runs) + 4;
                let bottom_of_text = text_top + MESSAGE_HEIGHT;
                let top = bottom_of_text - 9 - 1;
                if over(local, (left, top, left + 9, bottom_of_text)) {
                    tooltip = Some(tag.tooltip(lang));
                    frame.sprite(atlas, "icon/chat_modified", left, top, 9, 9);
                }
            }
        }
        self.scroll_bar(
            frame,
            atlas,
            (chat_bottom, max_width),
            visible.len(),
            focused,
        );
        frame.scale_from(first, scale, (4.0 * scale, 0.0));
        let tag_tooltip = tooltip.map(|runs| split(&runs, 210, font));
        // `componentHoverEffect`: a tag's tooltip comes first, then a hover
        // event's text, split to half the screen (at least 200).
        tag_tooltip.or_else(|| hover_lines(hovered, lang, font, gui_width))
    }

    /// `ClickableStyleFinder`: the style a press at `mouse` lands on in an
    /// open chat, the last glyph's that has a click event (or, with
    /// `insertions`, an insertion).
    pub fn clicked_style(
        &self,
        font: &Font,
        (_, gui_height): (i32, i32),
        mouse: (i32, i32),
        insertions: bool,
    ) -> Option<Style> {
        let scale = self.settings.scale as f32;
        let chat_bottom = ((gui_height - BOTTOM_MARGIN) as f32 / scale).floor() as i32;
        let entry_height = self.entry_height();
        let spacing = self.settings.line_spacing;
        let entry_bottom_to_message_y =
            (8.0 * (spacing + 1.0) - 4.0 * spacing + 0.5).floor() as i32;
        let shown = self
            .lines
            .len()
            .saturating_sub(self.scroll)
            .min(self.lines_per_page(true));
        let point = pose_undone(scale, mouse);
        let mut found = None;
        for i in (0..shown).rev() {
            let text_top = chat_bottom - i as i32 * entry_height - entry_bottom_to_message_y;
            let line = &self.lines[i + self.scroll];
            if let Some(style) =
                crate::gui::text_hit::style_under(font, &line.runs, (0, text_top), point)
                && (style.click.is_some() || (insertions && style.insertion.is_some()))
            {
                found = Some(style.clone());
            }
        }
        found
    }

    /// The scroll bar of a chat that is open (`extractRenderState`'s last
    /// part): as tall as the lines shown are of all of them.
    pub(super) fn scroll_bar(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        (chat_bottom, max_width): (i32, i32),
        count: usize,
        focused: bool,
    ) {
        let total = self.lines.len() as i32;
        if total == 0 || !focused {
            return;
        }
        let chat_height = count as i32 * self.entry_height();
        let virtual_height = total * self.entry_height();
        if virtual_height == chat_height {
            return;
        }
        let y = self.scroll as i32 * chat_height / total - chat_bottom;
        let height = chat_height * chat_height / virtual_height;
        let alpha = if y > 0 { 170_u32 } else { 96 };
        let colour = if self.new_message {
            13_382_451
        } else {
            3_355_562
        };
        let start_x = max_width + 4;
        frame.fill(
            atlas,
            (start_x, -y, start_x + 2, -y - height),
            alpha << 24 | colour,
        );
        frame.fill(
            atlas,
            (start_x + 2, -y, start_x + 1, -y - height),
            alpha << 24 | 13_421_772,
        );
    }
}
