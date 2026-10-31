//! The spectator menu: `SpectatorMenu`, its root, player and team categories, and `SpectatorGui`'s
//! state (the fade, the wheel and the action key), without the drawing.
//!
//! A spectator's number keys do not pick a hotbar slot but a slot of this menu: the first press
//! selects it, a second press of an enabled one runs it. Each page shows six entries of a
//! category between a scroll arrow and the close slot.
//!
//! Not ported: an item's live state. The players and the teams are read when the menu opens, so a
//! player who turns spectator while it is open still shows enabled.

use photon_core::text::Component;

/// `GameType.SPECTATOR`'s id.
const SPECTATOR: i32 = 3;
/// What one page of a category holds between the scroll arrows.
const PER_PAGE: i32 = 6;
/// `SpectatorGui.FADE_OUT_DELAY` and `FADE_OUT_TIME`, in milliseconds.
const FADE_OUT_DELAY: i64 = 5000;
const FADE_OUT_TIME: f32 = 2000.0;

/// A player the teleport categories list: the profile's id, its name and its game mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub id: u128,
    pub name: String,
    pub game_mode: i32,
}

/// A team with the players of it that can be teleported to, in the team's order.
#[derive(Debug, Clone, PartialEq)]
pub struct TeamEntry {
    pub name: Component,
    /// The team colour's `0xRRGGBB`, when it has one.
    pub colour: Option<u32>,
    pub players: Vec<Candidate>,
    /// The player whose face is the team's icon (the reference picks one at random).
    pub icon: usize,
}

/// What the menu is built from: the listed players and the teams.
#[derive(Debug, Clone, Default)]
pub struct Sources {
    pub players: Vec<Candidate>,
    pub teams: Vec<TeamEntry>,
}

/// `SpectatorMenuItem`.
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    /// `TeleportToPlayerMenuCategory` as an item of the root.
    TeleportToPlayer(Vec<Item>),
    /// `TeleportToTeamMenuCategory` as an item of the root.
    TeleportToTeam(Vec<Item>),
    /// `TeamSelectionItem`: opens the team's players.
    Team(TeamEntry),
    /// `PlayerMenuItem`.
    Player(Candidate),
    Close,
    Scroll {
        direction: i32,
        enabled: bool,
    },
    /// `EMPTY_SLOT`.
    Empty,
}

/// What selecting an item asks of the game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// `ServerboundTeleportToEntityPacket` to this profile.
    Teleport(u128),
}

impl Item {
    /// `SpectatorMenuItem.isEnabled`.
    pub fn is_enabled(&self) -> bool {
        match self {
            Self::TeleportToPlayer(items) | Self::TeleportToTeam(items) => !items.is_empty(),
            Self::Player(player) => player.game_mode != SPECTATOR,
            Self::Team(_) | Self::Close => true,
            Self::Scroll { enabled, .. } => *enabled,
            Self::Empty => false,
        }
    }

    /// `SpectatorMenuItem.getName`: a translation key or the player's name.
    pub fn name(&self) -> Component {
        match self {
            Self::TeleportToPlayer(_) => Component::translatable("spectatorMenu.teleport"),
            Self::TeleportToTeam(_) => Component::translatable("spectatorMenu.team_teleport"),
            Self::Team(team) => team.name.clone(),
            Self::Player(player) => Component::literal(player.name.clone()),
            Self::Close => Component::translatable("spectatorMenu.close"),
            Self::Scroll { direction, .. } if *direction < 0 => {
                Component::translatable("spectatorMenu.previous_page")
            }
            Self::Scroll { .. } => Component::translatable("spectatorMenu.next_page"),
            Self::Empty => Component::empty(),
        }
    }
}

/// `SpectatorMenuCategory`: its entries and the prompt shown with nothing selected.
#[derive(Debug, Clone, PartialEq)]
struct Category {
    items: Vec<Item>,
    prompt: &'static str,
}

/// `UUID.compareTo`: the most significant half then the least, each as a signed number.
fn uuid_order(a: u128, b: u128) -> std::cmp::Ordering {
    let halves = |id: u128| ((id >> 64) as u64 as i64, id as u64 as i64);
    halves(a).cmp(&halves(b))
}

