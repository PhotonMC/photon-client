//! The data component types in registry order, as `DataComponents` registers
//! them: each a named id.
//!
//! This table is maintained by hand; the parity families compare it with the reference.

pub const CUSTOM_DATA: i32 = 0;
pub const MAX_STACK_SIZE: i32 = 1;
pub const MAX_DAMAGE: i32 = 2;
pub const DAMAGE: i32 = 3;
pub const UNBREAKABLE: i32 = 4;
pub const USE_EFFECTS: i32 = 5;
pub const CUSTOM_NAME: i32 = 6;
pub const MINIMUM_ATTACK_CHARGE: i32 = 7;
pub const DAMAGE_TYPE: i32 = 8;
pub const ITEM_NAME: i32 = 9;
pub const ITEM_MODEL: i32 = 10;
pub const LORE: i32 = 11;
pub const RARITY: i32 = 12;
pub const ENCHANTMENTS: i32 = 13;
pub const CAN_PLACE_ON: i32 = 14;
pub const CAN_BREAK: i32 = 15;
pub const ATTRIBUTE_MODIFIERS: i32 = 16;
pub const CUSTOM_MODEL_DATA: i32 = 17;
pub const TOOLTIP_DISPLAY: i32 = 18;
pub const REPAIR_COST: i32 = 19;
pub const CREATIVE_SLOT_LOCK: i32 = 20;
pub const ENCHANTMENT_GLINT_OVERRIDE: i32 = 21;
pub const INTANGIBLE_PROJECTILE: i32 = 22;
pub const FOOD: i32 = 23;
pub const CONSUMABLE: i32 = 24;
pub const USE_REMAINDER: i32 = 25;
pub const USE_COOLDOWN: i32 = 26;
pub const DAMAGE_RESISTANT: i32 = 27;
pub const TOOL: i32 = 28;
pub const WEAPON: i32 = 29;
pub const ATTACK_RANGE: i32 = 30;
pub const ENCHANTABLE: i32 = 31;
pub const EQUIPPABLE: i32 = 32;
pub const REPAIRABLE: i32 = 33;
pub const GLIDER: i32 = 34;
pub const TOOLTIP_STYLE: i32 = 35;
pub const DEATH_PROTECTION: i32 = 36;
pub const BLOCKS_ATTACKS: i32 = 37;
pub const PIERCING_WEAPON: i32 = 38;
pub const KINETIC_WEAPON: i32 = 39;
pub const SWING_ANIMATION: i32 = 40;
pub const ADDITIONAL_TRADE_COST: i32 = 41;
pub const STORED_ENCHANTMENTS: i32 = 42;
pub const DYE: i32 = 43;
pub const DYED_COLOR: i32 = 44;
pub const MAP_COLOR: i32 = 45;
pub const MAP_ID: i32 = 46;
pub const MAP_DECORATIONS: i32 = 47;
pub const MAP_POST_PROCESSING: i32 = 48;
pub const CHARGED_PROJECTILES: i32 = 49;
pub const BUNDLE_CONTENTS: i32 = 50;
pub const POTION_CONTENTS: i32 = 51;
pub const POTION_DURATION_SCALE: i32 = 52;
pub const SUSPICIOUS_STEW_EFFECTS: i32 = 53;
pub const WRITABLE_BOOK_CONTENT: i32 = 54;
pub const WRITTEN_BOOK_CONTENT: i32 = 55;
pub const TRIM: i32 = 56;
pub const DEBUG_STICK_STATE: i32 = 57;
pub const ENTITY_DATA: i32 = 58;
pub const BUCKET_ENTITY_DATA: i32 = 59;
pub const BLOCK_ENTITY_DATA: i32 = 60;
pub const INSTRUMENT: i32 = 61;
pub const PROVIDES_TRIM_MATERIAL: i32 = 62;
pub const OMINOUS_BOTTLE_AMPLIFIER: i32 = 63;
pub const JUKEBOX_PLAYABLE: i32 = 64;
pub const PROVIDES_BANNER_PATTERNS: i32 = 65;
pub const RECIPES: i32 = 66;
pub const LODESTONE_TRACKER: i32 = 67;
pub const FIREWORK_EXPLOSION: i32 = 68;
pub const FIREWORKS: i32 = 69;
pub const PROFILE: i32 = 70;
pub const NOTE_BLOCK_SOUND: i32 = 71;
pub const BANNER_PATTERNS: i32 = 72;
pub const BASE_COLOR: i32 = 73;
pub const POT_DECORATIONS: i32 = 74;
pub const CONTAINER: i32 = 75;
pub const BLOCK_STATE: i32 = 76;
pub const BEES: i32 = 77;
pub const SULFUR_CUBE_CONTENT: i32 = 78;
pub const LOCK: i32 = 79;
pub const CONTAINER_LOOT: i32 = 80;
pub const BREAK_SOUND: i32 = 81;
pub const VILLAGER_VARIANT: i32 = 82;
pub const WOLF_VARIANT: i32 = 83;
pub const WOLF_SOUND_VARIANT: i32 = 84;
pub const WOLF_COLLAR: i32 = 85;
pub const FOX_VARIANT: i32 = 86;
pub const SALMON_SIZE: i32 = 87;
pub const PARROT_VARIANT: i32 = 88;
pub const TROPICAL_FISH_PATTERN: i32 = 89;
pub const TROPICAL_FISH_BASE_COLOR: i32 = 90;
pub const TROPICAL_FISH_PATTERN_COLOR: i32 = 91;
pub const MOOSHROOM_VARIANT: i32 = 92;
pub const RABBIT_VARIANT: i32 = 93;
pub const PIG_VARIANT: i32 = 94;
pub const PIG_SOUND_VARIANT: i32 = 95;
pub const COW_VARIANT: i32 = 96;
pub const COW_SOUND_VARIANT: i32 = 97;
pub const CHICKEN_VARIANT: i32 = 98;
pub const CHICKEN_SOUND_VARIANT: i32 = 99;
pub const ZOMBIE_NAUTILUS_VARIANT: i32 = 100;
pub const FROG_VARIANT: i32 = 101;
pub const HORSE_VARIANT: i32 = 102;
pub const PAINTING_VARIANT: i32 = 103;
pub const LLAMA_VARIANT: i32 = 104;
pub const AXOLOTL_VARIANT: i32 = 105;
pub const CAT_VARIANT: i32 = 106;
pub const CAT_SOUND_VARIANT: i32 = 107;
pub const CAT_COLLAR: i32 = 108;
pub const SHEEP_COLOR: i32 = 109;
pub const SHULKER_COLOR: i32 = 110;

