//! The tab list (`PlayerTabOverlay`): the players in columns of up to twenty
//! between a header and a footer, each with its ping.
//!
//! Not drawn: heads (an online-mode server's, which need skins).

use crate::{FontRuns, FrameRuns};
use std::collections::HashMap;

use photon_game::text::{Lang, Run};

use crate::chat::split;
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

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
    /// The name of the player's team, which orders players before their names.
    pub team: String,
    /// The score on the list's objective, and how it is written (none for
    /// hearts or when the player has no score).
    pub score: i32,
    pub score_text: Option<Vec<Run>>,
}

impl TabEntry {
    /// A listed player: the name shown (the display name the server gave, or
    /// the profile name), and what orders it. A spectator's name is italic
    /// where its own style says nothing of it (`decorateName`).
    pub fn new(
        profile_name: &str,
        display: Option<Vec<Run>>,
        game_mode: i32,
        latency: i32,
        order: i32,
    ) -> Self {
        let spectator = game_mode == 3;
        let mut name = display.unwrap_or_else(|| crate::chat::plain_runs(profile_name));
        if spectator {
            for run in &mut name {
                run.style.italic = run.style.italic.or(Some(true));
            }
        }
        Self {
            name,
            spectator,
            latency,
            order,
            profile_name: profile_name.to_owned(),
            team: String::new(),
            score: 0,
            score_text: None,
        }
    }

    /// The player's team (by name) and score on the list's objective.
    pub fn with_scoreboard(mut self, team: &str, score: i32, score_text: Option<Vec<Run>>) -> Self {
        team.clone_into(&mut self.team);
        self.score = score;
        self.score_text = score_text;
        self
    }
}

/// A listed player as the server describes it.
pub struct TabPlayer<'a> {
    pub name: &'a str,
    pub display_name: Option<&'a photon_game::text::Component>,
    pub game_mode: i32,
    pub latency: i32,
    pub list_order: i32,
}

/// The players as the list shows them, in its order, and the objective it
/// shows scores of: a name is the tab list display name, else the team's
/// dressing of the profile name (`getNameForDisplay`).
pub fn entries<'a>(
    players: impl IntoIterator<Item = TabPlayer<'a>>,
    board: &photon_game::scoreboard::Scoreboard,
    lang: &Lang,
) -> (Vec<TabEntry>, Option<TabObjective>) {
    use photon_game::scoreboard::{NumberFormat, SLOT_LIST};
    use photon_game::text::{Component, Style};
    let objective = board.display_objective(SLOT_LIST);
    let default = NumberFormat::player_list_default();
    let mut entries: Vec<TabEntry> = players
        .into_iter()
        .map(|player| {
            let dressed = match player.display_name {
                Some(component) => Some(component.clone()),
                None => board
                    .team_of(player.name)
                    .map(|(_, team)| team.formatted_name(&Component::literal(player.name))),
            };
            let name = dressed.map(|component| lang.runs(&component, &Style::default()));
            let (score, text) = match objective {
                Some((objective_name, objective)) => {
                    let info = board.score(player.name, objective_name);
                    let text = (!objective.hearts)
                        .then(|| {
                            info.map(|info| {
                                let format = objective.number_format.as_ref().unwrap_or(&default);
                                let component = info
                                    .number_format
                                    .as_ref()
                                    .unwrap_or(format)
                                    .format(info.value);
                                lang.runs(&component, &Style::default())
                            })
                        })
                        .flatten();
                    (info.map_or(0, |info| info.value), text)
                }
                None => (0, None),
            };
            let team = board.team_of(player.name).map_or("", |(name, _)| name);
            TabEntry::new(
                player.name,
                name,
                player.game_mode,
                player.latency,
                player.list_order,
            )
            .with_scoreboard(team, score, text)
        })
        .collect();
    sort(&mut entries);
    let shown = objective.map(|(_, objective)| TabObjective {
        hearts: objective.hearts,
    });
    (entries, shown)
}

