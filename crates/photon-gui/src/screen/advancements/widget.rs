//! One advancement on a tab (`AdvancementWidget`): where it is, what its box says when it is
//! hovered, and the lines that join it to its parent.

use crate::chat::split;
use crate::{FontRuns, FrameRuns};
use photon_game::advancements::{AdvancementProgress, AdvancementType, DisplayInfo};
use photon_game::text::{Component, Lang, Run, Style};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

/// `TEST_SPLIT_OFFSETS`.
const TEST_SPLIT_OFFSETS: [i32; 5] = [0, 10, -10, 25, -25];

/// `ChatFormatting.getColor` of the colour a frame type writes its description in.
fn chat_colour(frame: AdvancementType) -> u32 {
    match frame {
        AdvancementType::Task | AdvancementType::Goal => 0x55FF55,
        AdvancementType::Challenge => 0xAA00AA,
    }
}

/// `StringSplitter.stringWidth` of lines of runs: the widest, as the game adds its advances.
fn widest(lines: &[Vec<Run>], font: &Font) -> f32 {
    lines
        .iter()
        .map(|line| {
            line.iter()
                .flat_map(|run| {
                    let bold = run.style.bold == Some(true);
                    run.text.chars().map(move |c| (c, bold))
                })
                .map(|(c, bold)| font.glyph(c).map_or(0.0, |glyph| glyph.advance(bold)))
                .sum::<f32>()
        })
        .fold(0.0, f32::max)
}

/// `findOptimalLines`: the description split so that its widest line is within ten of the width it
/// would like, at the first margin that does, or the nearest.
fn optimal_lines(runs: &[Run], preferred: i32, font: &Font) -> Vec<Vec<Run>> {
    let mut best: Option<Vec<Vec<Run>>> = None;
    let mut best_distance = f32::MAX;
    for margin in TEST_SPLIT_OFFSETS {
        let lines = split(runs, preferred - margin, font);
        let distance = (widest(&lines, font) - preferred as f32).abs();
        if distance <= 10.0 {
            return lines;
        }
        if distance < best_distance {
            best_distance = distance;
            best = Some(lines);
        }
    }
    best.unwrap_or_default()
}

pub(super) struct AdvancementWidget {
    pub(super) id: String,
    pub(super) display: DisplayInfo,
    pub(super) x: i32,
    pub(super) y: i32,
    pub(super) width: i32,
    title_lines: Vec<Vec<Run>>,
    description: Vec<Vec<Run>>,
    pub(super) parent: Option<usize>,
    pub(super) children: Vec<usize>,
    pub(super) progress: Option<AdvancementProgress>,
}

impl AdvancementWidget {
    /// The constructor: the box's width from the title and the description as the font fits them.
    pub(super) fn new(
        id: &str,
        display: &DisplayInfo,
        requirements: usize,
        (font, lang): (&Font, &Lang),
    ) -> Self {
        let title_runs = lang.runs(&display.title, &Style::default());
        let title_lines = split(&title_runs, 163, font);
        let title_width = title_lines
            .iter()
            .map(|line| font.width_of_runs(line))
            .max()
            .unwrap_or(0)
            .max(80);
        let mut longest = 29 + title_width + max_progress_width(requirements, (font, lang));
        // `ComponentUtils.mergeStyles`: the frame's colour over the description's own.
        let mut description = display.description.clone();
        description.style.colour = Some(chat_colour(display.frame));
        let runs = lang.runs(&description, &Style::default());
        let description = optimal_lines(&runs, longest, font);
        for line in &description {
            longest = longest.max(font.width_of_runs(line));
        }
        Self {
            id: id.to_owned(),
            display: display.clone(),
            x: (display.x * 28.0).floor() as i32,
            y: (display.y * 27.0).floor() as i32,
            width: longest + 3 + 5,
            title_lines,
            description,
            parent: None,
            children: Vec::new(),
            progress: None,
        }
    }

    /// Whether it shows: a hidden one only once it is done.
    fn shown(&self) -> bool {
        !self.display.hidden
            || self
                .progress
                .as_ref()
                .is_some_and(AdvancementProgress::is_done)
    }

    /// `isMouseOver`: the frame's square, edges included.
    pub(super) fn is_mouse_over(
        &self,
        (scroll_x, scroll_y): (i32, i32),
        (mx, my): (i32, i32),
    ) -> bool {
        if !self.shown() {
            return false;
        }
        let (x0, y0) = (scroll_x + self.x, scroll_y + self.y);
        mx >= x0 && mx <= x0 + 26 && my >= y0 && my <= y0 + 26
    }

    fn percent(&self) -> f32 {
        self.progress
            .as_ref()
            .map_or(0.0, AdvancementProgress::percent)
    }

