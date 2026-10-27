use super::*;

pub(super) fn map_data_case() -> Result<Value, String> {
    let mut rows = vec![json!({
        "label": "colors",
        "colors": (0..=255u8)
            .map(|packed| photon_data::map::packed_color(packed) as i32)
            .collect::<Vec<_>>(),
    })];
    for (id, kind) in photon_data::map::decoration_types().iter().enumerate() {
        rows.push(json!({"label": kind.name, "id": id, "asset": kind.asset,
            "show_on_item_frame": kind.show_on_item_frame, "map_color": kind.map_color,
            "track_count": kind.track_count}));
    }
    Ok(json!({"rows": rows}))
}

pub(super) fn painting_variants_case() -> Result<Value, String> {
    let rows: Vec<Value> = photon_data::entity_type::painting_variants()
        .map(|variant| {
            json!({"label": variant.name, "name": variant.name, "asset": variant.asset,
                "width": variant.width, "height": variant.height})
        })
        .collect();
    Ok(json!({"rows": rows}))
}

pub(super) fn wolf_variants_case() -> Result<Value, String> {
    let rows: Vec<Value> = photon_data::entity_type::wolf_variants()
        .map(|variant| {
            let [wild, tame, angry, baby_wild, baby_tame, baby_angry] = variant.assets;
            json!({"label": variant.name, "name": variant.name, "wild": wild, "tame": tame,
                "angry": angry, "baby_wild": baby_wild, "baby_tame": baby_tame,
                "baby_angry": baby_angry})
        })
        .collect();
    Ok(json!({"rows": rows}))
}

pub(super) fn entity_variants_case() -> Result<Value, String> {
    let rows: Vec<Value> = ["cat", "chicken", "cow", "frog", "pig", "zombie_nautilus"]
        .iter()
        .flat_map(|kind| photon_data::entity_type::variants(kind))
        .map(|variant| {
            json!({
                "label": format!("{}:{}", variant.kind, variant.name),
                "kind": variant.kind,
                "name": variant.name,
                "asset": variant.asset,
                "baby_asset": variant.baby_asset,
                "model": variant.model,
            })
        })
        .collect();
    Ok(json!({"rows": rows}))
}

/// The sheared flag of a bogged: a boolean on its own id.
fn bogged_field() -> (&'static str, i64, i64) {
    (
        "bogged-sheared",
        i64::from(photon_world::BOGGED_SHEARED_DATA_ID),
        8,
    )
}

