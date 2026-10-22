use super::light::packed_light;
use super::wear::burning_light;
use super::*;
use crate::equipment::arm_poses;

/// Whether a player hangs upside down: the cape shows and the profile is named Dinnerbone or
/// Grumm (not asleep, which turns the body its own way).
pub(super) fn flipped(
    session: &PlaySession,
    entity: &photon_world::ClientEntity,
    snapshot: &photon_world::EntitySnapshot,
) -> bool {
    snapshot.pose != photon_world::ClientEntityPose::Sleeping
        && session
            .player_name(entity.uuid)
            .is_some_and(|name| entity.is_upside_down_player(name))
}

/// Where another player's model stands in the frame, in the skin that is drawn.
pub(super) fn remote_placement(
    (session, entity, snapshot): (
        &PlaySession,
        &photon_world::ClientEntity,
        &photon_world::EntitySnapshot,
    ),
    (view, skin): (FrameView, photon_render::entity::textures::TextureRect),
) -> Placement {
    let crouching = snapshot.pose == photon_world::ClientEntityPose::Crouching;
    let (x, y, z) = snapshot.position;
    let eye = Vec3::new(x, y + if crouching { 1.27 } else { 1.62 }, z);
    let partial = view.partial as f32;
    Placement {
        feet: [
            (x - f64::from(view.origin[0])) as f32,
            (y - f64::from(view.origin[1])) as f32,
            (z - f64::from(view.origin[2])) as f32,
        ],
        body_rot: snapshot.body_yaw,
        x_rot: snapshot.pitch,
        swim_amount: snapshot.swim_amount,
        in_water: false,
        visually_swimming: snapshot.pose == photon_world::ClientEntityPose::Swimming,
        crouching,
        light: burning_light(packed_light(session.world(), eye), snapshot.burning),
        skin,
        death_time: snapshot.death_time,
        red_overlay: snapshot.red_overlay,
        fall_flying: arm_poses::fall_flying(entity, snapshot, partial),
        auto_spin: entity
            .is_auto_spin_attack()
            .then(|| entity.tick_count() as f32 + partial),
        upside_down: snapshot.upside_down,
    }
}
