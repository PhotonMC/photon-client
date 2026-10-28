//! The name tags over entities that are not players: which renderer's rules decide, the
//! custom name and its visibility, the crosshair, the viewer, the distance, and where the tag
//! hangs for grown, baby and scaled entities of different kinds.

use serde_json::{Value, json};

use super::locator::{TAG, Tag, tag};
use crate::bits::f64_bits;
use crate::line::float_text;

/// One entity's scenario on top of the player scenario's camera and light.
struct Mob {
    kind: &'static str,
    renderer: &'static str,
    name: Option<&'static str>,
    visible: bool,
    picked: bool,
    baby: bool,
    scale: Option<f64>,
    /// `ArmorStand.DATA_CLIENT_FLAGS`: small is 1, marker 16.
    stand_flags: Option<u8>,
    pose: Option<&'static str>,
    reach: Option<f64>,
    below: Option<f64>,
    tag: Tag,
}

const MOB: Mob = Mob {
    kind: "minecraft:zombie",
    renderer: "mob",
    name: Some("Bob"),
    visible: true,
    picked: false,
    baby: false,
    scale: None,
    stand_flags: None,
    pose: None,
    reach: None,
    below: None,
    tag: TAG,
};

fn mob(label: &str, mob: Mob) -> Value {
    let mut row = tag(label, mob.tag);
    row["type"] = json!(mob.kind);
    row["renderer"] = json!(mob.renderer);
    row["custom_name_visible"] = json!(mob.visible);
    row["picked"] = json!(mob.picked);
    row["baby"] = json!(mob.baby);
    if let Some(name) = mob.name {
        row["custom_name"] = json!(name);
    }
    if let Some(scale) = mob.scale {
        row["scale"] = json!(f64_bits(scale));
    }
    if let Some(pose) = mob.pose {
        row["pose"] = json!(pose);
    }
    if let Some(flags) = mob.stand_flags {
        row["stand_flags"] = json!(flags);
    }
    if let Some(reach) = mob.below {
        row["below_name_distance"] = json!(f64_bits(reach));
    }
    if let Some(reach) = mob.reach {
        row["name_tag_distance"] = json!(f64_bits(reach));
    }
    row
}

fn at(distance: f64) -> Tag {
    Tag { position: [0.5 + distance, 64.0, 0.5], ..TAG }
}

fn visibility_rows() -> Vec<Value> {
    let plain = Mob { kind: "minecraft:minecart", renderer: "entity", ..MOB };
    vec![
        mob("mob-visible", MOB),
        mob("mob-hidden", Mob { visible: false, ..MOB }),
        mob("mob-hidden-picked", Mob { visible: false, picked: true, ..MOB }),
        mob("mob-visible-unnamed", Mob { name: None, ..MOB }),
        mob("mob-unnamed-picked", Mob { name: None, visible: false, picked: true, ..MOB }),
        mob("mob-invisible", Mob { tag: Tag { invisible: true, ..TAG }, ..MOB }),
        mob("mob-invisible-spectating", Mob { tag: Tag { invisible: true, spectator: true, ..TAG }, ..MOB }),
        mob("mob-ridden", Mob { tag: Tag { vehicle: true, ..TAG }, ..MOB }),
        mob("mob-hud-hidden", Mob { tag: Tag { hud_hidden: true, ..TAG }, ..MOB }),
        mob("mob-sneaking", Mob { tag: Tag { shift: true, ..TAG }, ..MOB }),
        mob("entity-visible", Mob { ..plain }),
        mob("entity-hidden", Mob { visible: false, ..plain }),
        mob("entity-hidden-picked", Mob { visible: false, picked: true, ..plain }),
        mob("entity-ridden", Mob { tag: Tag { vehicle: true, ..TAG }, ..plain }),
        mob("entity-invisible", Mob { tag: Tag { invisible: true, ..TAG }, ..plain }),
        mob("entity-hud-hidden", Mob { tag: Tag { hud_hidden: true, ..TAG }, ..plain }),
        mob("dragon-visible", Mob { kind: "minecraft:ender_dragon", renderer: "entity", ..MOB }),
        mob(
            "dragon-ridden",
            Mob { kind: "minecraft:ender_dragon", renderer: "entity", tag: Tag { vehicle: true, ..TAG }, ..MOB },
        ),
        mob(
            "dragon-invisible",
            Mob { kind: "minecraft:ender_dragon", renderer: "entity", tag: Tag { invisible: true, ..TAG }, ..MOB },
        ),
        mob("stand-visible", Mob { kind: "minecraft:armor_stand", renderer: "armor_stand", ..MOB }),
        mob("stand-small", Mob { kind: "minecraft:armor_stand", renderer: "armor_stand", stand_flags: Some(1), ..MOB }),
        mob(
            "stand-marker",
            Mob { kind: "minecraft:armor_stand", renderer: "armor_stand", stand_flags: Some(16), ..MOB },
        ),
        mob(
            "stand-small-marker",
            Mob { kind: "minecraft:armor_stand", renderer: "armor_stand", stand_flags: Some(25), ..MOB },
        ),
        mob("villager-sleeping", Mob { kind: "minecraft:villager", pose: Some("sleeping"), ..MOB }),
        mob(
            "villager-sleeping-scaled",
            Mob { kind: "minecraft:villager", pose: Some("sleeping"), scale: Some(2.0), ..MOB },
        ),
        mob("zombie-sleeping", Mob { pose: Some("sleeping"), ..MOB }),
        mob("stand-hidden", Mob { kind: "minecraft:armor_stand", renderer: "armor_stand", visible: false, ..MOB }),
        mob(
            "stand-hidden-picked",
            Mob { kind: "minecraft:armor_stand", renderer: "armor_stand", visible: false, picked: true, ..MOB },
        ),
        mob(
            "stand-ridden",
            Mob { kind: "minecraft:armor_stand", renderer: "armor_stand", tag: Tag { vehicle: true, ..TAG }, ..MOB },
        ),
    ]
}

