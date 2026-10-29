use photon_core::ids::EntityTypeId;

/// `(float) (Math.PI / 180.0)`.
pub(super) const DEG_TO_RAD: f32 = (std::f64::consts::PI / 180.0) as f32;

/// Which mount it is, for the parts of its tick that differ.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MountKind {
    /// A saddled horse, donkey, mule or undead horse.
    Horse,
    /// A saddled pig steered with a carrot on a stick.
    Pig,
    /// A saddled strider steered with a warped fungus on a stick, which walks on lava.
    Strider,
    /// A saddled camel (or camel husk), a horse that dashes and sits.
    Camel,
}

/// What a ridden mount keeps from one tick to the next beyond its place and motion.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MountState {
    pub fall_distance: f64,
    pub horizontal_collision: bool,
    pub vertical_collision: bool,
    pub vertical_collision_below: bool,
    /// `Entity.stuckSpeedMultiplier`, spent by the next move.
    pub stuck_speed_multiplier: (f64, f64, f64),
    pub no_jump_delay: i32,
    /// `AbstractHorse.playerJumpPendingScale`: how hard the rider asked to jump, until the horse
    /// is on the ground to do it.
    pub jump_pending_scale: f32,
    /// How deep the lava it stands in reached at the last look (`getFluidHeight(LAVA)`), which a
    /// strider's warmth reads the next tick.
    pub lava_height: f64,
    /// Whether it was in water or lava at the last look (`Entity.isInLiquid`).
    pub in_liquid: bool,
}

/// The values are finite, so the comparison is a total one.
impl Eq for MountState {}

impl Default for MountState {
    fn default() -> Self {
        Self {
            fall_distance: 0.0,
            horizontal_collision: false,
            vertical_collision: false,
            vertical_collision_below: false,
            stuck_speed_multiplier: (0.0, 0.0, 0.0),
            no_jump_delay: 0,
            jump_pending_scale: 0.0,
            lava_height: 0.0,
            in_liquid: false,
        }
    }
}

/// The player who steers the mount, as the mount's tick reads it: the keys' effect on the player's
/// own movement input (`xxa`, `zza`) and where the player looks.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Controller {
    pub xxa: f32,
    pub zza: f32,
    pub y_rot: f32,
    pub x_rot: f32,
    /// Whether a hand of the player holds a carrot on a stick, which steers a saddled pig.
    pub holds_carrot_on_a_stick: bool,
    /// Likewise for a warped fungus on a stick, which steers a saddled strider.
    pub holds_warped_fungus_on_a_stick: bool,
    /// `Entity.isSprinting`, which a camel's speed reads.
    pub sprinting: bool,
}

/// The values are finite, so the comparison is a total one.
impl Eq for Controller {}

/// The attributes the mount's movement reads.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Attributes {
    pub(crate) speed: f32,
    /// The movement speed attribute itself, which a camel's dash reads.
    pub(crate) base_speed: f64,
    pub(crate) gravity: f64,
    pub(crate) step_height: f32,
    pub(crate) jump_strength: f32,
    pub(crate) friction_modifier: f32,
    pub(crate) air_drag_modifier: f32,
    pub(crate) water_movement_efficiency: f32,
}

/// Whether an entity type is one of the horses a saddle makes rideable (`AbstractHorse` that is
/// not a llama).
pub(crate) fn is_horse_type(type_id: EntityTypeId) -> bool {
    [
        "minecraft:horse",
        "minecraft:donkey",
        "minecraft:mule",
        "minecraft:skeleton_horse",
        "minecraft:zombie_horse",
        "minecraft:camel",
        "minecraft:camel_husk",
    ]
    .iter()
    .any(|name| photon_data::entity_type::id(name) == Some(type_id))
}

/// The attributes a ridden mount moves by, read from what the server sent.
pub(super) fn mount_attributes(
    entity: crate::entity::EntityView<'_>,
    kind: MountKind,
) -> Attributes {
    let attribute = |name: &str, default: f64| entity.attribute_value(name).unwrap_or(default);
    Attributes {
        // `getRiddenSpeed`: a pig's is its speed times 0.225 (and its boost, which is one here).
        base_speed: attribute("minecraft:movement_speed", 0.7),
        speed: match kind {
            MountKind::Horse => attribute("minecraft:movement_speed", 0.7) as f32,
            // `Camel.getRiddenSpeed`: the attribute, and the sprint's bonus the body adds.
            MountKind::Camel => attribute("minecraft:movement_speed", 0.09) as f32,
            MountKind::Pig => (attribute("minecraft:movement_speed", 0.25) * 0.225 * 1.0) as f32,
            // `Strider.getRiddenSpeed`: slower while it is cold (suffocating).
            MountKind::Strider => {
                let suffocating = entity.is_suffocating();
                (attribute("minecraft:movement_speed", 0.175)
                    * f64::from(if suffocating { 0.35_f32 } else { 0.55_f32 })
                    * 1.0) as f32
            }
        },
        gravity: attribute("minecraft:gravity", 0.08),
        step_height: attribute("minecraft:step_height", 0.6) as f32,
        jump_strength: attribute("minecraft:jump_strength", 0.42) as f32,
        friction_modifier: attribute("minecraft:friction_modifier", 1.0) as f32,
        air_drag_modifier: attribute("minecraft:air_drag_modifier", 1.0) as f32,
        water_movement_efficiency: attribute("minecraft:water_movement_efficiency", 0.0) as f32,
    }
}

#[cfg(test)]
mod tests {

    use crate::{ClientWorld, DimensionBounds, EntityInput};

    use photon_core::binary::Uuid;
    use photon_core::ids::EntityId;

    #[test]
    fn a_horse_has_the_attributes_its_riding_reads() {
        let mut world = ClientWorld::new(DimensionBounds::new(-64, 384));
        let id = photon_data::entity_type::id("minecraft:horse").expect("horse");
        assert!(world.apply_entity(EntityInput {
            entity_id: EntityId(5),
            uuid: Uuid::new(0, 5),
            type_id: id,
            x: 0.0,
            y: 0.0,
            z: 0.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        let horse = world
            .entities()
            .all()
            .find(|entity| entity.id() == EntityId(5))
            .unwrap();
        // The values a saddled horse's riding reads, as the type starts with them.
        for (name, value) in [
            ("minecraft:gravity", 0.08),
            ("minecraft:step_height", 1.0),
            ("minecraft:jump_strength", 0.7),
            ("minecraft:friction_modifier", 1.0),
            ("minecraft:air_drag_modifier", 1.0),
            ("minecraft:water_movement_efficiency", 0.0),
        ] {
            assert_eq!(horse.attribute_value(name), Some(value), "{name}");
        }
        assert_eq!(
            horse
                .attribute_value("minecraft:movement_speed")
                .map(|speed| speed as f32),
            Some(0.225)
        );
    }
}
