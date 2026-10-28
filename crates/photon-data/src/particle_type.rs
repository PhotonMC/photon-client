//! The particle types of the reference's registry, in the order of their ids.
//!
//! The table is written from `ParticleTypes` and compared with the reference's own registry
//! on every parity run (`particle_types`).

use photon_core::version::{Version, VersionSpan};

/// A particle type of the registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParticleType {
    /// The name without the namespace (`smoke`).
    pub name: &'static str,
    /// Whether the type ignores the particle limiter.
    pub overrides_limiter: bool,
    /// Whether the type carries nothing but itself on the wire (`SimpleParticleType`).
    pub simple: bool,
    /// Whether the type carries a block state (`BlockParticleOption`).
    pub block_state: bool,
    /// The versions that have the type.
    pub versions: VersionSpan,
}

impl ParticleType {
    /// Whether the type exists in `version`.
    pub const fn exists_in(&self, version: Version) -> bool {
        self.versions.contains(version)
    }
}

const fn particle(
    name: &'static str,
    overrides_limiter: bool,
    simple: bool,
    block_state: bool,
) -> ParticleType {
    ParticleType {
        name,
        overrides_limiter,
        simple,
        block_state,
        versions: VersionSpan::ALL,
    }
}

/// Every particle type, in the order of the ids the wire carries.
pub const PARTICLE_TYPES: [ParticleType; 125] = [
    particle("angry_villager", false, true, false),
    particle("block", false, false, true),
    particle("block_marker", true, false, true),
    particle("bubble", false, true, false),
    particle("sulfur_bubbles", false, true, false),
    particle("noxious_gas", false, true, false),
    particle("noxious_gas_cloud", false, true, false),
    particle("geyser", true, false, false),
    particle("geyser_base", true, false, false),
    particle("geyser_poof", true, false, false),
    particle("geyser_plume", true, false, false),
    particle("cloud", false, true, false),
    particle("copper_fire_flame", false, true, false),
    particle("crit", false, true, false),
    particle("damage_indicator", true, true, false),
    particle("dragon_breath", false, false, false),
    particle("dripping_lava", false, true, false),
    particle("falling_lava", false, true, false),
    particle("landing_lava", false, true, false),
    particle("dripping_water", false, true, false),
    particle("falling_water", false, true, false),
    particle("dust", false, false, false),
    particle("dust_color_transition", false, false, false),
    particle("effect", false, false, false),
    particle("elder_guardian", true, true, false),
    particle("enchanted_hit", false, true, false),
    particle("enchant", false, true, false),
    particle("end_rod", false, true, false),
    particle("entity_effect", false, false, false),
    particle("explosion_emitter", true, true, false),
    particle("explosion", true, true, false),
    particle("gust", true, true, false),
    particle("small_gust", false, true, false),
    particle("gust_emitter_large", true, true, false),
    particle("gust_emitter_small", true, true, false),
    particle("sonic_boom", true, true, false),
    particle("falling_dust", false, false, true),
    particle("firework", false, true, false),
    particle("fishing", false, true, false),
    particle("flame", false, true, false),
    particle("infested", false, true, false),
    particle("cherry_leaves", false, true, false),
    particle("pale_oak_leaves", false, true, false),
    particle("tinted_leaves", false, false, false),
    particle("sculk_soul", false, true, false),
    particle("sculk_charge", true, false, false),
    particle("sculk_charge_pop", true, true, false),
    particle("soul_fire_flame", false, true, false),
    particle("soul", false, true, false),
    particle("flash", false, false, false),
    particle("happy_villager", false, true, false),
    particle("composter", false, true, false),
    particle("heart", false, true, false),
    particle("instant_effect", false, false, false),
    particle("item", false, false, false),
    particle("vibration", true, false, false),
    particle("trail", false, false, false),
    particle("pause_mob_growth", false, true, false),
    particle("reset_mob_growth", false, true, false),
    particle("item_slime", false, true, false),
    particle("item_cobweb", false, true, false),
    particle("item_snowball", false, true, false),
    particle("large_smoke", false, true, false),
    particle("lava", false, true, false),
    particle("mycelium", false, true, false),
    particle("note", false, true, false),
    particle("poof", true, true, false),
    particle("portal", false, true, false),
    particle("rain", false, true, false),
    particle("smoke", false, true, false),
    particle("white_smoke", false, true, false),
    particle("sneeze", false, true, false),
    particle("spit", true, true, false),
    particle("squid_ink", true, true, false),
    particle("sweep_attack", true, true, false),
    particle("totem_of_undying", false, true, false),
    particle("underwater", false, true, false),
    particle("splash", false, true, false),
    particle("witch", false, true, false),
    particle("bubble_pop", false, true, false),
    particle("current_down", false, true, false),
    particle("bubble_column_up", false, true, false),
    particle("nautilus", false, true, false),
    particle("dolphin", false, true, false),
    particle("campfire_cosy_smoke", true, true, false),
    particle("campfire_signal_smoke", true, true, false),
    particle("dripping_honey", false, true, false),
    particle("falling_honey", false, true, false),
    particle("landing_honey", false, true, false),
    particle("falling_nectar", false, true, false),
    particle("falling_spore_blossom", false, true, false),
    particle("ash", false, true, false),
    particle("crimson_spore", false, true, false),
    particle("warped_spore", false, true, false),
    particle("spore_blossom_air", false, true, false),
    particle("dripping_obsidian_tear", false, true, false),
    particle("falling_obsidian_tear", false, true, false),
    particle("landing_obsidian_tear", false, true, false),
    particle("reverse_portal", false, true, false),
    particle("white_ash", false, true, false),
    particle("small_flame", false, true, false),
    particle("snowflake", false, true, false),
    particle("dripping_dripstone_lava", false, true, false),
    particle("falling_dripstone_lava", false, true, false),
    particle("dripping_dripstone_water", false, true, false),
    particle("falling_dripstone_water", false, true, false),
    particle("glow_squid_ink", true, true, false),
    particle("glow", true, true, false),
    particle("wax_on", true, true, false),
    particle("wax_off", true, true, false),
    particle("electric_spark", true, true, false),
    particle("scrape", true, true, false),
    particle("shriek", false, false, false),
    particle("egg_crack", false, true, false),
    particle("dust_plume", false, true, false),
    particle("trial_spawner_detection", true, true, false),
    particle("trial_spawner_detection_ominous", true, true, false),
    particle("vault_connection", true, true, false),
    particle("dust_pillar", false, false, true),
    particle("ominous_spawning", true, true, false),
    particle("raid_omen", false, true, false),
    particle("trial_omen", false, true, false),
    particle("block_crumble", false, false, true),
    particle("firefly", false, true, false),
    particle("sulfur_cube_goo", false, true, false),
];

