use super::input::Part;
use super::*;
use crate::FrameRuns;
use crate::widgets::scroll_area::SCROLLBAR_WIDTH;
use photon_render::gui::{GuiAtlas, GuiFrame};

/// `extractContent`'s incompatible backdrop and the icon's hover overlay.
const INCOMPATIBLE_BACKDROP: u32 = -8_978_432_i32 as u32;
const HOVER_OVERLAY: u32 = -1_601_138_544_i32 as u32;

/// Where an entry's content starts, whether its list has a bar, and the pointer if it is over it.
struct EntryAt {
    content: (i32, i32),
    /// The width of the row's content, and of the text beside the icon.
    list: usize,
    content_width: i32,
    text_width: i32,
    hover: Option<(i32, i32)>,
}

impl PackScreen {
    pub fn draw(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        lang: &Lang,
        mouse: (i32, i32),
    ) {
        let (width, height) = self.size;
        let background = if self.in_world {
            "inworld_menu_background"
        } else {
            "menu_background"
        };
        frame.tile_texture(atlas, background, (32, 32), (0, 0, width, height));
        self.title.draw(frame, atlas, font, &self.arena, false);
        self.drop_info.draw(frame, atlas, font, &self.arena, false);
        self.draw_search(frame, atlas, font);
        for list in 0..2 {
            self.draw_list(frame, atlas, font, lang, list, mouse);
        }
        for widget in [&self.open_folder, &self.done] {
            let highlighted = widget.is_active() && widget.contains(&self.arena, mouse);
            widget.draw(frame, atlas, font, &self.arena, highlighted);
        }
    }