/// Every component type's name, by id.
#[rustfmt::skip]
pub static NAMES: [&str; 111] = [
    "minecraft:custom_data",
    "minecraft:max_stack_size",
    "minecraft:max_damage",
    "minecraft:damage",
    "minecraft:unbreakable",
    "minecraft:use_effects",
    "minecraft:custom_name",
    "minecraft:minimum_attack_charge",
    "minecraft:damage_type",
    "minecraft:item_name",
    "minecraft:item_model",
    "minecraft:lore",
    "minecraft:rarity",
    "minecraft:enchantments",
    "minecraft:can_place_on",
    "minecraft:can_break",
    "minecraft:attribute_modifiers",
    "minecraft:custom_model_data",
    "minecraft:tooltip_display",
    "minecraft:repair_cost",
    "minecraft:creative_slot_lock",
    "minecraft:enchantment_glint_override",
    "minecraft:intangible_projectile",
    "minecraft:food",
    "minecraft:consumable",
    "minecraft:use_remainder",
    "minecraft:use_cooldown",
    "minecraft:damage_resistant",
    "minecraft:tool",
    "minecraft:weapon",
    "minecraft:attack_range",
    "minecraft:enchantable",
    "minecraft:equippable",
    "minecraft:repairable",
    "minecraft:glider",
    "minecraft:tooltip_style",
    "minecraft:death_protection",
    "minecraft:blocks_attacks",
    "minecraft:piercing_weapon",
    "minecraft:kinetic_weapon",
    "minecraft:swing_animation",
    "minecraft:additional_trade_cost",
    "minecraft:stored_enchantments",
    "minecraft:dye",
    "minecraft:dyed_color",
    "minecraft:map_color",
    "minecraft:map_id",
    "minecraft:map_decorations",
    "minecraft:map_post_processing",
    "minecraft:charged_projectiles",
    "minecraft:bundle_contents",
    "minecraft:potion_contents",
    "minecraft:potion_duration_scale",
    "minecraft:suspicious_stew_effects",
    "minecraft:writable_book_content",
    "minecraft:written_book_content",
    "minecraft:trim",
    "minecraft:debug_stick_state",
    "minecraft:entity_data",
    "minecraft:bucket_entity_data",
    "minecraft:block_entity_data",
    "minecraft:instrument",
    "minecraft:provides_trim_material",
    "minecraft:ominous_bottle_amplifier",
    "minecraft:jukebox_playable",
    "minecraft:provides_banner_patterns",
    "minecraft:recipes",
    "minecraft:lodestone_tracker",
    "minecraft:firework_explosion",
    "minecraft:fireworks",
    "minecraft:profile",
    "minecraft:note_block_sound",
    "minecraft:banner_patterns",
    "minecraft:base_color",
    "minecraft:pot_decorations",
    "minecraft:container",
    "minecraft:block_state",
    "minecraft:bees",
    "minecraft:sulfur_cube_content",
    "minecraft:lock",
    "minecraft:container_loot",
    "minecraft:break_sound",
    "minecraft:villager/variant",
    "minecraft:wolf/variant",
    "minecraft:wolf/sound_variant",
    "minecraft:wolf/collar",
    "minecraft:fox/variant",
    "minecraft:salmon/size",
    "minecraft:parrot/variant",
    "minecraft:tropical_fish/pattern",
    "minecraft:tropical_fish/base_color",
    "minecraft:tropical_fish/pattern_color",
    "minecraft:mooshroom/variant",
    "minecraft:rabbit/variant",
    "minecraft:pig/variant",
    "minecraft:pig/sound_variant",
    "minecraft:cow/variant",
    "minecraft:cow/sound_variant",
    "minecraft:chicken/variant",
    "minecraft:chicken/sound_variant",
    "minecraft:zombie_nautilus/variant",
    "minecraft:frog/variant",
    "minecraft:horse/variant",
    "minecraft:painting/variant",
    "minecraft:llama/variant",
    "minecraft:axolotl/variant",
    "minecraft:cat/variant",
    "minecraft:cat/sound_variant",
    "minecraft:cat/collar",
    "minecraft:sheep/color",
    "minecraft:shulker/color",
];

/// A component type's id by name.
pub fn id(name: &str) -> Option<i32> {
    NAMES
        .iter()
        .position(|known| *known == name)
        .map(|id| id as i32)
}
