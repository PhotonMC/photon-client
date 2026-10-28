//! Every mob effect, in registry order, as the reference's `MobEffects`
//! declares it.
//!
//! This table is maintained by hand; the parity families compare it with the reference.

use crate::item::effect::{EffectModifier, MobEffect};

#[rustfmt::skip]
pub static MOB_EFFECTS: [MobEffect; 40] = [
    MobEffect::beneficial("minecraft:speed", 0x33EBFF).modifying(&[EffectModifier { attribute: "minecraft:movement_speed", amount: 0.2_f32 as f64, operation: 2 }]),
    MobEffect::harmful("minecraft:slowness", 0x8BAFE0).modifying(&[EffectModifier { attribute: "minecraft:movement_speed", amount: -0.15_f32 as f64, operation: 2 }]),
    MobEffect::beneficial("minecraft:haste", 0xD9C043).modifying(&[EffectModifier { attribute: "minecraft:attack_speed", amount: 0.1_f32 as f64, operation: 2 }]),
    MobEffect::harmful("minecraft:mining_fatigue", 0x4A4217).modifying(&[EffectModifier { attribute: "minecraft:attack_speed", amount: -0.1_f32 as f64, operation: 2 }]),
    MobEffect::beneficial("minecraft:strength", 0xFFC700).modifying(&[EffectModifier { attribute: "minecraft:attack_damage", amount: 3.0, operation: 0 }]),
    MobEffect::beneficial("minecraft:instant_health", 0xF82423),
    MobEffect::harmful("minecraft:instant_damage", 0xA9656A),
    MobEffect::beneficial("minecraft:jump_boost", 0xFDFF84).modifying(&[EffectModifier { attribute: "minecraft:safe_fall_distance", amount: 1.0, operation: 0 }]),
    MobEffect::harmful("minecraft:nausea", 0x551D4A),
    MobEffect::beneficial("minecraft:regeneration", 0xCD5CAB),
    MobEffect::beneficial("minecraft:resistance", 0x9146F0),
    MobEffect::beneficial("minecraft:fire_resistance", 0xFF9900),
    MobEffect::beneficial("minecraft:water_breathing", 0x98DAC0),
    MobEffect::beneficial("minecraft:invisibility", 0xF6F6F6).modifying(&[EffectModifier { attribute: "minecraft:waypoint_transmit_range", amount: -1.0, operation: 2 }]),
    MobEffect::harmful("minecraft:blindness", 0x1F1F23),
    MobEffect::beneficial("minecraft:night_vision", 0xC2FF66),
    MobEffect::harmful("minecraft:hunger", 0x587653),
    MobEffect::harmful("minecraft:weakness", 0x484D48).modifying(&[EffectModifier { attribute: "minecraft:attack_damage", amount: -4.0, operation: 0 }]),
    MobEffect::harmful("minecraft:poison", 0x87A363),
    MobEffect::harmful("minecraft:wither", 0x736156),
    MobEffect::beneficial("minecraft:health_boost", 0xF87D23).modifying(&[EffectModifier { attribute: "minecraft:max_health", amount: 4.0, operation: 0 }]),
    MobEffect::beneficial("minecraft:absorption", 0x2552A5).modifying(&[EffectModifier { attribute: "minecraft:max_absorption", amount: 4.0, operation: 0 }]),
    MobEffect::beneficial("minecraft:saturation", 0xF82423),
    MobEffect::neutral("minecraft:glowing", 0x94A061),
    MobEffect::harmful("minecraft:levitation", 0xCEFFFF),
    MobEffect::beneficial("minecraft:luck", 0x59C106).modifying(&[EffectModifier { attribute: "minecraft:luck", amount: 1.0, operation: 0 }]),
    MobEffect::harmful("minecraft:unluck", 0xC0A44D).modifying(&[EffectModifier { attribute: "minecraft:luck", amount: -1.0, operation: 0 }]),
    MobEffect::beneficial("minecraft:slow_falling", 0xF3CFB9),
    MobEffect::beneficial("minecraft:conduit_power", 0x1DC2D1),
    MobEffect::beneficial("minecraft:dolphins_grace", 0x88A3BE),
    MobEffect::neutral("minecraft:bad_omen", 0x0B6138),
    MobEffect::beneficial("minecraft:hero_of_the_village", 0x44FF44),
    MobEffect::harmful("minecraft:darkness", 0x292721),
    MobEffect::neutral("minecraft:trial_omen", 0x16A6A6),
    MobEffect::neutral("minecraft:raid_omen", 0xDE4058),
    MobEffect::harmful("minecraft:wind_charged", 0xBDC9FF),
    MobEffect::harmful("minecraft:weaving", 0x78695A),
    MobEffect::harmful("minecraft:oozing", 0x99FFA3),
    MobEffect::harmful("minecraft:infested", 0x8C9B8C),
    MobEffect::beneficial("minecraft:breath_of_the_nautilus", 0x00FFEE),
];
