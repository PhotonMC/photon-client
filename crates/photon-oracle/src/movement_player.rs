use super::*;
use photon_world::local_player::LocalPlayer;

/// What a row sets on its new player: the camera, and the abilities packet's flying speed, which
/// comes after the game mode as the packet does.
pub(super) fn start_player(player: &mut LocalPlayer, row: &Value) -> Result<(), String> {
    // A row whose view is another entity's (SetCamera) has no controlled camera.
    player.set_controlled_camera(row.get("camera_entity").is_none());
    if row.get("flying_speed").is_none() {
        return Ok(());
    }
    let abilities = player.abilities();
    player.set_abilities(photon_world::ClientPlayerAbilities::new(
        abilities.invulnerable(),
        abilities.flying(),
        abilities.may_fly(),
        abilities.instabuild(),
        u32::from_str_radix(string(row, "flying_speed")?, 16)
            .map(f32::from_bits)
            .map_err(|error| format!("flying_speed: {error}"))?,
        abilities.walking_speed(),
    ));
    Ok(())
}