/// The name (`smoke`, without the namespace) of the type with this id.
pub fn name(id: i32) -> Option<&'static str> {
    usize::try_from(id)
        .ok()
        .and_then(|index| PARTICLE_TYPES.get(index))
        .map(|particle| particle.name)
}

/// The id of the type with this name (`smoke`, without the namespace).
pub fn id(name: &str) -> Option<i32> {
    PARTICLE_TYPES
        .iter()
        .position(|particle| particle.name == name)
        .map(|index| index as i32)
}

/// Whether the type ignores the particle limiter (`ParticleType.getOverrideLimiter`).
pub fn overrides_limiter(id: i32) -> bool {
    usize::try_from(id)
        .ok()
        .and_then(|index| PARTICLE_TYPES.get(index))
        .is_some_and(|particle| particle.overrides_limiter)
}

/// Whether the type is a `SimpleParticleType`: nothing follows its id on the wire.
pub fn is_simple(id: i32) -> bool {
    usize::try_from(id)
        .ok()
        .and_then(|index| PARTICLE_TYPES.get(index))
        .is_some_and(|particle| particle.simple)
}

/// Whether the type carries a block state after its id on the wire.
pub fn carries_block_state(id: i32) -> bool {
    usize::try_from(id)
        .ok()
        .and_then(|index| PARTICLE_TYPES.get(index))
        .is_some_and(|particle| particle.block_state)
}
