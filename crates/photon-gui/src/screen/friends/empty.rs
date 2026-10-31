//! The text blocks centred in the room of the lists: the empty friends list with its picture and
//! the Microsoft account link, the empty requests list and the errors.

use super::draw::inside;
use super::*;
use crate::FontRuns;

const PICTURE: (i32, i32) = (128, 48);
const PICTURE_GAP: i32 = 8;
/// `FocusableTextWidget.DEFAULT_PADDING`.
const TEXT_PADDING: i32 = 4;

pub(super) struct Block {
    /// Where the picture is, when there is one.
    pub picture: Option<(i32, i32)>,
    /// The lines of text, each with its corner.
    pub lines: Vec<(i32, i32, Vec<Run>)>,
}

impl FriendsOverlay {
    pub(super) fn centred_block(
        &self,
        font: &Font,
        (x, y, width, height): (i32, i32, i32, i32),
        runs: &[Run],
        picture: bool,
    ) -> Block {
        // `FocusableTextWidget` keeps a padding around its text, which the width and height count.
        let lines = crate::chat::split(runs, width - 2 * BORDER - 2 * TEXT_PADDING, font);
        let above = if picture { PICTURE.1 + PICTURE_GAP } else { 0 };
        let text_height = lines.len() as i32 * 9 + 2 * TEXT_PADDING;
        // Matched to the oracle rows: a single line sits one pixel lower than the half.
        let top = y + (height - above - text_height + 1) / 2;
        Block {
            picture: picture.then(|| (x + (width - PICTURE.0) / 2, top)),
            lines: lines
                .into_iter()
                .enumerate()
                .map(|(index, line)| {
                    let at = x + width / 2 - font.width_of_runs(&line) / 2;
                    (at, top + above + TEXT_PADDING + index as i32 * 9, line)
                })
                .collect(),
        }
    }

    /// Whether `at` is on the link of the empty friends list.
    pub(super) fn empty_link_at(&self, at: (i32, i32), font: &Font) -> bool {
        if self.tab != Tab::Friends || self.view.friends != Listing::Empty {
            return false;
        }
        let block = self.centred_block(font, self.list_area(), &self.texts.empty, true);
        block.lines.iter().any(|(x, y, line)| {
            let mut left = *x;
            line.iter().any(|run| {
                let width = font.width_of_runs(std::slice::from_ref(run));
                let hit = run.style.underlined == Some(true) && inside(at, (left, *y, width, 9));
                left += width;
                hit
            })
        })
    }
}
