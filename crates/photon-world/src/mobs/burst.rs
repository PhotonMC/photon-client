//! The signs an animal shows around itself when the server tells it so: hearts for love
//! (`Animal.handleEntityEvent` 18) and for a taming that worked (7), smoke for one that did not
//! (6, `TamableAnimal` and `AbstractHorse.spawnTamingParticles`).

/// Seven particles at random places in the entity's box, each moving by a small gaussian push.
pub(crate) const BURST_COUNT: i32 = 7;
/// The entity event of love (`Animal`).
pub(crate) const LOVE_EVENT: i8 = 18;
/// The entity events of taming: it failed, it worked.
pub(crate) const TAMING_FAILED_EVENT: i8 = 6;
pub(crate) const TAMING_WORKED_EVENT: i8 = 7;

/// A burst the particle engine is to draw: where the entity stands, how big its box is and which
/// particle it is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EntityBurst {
    pub at: (f64, f64, f64),
    pub size: (f32, f32),
    pub particle: &'static str,
}

/// Whether the entity type is a `TamableAnimal` or an `AbstractHorse`, which answer 6 and 7.
fn can_be_tamed(name: &str) -> bool {
    matches!(
        name,
        "minecraft:wolf"
            | "minecraft:cat"
            | "minecraft:parrot"
            | "minecraft:horse"
            | "minecraft:donkey"
            | "minecraft:mule"
            | "minecraft:skeleton_horse"
            | "minecraft:zombie_horse"
            | "minecraft:camel"
            | "minecraft:camel_husk"
            | "minecraft:llama"
            | "minecraft:trader_llama"
            | "minecraft:nautilus"
            | "minecraft:zombie_nautilus"
    )
}

/// The particle an event draws around an entity of this type, if it answers the event.
pub(crate) fn particle_of(name: &str, animal: bool, event: i8) -> Option<&'static str> {
    match event {
        LOVE_EVENT if animal => Some("heart"),
        TAMING_WORKED_EVENT if can_be_tamed(name) => Some("heart"),
        TAMING_FAILED_EVENT if can_be_tamed(name) => Some("smoke"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn love_shows_hearts_on_animals_only() {
        assert_eq!(
            particle_of("minecraft:pig", true, LOVE_EVENT),
            Some("heart")
        );
        assert_eq!(particle_of("minecraft:zombie", false, LOVE_EVENT), None);
    }

    #[test]
    fn taming_shows_hearts_or_smoke_on_tamable_types() {
        assert_eq!(particle_of("minecraft:wolf", true, 7), Some("heart"));
        assert_eq!(particle_of("minecraft:horse", true, 6), Some("smoke"));
        assert_eq!(particle_of("minecraft:pig", true, 7), None);
    }
}
