//! `EntityRenderer.getShadowRadius` and `getShadowStrength`: the shadow an entity's renderer casts.

use photon_world::EntityView;

/// The share of the renderer's radius that the age takes: `LivingEntity.getAgeScale`, which a few
/// species declare their own for.
fn age_scale(entity: EntityView<'_>, name: &str) -> f32 {
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

/// What a type's shadow depends on, which does not change from one entity of it to the next:
/// the rest is the entity's own scale and age.
#[derive(Clone, Copy)]
pub(super) enum TypeShadow {
    /// Casts none, or is shaped by its own data (a display).
    None,
    Player,
    CubeMob,
    Pufferfish,
    /// The renderer's radius and strength, with how the scale and the age change the radius.
    Table {
        radius: f32,
        strength: f32,
        name: &'static str,
        scaled: bool,
        mob: bool,
    },
}

impl TypeShadow {
    /// The facts of the type of `entity`.
    pub(super) fn of_type(entity: EntityView<'_>) -> Self {
        let Some(name) = photon_data::entity_type::name(entity.type_id) else {
            return Self::None;
        };
        if entity.is_player() {
            return Self::Player;
        }
        if entity.is_cube_mob() {
            return Self::CubeMob;
        }
        if name == "minecraft:pufferfish" {
            return Self::Pufferfish;
        }
        let Some((radius, strength)) = photon_data::entity_shadow::of(name) else {
            return Self::None;
        };
        Self::Table {
            radius,
            strength,
            name,
            // The dragon's renderer is a plain `EntityRenderer`, so it has neither multiplier.
            scaled: entity.is_living_entity() && name != "minecraft:ender_dragon",
            mob: entity.is_mob(),
        }
    }
}

/// The radius and strength of the shadow of an entity of a type the renderer table lists, `None`
/// for one that casts none or is shaped by its own data (a display).
pub(super) fn of(entity: EntityView<'_>, facts: TypeShadow) -> Option<(f32, f32)> {
    let (radius, strength, name, mob) = match facts {
        TypeShadow::None => return None,
        TypeShadow::Player => return Some((0.5 * entity.scale(), 1.0)),
        // `AbstractCubeMobRenderer` and `PufferfishRenderer` replace the whole radius.
        TypeShadow::CubeMob => return Some((entity.cube_size() as f32 * 0.25, 1.0)),
        TypeShadow::Pufferfish => return Some((0.1 + 0.1 * entity.puff_state() as f32, 1.0)),
        TypeShadow::Table {
            radius,
            strength,
            scaled: false,
            ..
        } => return Some((radius, strength)),
        TypeShadow::Table {
            radius,
            strength,
            name,
            mob,
            ..
        } => (radius, strength, name, mob),
    };
    // `LivingEntityRenderer` scales by the scale attribute; `MobRenderer` also by the age.
    let mut radius = radius * entity.scale();
    if mob {
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
