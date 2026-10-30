//! Which entities a frame leaves out (`LevelExtractor.isEntityVisible`): those too far from the
//! camera for their type, those whose renderer's box is wholly outside the view cone, and those
//! standing in a section that has not shown yet.

use photon_core::ids::EntityId;
use std::collections::BTreeSet;

use photon_core::aabb::Aabb;
use photon_render::visibility::Frustum;
use photon_world::{ClientWorld, EntityView};

use crate::world::sections::SectionMeshes;

/// The camera a frame culls from.
pub struct CullView<'a> {
    pub frustum: &'a Frustum,
    /// `Camera.position`.
    pub camera: (f64, f64, f64),
    /// `Options.getEffectiveRenderDistance`, in chunks.
    pub render_distance: i32,
}

/// `Entity.viewScale` as `LevelExtractor.extractVisibleEntities` sets it: the render distance
/// over eight, held between one and two and a half, times the entity distance scaling option
/// (its default, one).
fn view_scale(render_distance: i32) -> f64 {
    (f64::from(render_distance) / 8.0).clamp(1.0, 2.5)
}

/// The entities the frame leaves out.
pub fn out_of_view(
    world: &ClientWorld,
    sections: &SectionMeshes,
    view: &CullView,
) -> BTreeSet<EntityId> {
    let scale = view_scale(view.render_distance);
    let bounds = world.dimension();
    world
        .entities()
        .all()
        .filter(|entity| {
            let shown = should_render(world, *entity, view, scale)
                || world.entity_carries_local_player(entity.id());
            if !shown {
                return true;
            }
            let (x, y, z) = entity.position();
            let block = [x, y, z].map(photon_core::math::floor_f64);
            let outside = block[1] < bounds.min_y || block[1] >= bounds.min_y + bounds.height;
            let section = block.map(|value| value.div_euclid(16));
            !outside && !sections.compiled_and_visible((section[0], section[1], section[2]))
        })
        .map(|entity| entity.id())
        .collect()
}

/// `EntityRenderer.shouldRender` with the overrides that widen it: a guardian's beam, an end
/// crystal's beam and a shulker sliding to its block. (The fishing hook's owner and the falling
/// block's landing narrow it; their renderers test those.)
fn should_render(world: &ClientWorld, entity: EntityView<'_>, view: &CullView, scale: f64) -> bool {
    if !entity.should_render(view.camera, scale) {
        return false;
    }
    let Some(culling) = entity.culling_box() else {
        return true;
    };
    let mut aabb = culling.inflate_all(0.5);
    if aabb.has_nan() || aabb.size() == 0.0 {
        let (x, y, z) = entity.position();
        aabb = Aabb::new(x - 2.0, y - 2.0, z - 2.0, x + 2.0, y + 2.0, z + 2.0);
    }
    if visible(view.frustum, aabb) {
        return true;
    }
    // `Leashable`: the lead to its holder may cross the view.
    if let Some(holder) = entity
        .leash_holder_id()
        .and_then(|id| world.entities().get(&id))
    {
        let held = holder
            .culling_box()
            .unwrap_or_else(|| holder.bounding_box());
        if visible(view.frustum, held) || visible(view.frustum, aabb.minmax(held)) {
            return true;
        }
    }
    let name = photon_data::entity_type::name(entity.type_id).unwrap_or_default();
    match name {
        "minecraft:end_crystal" => entity
            .end_crystal()
            .is_some_and(|crystal| crystal.beam_target().is_some()),
        "minecraft:guardian" | "minecraft:elder_guardian" => {
            guardian_beam_visible(world, entity, view.frustum)
        }
        "minecraft:shulker" => shulker_slide_visible(entity, view.frustum),
        _ => false,
    }
}

fn visible(frustum: &Frustum, aabb: Aabb) -> bool {
    frustum.is_visible(
        [aabb.min_x, aabb.min_y, aabb.min_z],
        [aabb.max_x, aabb.max_y, aabb.max_z],
    )
}

/// `GuardianRenderer.shouldRender`: the line from its eyes to the middle of what it attacks.
fn guardian_beam_visible(world: &ClientWorld, guardian: EntityView<'_>, frustum: &Frustum) -> bool {
    let Some(target) = guardian.guardian_attack_target().and_then(|id| {
        world
            .entities()
            .all()
            .find(|target| target.id() == EntityId(id))
    }) else {
        return false;
    };
    let (tx, ty, tz) = target.interpolated_position(1.0);
    let ty = ty + f64::from(target.bb_height()) * 0.5;
    let (sx, sy, sz) = guardian.interpolated_position(1.0);
    let sy = sy + f64::from(guardian.eye_height());
    visible(frustum, Aabb::new(sx, sy, sz, tx, ty, tz))
}

/// `ShulkerRenderer.shouldRender`: from where its slide starts to its block. The start is the
/// slide's offset, taken as a place, as the reference does.
fn shulker_slide_visible(shulker: EntityView<'_>, frustum: &Frustum) -> bool {
    let Some([ox, oy, oz]) = shulker.shulker_render_offset(0.0) else {
        return false;
    };
    // `EntityType.SHULKER`'s size, one block each way.
    let (half_width, half_height) = (0.5, 0.5);
    let (x, y, z) = shulker.position();
    let target = [x, y, z].map(|value| photon_core::math::floor_f64(value) as f64);
    let aabb = Aabb::new(
        ox,
        oy + half_height,
        oz,
        target[0] + 0.5,
        target[1] + half_height,
        target[2] + 0.5,
    );
    visible(frustum, aabb.inflate(half_width, half_height, half_width))
}

impl super::Play {
    /// Tells the session which entities this frame leaves out. The camera entity is one of them
    /// unless the camera leaves its eyes or it sleeps (`LevelExtractor`).
    pub(super) fn leave_out_of_view(&mut self, frame: &super::FrameCamera) {
        let mut out_of_view = if self.sections.culling() {
            out_of_view(
                self.session.world(),
                &self.sections,
                &CullView {
                    frustum: &frame.frustum,
                    camera: (frame.camera.x, frame.camera.y, frame.camera.z),
                    render_distance: frame.render_distance,
                },
            )
        } else {
            Default::default()
        };
        let world = self.session.world();
        if let Some(camera) = self.session.camera_entity()
            && self.controls.camera_type == crate::view::CameraType::FirstPerson
            && !world
                .entities()
                .at(camera)
                .is_some_and(|entity| entity.is_sleeping())
        {
            out_of_view.insert(camera);
        }
        self.session.set_entities_out_of_view(out_of_view);
    }
}