    /// `extractRenderState`'s frame and icon, for this one only.
    pub(super) fn draw_frame(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, at: (i32, i32)) {
        if !self.shown() {
            return;
        }
        let kind = if self.percent() >= 1.0 {
            "obtained"
        } else {
            "unobtained"
        };
        frame.sprite(
            atlas,
            &frame_sprite(self.display.frame, kind),
            at.0 + self.x + 3,
            at.1 + self.y,
            26,
            26,
        );
    }

    /// Where its icon goes, if it shows.
    pub(super) fn icon_place(&self, at: (i32, i32)) -> Option<(i32, i32)> {
        self.shown()
            .then_some((at.0 + self.x + 8, at.1 + self.y + 5))
    }

    /// `extractConnectivity` for the line to its parent: the dark outline under, the white over.
    pub(super) fn draw_connection(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        (parent_x, parent_y): (i32, i32),
        (at, background): ((i32, i32), bool),
    ) {
        let dep_x = at.0 + parent_x + 13;
        let split_x = at.0 + parent_x + 26 + 4;
        let dep_y = at.1 + parent_y + 13;
        let my_x = at.0 + self.x + 13;
        let my_y = at.1 + self.y + 13;
        let mut line = |x0: i32, x1: i32, y: i32, colour: u32| {
            let (low, high) = (x0.min(x1), x0.max(x1));
            frame.fill(atlas, (low, y, high + 1, y + 1), colour);
        };
        if background {
            let black = 0xFF00_0000;
            line(split_x, dep_x, dep_y - 1, black);
            line(split_x + 1, dep_x, dep_y, black);
            line(split_x, dep_x, dep_y + 1, black);
            line(my_x, split_x - 1, my_y - 1, black);
            line(my_x, split_x - 1, my_y, black);
            line(my_x, split_x - 1, my_y + 1, black);
            vertical(frame, atlas, split_x - 1, (my_y, dep_y), black);
            vertical(frame, atlas, split_x + 1, (my_y, dep_y), black);
        } else {
            let white = 0xFFFF_FFFF;
            line(split_x, dep_x, dep_y, white);
            line(my_x, split_x, my_y, white);
            vertical(frame, atlas, split_x, (my_y, dep_y), white);
        }
    }

    /// `extractHover`: the box with its title and description, over the others. `scroll` is the tab's,
    /// `window` the screen's left and top, `screen_width` the screen's, and `origin` where the tab's
    /// contents start.
    pub(super) fn draw_hover(
        &self,
        frame: &mut GuiFrame,
        (atlas, font, lang): (&GuiAtlas, &Font, &Lang),
        (scroll, origin, screen_width): ((i32, i32), (i32, i32), i32),
    ) {
        let layout = HoverLayout::new(self, lang, scroll, screen_width);
        let (ox, oy) = origin;
        let background_height = layout.title_bar_height + layout.description_height;
        if !self.description.is_empty() {
            let top = if layout.top_side {
                layout.title_bar_bottom - background_height
            } else {
                layout.title_top
            };
            frame.nine_slice(
                atlas,
                "advancements/title_box",
                (200, 26),
                10,
                false,
                (
                    ox + layout.title_left,
                    oy + top,
                    self.width,
                    background_height,
                ),
            );
        }
        self.draw_box(frame, atlas, &layout, origin);
        let frame_kind = layout.icon_frame;
        frame.sprite(
            atlas,
            &frame_sprite(self.display.frame, frame_kind),
            ox + scroll.0 + self.x + 3,
            oy + scroll.1 + self.y,
            26,
            26,
        );
        self.draw_hover_text(frame, (atlas, font), &layout, (scroll, origin));
    }

    /// The box under the title: whole, or in two kinds where the progress ends.
    fn draw_box(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        layout: &HoverLayout,
        (ox, oy): (i32, i32),
    ) {
        let left = ox + layout.title_left;
        let top = oy + layout.title_top;
        let height = layout.title_bar_height;
        let box_sprite = |kind: &str| format!("advancements/box_{kind}");
        if layout.first_half_kind == layout.second_half_kind {
            frame.nine_slice(
                atlas,
                &box_sprite(layout.first_half_kind),
                (200, 26),
                10,
                false,
                (left, top, self.width, height),
            );
        } else {
            let second = self.width - layout.first_half_width;
            let sizes = (((200, 26), 10), (200, height));
            frame.nine_slice_region(
                atlas,
                &box_sprite(layout.first_half_kind),
                sizes,
                (0, 0),
                (left, top, layout.first_half_width, height),
            );
            frame.nine_slice_region(
                atlas,
                &box_sprite(layout.second_half_kind),
                sizes,
                (200 - second, 0),
                (left + layout.first_half_width, top, second, height),
            );
        }
    }