pub(super) fn entity_variant_fields_case() -> Result<Value, String> {
    use photon_world::{
        ARMOR_STAND_FLAGS_DATA_ID, ARMOR_STAND_HEAD_POSE_DATA_ID, AVATAR_MAIN_HAND_DATA_ID,
        BEE_ANGER_END_DATA_ID, BEE_FLAGS_DATA_ID, CAMEL_DASH_DATA_ID,
        CAMEL_LAST_POSE_CHANGE_DATA_ID, CHICKEN_VARIANT_DATA_ID, CHICKEN_VARIANT_SERIALIZER_ID,
        COW_VARIANT_DATA_ID, COW_VARIANT_SERIALIZER_ID, CREEPER_IGNITED_DATA_ID,
        CREEPER_POWERED_DATA_ID, CREEPER_SWELL_DIR_DATA_ID, ENDERMAN_CARRY_DATA_ID,
        ENDERMAN_CREEPY_DATA_ID, EXPERIENCE_ORB_VALUE_DATA_ID, GOAT_LEFT_HORN_DATA_ID,
        GOAT_RIGHT_HORN_DATA_ID, HORSE_CHEST_DATA_ID, HORSE_FLAGS_DATA_ID, HORSE_VARIANT_DATA_ID,
        HUMANOID_ARM_SERIALIZER_ID, LLAMA_VARIANT_DATA_ID, LONG_SERIALIZER_ID,
        MOOSHROOM_TYPE_DATA_ID, OPTIONAL_BLOCK_STATE_SERIALIZER_ID, PIG_VARIANT_DATA_ID,
        PIG_VARIANT_SERIALIZER_ID, PIGLIN_BABY_DATA_ID, PIGLIN_DANCING_DATA_ID,
        RABBIT_TYPE_DATA_ID, RAIDER_CELEBRATING_DATA_ID, ROTATIONS_SERIALIZER_ID,
        SHEEP_WOOL_DATA_ID, SNOW_GOLEM_PUMPKIN_DATA_ID, SPELLCASTER_CASTING_DATA_ID,
        VILLAGER_DATA_DATA_ID, VILLAGER_DATA_SERIALIZER_ID, VILLAGER_UNHAPPY_DATA_ID,
        ZOMBIE_VILLAGER_DATA_DATA_ID,
    };
    let fields: &[(&str, i64, i64)] = &[
        (
            "pig",
            i64::from(PIG_VARIANT_DATA_ID),
            i64::from(PIG_VARIANT_SERIALIZER_ID),
        ),
        (
            "cow",
            i64::from(COW_VARIANT_DATA_ID),
            i64::from(COW_VARIANT_SERIALIZER_ID),
        ),
        (
            "experienceorb",
            i64::from(EXPERIENCE_ORB_VALUE_DATA_ID),
            i64::from(1),
        ),
        ("piglin-baby", i64::from(PIGLIN_BABY_DATA_ID), i64::from(8)),
        (
            "piglin-dancing",
            i64::from(PIGLIN_DANCING_DATA_ID),
            i64::from(8),
        ),
        (
            "avatar-main-hand",
            i64::from(AVATAR_MAIN_HAND_DATA_ID),
            i64::from(HUMANOID_ARM_SERIALIZER_ID),
        ),
        ("sheep-wool", i64::from(SHEEP_WOOL_DATA_ID), i64::from(0)),
        (
            "chicken",
            i64::from(CHICKEN_VARIANT_DATA_ID),
            i64::from(CHICKEN_VARIANT_SERIALIZER_ID),
        ),
        (
            "armor-stand-flags",
            i64::from(ARMOR_STAND_FLAGS_DATA_ID),
            i64::from(0),
        ),
        (
            "armor-stand-head",
            i64::from(ARMOR_STAND_HEAD_POSE_DATA_ID),
            i64::from(ROTATIONS_SERIALIZER_ID),
        ),
        (
            "armor-stand-body",
            i64::from(ARMOR_STAND_HEAD_POSE_DATA_ID + 1),
            i64::from(ROTATIONS_SERIALIZER_ID),
        ),
        (
            "armor-stand-left-arm",
            i64::from(ARMOR_STAND_HEAD_POSE_DATA_ID + 2),
            i64::from(ROTATIONS_SERIALIZER_ID),
        ),
        (
            "armor-stand-right-arm",
            i64::from(ARMOR_STAND_HEAD_POSE_DATA_ID + 3),
            i64::from(ROTATIONS_SERIALIZER_ID),
        ),
        (
            "armor-stand-left-leg",
            i64::from(ARMOR_STAND_HEAD_POSE_DATA_ID + 4),
            i64::from(ROTATIONS_SERIALIZER_ID),
        ),
        (
            "armor-stand-right-leg",
            i64::from(ARMOR_STAND_HEAD_POSE_DATA_ID + 5),
            i64::from(ROTATIONS_SERIALIZER_ID),
        ),
        (
            "villager-data",
            i64::from(VILLAGER_DATA_DATA_ID),
            i64::from(VILLAGER_DATA_SERIALIZER_ID),
        ),
        (
            "villager-unhappy",
            i64::from(VILLAGER_UNHAPPY_DATA_ID),
            i64::from(1),
        ),
        ("horse-flags", i64::from(HORSE_FLAGS_DATA_ID), i64::from(0)),
        (
            "horse-variant",
            i64::from(HORSE_VARIANT_DATA_ID),
            i64::from(1),
        ),
        ("horse-chest", i64::from(HORSE_CHEST_DATA_ID), i64::from(8)),
        (
            "llama-variant",
            i64::from(LLAMA_VARIANT_DATA_ID),
            i64::from(1),
        ),
        ("camel-dash", i64::from(CAMEL_DASH_DATA_ID), i64::from(8)),
        (
            "camel-last-pose",
            i64::from(CAMEL_LAST_POSE_CHANGE_DATA_ID),
            i64::from(LONG_SERIALIZER_ID),
        ),
        (
            "tamable-flags",
            i64::from(photon_world::TAMABLE_FLAGS_DATA_ID),
            i64::from(0),
        ),
        (
            "wolf-interested",
            i64::from(photon_world::WOLF_INTERESTED_DATA_ID),
            i64::from(8),
        ),
        (
            "wolf-collar",
            i64::from(photon_world::WOLF_COLLAR_DATA_ID),
            i64::from(1),
        ),
        (
            "wolf-anger-end",
            i64::from(photon_world::WOLF_ANGER_END_DATA_ID),
            i64::from(LONG_SERIALIZER_ID),
        ),
        (
            "wolf-variant",
            i64::from(photon_world::WOLF_VARIANT_DATA_ID),
            i64::from(photon_world::WOLF_VARIANT_SERIALIZER_ID),
        ),
        (
            "polar-bear-standing",
            i64::from(photon_world::POLAR_BEAR_STANDING_DATA_ID),
            i64::from(8),
        ),
        (
            "strider-suffocating",
            i64::from(photon_world::STRIDER_SUFFOCATING_DATA_ID),
            i64::from(8),
        ),
        (
            "hoglin-immune",
            i64::from(photon_world::HOGLIN_IMMUNE_DATA_ID),
            i64::from(8),
        ),
        (
            "copper-golem-weather",
            i64::from(photon_world::COPPER_GOLEM_WEATHER_DATA_ID),
            38,
        ),
        (
            "copper-golem-state",
            i64::from(photon_world::COPPER_GOLEM_STATE_DATA_ID),
            37,
        ),
        (
            "panda-unhappy",
            i64::from(photon_world::PANDA_UNHAPPY_DATA_ID),
            i64::from(1),
        ),
        (
            "panda-sneeze",
            i64::from(photon_world::PANDA_SNEEZE_DATA_ID),
            i64::from(1),
        ),
        (
            "panda-eat",
            i64::from(photon_world::PANDA_EAT_DATA_ID),
            i64::from(1),
        ),
        (
            "panda-main-gene",
            i64::from(photon_world::PANDA_MAIN_GENE_DATA_ID),
            i64::from(0),
        ),
        (
            "panda-hidden-gene",
            i64::from(photon_world::PANDA_HIDDEN_GENE_DATA_ID),
            i64::from(0),
        ),
        (
            "panda-flags",
            i64::from(photon_world::PANDA_FLAGS_DATA_ID),
            i64::from(0),
        ),
        (
            "fox-variant",
            i64::from(photon_world::FOX_VARIANT_DATA_ID),
            i64::from(1),
        ),
        (
            "fox-flags",
            i64::from(photon_world::FOX_FLAGS_DATA_ID),
            i64::from(0),
        ),
        (
            "cat-variant",
            i64::from(photon_world::CAT_VARIANT_DATA_ID),
            i64::from(photon_world::CAT_VARIANT_SERIALIZER_ID),
        ),
        (
            "cat-lying",
            i64::from(photon_world::CAT_LYING_DATA_ID),
            i64::from(8),
        ),
        (
            "cat-relax",
            i64::from(photon_world::CAT_RELAX_DATA_ID),
            i64::from(8),
        ),
        (
            "cat-collar",
            i64::from(photon_world::CAT_COLLAR_DATA_ID),
            i64::from(1),
        ),
        ("bee-flags", i64::from(BEE_FLAGS_DATA_ID), i64::from(0)),
        (
            "bee-anger-end",
            i64::from(BEE_ANGER_END_DATA_ID),
            i64::from(LONG_SERIALIZER_ID),
        ),
        ("rabbit-type", i64::from(RABBIT_TYPE_DATA_ID), i64::from(1)),
        (
            "goat-left-horn",
            i64::from(GOAT_LEFT_HORN_DATA_ID),
            i64::from(8),
        ),
        (
            "goat-right-horn",
            i64::from(GOAT_RIGHT_HORN_DATA_ID),
            i64::from(8),
        ),
        (
            "mooshroom-type",
            i64::from(MOOSHROOM_TYPE_DATA_ID),
            i64::from(1),
        ),
        (
            "snow-golem-pumpkin",
            i64::from(SNOW_GOLEM_PUMPKIN_DATA_ID),
            i64::from(0),
        ),
        (
            "enderman-carry",
            i64::from(ENDERMAN_CARRY_DATA_ID),
            i64::from(OPTIONAL_BLOCK_STATE_SERIALIZER_ID),
        ),
        (
            "enderman-creepy",
            i64::from(ENDERMAN_CREEPY_DATA_ID),
            i64::from(8),
        ),
        (
            "zombie-villager-data",
            i64::from(ZOMBIE_VILLAGER_DATA_DATA_ID),
            i64::from(VILLAGER_DATA_SERIALIZER_ID),
        ),
        (
            "raider-celebrating",
            i64::from(RAIDER_CELEBRATING_DATA_ID),
            i64::from(8),
        ),
        (
            "spellcaster-casting",
            i64::from(SPELLCASTER_CASTING_DATA_ID),
            i64::from(0),
        ),
        (
            "creeper-powered",
            i64::from(CREEPER_POWERED_DATA_ID),
            i64::from(8),
        ),
        (
            "creeper-swell-dir",
            i64::from(CREEPER_SWELL_DIR_DATA_ID),
            i64::from(1),
        ),
        (
            "creeper-ignited",
            i64::from(CREEPER_IGNITED_DATA_ID),
            i64::from(8),
        ),
        (
            "frog-variant",
            i64::from(photon_world::FROG_VARIANT_DATA_ID),
            i64::from(photon_world::FROG_VARIANT_SERIALIZER_ID),
        ),
        (
            "zombie-nautilus-variant",
            i64::from(photon_world::ZOMBIE_NAUTILUS_VARIANT_DATA_ID),
            i64::from(photon_world::ZOMBIE_NAUTILUS_VARIANT_SERIALIZER_ID),
        ),
        (
            "boat-hurt",
            i64::from(photon_world::BOAT_HURT_DATA_ID),
            i64::from(1),
        ),
        (
            "boat-hurt-dir",
            i64::from(photon_world::BOAT_HURT_DIR_DATA_ID),
            i64::from(1),
        ),
        (
            "boat-damage",
            i64::from(photon_world::BOAT_DAMAGE_DATA_ID),
            i64::from(3),
        ),
        (
            "boat-paddle-left",
            i64::from(photon_world::BOAT_PADDLE_LEFT_DATA_ID),
            i64::from(8),
        ),
        (
            "boat-paddle-right",
            i64::from(photon_world::BOAT_PADDLE_RIGHT_DATA_ID),
            i64::from(8),
        ),
        (
            "boat-bubble-time",
            i64::from(photon_world::BOAT_BUBBLE_TIME_DATA_ID),
            i64::from(1),
        ),
        (
            "parrot-variant",
            i64::from(photon_world::PARROT_VARIANT_DATA_ID),
            i64::from(1),
        ),
        (
            "bat-flags",
            i64::from(photon_world::BAT_FLAGS_DATA_ID),
            i64::from(0),
        ),
        (
            "vex-flags",
            i64::from(photon_world::VEX_FLAGS_DATA_ID),
            i64::from(0),
        ),
        (
            "allay-dancing",
            i64::from(photon_world::ALLAY_DANCING_DATA_ID),
            i64::from(8),
        ),
        (
            "ghast-charging",
            i64::from(photon_world::GHAST_CHARGING_DATA_ID),
            i64::from(8),
        ),
        (
            "phantom-size",
            i64::from(photon_world::PHANTOM_SIZE_DATA_ID),
            i64::from(1),
        ),
        (
            "guardian-moving",
            i64::from(photon_world::GUARDIAN_MOVING_DATA_ID),
            i64::from(8),
        ),
        (
            "guardian-attack-target",
            i64::from(photon_world::GUARDIAN_ATTACK_TARGET_DATA_ID),
            i64::from(1),
        ),
        (
            "axolotl-variant",
            i64::from(photon_world::AXOLOTL_VARIANT_DATA_ID),
            i64::from(1),
        ),
        (
            "axolotl-playing-dead",
            i64::from(photon_world::AXOLOTL_PLAYING_DEAD_DATA_ID),
            i64::from(8),
        ),
        (
            "turtle-has-egg",
            i64::from(photon_world::TURTLE_HAS_EGG_DATA_ID),
            i64::from(8),
        ),
        (
            "turtle-laying-egg",
            i64::from(photon_world::TURTLE_LAYING_EGG_DATA_ID),
            i64::from(8),
        ),
        (
            "salmon-type",
            i64::from(photon_world::SALMON_TYPE_DATA_ID),
            i64::from(1),
        ),
        (
            "tropical-fish-variant",
            i64::from(photon_world::TROPICAL_FISH_VARIANT_DATA_ID),
            i64::from(1),
        ),
        (
            "pufferfish-puff-state",
            i64::from(photon_world::PUFFERFISH_PUFF_STATE_DATA_ID),
            i64::from(1),
        ),
    ];
    Ok(json!({"rows": fields
        .iter().chain([&bogged_field()])
        .map(|(label, data_id, serializer)| {
            json!({"label": label, "data_id": data_id, "serializer": serializer})
        })
        .collect::<Vec<_>>()}))
}

pub(super) fn entity_data_serializers_case() -> Result<Value, String> {
    let rows: Vec<Value> = (0..)
        .map_while(|id| photon_data::entity_type::data_serializer(id).map(|name| (id, name)))
        .map(|(id, name)| json!({"label": name, "id": id}))
        .collect();
    Ok(json!({"rows": rows}))
}

pub(super) fn entity_types_case() -> Result<Value, String> {
    let rows: Vec<Value> = (0..)
        .map_while(|id| {
            photon_data::entity_type::name(photon_core::ids::EntityTypeId(id))
                .map(|name| (id, name))
        })
        .map(|(id, name)| {
            json!({"label": name, "id": id,
                "allowed_in_peaceful": photon_data::entity_type::allowed_in_peaceful(name)})
        })
        .collect();
    Ok(json!({"rows": rows}))
}