fn distance_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for distance in [20.0, 31.9, 32.1, 63.9, 64.1] {
        let tag = at(distance);
        rows.push(mob(&format!("mob-{}", float_text(distance)), Mob { tag, ..MOB }));
        rows.push(mob(
            &format!("mob-sneaking-{}", float_text(distance)),
            Mob { tag: Tag { shift: true, ..tag }, ..MOB },
        ));
        rows.push(mob(
            &format!("entity-{}", float_text(distance)),
            Mob { kind: "minecraft:minecart", renderer: "entity", tag, ..MOB },
        ));
    }
    for distance in [8.0, 9.9, 10.1, 100.0, 127.9, 128.1] {
        rows.push(mob(
            &format!("mob-reach-{}", float_text(distance)),
            Mob { tag: at(distance), reach: Some(if distance > 50.0 { 128.0 } else { 10.0 }), ..MOB },
        ));
    }
    rows
}

fn attachment_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for kind in [
        "minecraft:zombie",
        "minecraft:cow",
        "minecraft:villager",
        "minecraft:ghast",
        "minecraft:enderman",
        "minecraft:spider",
        "minecraft:chicken",
        "minecraft:slime",
        "minecraft:iron_golem",
        "minecraft:wolf",
        "minecraft:horse",
        "minecraft:ender_dragon",
        "minecraft:pig",
        "minecraft:squid",
        "minecraft:allay",
    ] {
        // The dragon's renderer is a plain `EntityRenderer`.
        let renderer = if kind == "minecraft:ender_dragon" { "entity" } else { "mob" };
        rows.push(mob(&format!("kind-{kind}"), Mob { kind, renderer, ..MOB }));
    }
    for kind in ["minecraft:zombie", "minecraft:cow", "minecraft:villager", "minecraft:pig", "minecraft:wolf"] {
        rows.push(mob(&format!("baby-{kind}"), Mob { kind, baby: true, ..MOB }));
    }
    for scale in [0.5, 2.0, 7.25] {
        rows.push(mob(&format!("scaled-{}", float_text(scale)), Mob { scale: Some(scale), ..MOB }));
    }
    rows.push(mob("baby-scaled", Mob { baby: true, scale: Some(2.0), ..MOB }));
    for (label, yaw, pitch) in [("look-east", -90.0, 0.0), ("odd-angles", 123.456, 17.25)] {
        rows.push(mob(label, Mob { tag: Tag { camera_rot: (yaw, pitch), ..TAG }, ..MOB }));
    }
    rows.push(mob("turned", Mob { tag: Tag { y_rot: -73.5, ..TAG }, ..MOB }));
    rows.push(mob("dark-cave", Mob { tag: Tag { light: 0, ..TAG }, ..MOB }));
    rows
}

