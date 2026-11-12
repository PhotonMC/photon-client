//! The cushions (`CushionRenderer`): the model in the cushion's dye colour, turned to the side its
//! facing snaps to, flipped over and lowered onto the block. Only the newest version has them,
//! so the colour textures are optional.

use photon_core::direction::Direction;
use photon_net::connection::PlaySession;
use photon_render::entity::textures::EntityTextures;
use photon_render::joml::Pose;
use photon_render::objects::boat::boat_vertices;
use photon_render::objects::cushion::cushion_pose;
use photon_render::scene::Vertex;

use crate::mobs::{MobModels, placement};
use crate::overlays::FrameView;

/// The model layer (`ModelLayers.CUSHION`).
const LAYER: &str = concat!("minecraft:", "cushion#main");

/// The textures (under `textures/`) of a cushion of each dye colour, in the order of
/// `DyeColor.getId`.
const TEXTURES: [&str; 16] = [
    "entity/cushion/white_cushion",
    "entity/cushion/orange_cushion",
    "entity/cushion/magenta_cushion",
    "entity/cushion/light_blue_cushion",
    "entity/cushion/yellow_cushion",
    "entity/cushion/lime_cushion",
    "entity/cushion/pink_cushion",
    "entity/cushion/gray_cushion",
    "entity/cushion/light_gray_cushion",
    "entity/cushion/cyan_cushion",
    "entity/cushion/purple_cushion",
    "entity/cushion/blue_cushion",
    "entity/cushion/brown_cushion",
    "entity/cushion/green_cushion",
    "entity/cushion/red_cushion",
    "entity/cushion/black_cushion",
];

/// The paths (under `textures/`) of the textures the cushions wear: none in a version without
/// the cushion's model layer.
pub fn textures() -> impl Iterator<Item = &'static str> {
    let has_layer = photon_data::entity_model::layer(LAYER).is_some();
    TEXTURES.into_iter().filter(move |_| has_layer)
}

/// The texture of the dye colour with this id; an id past the sixteen is none (the reference
/// would fail on it).
fn texture_of(colour: i32) -> Option<&'static str> {
    usize::try_from(colour)
        .ok()
        .and_then(|index| TEXTURES.get(index))
        .copied()
}

/// `CushionRenderer` for the cushions the server added.
pub fn remote_cushions(
    session: &PlaySession,
    models: &mut MobModels,
    textures: &EntityTextures,
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    let world = session.world();
    for entity in world
        .entities()
        .drawn_of_kind(|entity| entity.type_id == photon_data::entity_type::ids::CUSHION)
    {
        let partial = view.partial_of(world, entity.id()) as f32;
        let (Some(snapshot), Some(texture)) = (
            world.entity_snapshot_of(entity, partial),
            texture_of(entity.cushion_colour()),
        ) else {
            continue;
        };
        let (Some(rect), Some(model)) = (textures.rect(texture), models.model(LAYER)) else {
            continue;
        };
        let placed = placement(world, &snapshot, view, entity.scale());
        let mut base = Pose::default();
        base.translate(placed.feet[0], placed.feet[1], placed.feet[2]);
        let direction = Direction::from_y_rot(f64::from(entity.y_rot_at(partial)));
        let pose = cushion_pose(&base, direction);
        // A glowing cushion's body joins the outline.
        vertices.extend(crate::mobs::with_outline(
            entity,
            boat_vertices(model, &pose, placed.light, rect),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::{TEXTURES, texture_of, textures};

    #[test]
    fn a_dye_colour_has_its_texture() {
        assert_eq!(texture_of(0), Some("entity/cushion/white_cushion"));
        assert_eq!(texture_of(8), Some("entity/cushion/light_gray_cushion"));
        assert_eq!(texture_of(15), Some("entity/cushion/black_cushion"));
        assert_eq!(texture_of(16), None);
        assert_eq!(texture_of(-1), None);
    }

    #[test]
    fn every_texture_has_a_layer_in_the_newest_version() {
        // The lists drop a texture whose layer the played version lacks; the newest has them all.
        assert!(photon_data::entity_model::layer(super::LAYER).is_some());
        assert_eq!(textures().count(), TEXTURES.len());
        assert!(crate::objects::boats::textures().any(|name| name == "entity/chest_boat/poplar"));
    }
}
