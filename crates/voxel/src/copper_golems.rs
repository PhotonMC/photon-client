//! The copper golem (`CopperGolemRenderer`): its texture by how far it has
//! oxidized, its glowing eyes (`LivingEntityEmissiveLayer`), and its pose, the
//! walk (with arms raised while it holds something), the idle head spin and the
//! chest animations.
//!
//! What it holds in its hands (`ItemInHandLayer`), a head item
//! (`CustomHeadLayer`) and the block on its antenna (`BlockDecorationLayer`,
//! from the block item in the saddle slot) are carried with it.

use voxel_net::connection::PlaySession;
use voxel_render::entity_textures::EntityTextures;
use voxel_render::held_item::{
    copper_golem_antenna_pose, copper_golem_hand_pose, copper_golem_head_pose,
};
use voxel_render::mob_model::{
    CopperGolemModelState, QuadrupedState, mob_vertices, mob_vertices_glow, placement_pose,
};
use voxel_render::player_model::{Arm, Part};
use voxel_render::scene::Vertex;
use voxel_world::EquipmentSlot;

use crate::held_items::{HeldItem, Place, block_item_state};
use crate::mobs::{MobModels, placement};
use crate::overlays::FrameView;

const LAYER: &str = "minecraft:copper_golem#main";

/// `CopperGolemOxidationLevels`: the body's and the eyes' textures, from
/// unaffected to oxidized.
const TEXTURES: [(&str, &str); 4] = [
    (
        "entity/copper_golem/copper_golem",
        "entity/copper_golem/copper_golem_eyes",
    ),
    (
        "entity/copper_golem/copper_golem_exposed",
        "entity/copper_golem/copper_golem_eyes_exposed",
    ),
    (
        "entity/copper_golem/copper_golem_weathered",
        "entity/copper_golem/copper_golem_eyes_weathered",
    ),
    (
        "entity/copper_golem/copper_golem_oxidized",
        "entity/copper_golem/copper_golem_eyes_oxidized",
    ),
];

/// The paths (under `textures/`) of the textures a copper golem wears.
pub fn textures() -> impl Iterator<Item = &'static str> {
    TEXTURES.into_iter().flat_map(|(body, eyes)| [body, eyes])
}

/// `CopperGolemRenderer` for the copper golems the server added.
pub fn remote_copper_golems(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    (view, held): (FrameView, &mut Vec<HeldItem>),
) -> Vec<Vertex> {
    let Some(golem) = voxel_data::entity_type::id("minecraft:copper_golem") else {
        return Vec::new();
    };
    let world = session.world();
    let partial = view.partial as f32;
    let mut vertices = Vec::new();
    let mut glow = Vec::new();
    for entity in world.entities().filter(|entity| entity.type_id == golem) {
        let (body, eyes) = TEXTURES[usize::from(entity.copper_golem_weather().min(3))];
        let (Some(snapshot), Some(rect), Some(model)) = (
            world.entity_snapshot(entity.id(), partial),
            textures.rect(body),
            models.model(LAYER),
        ) else {
            continue;
        };
        let (right, left) = (
            entity.item_held_by_arm(false),
            entity.item_held_by_arm(true),
        );
        model.setup_copper_golem(&CopperGolemModelState {
            quadruped: QuadrupedState {
                head_y_rot: voxel_core::math::wrap_degrees_f32(
                    snapshot.head_yaw - snapshot.body_yaw,
                ),
                x_rot: snapshot.pitch,
                walk_position: snapshot.walk_position,
                walk_speed: snapshot.walk_speed,
            },
            age_in_ticks: snapshot.age_in_ticks,
            holds_item: right.is_some() || left.is_some(),
            animations: entity
                .copper_golem()
                .map(|golem| golem.playing)
                .unwrap_or_default(),
        });
        let placed = placement(world, &snapshot, view, entity.scale());
        vertices.extend(mob_vertices(model, &placed, rect));
        carried(
            model.root(),
            entity,
            (&placement_pose(&placed), placed.light),
            held,
        );
        // The eyes: the same model, as posed, in the glowing texture.
        if let Some(eyes) = textures.rect(eyes) {
            glow.extend(mob_vertices_glow(model, &placed, eyes));
        }
    }
    models.glow.extend(glow);
    vertices
}

