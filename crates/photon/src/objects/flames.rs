//! The flames over burning entities (`EntityRenderDispatcher.submit`): whatever the entity is, and
//! whether or not its body is drawn.

use photon_core::math::Vec3;
use photon_net::connection::PlaySession;
use photon_render::block::world_mesh::BlockRenderer;
use photon_render::entity::name_tag::camera_orientation;
use photon_render::joml::Quaternion;
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
        let size = entity.leash_size();
        let body = Body {
            position: snapshot.position,
            size: (size.width, size.height),
            eye_height: size.eye_height,
        };
        vertices.extend(flame_vertices(
            &flame(world, &body, (view, twist)),
            [first, second],
        ));
    }
}

/// `EntityRenderDispatcher.submit` for the player who looks from outside their body, who is not
/// among the entities. Not drawn for a spectator, and drawn whether or not it is invisible.
pub fn local_flames(
    session: &PlaySession,
    (renderer, view): (&BlockRenderer, FrameView),
    vertices: &mut Vec<Vertex>,
) {
    let world = session.world();
    let Some(player) = session.player() else {
        return;
    };
    let burning = world
        .local_player()
        .is_some_and(photon_world::ClientPlayerState::is_on_fire);
    let (Some(first), Some(second)) = (
        renderer.sprite_rect("minecraft:block/fire_0"),
        renderer.sprite_rect("minecraft:block/fire_1"),
    ) else {
        return;
    };
    if !burning || player.is_spectator() {
        return;
    }
    let twist = camera_twist(&camera_orientation(view.rotation.0, view.rotation.1));
    let at = player.interpolated_position(view.partial);
    let body = Body {
        position: (at.x, at.y, at.z),
        size: (player.bb_width(), player.bb_height()),
        eye_height: player.eye_height(),
    };
    vertices.extend(flame_vertices(
        &flame(world, &body, (view, twist)),
        [first, second],
    ));
}

/// Where an entity is and how big.
struct Body {
    position: (f64, f64, f64),
    size: (f32, f32),
    eye_height: f32,
}

/// What an entity's flames are drawn from, lit where its eyes are.
fn flame(
    world: &photon_world::ClientWorld,
    body: &Body,
    (view, twist): (FrameView, Quaternion),
) -> Flame {
    let (x, y, z) = body.position;
    Flame {
        origin: [
            (x - f64::from(view.origin[0])) as f32,
            (y - f64::from(view.origin[1])) as f32,
            (z - f64::from(view.origin[2])) as f32,
        ],
        size: body.size,
        light: packed_light(world, Vec3::new(x, y + f64::from(body.eye_height), z)),
        twist,
    }
}
