//! `Parrot.aiStep`'s draw from the level's random source: now and then a parrot looks for a mob to
//! imitate. The client plays no sound for it, but the draws are taken from the shared source, and
//! so shift everything drawn after them.

use photon_core::ids::EntityId;
use photon_core::random::RandomSource;

use super::engine::ParticleEngine;
use super::level_events::LevelEventContext;

/// The mobs a parrot imitates (`Parrot.MOB_SOUND_MAP`'s keys).
const IMITATED: [&str; 42] = [
    "minecraft:blaze",
    "minecraft:bogged",
    "minecraft:breeze",
    "minecraft:camel_husk",
    "minecraft:cave_spider",
    "minecraft:creaking",
    "minecraft:creeper",
    "minecraft:drowned",
    "minecraft:elder_guardian",
    "minecraft:ender_dragon",
    "minecraft:endermite",
    "minecraft:evoker",
    "minecraft:ghast",
    "minecraft:happy_ghast",
    "minecraft:guardian",
    "minecraft:hoglin",
    "minecraft:husk",
    "minecraft:illusioner",
    "minecraft:magma_cube",
    "minecraft:parched",
    "minecraft:phantom",
    "minecraft:piglin",
    "minecraft:piglin_brute",
    "minecraft:pillager",
    "minecraft:ravager",
    "minecraft:shulker",
    "minecraft:silverfish",
    "minecraft:skeleton",
    "minecraft:slime",
    "minecraft:spider",
    "minecraft:stray",
    "minecraft:vex",
    "minecraft:vindicator",
    "minecraft:warden",
    "minecraft:witch",
    "minecraft:wither",
    "minecraft:wither_skeleton",
    "minecraft:zoglin",
    "minecraft:zombie",
    "minecraft:zombie_horse",
    "minecraft:zombie_nautilus",
    "minecraft:zombie_villager",
];

impl ParticleEngine {
    /// The start of `Parrot.aiStep`: one in four hundred ticks the parrot looks for a mob.
    pub(super) fn parrot_imitation<F: FnMut() -> RandomSource>(
        &mut self,
        id: EntityId,
        context: &mut LevelEventContext<'_, F>,
    ) {
        if context.level_random.next_int_bound(400) != Ok(0) {
            return;
        }
        let Some(parrot) = context.world.entities().get(&id) else {
            return;
        };
        // `imitateNearbyMobs`: a living, loud parrot, half the time.
        if parrot.is_dead_or_dying() || parrot.is_silent() {
            return;
        }
        if context.level_random.next_int_bound(2) != Ok(0) {
            return;
        }
        let reach = parrot.bounding_box().inflate_all(20.0);
        let mobs: Vec<bool> = context
            .world
            .entities()
            .values()
            .filter(|entity| {
                photon_data::entity_type::name(entity.type_id)
                    .is_some_and(|name| IMITATED.contains(&name))
                    && entity.bounding_box().intersects(reach)
            })
            .map(|entity| entity.is_silent())
            .collect();
        if mobs.is_empty() {
            return;
        }
        let Ok(pick) = context.level_random.next_int_bound(mobs.len() as i32) else {
            return;
        };
        // `getPitch`, for a sound the client does not play.
        if !mobs[pick as usize] {
            context.level_random.next_float();
            context.level_random.next_float();
        }
    }
}