/// What the golem carries, each where its layer puts it: the stacks in its
/// hands and on its head, and the block on its antenna.
fn carried(
    root: &Part,
    entity: &voxel_world::ClientEntity,
    (base, light): (&voxel_render::joml::Pose, i32),
    held: &mut Vec<HeldItem>,
) {
    fn child<'a>(of: &'a Part, name: &str) -> Option<&'a Part> {
        of.children().iter().find(|part| part.name() == name)
    }
    let Some(body) = child(root, "body") else {
        return;
    };
    let (Some(head), Some(right_arm), Some(left_arm)) = (
        child(body, "head"),
        child(body, "right_arm"),
        child(body, "left_arm"),
    ) else {
        return;
    };
    let idle = entity.copper_golem_action() == voxel_world::copper_golem::GolemAction::Idle;
    for (side, arm, place) in [
        (Arm::Right, right_arm, Place::Right),
        (Arm::Left, left_arm, Place::Left),
    ] {
        if let Some(stack) = entity.item_held_by_arm(side == Arm::Left) {
            let pose = copper_golem_hand_pose(base, (root, body, arm), side, idle);
            held.push(HeldItem::stack(stack, pose, place, light));
        }
    }
    if let Some(stack) = entity.equipment(EquipmentSlot::Head) {
        let pose = copper_golem_head_pose(base, (root, body, head));
        held.push(HeldItem::stack(stack, pose, Place::Head, light));
    }
    if let Some(state) = entity
        .equipment(EquipmentSlot::Saddle)
        .and_then(block_item_state)
    {
        held.push(HeldItem::block(
            state,
            copper_golem_antenna_pose(base, (root, body, head)),
            light,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use voxel_game::item_components::{Patch, ids};
    use voxel_world::ClientItemStackState;

    fn with_state(item: &str, properties: &[(&str, &str)]) -> ClientItemStackState {
        let mut value = Vec::new();
        voxel_core::wire::write_varint(properties.len() as i32, &mut value);
        for (name, text) in properties {
            for part in [name, text] {
                voxel_core::wire::write_varint(part.len() as i32, &mut value);
                value.extend_from_slice(part.as_bytes());
            }
        }
        let patch = Patch {
            added: vec![(ids::BLOCK_STATE, value)],
            removed: Vec::new(),
        };
        let id = voxel_data::item::registry::index(item).expect("item") as i32;
        ClientItemStackState::new(1, id).with_patch_bytes(patch.trusted())
    }

    #[test]
    fn the_antenna_shows_the_block_item_with_its_state_properties() {
        let block = voxel_data::block::registry::block("minecraft:sea_pickle").expect("block");
        let first = block.first_state_id();
        let state_of = |stack| block_item_state(&stack).map(|state| state - first);
        let default = block.default_state_id() - first;
        let plain = ClientItemStackState::new(
            1,
            voxel_data::item::registry::index("minecraft:sea_pickle").expect("item") as i32,
        );
        assert_eq!(state_of(plain), Some(default));
        let three = state_of(with_state("minecraft:sea_pickle", &[("pickles", "3")]));
        assert!(three.is_some() && three != Some(default));
        // A value the property cannot hold, and a property the block lacks, change nothing.
        assert_eq!(
            state_of(with_state(
                "minecraft:sea_pickle",
                &[("pickles", "9"), ("nothing", "1")]
            )),
            Some(default)
        );
        // An item that places no block shows no block.
        let stick = voxel_data::item::registry::index("minecraft:stick").expect("item") as i32;
        assert_eq!(block_item_state(&ClientItemStackState::new(1, stick)), None);
    }
}