    fn draw_search(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font) {
        let rect = self.arena.0[self.search_rect];
        let focused = self.search_focused();
        let sprite = if focused {
            "widget/text_field_highlighted"
        } else {
            "widget/text_field"
        };
        frame.nine_slice(
            atlas,
            sprite,
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
    }

    fn draw_list(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        lang: &Lang,
        list: usize,
        mouse: (i32, i32),
    ) {
        let place = self.place();
        let x = self.list_x(list);
        let prefix = if self.in_world { "inworld_" } else { "" };
        frame.tile_texture_from(
            atlas,
            &format!("{prefix}menu_list_background"),
            (32, 32),
            (x + LIST_WIDTH, place.bottom() + self.scroll[list] as i32),
            (x, place.y, LIST_WIDTH, place.height),
        );
        let mark = frame.mark();
        let scrollable = self.scrollable(list);
        self.draw_heading(frame, atlas, font, list, scrollable);
        let hovered = self
            .entry_at(list, mouse)
            .map(|(view, rel)| (view.id.clone(), rel));
        for (index, view) in self.shown(list).into_iter().enumerate() {
            let top = self.entry_y(list, index + 1);
            if top + ENTRY_HEIGHT < place.y || top > place.bottom() {
                continue;
            }
            let hover = hovered
                .as_ref()
                .filter(|(id, _)| *id == view.id)
                .map(|(_, rel)| *rel);
            let bar = if scrollable { SCROLLBAR_WIDTH } else { 0 };
            let at = EntryAt {
                content: (x + 2 * PADDING, top + PADDING),
                list,
                content_width: ROW_WIDTH - 2 * PADDING - bar,
                text_width: TEXT_WIDTH - bar,
                hover,
            };
            self.draw_selection(frame, atlas, list, &view.id, (x, top), bar);
            self.draw_entry(frame, atlas, (font, lang), view, &at);
        }
        frame.clip_from(mark, (x, place.y, x + LIST_WIDTH, place.bottom()));
        self.draw_edges(frame, atlas, list);
    }

    /// The list's title, centred over its rows.
    fn draw_heading(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        list: usize,
        scrollable: bool,
    ) {
        let row_width = ROW_WIDTH - if scrollable { SCROLLBAR_WIDTH } else { 0 };
        let header = &self.headers[list];
        frame.styled_text(
            atlas,
            font,
            header,
            self.list_x(list) + PADDING + row_width / 2 - font.width_of_runs(header) / 2,
            self.entry_y(list, 0) + PADDING + (HEADER_ENTRY_HEIGHT - 4) / 2 - 4,
            0xFFFF_FFFF,
            true,
        );
    }

    /// The lines above and below the list, and its bar.
    fn draw_edges(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, list: usize) {
        let place = self.place();
        let x = self.list_x(list);
        let prefix = if self.in_world { "inworld_" } else { "" };
        for (name, y) in [
            ("header_separator", place.y - 2),
            ("footer_separator", place.bottom()),
        ] {
            let sprite = format!("{prefix}{name}");
            frame.blit_texture(atlas, &sprite, (32, 2), (0, 0), x, y, LIST_WIDTH, 2);
        }
        let content = self.content_height(list);
        place.draw_scrollbar(frame, atlas, self.bar_x(list), self.scroll[list], content);
    }

    fn draw_entry(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        (font, lang): (&Font, &Lang),
        view: &PackView,
        at: &EntryAt,
    ) {
        let (cx, cy) = at.content;
        if !view.compatibility.is_compatible() {
            frame.fill(
                atlas,
                (cx - 1, cy - 1, cx + at.content_width + 1, cy + 32 + 1),
                INCOMPATIBLE_BACKDROP,
            );
        }
        let icon = view.icon.as_deref().unwrap_or("textures/misc/unknown_pack");
        frame.blit_texture(atlas, icon, (ICON, ICON), (0, 0), cx, cy, ICON, ICON);
        let mut title = view.title.clone();
        let mut description = view.description.clone();
        let hovered = at.hover.filter(|_| !(view.fixed_position && view.required));
        let selected = self.is_selected(at.list, view);
        let overlay = hovered.is_some() || selected;
        if overlay && !(view.fixed_position && view.required) {
            let part = hovered.and_then(|rel| self.part_at(view, rel));
            frame.fill(atlas, (cx, cy, cx + ICON, cy + ICON), HOVER_OVERLAY);
            if !view.compatibility.is_compatible() {
                let key = format!("pack.incompatible.{}", view.compatibility.key());
                title = lang.runs(
                    &Component::translatable("pack.incompatible"),
                    &Style::default(),
                );
                let gray = Style {
                    colour: Some(0x80_8080),
                    ..Style::default()
                };
                description = lang.runs(&Component::translatable(key), &gray);
            }
            self.draw_overlay(frame, atlas, view, at.content, part);
        }
        let limit = at.text_width;
        let title = fit(font, title, limit);
        frame.styled_text(
            atlas,
            font,
            &title,
            cx + ICON + 2,
            cy + 1,
            0xFFFF_FFFF,
            true,
        );
        for (row, line) in description_lines(font, &description, limit)
            .into_iter()
            .enumerate()
        {
            let y = cy + 12 + row as i32 * 9;
            frame.styled_text(atlas, font, &line, cx + ICON + 2, y, 0xFF80_8080, true);
        }
    }

    /// Whether the entry is its list's selection and that list has the focus.
    fn is_selected(&self, list: usize, view: &PackView) -> bool {
        self.focus == Some(Focus::List(list))
            && matches!(&self.selected[list], Some(Selected::Pack(id)) if *id == view.id)
    }

    /// The outline of the selected entry: white while its list has the focus, gray otherwise.
    fn draw_selection(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        list: usize,
        id: &str,
        (x, top): (i32, i32),
        bar: i32,
    ) {
        if !matches!(&self.selected[list], Some(Selected::Pack(selected)) if selected == id) {
            return;
        }
        let outline = if self.focus == Some(Focus::List(list)) {
            0xFFFF_FFFF
        } else {
            0xFF80_8080
        };
        let left = x + (LIST_WIDTH - ROW_WIDTH) / 2;
        let rect = (left, top, left + ROW_WIDTH - bar, top + ENTRY_HEIGHT);
        frame.fill(atlas, rect, outline);
        let inner = (rect.0 + 1, rect.1 + 1, rect.2 - 1, rect.3 - 1);
        frame.fill(atlas, inner, 0xFF00_0000);
    }

    /// The icon's arrows: select, or unselect, up and down, highlighted under the pointer.
    fn draw_overlay(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        view: &PackView,
        (cx, cy): (i32, i32),
        part: Option<Part>,
    ) {
        let id = &view.id;
        let mut sprite = |name: &str, lit: bool| {
            let suffix = if lit { "_highlighted" } else { "" };
            let sprite = format!("transferable_list/{name}{suffix}");
            frame.sprite(atlas, &sprite, cx, cy, ICON, ICON);
        };
        if self.model.can_select(id) {
            sprite("select", part == Some(Part::Select));
            return;
        }
        if self.model.can_unselect(id) {
            sprite("unselect", part == Some(Part::Unselect));
        }
        if self.model.can_move_up(id) {
            sprite("move_up", part == Some(Part::Up));
        }
        if self.model.can_move_down(id) {
            sprite("move_down", part == Some(Part::Down));
        }
    }
}

/// `StringWidget.setMaxWidth`: a text wider than the room ends in an ellipsis.
fn fit(font: &Font, mut runs: Vec<Run>, limit: i32) -> Vec<Run> {
    if font.width_of_runs(&runs) > limit {
        ellipsize(font, &mut runs, limit);
    }
    runs
}
