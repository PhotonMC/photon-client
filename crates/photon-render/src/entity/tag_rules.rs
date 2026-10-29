//! The name tag over an entity that is not a player (`EntityRenderer`, `LivingEntityRenderer` and
//! `MobRenderer` `.shouldShowName`, `ArmorStandRenderer.shouldShowName`): which of them show
//! theirs, and where it hangs. Teams and the camera entity are not carried.

use photon_core::math::Vec3;
use photon_world::EntityView;

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
    /// `ItemFrameRenderer`: the crosshair is on it and the HUD is up (the item's custom name is
    /// what the tag says).
    ItemFrame,
}

/// `Entity.getAttachments().getNullable(NAME_TAG, 0, yRot)` before the entity's scale: the
/// table's point of a grown or a baby entity of the type. No type has a sideways part, so the
/// entity's turn does not move it.
fn attachment(entity: EntityView<'_>) -> Option<Vec3> {
    if entity.is_living_entity() && entity.pose() == photon_world::ClientEntityPose::Sleeping {
        // `LivingEntity.SLEEPING_DIMENSIONS`: 0.2 high whatever the type, and not scaled.
        return Some(Vec3::new(0.0, 0.2_f32 as f64, 0.0));
    }
    if let Some(height) = entity.pose_height() {
        // A pose's own size has no attachments but the default: the tag sits at its top.
        return Some(Vec3::new(
            0.0,
            f64::from(height) * f64::from(entity.scale()),
            0.0,
        ));
    }
    let pick = photon_data::entity_type::pick(entity.type_id)?;
    let (x, y, z) = if entity.is_baby() {
        pick.baby_name_tag.or(pick.name_tag)
    } else {
        pick.name_tag
    }?;
    let scale = f64::from(entity.scale()) * stand_size(entity) * f64::from(entity.pose_scale());
    Some(Vec3::new(x * scale, y * scale, z * scale))
}

/// `ArmorStand.getDimensions`: a marker has no size at all (its tag sits at its feet) and a small
/// stand is the stand's dimensions halved.
fn stand_size(entity: EntityView<'_>) -> f64 {
    if rules_of(entity) != Some(NameRules::ArmorStand) {
        return 1.0;
    }
    let stand = entity.armor_stand();
    if stand.is_marker() {
        0.0
    } else if stand.is_small() {
        0.5
    } else {
        1.0
    }
}

/// Which renderer's rules name an entity, or none where it makes no tag (yet): a player names
/// itself by its profile, an item frame by the item in it, an item, a falling block and a
/// villager by what they are rather than their type (unless they carry a custom name), and a
/// lightning bolt or an ominous item spawner submits no name at all. Read from the renderer
/// classes: the ender dragon's extends `EntityRenderer` itself, so it follows the plain rules
/// though it is a mob.
pub fn rules_of(entity: EntityView<'_>) -> Option<NameRules> {
    let name = photon_data::entity_type::name(entity.type_id)?;
    match name {
        "minecraft:player" | "minecraft:lightning_bolt" | "minecraft:ominous_item_spawner" => None,
        // `AvatarRenderer.shouldShowName`: `LivingEntityRenderer`'s, then a visible name or a
        // custom one under the crosshair, as a mob's.
        "minecraft:mannequin" => Some(NameRules::Mob),
        "minecraft:item" | "minecraft:falling_block" | "minecraft:villager"
            if !entity.has_custom_name() =>
        {
            None
        }
        "minecraft:armor_stand" => Some(NameRules::ArmorStand),
        "minecraft:item_frame" | "minecraft:glow_item_frame" => Some(NameRules::ItemFrame),
        _ => class_rules(entity, name),
    }
}

/// The rules of the renderer class an entity's type falls under.
fn class_rules(entity: EntityView<'_>, name: &str) -> Option<NameRules> {
    if name == "minecraft:ender_dragon" {
        Some(NameRules::Entity)
    } else if entity.is_mob() {
        Some(NameRules::Mob)
    } else if entity.is_living_entity() {
        None
    } else {
        Some(NameRules::Entity)
    }
}

/// `EntityRenderer.extractNameTags` for an entity that is not a player, with the name it would
/// show; `picked` is whether the crosshair is on it (`crosshairPickEntity`).
pub fn entity_name_tag(
    entity: EntityView<'_>,
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
        NameRules::ItemFrame => picked && !viewer.hud_hidden,
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
        score: None,
    })
}