/// The scoreboard name of the entity of a row (`Entity.getScoreboardName`): its unique id.
const MOB_NAME: &str = "00000000-0000-0000-0000-000000000002";
/// The local player of the reference scene.
const LOCAL: &str = "Parity";

/// How the viewer stands to the team of the entity.
#[derive(Clone, Copy)]
enum Standing {
    Alone,
    Same,
    Other,
}

/// A team holding the entity, and the viewer as `viewer` says.
fn teams(subject: &str, viewer: Standing, look: (u8, bool), dress: [&str; 3]) -> Value {
    let mut own = vec![json!(subject)];
    let mut teams = Vec::new();
    match viewer {
        Standing::Same => own.push(json!(LOCAL)),
        Standing::Other => teams.push(json!({"name": "other", "players": [LOCAL]})),
        Standing::Alone => {}
    }
    let mut team = json!({
        "name": "own", "players": own, "visibility": look.0, "friendly": look.1,
    });
    let [prefix, suffix, colour] = dress;
    if !prefix.is_empty() {
        team["prefix"] = json!(prefix);
    }
    if !suffix.is_empty() {
        team["suffix"] = json!(suffix);
    }
    if let Ok(colour) = colour.parse::<u8>() {
        team["color"] = json!(colour);
    }
    teams.push(team);
    json!(teams)
}

fn with_teams(mut row: Value, teams: Value) -> Value {
    row["teams"] = teams;
    row
}

/// Every visibility against every standing of the viewer, for a player and for a mob.
fn team_visibility_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for visibility in 0..4_u8 {
        for (viewer, word) in [(Standing::Alone, "none"), (Standing::Same, "same"), (Standing::Other, "other")] {
            let look = (visibility, true);
            let label = format!("{visibility}-{word}");
            rows.push(with_teams(mob(&format!("team-mob-{label}"), MOB), teams(MOB_NAME, viewer, look, ["", "", ""])));
            rows.push(with_teams(
                tag(&format!("team-player-{label}"), TAG),
                teams(TAG.name, viewer, look, ["", "", ""]),
            ));
        }
    }
    rows
}

/// Where the team's answer replaces the plain gates, and where it does not.
fn team_gate_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for (word, tag) in [
        ("invisible", Tag { invisible: true, ..TAG }),
        ("hud-hidden", Tag { hud_hidden: true, ..TAG }),
        ("ridden", Tag { vehicle: true, ..TAG }),
        ("far-sneaking", Tag { shift: true, position: [40.5, 64.0, 0.5], ..TAG }),
    ] {
        for (visibility, friendly) in [(0_u8, true), (0, false), (2, true), (2, false)] {
            let look = (visibility, friendly);
            let label = format!("{word}-{visibility}-{friendly}");
            for viewer in [Standing::Same, Standing::Other] {
                let kind = if matches!(viewer, Standing::Same) { "same" } else { "other" };
                rows.push(with_teams(
                    mob(&format!("team-mob-{label}-{kind}"), Mob { tag, ..MOB }),
                    teams(MOB_NAME, viewer, look, ["", "", ""]),
                ));
                rows.push(with_teams(
                    super::locator::tag(&format!("team-player-{label}-{kind}"), tag),
                    teams(tag.name, viewer, look, ["", "", ""]),
                ));
            }
        }
    }
    rows
}

/// The renderers that do not look at a team's visibility still show its dressing.
fn team_plain_rows() -> Vec<Value> {
    let dress = ["[", "] suffix", "12"];
    let mut rows = Vec::new();
    for (word, kind, renderer) in
        [("entity", "minecraft:minecart", "entity"), ("stand", "minecraft:armor_stand", "armor_stand")]
    {
        for visibility in [0_u8, 1] {
            rows.push(with_teams(
                mob(&format!("team-{word}-{visibility}"), Mob { kind, renderer, ..MOB }),
                teams(MOB_NAME, Standing::Other, (visibility, true), dress),
            ));
        }
    }
    rows
}

