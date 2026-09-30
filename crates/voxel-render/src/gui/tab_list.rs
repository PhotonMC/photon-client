//! The tab list (`PlayerTabOverlay`): the players in columns of up to twenty
//! between a header and a footer, each with its ping.
//!
//! Not drawn: heads (an online-mode server's, which need skins), teams and
//! the scoreboard's score column.

use voxel_game::text::Run;

use crate::gui::chat::wrap;
use crate::gui::font::Font;
use crate::gui::{GuiAtlas, GuiFrame};

/// The most players shown, and the rows in a column.
const MAX_PLAYERS: usize = 80;
const MAX_ROWS: usize = 20;

/// One player of the list.
#[derive(Debug, Clone)]
pub struct TabEntry {
    pub name: Vec<Run>,
    pub spectator: bool,
    pub latency: i32,
    pub order: i32,
    /// The profile name, which orders players of one kind.
    pub profile_name: String,
}

/// The order players are listed in: a higher list order first, spectators
/// after the rest, then by name without regard to case.
pub fn sort(entries: &mut [TabEntry]) {
    entries.sort_by(|a, b| {
        b.order
            .cmp(&a.order)
            .then(a.spectator.cmp(&b.spectator))
            .then(
                a.profile_name
                    .to_lowercase()
                    .cmp(&b.profile_name.to_lowercase()),
            )
    });
}

/// `PlayerTabOverlay.getPingIcon`.
pub fn ping_sprite(latency: i32) -> &'static str {
    match latency {
        i32::MIN..=-1 => "icon/ping_unknown",
        0..=149 => "icon/ping_5",
        150..=299 => "icon/ping_4",
        300..=599 => "icon/ping_3",
        600..=999 => "icon/ping_2",
        _ => "icon/ping_1",
    }
}

/// How many columns and rows `n` players take.
pub fn grid(n: usize) -> (usize, usize) {
    let (mut cols, mut rows) = (1, n);
    while rows > MAX_ROWS {
        cols += 1;
        rows = n.div_ceil(cols);
    }
    (cols, rows)
}

/// `PlayerTabOverlay.extractRenderState`.
pub fn draw(
    frame: &mut GuiFrame,
    atlas: &GuiAtlas,
    font: &Font,
    screen_width: i32,
    entries: &[TabEntry],
    header: &[Run],
    footer: &[Run],
) {
    let entries = &entries[..entries.len().min(MAX_PLAYERS)];
    let (cols, rows) = grid(entries.len());
    let max_name = entries
        .iter()
        .map(|entry| font.width_of_runs(&entry.name))
        .max()
        .unwrap_or(0);
    let slot_width = (cols as i32 * (max_name + 13)).min(screen_width - 50) / cols as i32;
    let body_width = slot_width * cols as i32 + (cols as i32 - 1) * 5;
    let xxo = screen_width / 2 - body_width / 2;
    let split = |runs: &[Run]| {
        if runs.is_empty() {
            Vec::new()
        } else {
            wrap(runs, screen_width - 50, font)
        }
    };
    let (header_lines, footer_lines) = (split(header), split(footer));
    let widest = header_lines
        .iter()
        .chain(&footer_lines)
        .map(|line| font.width_of_runs(line))
        .fold(body_width, i32::max);
    let (left, right) = (
        screen_width / 2 - widest / 2 - 1,
        screen_width / 2 + widest / 2 + 1,
    );
    let panel = 0x8000_0000;
    let lines = |frame: &mut GuiFrame, lines: &[Vec<Run>], mut y: i32| {
        frame.fill(
            atlas,
            (left, y - 1, right, y + lines.len() as i32 * 9),
            panel,
        );
        for line in lines {
            let x = screen_width / 2 - font.width_of_runs(line) / 2;
            frame.styled_text(atlas, font, line, x, y, 0xFFFF_FFFF, true);
            y += 9;
        }
    };
    let mut y = 10;
    if !header_lines.is_empty() {
        lines(frame, &header_lines, y);
        y += header_lines.len() as i32 * 9;
    }
    y += 1;
    frame.fill(atlas, (left, y - 1, right, y + rows as i32 * 9), panel);
    for (index, entry) in entries.iter().enumerate() {
        let (col, row) = (index / rows, index % rows);
        let x = xxo + col as i32 * slot_width + col as i32 * 5;
        let row_y = y + row as i32 * 9;
        frame.fill(atlas, (x, row_y, x + slot_width, row_y + 8), 0x20FF_FFFF);
        let colour = if entry.spectator {
            0x90FF_FFFF
        } else {
            0xFFFF_FFFF
        };
        frame.styled_text(atlas, font, &entry.name, x, row_y, colour, true);
        frame.sprite(
            atlas,
            ping_sprite(entry.latency),
            x + slot_width - 11,
            row_y,
            10,
            8,
        );
    }
    if !footer_lines.is_empty() {
        y += rows as i32 * 9 + 1;
        lines(frame, &footer_lines, y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, spectator: bool, order: i32) -> TabEntry {
        TabEntry {
            name: Vec::new(),
            spectator,
            latency: 0,
            order,
            profile_name: name.to_owned(),
        }
    }

    #[test]
    fn players_sort_by_order_then_spectators_last_then_name() {
        let mut list = vec![
            entry("zed", false, 0),
            entry("Amy", true, 0),
            entry("bob", false, 0),
            entry("Cat", false, 5),
        ];
        sort(&mut list);
        let names: Vec<&str> = list.iter().map(|e| e.profile_name.as_str()).collect();
        assert_eq!(names, ["Cat", "bob", "zed", "Amy"]);
    }

    #[test]
    fn the_list_takes_a_column_per_twenty_players() {
        assert_eq!(grid(1), (1, 1));
        assert_eq!(grid(20), (1, 20));
        assert_eq!(grid(21), (2, 11));
        assert_eq!(grid(80), (4, 20));
    }

    #[test]
    fn ping_icons_follow_the_latency() {
        assert_eq!(ping_sprite(-1), "icon/ping_unknown");
        assert_eq!(ping_sprite(149), "icon/ping_5");
        assert_eq!(ping_sprite(150), "icon/ping_4");
        assert_eq!(ping_sprite(600), "icon/ping_2");
        assert_eq!(ping_sprite(5000), "icon/ping_1");
    }
}
