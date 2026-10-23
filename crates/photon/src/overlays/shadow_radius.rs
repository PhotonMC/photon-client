//! `EntityRenderer.getShadowRadius` and `getShadowStrength`: the shadow an entity's renderer casts.

use photon_world::ClientEntity;

/// The share of the renderer's radius that the age takes: `LivingEntity.getAgeScale`, which a few
/// species declare their own for.
fn age_scale(entity: &ClientEntity, name: &str) -> f32 {
    if !entity.is_baby() {
        return 1.0;
    }
    match name {
        "minecraft:camel" | "minecraft:armadillo" => 0.6,
        "minecraft:goat" => 0.55,
        "minecraft:dolphin" => 0.65,
        "minecraft:turtle" => 0.3,
        "minecraft:happy_ghast" => 0.2375,
        _ => 0.5,
    }
}

/// The radius and strength of the shadow of a type the renderer table lists, `None` for one that
/// casts none or is shaped by its own data (a display).
pub(super) fn of(entity: &ClientEntity) -> Option<(f32, f32)> {
    let name = photon_data::entity_type::name(entity.type_id)?;
    if entity.is_player() {
        return Some((0.5 * entity.scale(), 1.0));
    }
    // `AbstractCubeMobRenderer` and `PufferfishRenderer` replace the whole radius.
    if entity.is_cube_mob() {
        return Some((entity.cube_size() as f32 * 0.25, 1.0));
    }
    if name == "minecraft:pufferfish" {
        return Some((0.1 + 0.1 * entity.puff_state() as f32, 1.0));
    }
    let (radius, strength) = photon_data::entity_shadow::of(name)?;
    // The dragon's renderer is a plain `EntityRenderer`, so it has neither multiplier.
    if !entity.is_living_entity() || name == "minecraft:ender_dragon" {
        return Some((radius, strength));
    }
    // `LivingEntityRenderer` scales by the scale attribute; `MobRenderer` also by the age.
    let mut radius = radius * entity.scale();
    if entity.is_mob() {
        radius *= age_scale(entity, name);
        // Villagers, turtles and striders shrink a baby's shadow once more.
        if entity.is_baby() {
            radius *= match name {
                "minecraft:villager" | "minecraft:strider" => 0.5,
                "minecraft:turtle" => 0.83,
                _ => 1.0,
            };
        }
    }
    Some((radius, strength))
}
