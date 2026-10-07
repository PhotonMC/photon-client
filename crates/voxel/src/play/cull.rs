//! Which entities a frame can leave out: those wholly outside the view cone.
//!
//! `EntityRenderer.shouldRender` tests the entity's box grown by half a block (more for a few
//! renderers) against the frustum. The box tested here holds the reference's: it is grown further,
//! by the scale attribute and by where the entity is drawn between two ticks, so whatever the
//! reference draws is drawn here too, and what is left out could not reach the screen. The
//! renderers whose drawing reaches far past the entity (beams, lines, the dragon, the bolt, a
//! minecart's raised block) are never left out.

use std::collections::BTreeSet;

use voxel_render::visibility::Frustum;
use voxel_world::ClientWorld;

/// More than the largest growth a renderer gives its culling box (the illusioner's three
/// blocks sideways plus the half block every box gets).
const MARGIN: f64 = 3.5;

/// The types whose drawing (with every `*_minecart`) is not bounded by the box around them.
const NEVER_LEFT_OUT: [&str; 7] = [
    "minecraft:ender_dragon",
    "minecraft:lightning_bolt",
    "minecraft:fishing_bobber",
    "minecraft:guardian",
    "minecraft:elder_guardian",
    "minecraft:end_crystal",
    "minecraft:minecart",
];

/// The entities wholly outside `frustum` for a frame drawn `partial` of a tick on.
pub fn out_of_view(world: &ClientWorld, frustum: &Frustum, partial: f32) -> BTreeSet<i32> {
    world
        .entities()
        .all()
        .filter(|entity| {
            let name = voxel_data::entity_type::name(entity.type_id).unwrap_or_default();
            if NEVER_LEFT_OUT.contains(&name) || name.ends_with("_minecart") {
                return false;
            }
            let bounds = entity.bounding_box();
            let grown = f64::from(entity.scale().max(1.0)) - 1.0;
            let size = (bounds.max_x - bounds.min_x)
                .max(bounds.max_y - bounds.min_y)
                .max(bounds.max_z - bounds.min_z);
            let margin = MARGIN + size * grown;
            // Where it is drawn lies between where it was and where it is.
            let (x, y, z) = entity.position();
            let (dx, dy, dz) = entity.interpolated_position(partial);
            let shift = [dx - x, dy - y, dz - z];
            let min = [bounds.min_x, bounds.min_y, bounds.min_z];
            let max = [bounds.max_x, bounds.max_y, bounds.max_z];
            let low = std::array::from_fn(|axis| min[axis] + shift[axis].min(0.0) - margin);
            let high = std::array::from_fn(|axis| max[axis] + shift[axis].max(0.0) + margin);
            !frustum.is_visible(low, high)
        })
        .map(|entity| entity.id())
        .collect()
}