/// The team's prefix, suffix and colour on the name.
fn team_dress_rows() -> Vec<Value> {
    let dress = ["[", "] suffix", "12"];
    vec![
        with_teams(mob("team-mob-dressed", MOB), teams(MOB_NAME, Standing::Alone, (0, true), dress)),
        with_teams(tag("team-player-dressed", TAG), teams(TAG.name, Standing::Alone, (0, true), dress)),
        with_teams(
            mob("team-mob-unnamed-dressed", Mob { name: None, ..MOB }),
            teams(MOB_NAME, Standing::Alone, (0, true), dress),
        ),
    ]
}

/// A scoreboard of one objective shown below the name, scoring `owner`, with the number format
/// the objective names.
fn scored(mut row: Value, owner: &str, format: Option<&str>) -> Value {
    let mut objective = json!({"name": "kills", "title": "Kills"});
    if let Some(format) = format {
        objective["format"] = json!(format);
    }
    row["objectives"] = json!([objective]);
    row["scores"] = json!([{"objective": "kills", "owner": owner, "value": 7}]);
    row["slots"] = json!({"2": "kills"});
    row
}

/// The score line under a name: for a player and a mob, near and far, in each number format,
/// beside the name's own rules.
fn score_rows() -> Vec<Value> {
    let near = at(5.0);
    let mut rows = Vec::new();
    for (word, format) in
        [("plain", None), ("blank", Some("blank")), ("gold", Some("style:gold")), ("fixed", Some("fixed:many"))]
    {
        rows.push(scored(mob(&format!("score-mob-{word}"), Mob { tag: near, ..MOB }), MOB_NAME, format));
        rows.push(scored(
            tag(&format!("score-player-{word}"), Tag { position: [5.5, 64.0, 0.5], ..TAG }),
            TAG.name,
            format,
        ));
    }
    for distance in [9.5, 10.1] {
        rows.push(scored(
            mob(&format!("score-mob-{}", float_text(distance)), Mob { tag: at(distance), ..MOB }),
            MOB_NAME,
            None,
        ));
    }
    for (word, below) in [("short", 3.0), ("long", 20.0)] {
        for distance in [5.0, 15.0] {
            let label = format!("score-mob-below-{word}-{}", float_text(distance));
            rows.push(scored(mob(&label, Mob { tag: at(distance), below: Some(below), ..MOB }), MOB_NAME, None));
        }
    }
    rows.push(scored(mob("score-mob-other-owner", Mob { tag: near, ..MOB }), "someone", None));
    rows.push(scored(mob("score-mob-sneaking", Mob { tag: Tag { shift: true, ..near }, ..MOB }), MOB_NAME, None));
    rows.push(scored(mob("score-mob-hidden-name", Mob { tag: near, visible: false, ..MOB }), MOB_NAME, None));
    rows.push(scored(
        mob("score-entity", Mob { kind: "minecraft:minecart", renderer: "entity", tag: near, ..MOB }),
        MOB_NAME,
        None,
    ));
    rows.push(scored(
        tag("score-player-crouching", Tag { position: [5.5, 64.0, 0.5], crouching: true, shift: true, ..TAG }),
        TAG.name,
        None,
    ));
    rows.push(scored(
        tag("score-player-deadmau5", Tag { name: "deadmau5", position: [5.5, 64.0, 0.5], ..TAG }),
        "deadmau5",
        None,
    ));
    let mut sidebar = scored(mob("score-mob-sidebar-only", Mob { tag: near, ..MOB }), MOB_NAME, None);
    sidebar["slots"] = json!({"1": "kills"});
    rows.push(sidebar);
    rows
}

/// The rows for entities that are not players.
pub fn rows() -> Vec<Value> {
    [
        visibility_rows(),
        distance_rows(),
        attachment_rows(),
        team_visibility_rows(),
        team_gate_rows(),
        team_dress_rows(),
        team_plain_rows(),
        score_rows(),
    ]
    .concat()
}
