//! The particle types of the reference's registry, in the order of their ids.
//!
//! The table is written from `ParticleTypes` and compared with the reference's own registry
//! on every parity run (`particle_types`).

/// A particle type of the registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParticleType {
    /// The name without the namespace (`smoke`).
    pub name: &'static str,
    /// Whether the type ignores the particle limiter.
    pub overrides_limiter: bool,
    /// Whether the type carries nothing but itself on the wire (`SimpleParticleType`).
    pub simple: bool,
}

const fn particle(name: &'static str, overrides_limiter: bool, simple: bool) -> ParticleType {
    ParticleType {
        name,
        overrides_limiter,
        simple,
    }
}

/// Every particle type, in the order of the ids the wire carries.
pub const PARTICLE_TYPES: [ParticleType; 125] = [
    particle("angry_villager", false, true),
    particle("block", false, false),
    particle("block_marker", true, false),
    particle("bubble", false, true),
    particle("sulfur_bubbles", false, true),
    particle("noxious_gas", false, true),
    particle("noxious_gas_cloud", false, true),
    particle("geyser", true, false),
    particle("geyser_base", true, false),
    particle("geyser_poof", true, false),
    particle("geyser_plume", true, false),
    particle("cloud", false, true),
    particle("copper_fire_flame", false, true),
    particle("crit", false, true),
    particle("damage_indicator", true, true),
    particle("dragon_breath", false, false),
    particle("dripping_lava", false, true),
    particle("falling_lava", false, true),
    particle("landing_lava", false, true),
    particle("dripping_water", false, true),
    particle("falling_water", false, true),
    particle("dust", false, false),
    particle("dust_color_transition", false, false),
    particle("effect", false, false),
    particle("elder_guardian", true, true),
    particle("enchanted_hit", false, true),
    particle("enchant", false, true),
    particle("end_rod", false, true),
    particle("entity_effect", false, false),
    particle("explosion_emitter", true, true),
    particle("explosion", true, true),
    particle("gust", true, true),
    particle("small_gust", false, true),
    particle("gust_emitter_large", true, true),
    particle("gust_emitter_small", true, true),
    particle("sonic_boom", true, true),
    particle("falling_dust", false, false),
    particle("firework", false, true),
    particle("fishing", false, true),
    particle("flame", false, true),
    particle("infested", false, true),
    particle("cherry_leaves", false, true),
    particle("pale_oak_leaves", false, true),
    particle("tinted_leaves", false, false),
    particle("sculk_soul", false, true),
    particle("sculk_charge", true, false),
    particle("sculk_charge_pop", true, true),
    particle("soul_fire_flame", false, true),
    particle("soul", false, true),
    particle("flash", false, false),
    particle("happy_villager", false, true),
    particle("composter", false, true),
    particle("heart", false, true),
    particle("instant_effect", false, false),
    particle("item", false, false),
    particle("vibration", true, false),
    particle("trail", false, false),
    particle("pause_mob_growth", false, true),
    particle("reset_mob_growth", false, true),
    particle("item_slime", false, true),
    particle("item_cobweb", false, true),
    particle("item_snowball", false, true),
    particle("large_smoke", false, true),
    particle("lava", false, true),
    particle("mycelium", false, true),
    particle("note", false, true),
    particle("poof", true, true),
    particle("portal", false, true),
    particle("rain", false, true),
    particle("smoke", false, true),
    particle("white_smoke", false, true),
    particle("sneeze", false, true),
    particle("spit", true, true),
    particle("squid_ink", true, true),
    particle("sweep_attack", true, true),
    particle("totem_of_undying", false, true),
    particle("underwater", false, true),
    particle("splash", false, true),
    particle("witch", false, true),
    particle("bubble_pop", false, true),
    particle("current_down", false, true),
    particle("bubble_column_up", false, true),
    particle("nautilus", false, true),
    particle("dolphin", false, true),
    particle("campfire_cosy_smoke", true, true),
    particle("campfire_signal_smoke", true, true),
    particle("dripping_honey", false, true),
    particle("falling_honey", false, true),
    particle("landing_honey", false, true),
    particle("falling_nectar", false, true),
    particle("falling_spore_blossom", false, true),
    particle("ash", false, true),
    particle("crimson_spore", false, true),
    particle("warped_spore", false, true),
    particle("spore_blossom_air", false, true),
    particle("dripping_obsidian_tear", false, true),
    particle("falling_obsidian_tear", false, true),
    particle("landing_obsidian_tear", false, true),
    particle("reverse_portal", false, true),
    particle("white_ash", false, true),
    particle("small_flame", false, true),
    particle("snowflake", false, true),
    particle("dripping_dripstone_lava", false, true),
    particle("falling_dripstone_lava", false, true),
    particle("dripping_dripstone_water", false, true),
    particle("falling_dripstone_water", false, true),
    particle("glow_squid_ink", true, true),
    particle("glow", true, true),
    particle("wax_on", true, true),
    particle("wax_off", true, true),
    particle("electric_spark", true, true),
    particle("scrape", true, true),
    particle("shriek", false, false),
    particle("egg_crack", false, true),
    particle("dust_plume", false, true),
    particle("trial_spawner_detection", true, true),
    particle("trial_spawner_detection_ominous", true, true),
    particle("vault_connection", true, true),
    particle("dust_pillar", false, false),
    particle("ominous_spawning", true, true),
    particle("raid_omen", false, true),
    particle("trial_omen", false, true),
    particle("block_crumble", false, false),
    particle("firefly", false, true),
    particle("sulfur_cube_goo", false, true),
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