/// The order players are listed in: a higher list order first, spectators
/// after the rest, then by team name, then by name without regard to case.
pub fn sort(entries: &mut [TabEntry]) {
    entries.sort_by(|a, b| {
        b.order
            .cmp(&a.order)
            .then(a.spectator.cmp(&b.spectator))
            .then_with(|| a.team.cmp(&b.team))
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

/// `PlayerTabOverlay.HealthState`: what the hearts column remembers of a
/// player to blink when the score changes.
#[derive(Debug, Clone)]
struct HealthState {
    last_value: i32,
    displayed_value: i32,
    last_update_tick: i64,
    blink_until_tick: i64,
}

impl HealthState {
    fn new(value: i32) -> Self {
        Self {
            last_value: value,
            displayed_value: value,
            last_update_tick: 0,
            blink_until_tick: 0,
        }
    }

    fn update(&mut self, value: i32, tick: i64) {
        if value != self.last_value {
            let blink = if value < self.last_value { 20 } else { 10 };
            self.blink_until_tick = tick + blink;
            self.last_value = value;
            self.last_update_tick = tick;
        }
        if tick - self.last_update_tick > 20 {
            self.displayed_value = value;
        }
    }

    fn blinking(&self, tick: i64) -> bool {
        self.blink_until_tick > tick && (self.blink_until_tick - tick) % 6 >= 3
    }
}

/// The hearts column's memory, by player.
#[derive(Debug, Clone, Default)]
pub struct HealthStates(HashMap<String, HealthState>);

impl HealthStates {
    /// `setVisible`: a list hidden or shown again starts afresh.
    pub fn clear(&mut self) {
        self.0.clear();
    }
}

/// What the list shows beside names (`displayObjective`): scores as text, or
/// hearts.
#[derive(Debug, Clone, Copy)]
pub struct TabObjective {
    pub hearts: bool,
}

/// What `draw` needs besides the players: the objective, the hearts' memory,
/// the HUD's tick, and the language the hearts' text is put into words by.
pub struct TabScores<'a> {
    pub objective: Option<TabObjective>,
    pub health: &'a mut HealthStates,
    pub gui_ticks: i64,
    pub lang: &'a Lang,
}

/// `PlayerTabOverlay.extractTablistHearts`.
fn draw_hearts(
    frame: &mut GuiFrame,
    atlas: &GuiAtlas,
    font: &Font,
    scores: &mut TabScores,
    (key, score): (&str, i32),
    (left, right, y): (i32, i32, i32),
) {
    let health = scores
        .health
        .0
        .entry(key.to_owned())
        .or_insert_with(|| HealthState::new(score));
    health.update(score, scores.gui_ticks);
    let displayed = health.displayed_value;
    let blink = health.blinking(scores.gui_ticks);
    let full_hearts = -((-score.max(displayed)).div_euclid(2));
    let hearts_to_render = score.max(displayed.max(20)) / 2;
    if full_hearts <= 0 {
        return;
    }
    let per_heart = ((right - left - 4) as f32 / hearts_to_render as f32)
        .min(9.0)
        .floor() as i32;
    if per_heart <= 3 {
        let pct = (score as f32 / 20.0).clamp(0.0, 1.0);
        let colour =
            (((1.0 - pct) * 255.0) as i32 as u32) << 16 | (((pct * 255.0) as i32 as u32) << 8);
        let hearts = score as f32 / 2.0;
        let number = if hearts.fract() == 0.0 {
            format!("{hearts:.1}")
        } else {
            hearts.to_string()
        };
        let hp = scores.lang.runs(
            &photon_game::text::Component::translatable_with(
                "multiplayer.player.list.hp",
                vec![photon_game::text::Component::literal(number.clone())],
            ),
            &photon_game::text::Style::default(),
        );
        let text = if right - font.width_of_runs(&hp) >= left {
            hp
        } else {
            crate::chat::plain_runs(&number)
        };
        let x = (right + left - font.width_of_runs(&text)) / 2;
        frame.styled_text(atlas, font, &text, x, y, 0xFF00_0000 | colour, true);
        return;
    }
    let container = if blink {
        "hud/heart/container_blinking"
    } else {
        "hud/heart/container"
    };
    for heart in full_hearts..hearts_to_render {
        frame.sprite(atlas, container, left + heart * per_heart, y, 9, 9);
    }
    for heart in 0..full_hearts {
        let x = left + heart * per_heart;
        frame.sprite(atlas, container, x, y, 9, 9);
        if blink {
            if heart * 2 + 1 < displayed {
                frame.sprite(atlas, "hud/heart/full_blinking", x, y, 9, 9);
            }
            if heart * 2 + 1 == displayed {
                frame.sprite(atlas, "hud/heart/half_blinking", x, y, 9, 9);
            }
        }
        if heart * 2 + 1 < score {
            let sprite = if heart >= 10 {
                "hud/heart/absorbing_full_blinking"
            } else {
                "hud/heart/full"
            };
            frame.sprite(atlas, sprite, x, y, 9, 9);
        }
        if heart * 2 + 1 == score {
            let sprite = if heart >= 10 {
                "hud/heart/absorbing_half_blinking"
            } else {
                "hud/heart/half"
            };
            frame.sprite(atlas, sprite, x, y, 9, 9);
        }
    }
}

/// `PlayerTabOverlay.extractTablistScore`: a player's hearts or score in
/// the column between `left` and `right`, when it is wide enough.
fn draw_score(
    frame: &mut GuiFrame,
    atlas: &GuiAtlas,
    font: &Font,
    scores: &mut TabScores,
    entry: &TabEntry,
    (left, right, y): (i32, i32, i32),
) {
    let Some(objective) = scores.objective else {
        return;
    };
    if right - left <= 5 {
        return;
    }
    if objective.hearts {
        draw_hearts(
            frame,
            atlas,
            font,
            scores,
            (&entry.profile_name, entry.score),
            (left, right, y),
        );
    } else if let Some(text) = &entry.score_text {
        let width = font.width_of_runs(text);
        frame.styled_text(atlas, font, text, right - width, y, 0xFFFF_FFFF, true);
    }
}

/// `PlayerTabOverlay.extractRenderState`.
pub fn draw(
    frame: &mut GuiFrame,
    (atlas, font): (&GuiAtlas, &Font),
    screen_width: i32,
    entries: &[TabEntry],
    (header, footer): (&[Run], &[Run]),
    mut scores: TabScores,
) {
    let entries = &entries[..entries.len().min(MAX_PLAYERS)];
    let (cols, rows) = grid(entries.len());
    let max_name = entries
        .iter()
        .map(|entry| font.width_of_runs(&entry.name))
        .max()
        .unwrap_or(0);
    let spacer = font.width(" ");
    let max_score = entries
        .iter()
        .filter_map(|entry| entry.score_text.as_ref())
        .map(|text| font.width_of_runs(text))
        .filter(|width| *width > 0)
        .map(|width| spacer + width)
        .max()
        .unwrap_or(0);
    let score_width = match scores.objective {
        None => 0,
        Some(TabObjective { hearts: true }) => 90,
        Some(TabObjective { hearts: false }) => max_score,
    };
    scores
        .health
        .0
        .retain(|key, _| entries.iter().any(|entry| entry.profile_name == *key));
    let slot_width =
        (cols as i32 * (max_name + score_width + 13)).min(screen_width - 50) / cols as i32;
    let body_width = slot_width * cols as i32 + (cols as i32 - 1) * 5;
    let xxo = screen_width / 2 - body_width / 2;
    let split = |runs: &[Run]| {
        if runs.is_empty() {
            Vec::new()
        } else {
            split(runs, screen_width - 50, font)
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
        y += header_lines.len() as i32 * 9 + 1;
    }
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
        if !entry.spectator {
            let left = x + max_name + 1;
            draw_score(
                frame,
                atlas,
                font,
                &mut scores,
                entry,
                (left, left + score_width, row_y),
            );
        }
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
            team: String::new(),
            score: 0,
            score_text: None,
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
    fn a_team_orders_players_before_their_names() {
        let mut list = vec![entry("amy", false, 0), entry("zed", false, 0)];
        list[0].team = "b".to_owned();
        list[1].team = "a".to_owned();
        sort(&mut list);
        assert_eq!(list[0].profile_name, "zed");
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
