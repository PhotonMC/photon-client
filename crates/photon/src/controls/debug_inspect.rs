//! F3+I (`KeyboardHandler.copyRecreateCommand`): a command that recreates what the crosshair is on
//! goes to the clipboard. A game master has the tag read from the server first.

use super::*;
use photon_game::game_mode::Aim;
use photon_net::connection::DebugQuery;

/// F3+I's `/setblock` for a block, from its serialized state.
fn setblock_command((x, y, z): (i32, i32, i32), state: &str) -> String {
    format!("/setblock {x} {y} {z} {state}")
}

/// F3+I's `/summon` for an entity: its type and its position, each number with two decimals.
fn summon_command(id: &str, position: [f64; 3]) -> String {
    let fixed = photon_gui::hud::debug::fixed;
    format!(
        "/summon {id} {} {} {}",
        fixed(position[0], 2),
        fixed(position[1], 2),
        fixed(position[2], 2)
    )
}

impl Controls {
    /// F3+I: the command for the block or entity the crosshair is on, and the `[Debug]:` line that
    /// says it came from the client. A game master has the tag asked of the server, and the
    /// command is copied when the answer comes (`receive_query_answers`).
    ///
    /// With Shift held a game master gets the tag from the client's own copy instead; that part
    /// is not ported, so nothing is copied then.
    pub(super) fn debug_copy_recreate(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        if session.player().is_none() || session.reduced_debug_info() {
            return;
        }
        let game_master = session.may_change_game_mode();
        if game_master && self.debug.shift {
            return;
        }
        let Some(target) = self.inspected(session) else {
            return;
        };
        if game_master {
            if let Err(error) = ask_server(session, target) {
                log::warn!("the tag query was not sent: {error}");
            }
            return;
        }
        let (command, feedback) = match target {
            Inspected::Block { pos, state } => {
                (setblock_command(pos, &state), "debug.inspect.client.block")
            }
            Inspected::Entity { name, position, .. } => (
                summon_command(&name, position),
                "debug.inspect.client.entity",
            ),
        };
        surface.set_clipboard(&command);
        self.debug_say(feedback, assets);
    }

    /// What the crosshair is on, if it is on a block or an entity the client knows the type of.
    fn inspected(&self, session: &PlaySession) -> Option<Inspected> {
        let player = session.player()?;
        let world = session.world();
        match crate::view::camera_aim(session, player, 1.0) {
            Aim::Block { x, y, z, .. } => Some(Inspected::Block {
                pos: (x, y, z),
                state: photon_world::block::rules::serialize_state(
                    world.block_state_or_air(x, y, z),
                ),
            }),
            Aim::Entity { id, position, .. } => {
                let name = world
                    .entities()
                    .all()
                    .find(|entity| entity.id() == id)
                    .and_then(|entity| photon_data::entity_type::name(entity.type_id))?;
                Some(Inspected::Entity {
                    id,
                    name: name.to_owned(),
                    position,
                })
            }
            Aim::Miss => None,
        }
    }

    /// The tags the server answered F3+I's questions with: the command goes to the clipboard.
    pub fn receive_query_answers(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        for answer in session.take_query_answers() {
            let (command, feedback) = match answer.query {
                DebugQuery::Block { pos, state } => (
                    block_command_with_tag(pos, &state, answer.tag),
                    "debug.inspect.server.block",
                ),
                DebugQuery::Entity { id, name, position } => {
                    // The entity as it stands when the answer comes, else where it stood.
                    let position = session
                        .world()
                        .entities()
                        .all()
                        .find(|entity| entity.id() == id)
                        .map_or(position, |entity| {
                            let (x, y, z) = entity.position();
                            [x, y, z]
                        });
                    (
                        entity_command_with_tag(&name, position, answer.tag),
                        "debug.inspect.server.entity",
                    )
                }
            };
            surface.set_clipboard(&command);
            self.debug_say(feedback, assets);
        }
    }
}

/// What F3+I found under the crosshair.
enum Inspected {
    Block {
        pos: (i32, i32, i32),
        state: String,
    },
    Entity {
        id: photon_core::ids::EntityId,
        name: String,
        position: [f64; 3],
    },
}

/// `queryBlockEntityTag` or `queryEntityTag`.
fn ask_server(
    session: &mut PlaySession,
    target: Inspected,
) -> std::result::Result<(), photon_net::connection::ConnectionError> {
    match target {
        Inspected::Block { pos, state } => session.query_block_entity_tag(pos, state),
        Inspected::Entity { id, name, position } => session.query_entity_tag(id, name, position),
    }
}

/// `copyCreateBlockCommand`: the state, then the tag as `Tag.toString` writes it.
fn block_command_with_tag(
    pos: (i32, i32, i32),
    state: &str,
    tag: Option<Vec<photon_core::nbt::Field>>,
) -> String {
    let mut description = state.to_owned();
    if let Some(fields) = tag {
        description.push_str(&photon_core::snbt::compact(
            &photon_core::nbt::FieldValue::Compound(fields),
        ));
    }
    setblock_command(pos, &description)
}

/// `copyCreateEntityCommand`: the tag, less its UUID and position, written as the pretty
/// printer writes it.
fn entity_command_with_tag(
    name: &str,
    position: [f64; 3],
    tag: Option<Vec<photon_core::nbt::Field>>,
) -> String {
    let command = summon_command(name, position);
    let Some(fields) = tag else {
        return command;
    };
    let text = photon_core::snbt::pretty_compound_without(&fields, &["UUID", "Pos"]);
    format!("{command} {text}")
}

#[cfg(test)]
mod tests {
    use super::{setblock_command, summon_command};

    #[test]
    fn the_commands_put_the_position_where_the_reference_does() {
        assert_eq!(
            setblock_command(
                (-3, 64, 12),
                "minecraft:lever[face=wall,facing=north,powered=false]"
            ),
            "/setblock -3 64 12 minecraft:lever[face=wall,facing=north,powered=false]"
        );
        assert_eq!(
            summon_command("minecraft:cow", [1.5, 64.0, -2.125]),
            "/summon minecraft:cow 1.50 64.00 -2.13"
        );
    }

    #[test]
    fn a_state_is_serialized_with_its_properties_in_name_order() {
        let default_of = |name: &str| {
            photon_data::block::registry::BLOCKS
                .iter()
                .find(|block| block.name == name)
                .expect("a block")
                .default_state_id()
        };
        let serialize = photon_world::block::rules::serialize_state;
        assert_eq!(serialize(default_of("minecraft:stone")), "minecraft:stone");
        assert_eq!(
            serialize(default_of("minecraft:lever")),
            "minecraft:lever[face=wall,facing=north,powered=false]"
        );
    }
}
