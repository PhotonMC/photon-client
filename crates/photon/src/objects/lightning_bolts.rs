//! The lightning bolts (`LightningBoltRenderer`): the jagged translucent column each one's seed
//! draws, added to what is behind it as the dragon's rays are.

use photon_net::connection::PlaySession;
use photon_render::joml::Pose;
use photon_render::objects::lightning::lightning_bolt;

use crate::mobs::MobModels;
use crate::mobs::monsters::ender_dragons::ray_vertex;
use crate::overlays::FrameView;

/// The bolts the server added, handed to the rays' buffer.
pub fn remote_lightning(session: &PlaySession, models: &mut MobModels, view: FrameView) {
    let world = session.world();
    let partial = view.partial as f32;
    for entity in world
        .entities()
        .drawn_of_kind(|entity| entity.is_lightning_bolt())
    {
        let (Some(state), Some(snapshot)) = (
            entity.lightning(),
            world.entity_snapshot_of(entity, partial),
        ) else {
            continue;
        };
        let (x, y, z) = snapshot.position;
        let mut base = Pose::default();
        base.translate(
            (x - f64::from(view.origin[0])) as f32,
            (y - f64::from(view.origin[1])) as f32,
            (z - f64::from(view.origin[2])) as f32,
        );
        for quad in lightning_bolt(&base, state.seed()).as_chunks::<4>().0 {
            models
                .rays
                .extend([0, 1, 2, 0, 2, 3].map(|index| ray_vertex(quad[index])));
        }
    }
}
