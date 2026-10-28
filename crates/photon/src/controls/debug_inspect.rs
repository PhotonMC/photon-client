//! F3+I (`KeyboardHandler.copyRecreateCommand`): a command that recreates what the crosshair is on
//! goes to the clipboard.

use super::*;
use photon_game::game_mode::Aim;

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
    /// F3+I for a player who is no game master: the command for the block or entity the crosshair
    /// is on, and the `[Debug]:` line that says it came from the client.
    ///
    /// A game master gets the tag added (from the server unless Shift is held); that part is not
    /// ported, so nothing is copied for one.
    pub(super) fn debug_copy_recreate(
        &mut self,
        session: &PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        let Some(player) = session.player() else {
            return;
        };
        if session.reduced_debug_info() || session.may_change_game_mode() {
            return;
        }
        let world = session.world();
        let (command, feedback) = match crate::view::camera_aim(session, player, 1.0) {
            Aim::Block { x, y, z, .. } => (
                setblock_command(
                    (x, y, z),
                    &photon_world::block::rules::serialize_state(world.block_state_or_air(x, y, z)),
                ),
                "debug.inspect.client.block",
            ),
            Aim::Entity { id, position, .. } => {
                let Some(name) = world
                    .entities()
                    .all()
                    .find(|entity| entity.id() == id)
                    .and_then(|entity| photon_data::entity_type::name(entity.type_id))
                else {
                    return;
                };
                (
                    summon_command(name, position),
                    "debug.inspect.client.entity",
                )
            }
            Aim::Miss => return,
        };
        surface.set_clipboard(&command);
        self.debug_say(feedback, assets);
    }
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
