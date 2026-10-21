//! The flames over burning entities (`EntityRenderDispatcher.submit`): whatever the entity is, and
//! whether or not its body is drawn.

use photon_core::math::Vec3;
use photon_net::connection::PlaySession;
use photon_render::block::world_mesh::BlockRenderer;
use photon_render::entity::name_tag::camera_orientation;
use photon_render::objects::flame::{Flame, camera_twist, flame_vertices};
use photon_render::scene::Vertex;

use crate::overlays::{FrameView, packed_light};

/// The flames of every entity that displays them. Not carried: the render offset of an enderman's
/// shake and of the other entities that have one.
pub fn flames(
    session: &PlaySession,
    renderer: &BlockRenderer,
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    let (Some(first), Some(second)) = (
        renderer.sprite_rect("minecraft:block/fire_0"),
        renderer.sprite_rect("minecraft:block/fire_1"),
    ) else {
        return;
    };
    let world = session.world();
    let partial = view.partial as f32;
    let twist = camera_twist(&camera_orientation(view.rotation.0, view.rotation.1));
    for entity in world.entities().drawn_with_invisible() {
        if !entity.displays_fire() {
            continue;
        }
        let Some(snapshot) = world.entity_snapshot(entity.id(), partial) else {
            continue;
        };
        let (x, y, z) = snapshot.position;
        let size = entity.leash_size();
        let flame = Flame {
            origin: [
                (x - f64::from(view.origin[0])) as f32,
                (y - f64::from(view.origin[1])) as f32,
                (z - f64::from(view.origin[2])) as f32,
            ],
            size: (size.width, size.height),
            light: packed_light(world, Vec3::new(x, y + f64::from(size.eye_height), z)),
            twist,
        };
        vertices.extend(flame_vertices(&flame, [first, second]));
    }
}
