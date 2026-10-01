//! The scoreboard the server describes (`Scoreboard`, `Objective`,
//! `PlayerTeam`, the number formats): objectives with scores, the slots they
//! show in, and teams that dress names. The server's packets change it;
//! nothing here changes it by itself.

use std::collections::{BTreeMap, HashMap};

use crate::text::{Component, Style};

/// `DisplaySlot` ids: the tab list, the sidebar, below names, then a sidebar
/// for each team colour.
pub const SLOT_LIST: u8 = 0;
pub const SLOT_SIDEBAR: u8 = 1;
const SLOT_COUNT: usize = 19;

/// `TeamColor`, in id order.
pub const TEAM_COLOURS: [&str; 16] = [
    "black",
    "dark_blue",
    "dark_green",
    "dark_aqua",
    "dark_red",
    "dark_purple",
    "gold",
    "gray",
    "dark_gray",
    "blue",
    "green",
    "aqua",
    "red",
    "light_purple",
    "yellow",
    "white",
];

/// `NumberFormat`: how a score is written.
#[derive(Debug, Clone, PartialEq)]
pub enum NumberFormat {
    Blank,
    Styled(Style),
    Fixed(Component),
}

impl NumberFormat {
    /// `StyledFormat.SIDEBAR_DEFAULT`.
    pub fn sidebar_default() -> Self {
        Self::Styled(Style::coloured("red"))
    }

    /// `StyledFormat.PLAYER_LIST_DEFAULT`.
    pub fn player_list_default() -> Self {
        Self::Styled(Style::coloured("yellow"))
    }