/// `TeleportToPlayerMenuCategory(profiles)`: the players that are not spectators, by profile id.
fn player_items(players: &[Candidate]) -> Vec<Item> {
    let mut listed: Vec<&Candidate> = players
        .iter()
        .filter(|player| player.game_mode != SPECTATOR)
        .collect();
    listed.sort_by(|a, b| uuid_order(a.id, b.id));
    listed.into_iter().cloned().map(Item::Player).collect()
}

/// `TeleportToTeamMenuCategory`: a team with a player to go to.
fn team_items(teams: &[TeamEntry]) -> Vec<Item> {
    teams
        .iter()
        .filter(|team| team.players.iter().any(|p| p.game_mode != SPECTATOR))
        .map(|team| {
            let mut team = team.clone();
            team.players.retain(|player| player.game_mode != SPECTATOR);
            team.icon = team.icon.min(team.players.len() - 1);
            Item::Team(team)
        })
        .collect()
}

/// `SpectatorMenu`.
#[derive(Debug, Clone)]
pub struct SpectatorMenu {
    category: Category,
    selected_slot: i32,
    page: i32,
    closed: bool,
}

impl SpectatorMenu {
    /// `new SpectatorMenu(listener)`: the root category, with the players and teams as they are.
    pub fn new(sources: &Sources) -> Self {
        let root = vec![
            Item::TeleportToPlayer(player_items(&sources.players)),
            Item::TeleportToTeam(team_items(&sources.teams)),
        ];
        Self {
            category: Category {
                items: root,
                prompt: "spectatorMenu.root.prompt",
            },
            selected_slot: -1,
            page: 0,
            closed: false,
        }
    }

    /// `getItem`: the page's slot, with the arrows and the close slot.
    pub fn item(&self, slot: i32) -> Item {
        let items = &self.category.items;
        let index = slot + self.page * PER_PAGE;
        if self.page > 0 && slot == 0 {
            Item::Scroll {
                direction: -1,
                enabled: true,
            }
        } else if slot == 7 {
            Item::Scroll {
                direction: 1,
                enabled: usize::try_from(index).is_ok_and(|index| index < items.len()),
            }
        } else if slot == 8 {
            Item::Close
        } else {
            usize::try_from(index)
                .ok()
                .and_then(|index| items.get(index))
                .cloned()
                .unwrap_or(Item::Empty)
        }
    }

    /// `getItems`: the nine slots.
    pub fn items(&self) -> Vec<Item> {
        (0..9).map(|slot| self.item(slot)).collect()
    }

    pub fn selected_slot(&self) -> i32 {
        self.selected_slot
    }

    pub fn selected_item(&self) -> Item {
        self.item(self.selected_slot)
    }

    /// `getSelectedCategory().getPrompt()`: the translation key.
    pub fn prompt(&self) -> &'static str {
        self.category.prompt
    }

    /// Whether `exit` was asked.
    pub fn is_closed(&self) -> bool {
        self.closed
    }

    /// `selectSlot`: a second press of the selected, enabled slot runs it.
    pub fn select_slot(&mut self, slot: i32) -> Option<Action> {
        let item = self.item(slot);
        if item == Item::Empty {
            return None;
        }
        if self.selected_slot == slot && item.is_enabled() {
            return self.select_item(item);
        }
        self.selected_slot = slot;
        None
    }

    /// `SpectatorMenuItem.selectItem`.
    fn select_item(&mut self, item: Item) -> Option<Action> {
        match item {
            Item::TeleportToPlayer(items) => {
                self.select_category(items, "spectatorMenu.teleport.prompt");
            }
            Item::TeleportToTeam(items) => {
                self.select_category(items, "spectatorMenu.team_teleport.prompt");
            }
            Item::Team(team) => {
                let items = player_items(&team.players);
                self.select_category(items, "spectatorMenu.teleport.prompt");
            }
            Item::Player(player) => return Some(Action::Teleport(player.id)),
            Item::Close => self.closed = true,
            Item::Scroll { direction, .. } => self.page += direction,
            Item::Empty => {}
        }
        None
    }

    /// `selectCategory`: nothing selected, back on the first page.
    fn select_category(&mut self, items: Vec<Item>, prompt: &'static str) {
        self.category = Category { items, prompt };
        self.selected_slot = -1;
        self.page = 0;
    }
}

mod gui;
pub use gui::SpectatorGui;

#[cfg(test)]
mod tests;
