//! The cat, chicken, cow, frog, pig and zombie nautilus variants of the game's data pack.
//!
//! This table is maintained by hand; the parity families compare it with the reference.

use crate::entity_type::EntityVariant;

#[rustfmt::skip]
pub static ENTITY_VARIANTS: [EntityVariant; 25] = [
    EntityVariant { kind: "cat", name: "minecraft:all_black", asset: "minecraft:entity/cat/cat_all_black", baby_asset: "minecraft:entity/cat/cat_all_black_baby", model: "normal" },
    EntityVariant { kind: "cat", name: "minecraft:black", asset: "minecraft:entity/cat/cat_black", baby_asset: "minecraft:entity/cat/cat_black_baby", model: "normal" },
    EntityVariant { kind: "cat", name: "minecraft:british_shorthair", asset: "minecraft:entity/cat/cat_british_shorthair", baby_asset: "minecraft:entity/cat/cat_british_shorthair_baby", model: "normal" },
    EntityVariant { kind: "cat", name: "minecraft:calico", asset: "minecraft:entity/cat/cat_calico", baby_asset: "minecraft:entity/cat/cat_calico_baby", model: "normal" },
    EntityVariant { kind: "cat", name: "minecraft:jellie", asset: "minecraft:entity/cat/cat_jellie", baby_asset: "minecraft:entity/cat/cat_jellie_baby", model: "normal" },
    EntityVariant { kind: "cat", name: "minecraft:persian", asset: "minecraft:entity/cat/cat_persian", baby_asset: "minecraft:entity/cat/cat_persian_baby", model: "normal" },
    EntityVariant { kind: "cat", name: "minecraft:ragdoll", asset: "minecraft:entity/cat/cat_ragdoll", baby_asset: "minecraft:entity/cat/cat_ragdoll_baby", model: "normal" },
    EntityVariant { kind: "cat", name: "minecraft:red", asset: "minecraft:entity/cat/cat_red", baby_asset: "minecraft:entity/cat/cat_red_baby", model: "normal" },
    EntityVariant { kind: "cat", name: "minecraft:siamese", asset: "minecraft:entity/cat/cat_siamese", baby_asset: "minecraft:entity/cat/cat_siamese_baby", model: "normal" },
    EntityVariant { kind: "cat", name: "minecraft:tabby", asset: "minecraft:entity/cat/cat_tabby", baby_asset: "minecraft:entity/cat/cat_tabby_baby", model: "normal" },
    EntityVariant { kind: "cat", name: "minecraft:white", asset: "minecraft:entity/cat/cat_white", baby_asset: "minecraft:entity/cat/cat_white_baby", model: "normal" },
    EntityVariant { kind: "chicken", name: "minecraft:cold", asset: "minecraft:entity/chicken/chicken_cold", baby_asset: "minecraft:entity/chicken/chicken_cold_baby", model: "cold" },
    EntityVariant { kind: "chicken", name: "minecraft:temperate", asset: "minecraft:entity/chicken/chicken_temperate", baby_asset: "minecraft:entity/chicken/chicken_temperate_baby", model: "normal" },
    EntityVariant { kind: "chicken", name: "minecraft:warm", asset: "minecraft:entity/chicken/chicken_warm", baby_asset: "minecraft:entity/chicken/chicken_warm_baby", model: "normal" },
    EntityVariant { kind: "cow", name: "minecraft:cold", asset: "minecraft:entity/cow/cow_cold", baby_asset: "minecraft:entity/cow/cow_cold_baby", model: "cold" },
    EntityVariant { kind: "cow", name: "minecraft:temperate", asset: "minecraft:entity/cow/cow_temperate", baby_asset: "minecraft:entity/cow/cow_temperate_baby", model: "normal" },
    EntityVariant { kind: "cow", name: "minecraft:warm", asset: "minecraft:entity/cow/cow_warm", baby_asset: "minecraft:entity/cow/cow_warm_baby", model: "warm" },
    EntityVariant { kind: "frog", name: "minecraft:cold", asset: "minecraft:entity/frog/frog_cold", baby_asset: "", model: "normal" },
    EntityVariant { kind: "frog", name: "minecraft:temperate", asset: "minecraft:entity/frog/frog_temperate", baby_asset: "", model: "normal" },
    EntityVariant { kind: "frog", name: "minecraft:warm", asset: "minecraft:entity/frog/frog_warm", baby_asset: "", model: "normal" },
    EntityVariant { kind: "pig", name: "minecraft:cold", asset: "minecraft:entity/pig/pig_cold", baby_asset: "minecraft:entity/pig/pig_cold_baby", model: "cold" },
    EntityVariant { kind: "pig", name: "minecraft:temperate", asset: "minecraft:entity/pig/pig_temperate", baby_asset: "minecraft:entity/pig/pig_temperate_baby", model: "normal" },
    EntityVariant { kind: "pig", name: "minecraft:warm", asset: "minecraft:entity/pig/pig_warm", baby_asset: "minecraft:entity/pig/pig_warm_baby", model: "normal" },
    EntityVariant { kind: "zombie_nautilus", name: "minecraft:temperate", asset: "minecraft:entity/nautilus/zombie_nautilus", baby_asset: "", model: "normal" },
    EntityVariant { kind: "zombie_nautilus", name: "minecraft:warm", asset: "minecraft:entity/nautilus/zombie_nautilus_coral", baby_asset: "", model: "warm" },
];
