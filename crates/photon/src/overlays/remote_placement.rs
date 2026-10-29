use super::light::packed_light;
use super::wear::burning_light;
use super::*;
use crate::equipment::arm_poses;
use photon_render::entity::player_renderer::Sleep;
use photon_world::EntityView;

/// Whether an avatar is upside down: the cape shows and a player's profile, or a mannequin's custom
/// name, is Dinnerbone or Grumm.
pub(super) fn flipped(session: &PlaySession, entity: EntityView<'_>) -> bool {
    if !entity.is_player() {
        // `AvatarRenderer.isEntityUpsideDown` for a mannequin: the cape shows, then the living rule
        // (a custom name of Dinnerbone or Grumm).
        return entity.shows_cape() && entity.has_upside_down_name();
    }
    session
        .player_name(entity.uuid)
        .is_some_and(|name| entity.is_upside_down_player(name))
}

/// Where another player's model stands in the frame, in the skin that is drawn.
pub(super) fn remote_placement(
    (session, entity, snapshot): (&PlaySession, EntityView<'_>, &photon_world::EntitySnapshot),
    (view, skin): (FrameView, photon_render::entity::textures::TextureRect),
) -> Placement {
    let crouching = snapshot.pose == photon_world::ClientEntityPose::Crouching;
    let (x, y, z) = snapshot.position;
    let eye = Vec3::new(
        snapshot.light_probe.0,
        snapshot.light_probe.1,
        snapshot.light_probe.2,
    );
    let partial = view.partial as f32;
    let sleeping = (snapshot.pose == photon_world::ClientEntityPose::Sleeping)
        .then(|| in_bed(session.world(), entity.sleeping_pos(), snapshot.body_yaw));
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
        light: burning_light(
            packed_light(session.world(), eye),
            snapshot.full_block_light,
        ),
        skin,
        death_time: snapshot.death_time,
        red_overlay: snapshot.red_overlay,
        fall_flying: arm_poses::fall_flying(entity, snapshot, partial),
        auto_spin: entity
            .is_auto_spin_attack()
            .then(|| entity.tick_count() as f32 + partial),
        // A sleeper keeps the mirrored look but lies down instead of turning over.
        upside_down: snapshot.upside_down.filter(|_| sleeping.is_none()),
        sleeping,
    }
}

/// A sleeper's lying (`Entity.getBedOrientation` for the bed it sleeps in).
pub(super) fn in_bed(
    world: &photon_world::ClientWorld,
    bed: Option<photon_core::binary::BlockPos>,
    body_rot: f32,
) -> Sleep {
    let facing =
        bed.and_then(|pos| photon_world::block::bed::bed_orientation(world, (pos.x, pos.y, pos.z)));
    let step = facing.map(|facing| {
        let (x, _, z) = facing.step();
        (x, z)
    });
    // `getEyeHeight(Pose.STANDING)` of a player.
    Sleep::in_bed(step, 1.62, body_rot)
}