    pub fn format(&self, value: i32) -> Component {
        match self {
            Self::Blank => Component::empty(),
            Self::Styled(style) => Component::literal(value.to_string()).with_style(style),
            Self::Fixed(component) => component.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Objective {
    pub display_name: Component,
    /// `RenderType.HEARTS` rather than `INTEGER`.
    pub hearts: bool,
    pub number_format: Option<NumberFormat>,
}

/// A score of one owner on one objective.
#[derive(Debug, Clone, PartialEq)]
pub struct Score {
    pub value: i32,
    pub display: Option<Component>,
    pub number_format: Option<NumberFormat>,
}

impl Score {
    /// `ReadOnlyScoreInfo.formatValue`.
    pub fn format_value(&self, default: &NumberFormat) -> Component {
        self.number_format
            .as_ref()
            .unwrap_or(default)
            .format(self.value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Team {
    pub display_name: Component,
    pub prefix: Component,
    pub suffix: Component,
    /// A `TeamColor` id.
    pub colour: Option<u8>,
}

impl Team {
    /// `PlayerTeam.getFormattedName`: the prefix, the name and the suffix, in
    /// the team's colour.
    pub fn formatted_name(&self, name: &Component) -> Component {
        let mut result = Component::empty()
            .append(self.prefix.clone())
            .append(name.clone())
            .append(self.suffix.clone());
        if let Some(colour) = self.colour {
            result.style.colour = crate::text::named_colour(TEAM_COLOURS[usize::from(colour)]);
        }
        result
    }
}

/// What a `ClientboundSetPlayerTeamPacket` carries of a team's settings.
#[derive(Debug, Clone, PartialEq)]
pub struct TeamParameters {
    pub display_name: Component,
    pub prefix: Component,
    pub suffix: Component,
    pub colour: Option<u8>,
}

/// A packet that changes the scoreboard.
#[derive(Debug, Clone, PartialEq)]
pub enum ScoreboardEvent {
    /// `ClientboundSetObjectivePacket`: method 0 adds, 1 removes, 2 changes.
    Objective {
        name: String,
        method: u8,
        display_name: Component,
        hearts: bool,
        number_format: Option<NumberFormat>,
    },
    Score {
        owner: String,
        objective: String,
        value: i32,
        display: Option<Component>,
        number_format: Option<NumberFormat>,
    },
    ResetScore {
        owner: String,
        objective: Option<String>,
    },
    Display {
        slot: u8,
        objective: Option<String>,
    },
    /// `ClientboundSetPlayerTeamPacket`: method 0 adds, 1 removes, 2 changes,
    /// 3 and 4 add and remove players.
    Team {
        name: String,
        method: u8,
        parameters: Option<TeamParameters>,
        players: Vec<String>,
    },
}

#[derive(Debug, Clone, Default)]
pub struct Scoreboard {
    objectives: HashMap<String, Objective>,
    /// Owner, then objective name.
    scores: HashMap<String, HashMap<String, Score>>,
    display: [Option<String>; SLOT_COUNT],
    teams: BTreeMap<String, Team>,
    team_of: HashMap<String, String>,
}

/// `String.CASE_INSENSITIVE_ORDER`: char by char, upper then lower case.
fn insensitive(text: &str) -> impl Iterator<Item = char> + '_ {
    text.chars()
        .map(|c| c.to_uppercase().next().unwrap_or(c))
        .map(|c| c.to_lowercase().next().unwrap_or(c))
}

impl Scoreboard {
    /// `ClientPacketListener.handleAddObjective` and its siblings.
    pub fn apply(&mut self, event: ScoreboardEvent) {
        match event {
            ScoreboardEvent::Objective {
                name,
                method,
                display_name,
                hearts,
                number_format,
            } => self.apply_objective(name, method, display_name, hearts, number_format),
            ScoreboardEvent::Score {
                owner,
                objective,
                value,
                display,
                number_format,
            } => {
                if self.objectives.contains_key(&objective) {
                    self.scores.entry(owner).or_default().insert(
                        objective,
                        Score {
                            value,
                            display,
                            number_format,
                        },
                    );
                }
            }
            ScoreboardEvent::ResetScore { owner, objective } => match objective {
                None => {
                    self.scores.remove(&owner);
                }
                Some(objective) => {
                    if let Some(scores) = self.scores.get_mut(&owner) {
                        scores.remove(&objective);
                        if scores.is_empty() {
                            self.scores.remove(&owner);
                        }
                    }
                }
            },
            ScoreboardEvent::Display { slot, objective } => {
                // An id out of range is the first slot (`ByIdMap` ZERO).
                let slot = if usize::from(slot) < SLOT_COUNT {
                    slot
                } else {
                    0
                };
                self.display[usize::from(slot)] =
                    objective.filter(|name| self.objectives.contains_key(name));
            }
            ScoreboardEvent::Team {
                name,
                method,
                parameters,
                players,
            } => self.apply_team(name, method, parameters, players),
        }
    }

    fn apply_objective(
        &mut self,
        name: String,
        method: u8,
        display_name: Component,
        hearts: bool,
        number_format: Option<NumberFormat>,
    ) {
        match method {
            0 => {
                self.objectives.entry(name).or_insert(Objective {
                    display_name,
                    hearts,
                    number_format,
                });
            }
            1 => {
                self.objectives.remove(&name);
                for slot in &mut self.display {
                    if slot.as_deref() == Some(name.as_str()) {
                        *slot = None;
                    }
                }
                for scores in self.scores.values_mut() {
                    scores.remove(&name);
                }
            }
            2 => {
                if let Some(objective) = self.objectives.get_mut(&name) {
                    *objective = Objective {
                        display_name,
                        hearts,
                        number_format,
                    };
                }
            }
            _ => {}
        }
    }

    fn apply_team(
        &mut self,
        name: String,
        method: u8,
        parameters: Option<TeamParameters>,
        players: Vec<String>,
    ) {
        if method == 0 {
            self.teams.entry(name.clone()).or_insert_with(|| Team {
                display_name: Component::literal(name.clone()),
                prefix: Component::empty(),
                suffix: Component::empty(),
                colour: None,
            });
        }
        let Some(team) = self.teams.get_mut(&name) else {
            return;
        };
        if let Some(parameters) = parameters {
            team.display_name = parameters.display_name;
            team.prefix = parameters.prefix;
            team.suffix = parameters.suffix;
            team.colour = parameters.colour;
        }
        match method {
            0 | 3 => {
                for player in players {
                    self.team_of.insert(player, name.clone());
                }
            }
            4 => {
                for player in players {
                    if self.team_of.get(&player) == Some(&name) {
                        self.team_of.remove(&player);
                    }
                }
            }
            1 => {
                self.teams.remove(&name);
                self.team_of.retain(|_, team| *team != name);
            }
            _ => {}
        }
    }

    /// `getPlayersTeam`.
    pub fn team_of(&self, player: &str) -> Option<(&str, &Team)> {
        let name = self.team_of.get(player)?;
        self.teams.get(name).map(|team| (name.as_str(), team))
    }

    /// `PlayerTeam.formatNameForTeam` for a player's name.
    pub fn team_formatted(&self, player: &str, name: &Component) -> Component {
        match self.team_of(player) {
            Some((_, team)) => team.formatted_name(name),
            None => name.clone(),
        }
    }

    pub fn objective(&self, name: &str) -> Option<&Objective> {
        self.objectives.get(name)
    }

    /// `getDisplayObjective`.
    pub fn display_objective(&self, slot: u8) -> Option<(&str, &Objective)> {
        let name = self.display.get(usize::from(slot))?.as_deref()?;
        self.objectives.get(name).map(|objective| (name, objective))
    }

    /// `getPlayerScoreInfo`.
    pub fn score(&self, owner: &str, objective: &str) -> Option<&Score> {
        self.scores.get(owner)?.get(objective)
    }

    /// What `Hud.extractScoreboardSidebar` shows: the objective of the
    /// local player's team colour, or the sidebar's.
    pub fn sidebar_objective(&self, local_player: &str) -> Option<(&str, &Objective)> {
        let by_team = self
            .team_of(local_player)
            .and_then(|(_, team)| team.colour)
            .and_then(|colour| self.display_objective(3 + colour));
        by_team.or_else(|| self.display_objective(SLOT_SIDEBAR))
    }

    /// `Hud.displayScoreboardSidebar`'s lines: the visible scores, the
    /// highest first and by name among equals, 15 at most, each as the name
    /// a team dresses and the score written.
    pub fn sidebar_lines(&self, objective: &str) -> Vec<(Component, Component)> {
        let Some(objective_data) = self.objectives.get(objective) else {
            return Vec::new();
        };
        let default = NumberFormat::sidebar_default();
        let format = objective_data.number_format.as_ref().unwrap_or(&default);
        let mut scores: Vec<(&String, &Score)> = self
            .scores
            .iter()
            .filter(|(owner, _)| !owner.starts_with('#'))
            .filter_map(|(owner, scores)| scores.get(objective).map(|score| (owner, score)))
            .collect();
        scores.sort_by(|(a_owner, a), (b_owner, b)| {
            b.value
                .cmp(&a.value)
                .then_with(|| insensitive(a_owner).cmp(insensitive(b_owner)))
        });
        scores
            .into_iter()
            .take(15)
            .map(|(owner, score)| {
                let name = score
                    .display
                    .clone()
                    .unwrap_or_else(|| Component::literal(owner.clone()));
                (
                    self.team_formatted(owner, &name),
                    score
                        .number_format
                        .as_ref()
                        .unwrap_or(format)
                        .format(score.value),
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn objective(board: &mut Scoreboard, name: &str) {
        board.apply(ScoreboardEvent::Objective {
            name: name.to_owned(),
            method: 0,
            display_name: Component::literal(name),
            hearts: false,
            number_format: None,
        });
    }

    fn score(board: &mut Scoreboard, owner: &str, objective: &str, value: i32) {
        board.apply(ScoreboardEvent::Score {
            owner: owner.to_owned(),
            objective: objective.to_owned(),
            value,
            display: None,
            number_format: None,
        });
    }

    #[test]
    fn the_sidebar_lists_the_highest_first_and_hides_the_hidden() {
        let mut board = Scoreboard::default();
        objective(&mut board, "o");
        score(&mut board, "bob", "o", 5);
        score(&mut board, "Al", "o", 5);
        score(&mut board, "cy", "o", 9);
        score(&mut board, "#hidden", "o", 100);
        let lines = board.sidebar_lines("o");
        let names: Vec<&Component> = lines.iter().map(|(name, _)| name).collect();
        assert_eq!(
            names,
            [
                &Component::literal("cy"),
                &Component::literal("Al"),
                &Component::literal("bob")
            ]
        );
    }

    #[test]
    fn removing_an_objective_clears_its_slot_and_scores() {
        let mut board = Scoreboard::default();
        objective(&mut board, "o");
        score(&mut board, "a", "o", 1);
        board.apply(ScoreboardEvent::Display {
            slot: SLOT_SIDEBAR,
            objective: Some("o".to_owned()),
        });
        assert!(board.sidebar_objective("x").is_some());
        board.apply(ScoreboardEvent::Objective {
            name: "o".to_owned(),
            method: 1,
            display_name: Component::empty(),
            hearts: false,
            number_format: None,
        });
        assert!(board.sidebar_objective("x").is_none());
        assert!(board.score("a", "o").is_none());
    }

    #[test]
    fn a_team_dresses_its_members_and_picks_its_own_sidebar() {
        let mut board = Scoreboard::default();
        objective(&mut board, "all");
        objective(&mut board, "red_only");
        board.apply(ScoreboardEvent::Display {
            slot: SLOT_SIDEBAR,
            objective: Some("all".to_owned()),
        });
        board.apply(ScoreboardEvent::Display {
            slot: 3 + 12,
            objective: Some("red_only".to_owned()),
        });
        board.apply(ScoreboardEvent::Team {
            name: "t".to_owned(),
            method: 0,
            parameters: Some(TeamParameters {
                display_name: Component::literal("t"),
                prefix: Component::literal("["),
                suffix: Component::literal("]"),
                colour: Some(12),
            }),
            players: vec!["al".to_owned()],
        });
        let dressed = board.team_formatted("al", &Component::literal("al"));
        assert_eq!(dressed.style.colour, Some(0xFF_5555));
        assert_eq!(dressed.siblings.len(), 3);
        assert_eq!(board.sidebar_objective("al").unwrap().0, "red_only");
        assert_eq!(board.sidebar_objective("bo").unwrap().0, "all");
        board.apply(ScoreboardEvent::Team {
            name: "t".to_owned(),
            method: 1,
            parameters: None,
            players: Vec::new(),
        });
        assert!(board.team_of("al").is_none());
    }
}
