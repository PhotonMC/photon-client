//! The name tag over an entity that is not a player (`EntityRenderer`, `LivingEntityRenderer` and
//! `MobRenderer` `.shouldShowName`, `ArmorStandRenderer.shouldShowName`): which of them show
//! theirs, and where it hangs. Teams and the camera entity are not carried.

use photon_core::math::Vec3;
use photon_world::ClientEntity;

use super::name_tag::{NameTag, Teams, Viewer, distance_sq, living_shows};
use crate::gui::TextRun;

/// `Attributes.NAME_TAG_DISTANCE`'s default, and what an entity without the attribute uses.
const NAME_TAG_DISTANCE: f64 = 64.0;

/// Which renderer's rules decide whether a name shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameRules {
    /// `EntityRenderer`: a visible name, or a custom name under the crosshair.
    Entity,
    /// `MobRenderer`: those, and `LivingEntityRenderer`'s: not hidden, not riden, not itself.
    Mob,
    /// `ArmorStandRenderer`: a visible custom name.
    ArmorStand,
}

/// `Entity.getAttachments().getNullable(NAME_TAG, 0, yRot)` before the entity's scale: the
/// table's point of a grown or a baby entity of the type. No type has a sideways part, so the
/// entity's turn does not move it.
fn attachment(entity: &ClientEntity) -> Option<Vec3> {
    let pick = photon_data::entity_type::pick(entity.type_id)?;
    let (x, y, z) = if entity.is_baby() {
        pick.baby_name_tag.or(pick.name_tag)
    } else {
        pick.name_tag
    }?;
    let scale = f64::from(entity.scale());
    Some(Vec3::new(x * scale, y * scale, z * scale))
}

/// Which renderer's rules name an entity, or none where it makes no tag (yet): a player names
/// itself by its profile, an item frame by the item in it, an item, a falling block and a
/// villager by what they are rather than their type (unless they carry a custom name), and a
/// lightning bolt or an ominous item spawner submits no name at all. Read from the renderer
/// classes: the ender dragon's extends `EntityRenderer` itself, so it follows the plain rules
/// though it is a mob.
pub fn rules_of(entity: &ClientEntity) -> Option<NameRules> {
    let name = photon_data::entity_type::name(entity.type_id)?;
    match name {
        "minecraft:player"
        | "minecraft:mannequin"
        | "minecraft:item_frame"
        | "minecraft:glow_item_frame"
        | "minecraft:lightning_bolt"
        | "minecraft:ominous_item_spawner" => None,
        "minecraft:item" | "minecraft:falling_block" | "minecraft:villager"
            if !entity.has_custom_name() =>
        {
            None
        }
        "minecraft:armor_stand" => Some(NameRules::ArmorStand),
        "minecraft:ender_dragon" => Some(NameRules::Entity),
        _ if entity.is_mob() => Some(NameRules::Mob),
        _ if entity.is_living_entity() => None,
        _ => Some(NameRules::Entity),
    }
}

/// `Entity.getEyeHeight()` of a standing entity: the table's eye height of its type (a baby's
/// for a baby) in its scale.
pub fn eye_height(entity: &ClientEntity) -> Option<f64> {
    let pick = photon_data::entity_type::pick(entity.type_id)?;
    let (_, _, eye) = if entity.is_baby() {
        pick.baby.unwrap_or(pick.size)
    } else {
        pick.size
    };
    Some(f64::from(eye) * f64::from(entity.scale()))
}

/// `EntityRenderer.extractNameTags` for an entity that is not a player, with the name it would
/// show; `picked` is whether the crosshair is on it (`crosshairPickEntity`).
pub fn entity_name_tag(
    entity: &ClientEntity,
    runs: Vec<TextRun<'static>>,
    (rules, picked): (NameRules, bool),
    (viewer, teams): (Viewer, Teams),
) -> Option<NameTag> {
    let distance_sq = distance_sq(entity, viewer);
    let reach = entity
        .attribute_value("minecraft:name_tag_distance")
        .unwrap_or(NAME_TAG_DISTANCE);
    if distance_sq >= reach * reach {
        return None;
    }
    let named = entity.should_show_name() || entity.has_custom_name() && picked;
    let shows = match rules {
        NameRules::Entity => named,
        NameRules::Mob => living_shows(entity, viewer, teams, distance_sq) && named,
        NameRules::ArmorStand => entity.is_custom_name_visible(),
    };
    if !shows {
        return None;
    }
    Some(NameTag {
        runs,
        attachment: attachment(entity)?,
        offset: 0,
        see_through: !entity.is_shift_key_down(),
        render_offset: Vec3::ZERO,
    })
}
