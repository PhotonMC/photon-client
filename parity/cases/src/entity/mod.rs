//! The `entity` family: entity models, poses, renderers and their data,
//! mostly as rows that differ from a standing pose by a few values.

mod aquatic;
mod banner;
mod beacon;
mod bosses;
mod effects;
mod farm;
mod flyers;
mod golems;
mod guardian;
mod held;
mod humanoids;
mod items;
mod keyframed;
mod objects;
mod pets;
mod player;
pub(crate) mod pose;
mod projectiles;
mod shaking;
mod spear;
mod view_state;

use serde_json::Value;

use pose::{entity, table};

type Cases = fn() -> Vec<Value>;

fn tables() -> Vec<Value> {
    vec![
        table("data-serializers", "entity_data_serializers"),
        table("variant-fields", "entity_variant_fields"),
        table("variants", "entity_variants"),
        table("wolf-variants", "wolf_variants"),
        table("painting-variants", "painting_variants"),
        table("map-data", "map_data"),
    ]
}

/// The generators in the order the cases are listed.
const ORDER: &[Cases] = &[
    player::player_model,
    player::walk_animation,
    player::model_layer,
    tables,
    humanoids::zombie,
    humanoids::skeleton,
    humanoids::piglin,
    items::xp_orb,
    player::living_pose,
    held::held_item,
    || vec![entity("spear-table", "spear_table", vec![])],
    farm::sheep,
    farm::chicken,
    farm::sheep_colour,
    farm::sheep_eat,
    humanoids::spider,
    golems::creeper_render,
    items::equipment_layer,
    humanoids::villager,
    humanoids::illager_witch,
    golems::golem,
    farm::goat,
    items::animation,
    farm::rabbit,
    golems::insect,
    farm::equine,
    farm::camel,
    farm::llama,
    pets::wolf,
    pets::feline,
    pets::polar_bear,
    pets::hoglin,
    pets::fox,
    pets::panda,
    shaking::ravager,
    shaking::strider,
    shaking::shaking,
    aquatic::squid,
    bosses::cube_mob,
    bosses::shulker,
    aquatic::fish,
    aquatic::sea_animal,
    aquatic::axolotl,
    aquatic::frog,
    aquatic::nautilus,
    guardian::guardian,
    guardian::guardian_beam,
    flyers::airborne,
    flyers::flier,
    flyers::parrot,
    flyers::breeze,
    keyframed::warden,
    keyframed::creaking,
    keyframed::armadillo,
    keyframed::sniffer,
    keyframed::wither,
    bosses::ender_dragon,
    bosses::dragon_rays,
    bosses::end_crystal,
    bosses::crystal_beam,
    objects::minecart_pose,
    objects::item_frame_pose,
    objects::display_pose,
    items::glint,
    projectiles::text_display,
    projectiles::map_render,
    items::equippable_cases,
    player::camera_hurt,
    player::camera_spin,
    player::item_activation_pose,
    objects::chest_cases,
    objects::shulker_box,
    objects::bell,
    objects::enchant_table,
    objects::lectern,
    objects::decorated_pot,
    beacon::beacon_beam,
    beacon::gateway_beam,
    beacon::brushable_pose,
    beacon::shelf_pose,
    beacon::conduit_pose,
    beacon::campfire_pose,
    beacon::vault_pose,
    beacon::copper_statue_pose,
    beacon::end_portal_cube,
    view_state::mob_effect_blend,
    view_state::view_scales,
    objects::skull_block,
    objects::elder_guardian_particle,
    banner::banner_cases,
    objects::display_light,
    projectiles::arrow,
    projectiles::thrown_item,
    projectiles::charge,
    projectiles::small_projectile,
    effects::fishing_hook,
    projectiles::leash,
    player::elytra,
    player::avatar_pose,
    effects::primed_tnt,
    effects::lightning_bolt,
    effects::painting_geometry,
    effects::screen_effect,
    flyers::happy_ghast,
    flyers::flier_item,
    objects::boat,
    golems::copper_golem,
    golems::copper_golem_layer,
    || vec![table("attributes", "entity_attributes")],
    golems::villager_held,
    items::armor_stand,
    || vec![table("types", "entity_types")],
    items::quadruped,
    items::item_pose,
];

pub fn cases() -> Vec<Value> {
    ORDER.iter().flat_map(|generate| generate()).collect()
}