    /// The title lines, the progress, and the description above or under.
    fn draw_hover_text(
        &self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        layout: &HoverLayout,
        (scroll, (ox, oy)): ((i32, i32), (i32, i32)),
    ) {
        let (title_top, left_side) = (layout.title_top, layout.left_side);
        let description_left = layout.title_left + 5;
        let progress_y = oy + title_top + 9;
        let title_x = if left_side {
            ox + description_left
        } else {
            ox + scroll.0 + self.x + 32
        };
        multiline(
            frame,
            (atlas, font),
            &self.title_lines,
            (title_x, progress_y),
            0xFFFF_FFFF,
        );
        if let Some(text) = &layout.progress_text {
            let width = font.width_of_runs(text);
            let x = if left_side {
                ox + scroll.0 + self.x - width
            } else {
                ox + scroll.0 + self.x + self.width - width - 5
            };
            frame.styled_text(atlas, font, text, (x, progress_y), 0xFFFF_FFFF, true);
        }
        let description_y = if layout.top_side {
            title_top - layout.description_text_height + 1
        } else {
            layout.title_bar_bottom
        };
        multiline(
            frame,
            (atlas, font),
            &self.description,
            (ox + description_left, oy + description_y),
            0xFF00_FF00,
        );
    }
}

/// `GuiGraphicsExtractor.verticalLine`.
fn vertical(frame: &mut GuiFrame, atlas: &GuiAtlas, x: i32, (y0, y1): (i32, i32), colour: u32) {
    let (low, high) = (y0.min(y1), y0.max(y1));
    frame.fill(atlas, (x, low + 1, x + 1, high), colour);
}

/// The text of each line, nine units under the line before.
fn multiline(
    frame: &mut GuiFrame,
    (atlas, font): (&GuiAtlas, &Font),
    lines: &[Vec<Run>],
    (x, y): (i32, i32),
    colour: u32,
) {
    for (index, line) in lines.iter().enumerate() {
        frame.styled_text(atlas, font, line, (x, y + index as i32 * 9), colour, true);
    }
}

/// `AdvancementWidgetType.frameSprite`.
pub(super) fn frame_sprite(frame: AdvancementType, kind: &str) -> String {
    let name = match frame {
        AdvancementType::Task => "task",
        AdvancementType::Challenge => "challenge",
        AdvancementType::Goal => "goal",
    };
    format!("advancements/{name}_frame_{kind}")
}

/// `getMaxProgressWidth`: room for the progress text of a finished advancement.
fn max_progress_width(requirements: usize, (font, lang): (&Font, &Lang)) -> i32 {
    if requirements <= 1 {
        return 0;
    }
    font.width_of_runs(&progress_runs(requirements, requirements, lang)) + 8
}

/// `advancements.progress` for a count and a total.
pub(super) fn progress_runs(done: usize, total: usize, lang: &Lang) -> Vec<Run> {
    lang.runs(
        &Component::translatable_with(
            "advancements.progress",
            vec![
                Component::literal(done.to_string()),
                Component::literal(total.to_string()),
            ],
        ),
        &Style::default(),
    )
}

/// What `extractHover` works out before it draws.
struct HoverLayout {
    title_bar_height: i32,
    title_top: i32,
    title_bar_bottom: i32,
    description_text_height: i32,
    description_height: i32,
    left_side: bool,
    top_side: bool,
    title_left: i32,
    first_half_width: i32,
    first_half_kind: &'static str,
    second_half_kind: &'static str,
    icon_frame: &'static str,
    progress_text: Option<Vec<Run>>,
}

impl HoverLayout {
    fn new(widget: &AdvancementWidget, lang: &Lang, scroll: (i32, i32), screen_width: i32) -> Self {
        let title_bar_height = 9 * widget.title_lines.len() as i32 + 9 + 8;
        let title_top = scroll.1 + widget.y + (26 - title_bar_height) / 2;
        let title_bar_bottom = title_top + title_bar_height;
        let description_text_height = widget.description.len() as i32 * 9;
        let description_height = 6 + description_text_height;
        let progress_text = widget
            .progress
            .as_ref()
            .and_then(AdvancementProgress::progress_counts)
            .map(|(done, total)| progress_runs(done, total, lang));
        let amount = widget.percent();
        let mut first_half_width = (amount * widget.width as f32).floor() as i32;
        let (first, second, icon) = if amount >= 1.0 {
            first_half_width = widget.width / 2;
            ("obtained", "obtained", "obtained")
        } else if first_half_width < 2 {
            first_half_width = widget.width / 2;
            ("unobtained", "unobtained", "unobtained")
        } else if first_half_width > widget.width - 2 {
            first_half_width = widget.width / 2;
            ("obtained", "obtained", "unobtained")
        } else {
            ("obtained", "unobtained", "unobtained")
        };
        let left_side = scroll.0 + widget.x + widget.width + 26 >= screen_width;
        let title_left = if left_side {
            scroll.0 + widget.x - widget.width + 26 + 6
        } else {
            scroll.0 + widget.x
        };
        Self {
            title_bar_height,
            title_top,
            title_bar_bottom,
            description_text_height,
            description_height,
            left_side,
            top_side: title_bar_bottom + description_height >= 113,
            title_left,
            first_half_width,
            first_half_kind: first,
            second_half_kind: second,
            icon_frame: icon,
            progress_text,
        }
    }
}
