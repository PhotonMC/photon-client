//! The name an entity shows over its head (`Entity.getDisplayName` without a team): its custom
//! name, or the name of its type.

use photon_game::scoreboard::{NameTagVisibility, Scoreboard};
use photon_game::text::{Component, Lang, Style};
use photon_render::entity::name_tag::{TagVisibility, TeamRule, Teams};
use photon_render::gui::TextRun;
use photon_world::ClientEntity;

/// `Entity.getScoreboardName`: a player's profile name, the unique id of anything else.
fn scoreboard_name(entity: &ClientEntity, player_name: Option<&str>) -> String {
    match player_name {
        Some(name) if entity.is_player() => name.to_owned(),
        _ => entity.uuid.to_string_java(),
    }
}

/// `Entity.getTeamColor`: the colour of the entity's team, white for none or a team without one.
/// The outline of a glowing entity is drawn in it. `holder` is its scoreboard name.
pub fn team_colour(holder: &str, board: &Scoreboard) -> u32 {
    board
        .team_of(holder)
        .and_then(|(_, team)| team.colour)
        .and_then(|colour| photon_game::scoreboard::TEAM_COLOURS.get(usize::from(colour)))
        .and_then(|name| photon_game::text::named_colour(name))
        .unwrap_or(0xFF_FFFF)
}

/// The teams a tag's decision looks at, from the scoreboard: the entity's (by its scoreboard
/// name) and the viewer's (by the local player's name).
fn teams_of(board: &Scoreboard, entity_name: &str, viewer_name: &str) -> Teams {
    let entity = board.team_of(entity_name);
    let viewer = board.team_of(viewer_name);
    Teams {
        entity: entity.map(|(_, team)| TeamRule {
            visibility: match team.name_tag_visibility {
                NameTagVisibility::Always => TagVisibility::Always,
                NameTagVisibility::Never => TagVisibility::Never,
                NameTagVisibility::HideForOtherTeams => TagVisibility::HideForOtherTeams,
                NameTagVisibility::HideForOwnTeam => TagVisibility::HideForOwnTeam,
            },
            sees_friendly_invisibles: team.sees_friendly_invisibles,
        }),
        viewer_has_team: viewer.is_some(),
        same_team: matches!((entity, viewer), (Some((a, _)), Some((b, _))) if a == b),
    }
}

/// What a name tag is decided and written from.
pub struct TagInputs {
    /// The entity's display name.
    pub runs: Vec<TextRun<'static>>,
    /// The teams its rules look at.
    pub teams: Teams,
    /// The score line under the name (`Entity.belowNameDisplay`).
    pub score: Option<Vec<TextRun<'static>>>,
}

/// The inputs of an entity's tag; `player_name` is the profile name of a player. The one place
/// the client and the oracle both build them.
pub fn tag_inputs(
    entity: &ClientEntity,
    player_name: Option<&str>,
    (lang, board): (&Lang, &Scoreboard),
    viewer_name: &str,
) -> TagInputs {
    let own = scoreboard_name(entity, player_name);
    let runs = match player_name {
        Some(name) if entity.is_player() => player_name_runs(name, (lang, board)),
        _ => entity_name_runs(entity, (lang, board, &own)),
    };
    let score = below_name(entity, (board, &own)).map(|score| {
        let runs = lang.runs(&score, &Style::default());
        crate::draw_runs(&runs)
            .into_iter()
            .map(TextRun::into_owned)
            .collect()
    });
    TagInputs {
        runs,
        teams: teams_of(board, &own, viewer_name),
        score,
    }
}

/// `Entity.belowNameDisplay`: the below-name objective's score, or a mannequin's description
/// (`ClientMannequin`).
fn below_name(entity: &ClientEntity, (board, own): (&Scoreboard, &str)) -> Option<Component> {
    use photon_world::objects::mannequin::MannequinDescription;
    match entity.mannequin_description() {
        None => board.below_name(own),
        Some(MannequinDescription::Default) => {
            Some(Component::translatable("entity.minecraft.mannequin.label"))
        }
        Some(MannequinDescription::Hidden) => None,
        Some(MannequinDescription::Text(tag)) => Component::from_nbt(&tag),
    }
}

/// A player's name over their head, dressed by their team (`Player.getDisplayName`).
fn player_name_runs(name: &str, (lang, board): (&Lang, &Scoreboard)) -> Vec<TextRun<'static>> {
    let component = board.team_formatted(name, &Component::literal(name));
    let runs = lang.runs(&component, &Style::default());
    crate::draw_runs(&runs)
        .into_iter()
        .map(TextRun::into_owned)
        .collect()
}

/// `Entity.getName`: the custom name the server sent, else the type's name (`getTypeName`;
/// the types that name themselves otherwise, like a villager by its profession, are not told
/// apart). Styled the way the component says, in words of `lang`.
fn entity_name_runs(
    entity: &ClientEntity,
    (lang, board, scoreboard_name): (&Lang, &Scoreboard, &str),
) -> Vec<TextRun<'static>> {
    let component = entity
        .custom_name()
        .and_then(|tag| Component::from_nbt(&tag))
        .or_else(|| {
            let name = photon_data::entity_type::name(entity.type_id)?;
            // `Util.makeDescriptionId("entity", id)`.
            let (namespace, path) = name.split_once(':')?;
            Some(Component::translatable(format!(
                "entity.{namespace}.{}",
                path.replace('/', ".")
            )))
        });
    let Some(component) = component else {
        return Vec::new();
    };
    // `Entity.getDisplayName`: dressed by the team.
    let component = board.team_formatted(scoreboard_name, &component);
    let runs = lang.runs(&component, &Style::default());
    crate::draw_runs(&runs)
        .into_iter()
        .map(TextRun::into_owned)
        .collect()
}
