//! The armor stands the frame draws (`ArmorStandRenderer`): the stand itself,
//! turned to the stand's rotations, with the armor and the items it holds.

use photon_net::connection::PlaySession;
use photon_render::entity::mob_model::{ArmorStandState, mob_vertices_coloured, placement_pose};
use photon_render::entity::textures::EntityTextures;
use photon_render::scene::Vertex;
use photon_world::EquipmentSlot;

use crate::equipment::armor::{Armor, Worn};
use crate::equipment::held_items::{HEAD_TRANSFORMS, Hands, HeldItem};
use crate::mobs::{MobModels, placement};
use crate::overlays::FrameView;

/// `ArmorStandRenderer.DEFAULT_SKIN_LOCATION`, under `textures/`.
pub const TEXTURE: &str = "entity/armorstand/armorstand";

/// The armor stands of the world, each posed from the data the server sent.
/// A stand's wiggle when it is hit (`wiggle`) is not drawn, and neither is the elytra. An invisible
/// stand keeps its armor and items and leaves out only the body.
pub fn remote_armor_stands(
    session: &PlaySession,
    (models, armor): (&mut MobModels, &mut Armor),
    textures: &EntityTextures,
    (view, held): (FrameView, &mut Vec<HeldItem>),
    vertices: &mut Vec<Vertex>,
) {
    let Some(stand_type) = photon_data::entity_type::id("minecraft:armor_stand") else {
        return;
    };
    let world = session.world();
    let partial = view.partial as f32;
    for entity in world.drawn_mobs() {
        let Some(snapshot) = world
            .entity_snapshot(entity.id(), partial)
            .filter(|_| entity.type_id == stand_type)
        else {
            continue;
        };
        let data = entity.armor_stand();
        let state = ArmorStandState {
            poses: std::array::from_fn(|part| data.pose(part)),
            show_arms: data.shows_arms(),
            show_base_plate: data.shows_base_plate(),
            y_rot: snapshot.yaw,
        };
        // `setupRotations` turns the stand by its own rotation, not a body's.
        let mut placed = placement(world, &snapshot, view, entity.scale());
        placed.body_rot = snapshot.yaw;
        let prefix = if data.is_small() {
            "minecraft:armor_stand_small"
        } else {
            "minecraft:armor_stand"
        };
        let worn = Worn {
            head: entity.equipment(EquipmentSlot::Head),
            chest: entity.equipment(EquipmentSlot::Chest),
            legs: entity.equipment(EquipmentSlot::Legs),
            feet: entity.equipment(EquipmentSlot::Feet),
        };
        if let (Some(rect), Some(model)) = (
            textures.rect(TEXTURE),
            models.model(&format!("{prefix}#main")),
        ) {
            model.setup_armor_stand(&state, true);
            let hands = Hands {
                base: &placement_pose(&placed),
                root: model.root(),
                hand_shift: 0.0,
                baby: false,
                light: placed.light,
                right: entity.item_held_by_arm(false),
                left: entity.item_held_by_arm(true),
                head: worn.head_item(armor),
                head_transforms: HEAD_TRANSFORMS,
                eye: None,
                motions: Default::default(),
            };
            hands.collect(held);
            vertices.extend(crate::mobs::visible_body(entity, model, (&placed, rect)));
            vertices.extend(armor.skull_vertices(
                hands.skull(),
                0.0,
                textures,
                (placed.light, false),
            ));
        }
        vertices.extend(armor.vertices(
            worn,
            (prefix, false),
            textures,
            &|model| model.setup_armor_stand(&state, false),
            &|model, rect, colour| mob_vertices_coloured(model, &placed, rect, colour),
        ));
    }
}
