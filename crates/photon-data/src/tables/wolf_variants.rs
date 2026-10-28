//! The wolf variants of the game's data pack.
//!
//! This table is maintained by hand; the parity families compare it with the reference.

use crate::entity_type::WolfVariant;

#[rustfmt::skip]
pub static WOLF_VARIANTS: [WolfVariant; 9] = [
    WolfVariant { name: "minecraft:ashen", assets: ["minecraft:entity/wolf/wolf_ashen", "minecraft:entity/wolf/wolf_ashen_tame", "minecraft:entity/wolf/wolf_ashen_angry", "minecraft:entity/wolf/wolf_ashen_baby", "minecraft:entity/wolf/wolf_ashen_tame_baby", "minecraft:entity/wolf/wolf_ashen_angry_baby"] },
    WolfVariant { name: "minecraft:black", assets: ["minecraft:entity/wolf/wolf_black", "minecraft:entity/wolf/wolf_black_tame", "minecraft:entity/wolf/wolf_black_angry", "minecraft:entity/wolf/wolf_black_baby", "minecraft:entity/wolf/wolf_black_tame_baby", "minecraft:entity/wolf/wolf_black_angry_baby"] },
    WolfVariant { name: "minecraft:chestnut", assets: ["minecraft:entity/wolf/wolf_chestnut", "minecraft:entity/wolf/wolf_chestnut_tame", "minecraft:entity/wolf/wolf_chestnut_angry", "minecraft:entity/wolf/wolf_chestnut_baby", "minecraft:entity/wolf/wolf_chestnut_tame_baby", "minecraft:entity/wolf/wolf_chestnut_angry_baby"] },
    WolfVariant { name: "minecraft:pale", assets: ["minecraft:entity/wolf/wolf", "minecraft:entity/wolf/wolf_tame", "minecraft:entity/wolf/wolf_angry", "minecraft:entity/wolf/wolf_baby", "minecraft:entity/wolf/wolf_tame_baby", "minecraft:entity/wolf/wolf_angry_baby"] },
    WolfVariant { name: "minecraft:rusty", assets: ["minecraft:entity/wolf/wolf_rusty", "minecraft:entity/wolf/wolf_rusty_tame", "minecraft:entity/wolf/wolf_rusty_angry", "minecraft:entity/wolf/wolf_rusty_baby", "minecraft:entity/wolf/wolf_rusty_tame_baby", "minecraft:entity/wolf/wolf_rusty_angry_baby"] },
    WolfVariant { name: "minecraft:snowy", assets: ["minecraft:entity/wolf/wolf_snowy", "minecraft:entity/wolf/wolf_snowy_tame", "minecraft:entity/wolf/wolf_snowy_angry", "minecraft:entity/wolf/wolf_snowy_baby", "minecraft:entity/wolf/wolf_snowy_tame_baby", "minecraft:entity/wolf/wolf_snowy_angry_baby"] },
    WolfVariant { name: "minecraft:spotted", assets: ["minecraft:entity/wolf/wolf_spotted", "minecraft:entity/wolf/wolf_spotted_tame", "minecraft:entity/wolf/wolf_spotted_angry", "minecraft:entity/wolf/wolf_spotted_baby", "minecraft:entity/wolf/wolf_spotted_tame_baby", "minecraft:entity/wolf/wolf_spotted_angry_baby"] },
    WolfVariant { name: "minecraft:striped", assets: ["minecraft:entity/wolf/wolf_striped", "minecraft:entity/wolf/wolf_striped_tame", "minecraft:entity/wolf/wolf_striped_angry", "minecraft:entity/wolf/wolf_striped_baby", "minecraft:entity/wolf/wolf_striped_tame_baby", "minecraft:entity/wolf/wolf_striped_angry_baby"] },
    WolfVariant { name: "minecraft:woods", assets: ["minecraft:entity/wolf/wolf_woods", "minecraft:entity/wolf/wolf_woods_tame", "minecraft:entity/wolf/wolf_woods_angry", "minecraft:entity/wolf/wolf_woods_baby", "minecraft:entity/wolf/wolf_woods_tame_baby", "minecraft:entity/wolf/wolf_woods_angry_baby"] },
];
