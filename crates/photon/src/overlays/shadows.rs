use super::shadow_radius::{self, TypeShadow};
use super::*;
use crate::objects::displays;
use photon_world::{ClientWorld, EntityView, ShadowInput};
use std::collections::HashMap;

/// Whether an entity casts no shadow: an invisible one, or a spectator.
fn casts_no_shadow(entity: EntityView<'_>) -> bool {
    entity.is_invisible() || entity.is_player() && entity.is_spectator()
}

/// The squared distance from the camera to a place.
fn camera_distance_sq(camera: &TagCamera, at: (f64, f64, f64)) -> f64 {
    let (dx, dy, dz) = (
        camera.position.x - at.0,
        camera.position.y - at.1,
        camera.position.z - at.2,
    );
    dx * dx + dy * dy + dz * dz
}

/// An entity's shadow radius and strength, or `None` when it casts nothing this frame.
///
/// `entity_shadow_pieces` makes none for a radius of nothing, nor for a positive strength 16
/// blocks or more from the camera; skipping those before the snapshot leaves the vertices as
/// they were and spares the snapshot.
fn shadow_of(
    world: &ClientWorld,
    camera: &TagCamera,
    entity: EntityView<'_>,
    facts: TypeShadow,
    partial: f32,
) -> Option<(f32, f32)> {
    // `EntityRenderer.extractShadow`: an invisible entity casts none, nor does a spectator.
    if casts_no_shadow(entity) {
        return None;
    }
    if entity.display().is_some() {
        let snapshot = world.entity_snapshot_of(entity, partial)?;
        let at = (camera.position.x, camera.position.y, camera.position.z);
        return displays::display_shadow(entity, snapshot.position, at, partial);
    }
    let (radius, strength) = shadow_radius::of(entity, facts)?;
    let too_far = strength > 0.0 && camera_distance_sq(camera, entity.position()) >= 256.0;
    (radius > 0.0 && !too_far).then_some((radius, strength))
}

/// What casts a shadow this frame, in network id order, and the local player last.
fn casters(
    world: &ClientWorld,
    camera: &TagCamera,
    local: Option<Vec3>,
    partial: f32,
) -> Vec<Caster> {
    let mut casters: Vec<Caster> = Vec::new();
    // Only a display casts a shadow farther than 16 blocks from the camera (`shadow_of`), so the
    // rest are the entities of the sections around it.
    let entities = world.entities();
    let at = camera.position;
    let mut ids = entities.ids_near((at.x, at.y, at.z), 16.0);
    ids.extend(entities.display_ids());
    ids.sort_unstable();
    ids.dedup();
    // What a type's shadow depends on is looked up once per type, not once per entity.
    let mut types: HashMap<_, TypeShadow> = HashMap::new();
    for entity in ids.into_iter().filter_map(|id| entities.at(id)) {
        let facts = *types
            .entry(entity.type_id)
            .or_insert_with(|| TypeShadow::of_type(entity));
        let Some((radius, strength)) = shadow_of(world, camera, entity, facts, partial) else {
            continue;
        };
        let Some(snapshot) = world.entity_snapshot_of(entity, partial) else {
            continue;
        };
        casters.push(Caster {
            drawn: snapshot.position,
            ticked: entity.position(),
            radius,
            strength,
        });
    }
    if let Some(at) = local {
        casters.push(Caster {
            drawn: (at.x, at.y, at.z),
            ticked: (at.x, at.y, at.z),
            radius: 0.5,
            strength: 1.0,
        });
    }
    casters
}

/// `EntityRenderer.extractShadow` and `ShadowFeatureRenderer` for the
/// entities the frame draws: the shadow each casts on the blocks below it.
/// `shadowRadius` and `shadowStrength` are the renderers' (`shadow_radius`). `local` is the player's own place, when
/// the camera shows them.
pub fn entity_shadows(
    session: &PlaySession,
    textures: &photon_render::entity::textures::EntityTextures,
    camera: &TagCamera,
    local: Option<Vec3>,
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    let Some(rect) = textures.rect(SHADOW_TEXTURE) else {
        return;
    };
    let world = session.world();
    let casters = casters(world, camera, local, view.partial as f32);
    for Caster {
        drawn,
        ticked,
        radius,
        strength,
    } in casters
    {
        let radius = radius.min(32.0);
        let pieces = world.entity_shadow_pieces(ShadowInput {
            position: drawn,
            radius,
            strength,
            distance_sq: camera_distance_sq(camera, ticked),
            // The sky's dimming (`gameplay/sky_light_level`) is not carried.
            sky_darken: 0,
        });
        let translation = [
            (drawn.0 - camera.position.x) as f32,
            (drawn.1 - camera.position.y) as f32,
            (drawn.2 - camera.position.z) as f32,
        ];
        vertices.extend(photon_render::entity::shadow::vertices(
            &pieces,
            radius,
            translation,
            rect,
            camera.relative,
        ));
    }
}
