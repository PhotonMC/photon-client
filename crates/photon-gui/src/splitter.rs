//! `StringSplitter.splitLines` over styled text: the pieces are broken into lines of at most a
//! width, measuring through `StringDecomposer.iterateFormatted`, so the `\u{a7}` codes style the
//! text and take no room. The pieces keep their codes: the lines are drawn through the same
//! iteration, which is where a reset goes back to the piece's own style.

use photon_game::text::{Run, Style};
use photon_render::gui::font::Font;

use crate::runs::styled_by;

/// One piece: its characters and the style it starts in (`StringSplitter.LineComponent`).
struct Piece {
    chars: Vec<char>,
    style: Style,
}

impl Piece {
    fn run(&self) -> Run {
        Run {
            text: self.chars.iter().collect(),
            style: self.style.clone(),
        }
    }
}

/// `LineBreakFinder`: where the line that is being measured ends.
struct Finder<'a> {
    font: &'a Font,
    max: f32,
    line_break: Option<usize>,
    break_style: Style,
    seen_width: bool,
    width: f32,
    last_space: Option<(usize, Style)>,
    next: usize,
    offset: usize,
}

impl Finder<'_> {
    fn accept(&mut self, position: usize, style: &Style, c: char) -> bool {
        let position = position + self.offset;
        if c != '\n' {
            if c == ' ' {
                self.last_space = Some((position, style.clone()));
            }
            let bold = style.bold == Some(true);
            let width = self.font.glyph(c).map_or(0.0, |glyph| glyph.advance(bold));
            self.width += width;
            if !self.seen_width || self.width <= self.max {
                self.seen_width |= width != 0.0;
                self.next = position + 1;
                return true;
            }
            if let Some((space, space_style)) = self.last_space.clone() {
                return self.finish(space, space_style);
            }
        }
        self.finish(position, style.clone())
    }

    fn finish(&mut self, at: usize, style: Style) -> bool {
        self.line_break = Some(at);
        self.break_style = style;
        false
    }
}

/// `StringDecomposer.iterateFormatted`: the characters of `chars` with the style the codes give
/// them, a reset going back to `reset`. False if `accept` stopped it.
fn iterate(
    chars: &[char],
    start: &Style,
    reset: &Style,
    mut accept: impl FnMut(usize, &Style, char) -> bool,
) -> bool {
    let mut style = start.clone();
    let mut at = 0;
    while at < chars.len() {
        if chars[at] == '\u{a7}' {
            let Some(&code) = chars.get(at + 1) else {
                break;
            };
            style = styled_by(style, code.to_ascii_lowercase(), reset);
            at += 2;
            continue;
        }
        if !accept(at, &style, chars[at]) {
            return false;
        }
        at += 1;
    }
    true
}

/// `FlatComponents.splitAt`: the pieces before `skip_at`, leaving the rest of the pieces after the
/// `skip_size` characters skipped, the first of them in `style`.
fn split_at(pieces: &mut Vec<Piece>, skip_at: usize, skip_size: usize, style: &Style) -> Vec<Run> {
    let mut line = Vec::new();
    let mut position = skip_at;
    let mut skipping = false;
    // Every piece the loop visits is removed or replaced at the front, so it is always the first.
    let index = 0;
    while index < pieces.len() {
        let size = pieces[index].chars.len();
        if !skipping {
            if position > size {
                line.push(pieces.remove(index).run());
                position -= size;
                continue;
            }
            if position > 0 {
                line.push(Run {
                    text: pieces[index].chars[..position].iter().collect(),
                    style: pieces[index].style.clone(),
                });
            }
            position += skip_size;
            skipping = true;
        }
        if position <= size {
            let after = pieces[index].chars[position..].to_vec();
            if after.is_empty() {
                pieces.remove(index);
            } else {
                pieces[index] = Piece {
                    chars: after,
                    style: style.clone(),
                };
            }
            break;
        }
        pieces.remove(index);
        position -= size;
    }
    line
}

/// The lines of `runs` and whether each follows a soft break (one the width made).
pub fn split_lines(runs: &[Run], max: i32, font: &Font, initial: &Style) -> Vec<(Vec<Run>, bool)> {
    let mut pieces: Vec<Piece> = runs
        .iter()
        .filter(|run| !run.text.is_empty())
        .map(|run| Piece {
            chars: run.text.chars().collect(),
            style: run.style.clone(),
        })
        .collect();
    let (mut lines, mut wrapped, mut forced) = (Vec::new(), false, false);
    'restart: loop {
        let mut finder = Finder {
            font,
            max: (max as f32).max(1.0),
            line_break: None,
            break_style: Style::default(),
            seen_width: false,
            width: 0.0,
            last_space: None,
            next: 0,
            offset: 0,
        };
        for piece in &pieces {
            let done = iterate(&piece.chars, &piece.style, initial, |at, style, c| {
                finder.accept(at, style, c)
            });
            if !done {
                let at = finder.line_break.unwrap_or(finder.next);
                let flat: Vec<char> = pieces
                    .iter()
                    .flat_map(|p| p.chars.iter().copied())
                    .collect();
                let tail = flat[at];
                let newline = tail == '\n';
                let skip = usize::from(newline || tail == ' ');
                forced = newline;
                let style = finder.break_style.clone();
                lines.push((split_at(&mut pieces, at, skip, &style), wrapped));
                wrapped = !newline;
                continue 'restart;
            }
            finder.offset += piece.chars.len();
        }
        break;
    }
    if !pieces.is_empty() {
        lines.push((pieces.iter().map(Piece::run).collect(), wrapped));
    } else if forced {
        lines.push((Vec::new(), false));
    }
    lines
}
