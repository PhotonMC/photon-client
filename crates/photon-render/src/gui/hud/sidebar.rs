//! The scoreboard's sidebar (`Hud.displayScoreboardSidebar`): a title over
//! up to fifteen lines, each a name and its score, at the right edge.

use crate::gui::{FontRuns, FrameRuns};
use photon_game::text::Run;

use crate::gui::font::Font;
use crate::gui::{GuiAtlas, GuiFrame};

/// `Options.getBackgroundColor(0.3F)` and `(0.4F)` for the lines and the
/// title: black at `floor(opacity * 255)`.
const LINES_BACKGROUND: u32 = 0x4C00_0000;
const TITLE_BACKGROUND: u32 = 0x6600_0000;

/// One line: the name and the score as written.
pub type SidebarLine = (Vec<Run>, Vec<Run>);

pub fn draw(
    frame: &mut GuiFrame,
    atlas: &GuiAtlas,
    font: &Font,
    (screen_width, screen_height): (i32, i32),
    title: &[Run],
    lines: &[SidebarLine],
) {
    let title_width = font.width_of_runs(title);
    let spacer = font.width(": ");
    let mut width = title_width;
    for (name, score) in lines {
        let score_width = font.width_of_runs(score);
        width = width.max(
            font.width_of_runs(name)
                + if score_width > 0 {
                    spacer + score_width
                } else {
                    0
                },
        );
    }
    let count = lines.len() as i32;
    let height = count * 9;
    let bottom = screen_height / 2 + height / 3;
    let left = screen_width - width - 3;
    let right = screen_width - 3 + 2;
    let header_y = bottom - count * 9;
    frame.fill(
        atlas,
        (left - 2, header_y - 9 - 1, right, header_y - 1),
        TITLE_BACKGROUND,
    );
    frame.fill(
        atlas,
        (left - 2, header_y - 1, right, bottom),
        LINES_BACKGROUND,
    );
    frame.styled_text(
        atlas,
        font,
        title,
        left + width / 2 - title_width / 2,
        header_y - 9,
        0xFFFF_FFFF,
        false,
    );
    for (index, (name, score)) in lines.iter().enumerate() {
        let y = bottom - (count - index as i32) * 9;
        frame.styled_text(atlas, font, name, left, y, 0xFFFF_FFFF, false);
        let score_width = font.width_of_runs(score);
        frame.styled_text(
            atlas,
            font,
            score,
            right - score_width,
            y,
            0xFFFF_FFFF,
            false,
        );
    }
}
