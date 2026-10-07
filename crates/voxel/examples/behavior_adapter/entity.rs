use super::*;

pub(super) fn entity_result(request: &Value) -> Result<Value, String> {
    use voxel_render::player_model::{Arm, PlayerModel, PlayerRenderState};
    match string(request, "operation")? {
        "walk_animation" => return walk_animation_result(request),
        "model_layers" => return Ok(model_layers_result()),
        "quadruped_model" => return quadruped_model_result(request),
        "item_entity_pose" => return item_entity_pose_result(request),
        "living_pose" => return living_pose_result(request),
        "held_item_pose" => return held_item_pose_result(request),
        "spear_table" => return Ok(spear_table_result()),
        "sheep_model" => return sheep_model_result(request),
        "armor_stand_model" => return armor_stand_model_result(request),
        "equipment_layers" => return equipment_layers_result(),
        "iron_golem_model" => return iron_golem_model_result(request),
        "snow_golem_model" => return snow_golem_model_result(request),
        "enderman_model" => return enderman_model_result(request),
        "iron_golem_render" => return iron_golem_render_result(request),
        "entity_block_layers" => return entity_block_layers_result(request),
        "eyes_layers" => return eyes_layers_result(request),
        "swirl_layers" => return swirl_layers_result(request),
        "equine_model" => return equine_model_result(request),
        "camel_model" => return camel_model_result(request),
        "llama_model" => return llama_model_result(request),
        "wolf_model" => return wolf_model_result(request),
        "feline_model" => return feline_model_result(request),
        "polar_bear_model" => return polar_bear_model_result(request),
        "hoglin_model" => return hoglin_model_result(request),
        "copper_golem_model" => return copper_golem_model_result(request),
        "squid_model" => return squid_model_result(request),
        "magma_cube_model" => return magma_cube_model_result(request),
        "shulker_model" => return shulker_model_result(request),
        "sea_animal_model" => return sea_animal_model_result(request),
        "dolphin_item_pose" => return dolphin_item_pose_result(request),
        "axolotl_model" => return axolotl_model_result(request),
        "frog_model" => return frog_model_result(request),
        "nautilus_model" => return nautilus_model_result(request),
        "guardian_model" => return guardian_model_result(request),
        "guardian_beam" => return guardian_beam_result(request),
        "dragon_rays" => return dragon_rays_result(request),
        "minecart_pose" => return minecart_pose_result(request),
        "item_frame_pose" => return item_frame_pose_result(request),
        "display_pose" => return display_pose_result(request),
        "item_foil" => return item_foil_result(request),
        "map_render" => return map_render_result(request),
        "text_display_submit" => return text_display_submit_result(request),
        "item_equippable" => return item_equippable_result(request),
        "glint_layers" => return glint_layers_result(request),
        "camera_hurt" => return camera_hurt_result(request),
        "chest_model" => return chest_model_result(request),
        "chest_pose" => return chest_pose_result(request),
        "bell_pose" => return bell_pose_result(request),
        "banner_pose" => return banner_pose_result(request),
        "skull_block_pose" => return skull_block_pose_result(request),
        "shulker_box_pose" => return shulker_box_pose_result(request),
        "display_light" => return display_light_result(request),
        "arrow_pose" => return arrow_pose_result(request),
        "thrown_item_pose" => return thrown_item_pose_result(request),
        "dragon_fireball_quad" => return dragon_fireball_quad_result(request),
        "fishing_hook" => return fishing_hook_result(request),
        "leash_strip" => return leash_strip_result(request),
        "elytra_model" => return elytra_model_result(request),
        "avatar_pose" => return avatar_pose_result(request),
        "primed_tnt_pose" => return primed_tnt_pose_result(request),
        "lightning_bolt" => return lightning_bolt_result(request),
        "painting_geometry" => return painting_geometry_result(request),
        "small_projectile_pose" => return small_projectile_pose_result(request),
        "crystal_beam" => return crystal_beam_result(request),
        "airborne_model" => return airborne_model_result(request),
        "parrot_textures" => return parrot_textures_result(request),
        "flier_item_pose" => return flier_item_pose_result(request),
        "boat_model" => return boat_model_result(request),
        "boat_pose" => return boat_pose_result(request),
        "fish_model" => return fish_model_result(request),
        "fish_textures" => return fish_textures_result(request),
        "glow_squid_light" => return glow_squid_light_result(request),
        "copper_golem_layers" => return copper_golem_layers_result(request),
        "fox_model" => return fox_model_result(request),
        "panda_model" => return panda_model_result(request),
        "ravager_model" => return ravager_model_result(request),
        "strider_model" => return strider_model_result(request),
        "strider_textures" => return Ok(strider_textures_result()),
        "panda_textures" => return Ok(panda_textures_result()),
        "setup_rotations" => return setup_rotations_result(request),
        "shaking_rotations" => return shaking_rotations_result(request),
        "fox_textures" => return Ok(fox_textures_result()),
        "goat_model" => return goat_model_result(request),
        "villager_held_layers" => return villager_held_layers_result(request),
        "entity_attributes" => return Ok(entity_attributes_result()),
        "animation_definitions" => return Ok(animation_definitions_result()),
        "rabbit_model" => return rabbit_model_result(request),
        "segmented_model" => return segmented_model_result(request),
        "bee_model" => return bee_model_result(request),
        "keyframe_animation" => return keyframe_animation_result(request),
        "animation_states" => return animation_states_result(request),
        "illager_model" => return illager_model_result(request),
        "witch_model" => return witch_model_result(request),
        "villager_registries" => return Ok(villager_registries_result()),
        "villager_layers" => return villager_layers_result(request),
        "villager_model" => return villager_model_result(request),
        "spider_model" => return spider_model_result(request, false),
        "creeper_model" => return spider_model_result(request, true),
        "creeper_render" => return creeper_render_result(request),
        "chicken_model" => return chicken_model_result(request),
        "sheep_colors" => return sheep_colors_result(request),
        "sheep_eat" => return sheep_eat_result(request),
        "zombie_model" => return zombie_model_result(request),
        "skeleton_model" => return skeleton_model_result(request),
        "piglin_model" => return piglin_model_result(request),
        "experience_orb" => return experience_orb_result(request),
        "map_data" => {
            let mut rows = vec![json!({
                "label": "colors",
                "colors": (0..=255u8)
                    .map(|packed| voxel_data::map::packed_color(packed) as i32)
                    .collect::<Vec<_>>(),
            })];
            for (id, kind) in voxel_data::map::decoration_types().iter().enumerate() {
                rows.push(json!({"label": kind.name, "id": id, "asset": kind.asset,
                    "show_on_item_frame": kind.show_on_item_frame, "map_color": kind.map_color,
                    "track_count": kind.track_count}));
            }
            return Ok(json!({"rows": rows}));
        }
        "painting_variants" => {
            let rows: Vec<Value> = voxel_data::entity_type::painting_variants()
                .map(|variant| {
                    json!({"label": variant.name, "name": variant.name, "asset": variant.asset,
                        "width": variant.width, "height": variant.height})
                })
                .collect();
            return Ok(json!({"rows": rows}));
        }
        "wolf_variants" => {
            let rows: Vec<Value> = voxel_data::entity_type::wolf_variants()
                .map(|variant| {
                    let [wild, tame, angry, baby_wild, baby_tame, baby_angry] = variant.assets;
                    json!({"label": variant.name, "name": variant.name, "wild": wild, "tame": tame,
                        "angry": angry, "baby_wild": baby_wild, "baby_tame": baby_tame,
                        "baby_angry": baby_angry})
                })
                .collect();
            return Ok(json!({"rows": rows}));
        }
        "entity_variants" => {
            let rows: Vec<Value> = ["cat", "chicken", "cow", "frog", "pig", "zombie_nautilus"]
                .iter()
                .flat_map(|kind| voxel_data::entity_type::variants(kind))
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
            return Ok(json!({"rows": rows}));
        }
        "entity_variant_fields" => {
            use voxel_world::{
                ARMOR_STAND_FLAGS_DATA_ID, ARMOR_STAND_HEAD_POSE_DATA_ID, AVATAR_MAIN_HAND_DATA_ID,
                BEE_ANGER_END_DATA_ID, BEE_FLAGS_DATA_ID, CAMEL_DASH_DATA_ID,
                CAMEL_LAST_POSE_CHANGE_DATA_ID, CHICKEN_VARIANT_DATA_ID,
                CHICKEN_VARIANT_SERIALIZER_ID, COW_VARIANT_DATA_ID, COW_VARIANT_SERIALIZER_ID,
                CREEPER_IGNITED_DATA_ID, CREEPER_POWERED_DATA_ID, CREEPER_SWELL_DIR_DATA_ID,
                ENDERMAN_CARRY_DATA_ID, ENDERMAN_CREEPY_DATA_ID, EXPERIENCE_ORB_VALUE_DATA_ID,
                GOAT_LEFT_HORN_DATA_ID, GOAT_RIGHT_HORN_DATA_ID, HORSE_CHEST_DATA_ID,
                HORSE_FLAGS_DATA_ID, HORSE_VARIANT_DATA_ID, HUMANOID_ARM_SERIALIZER_ID,
                LLAMA_VARIANT_DATA_ID, LONG_SERIALIZER_ID, MOOSHROOM_TYPE_DATA_ID,
                OPTIONAL_BLOCK_STATE_SERIALIZER_ID, PIG_VARIANT_DATA_ID, PIG_VARIANT_SERIALIZER_ID,
                PIGLIN_BABY_DATA_ID, PIGLIN_DANCING_DATA_ID, RABBIT_TYPE_DATA_ID,
                RAIDER_CELEBRATING_DATA_ID, ROTATIONS_SERIALIZER_ID, SHEEP_WOOL_DATA_ID,
                SNOW_GOLEM_PUMPKIN_DATA_ID, SPELLCASTER_CASTING_DATA_ID, VILLAGER_DATA_DATA_ID,
                VILLAGER_DATA_SERIALIZER_ID, VILLAGER_UNHAPPY_DATA_ID,
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
                    i64::from(voxel_world::TAMABLE_FLAGS_DATA_ID),
                    i64::from(0),
                ),
                (
                    "wolf-interested",
                    i64::from(voxel_world::WOLF_INTERESTED_DATA_ID),
                    i64::from(8),
                ),
                (
                    "wolf-collar",
                    i64::from(voxel_world::WOLF_COLLAR_DATA_ID),
                    i64::from(1),
                ),
                (
                    "wolf-anger-end",
                    i64::from(voxel_world::WOLF_ANGER_END_DATA_ID),
                    i64::from(LONG_SERIALIZER_ID),
                ),
                (
                    "wolf-variant",
                    i64::from(voxel_world::WOLF_VARIANT_DATA_ID),
                    i64::from(voxel_world::WOLF_VARIANT_SERIALIZER_ID),
                ),
                (
                    "polar-bear-standing",
                    i64::from(voxel_world::POLAR_BEAR_STANDING_DATA_ID),
                    i64::from(8),
                ),
                (
                    "strider-suffocating",
                    i64::from(voxel_world::STRIDER_SUFFOCATING_DATA_ID),
                    i64::from(8),
                ),
                (
                    "hoglin-immune",
                    i64::from(voxel_world::HOGLIN_IMMUNE_DATA_ID),
                    i64::from(8),
                ),
                (
                    "copper-golem-weather",
                    i64::from(voxel_world::COPPER_GOLEM_WEATHER_DATA_ID),
                    38,
                ),
                (
                    "copper-golem-state",
                    i64::from(voxel_world::COPPER_GOLEM_STATE_DATA_ID),
                    37,
                ),
                (
                    "panda-unhappy",
                    i64::from(voxel_world::PANDA_UNHAPPY_DATA_ID),
                    i64::from(1),
                ),
                (
                    "panda-sneeze",
                    i64::from(voxel_world::PANDA_SNEEZE_DATA_ID),
                    i64::from(1),
                ),
                (
                    "panda-eat",
                    i64::from(voxel_world::PANDA_EAT_DATA_ID),
                    i64::from(1),
                ),
                (
                    "panda-main-gene",
                    i64::from(voxel_world::PANDA_MAIN_GENE_DATA_ID),
                    i64::from(0),
                ),
                (
                    "panda-hidden-gene",
                    i64::from(voxel_world::PANDA_HIDDEN_GENE_DATA_ID),
                    i64::from(0),
                ),
                (
                    "panda-flags",
                    i64::from(voxel_world::PANDA_FLAGS_DATA_ID),
                    i64::from(0),
                ),
                (
                    "fox-variant",
                    i64::from(voxel_world::FOX_VARIANT_DATA_ID),
                    i64::from(1),
                ),
                (
                    "fox-flags",
                    i64::from(voxel_world::FOX_FLAGS_DATA_ID),
                    i64::from(0),
                ),
                (
                    "cat-variant",
                    i64::from(voxel_world::CAT_VARIANT_DATA_ID),
                    i64::from(voxel_world::CAT_VARIANT_SERIALIZER_ID),
                ),
                (
                    "cat-lying",
                    i64::from(voxel_world::CAT_LYING_DATA_ID),
                    i64::from(8),
                ),
                (
                    "cat-relax",
                    i64::from(voxel_world::CAT_RELAX_DATA_ID),
                    i64::from(8),
                ),
                (
                    "cat-collar",
                    i64::from(voxel_world::CAT_COLLAR_DATA_ID),
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
                    i64::from(voxel_world::FROG_VARIANT_DATA_ID),
                    i64::from(voxel_world::FROG_VARIANT_SERIALIZER_ID),
                ),
                (
                    "zombie-nautilus-variant",
                    i64::from(voxel_world::ZOMBIE_NAUTILUS_VARIANT_DATA_ID),
                    i64::from(voxel_world::ZOMBIE_NAUTILUS_VARIANT_SERIALIZER_ID),
                ),
                (
                    "boat-hurt",
                    i64::from(voxel_world::BOAT_HURT_DATA_ID),
                    i64::from(1),
                ),
                (
                    "boat-hurt-dir",
                    i64::from(voxel_world::BOAT_HURT_DIR_DATA_ID),
                    i64::from(1),
                ),
                (
                    "boat-damage",
                    i64::from(voxel_world::BOAT_DAMAGE_DATA_ID),
                    i64::from(3),
                ),
                (
                    "boat-paddle-left",
                    i64::from(voxel_world::BOAT_PADDLE_LEFT_DATA_ID),
                    i64::from(8),
                ),
                (
                    "boat-paddle-right",
                    i64::from(voxel_world::BOAT_PADDLE_RIGHT_DATA_ID),
                    i64::from(8),
                ),
                (
                    "boat-bubble-time",
                    i64::from(voxel_world::BOAT_BUBBLE_TIME_DATA_ID),
                    i64::from(1),
                ),
                (
                    "parrot-variant",
                    i64::from(voxel_world::PARROT_VARIANT_DATA_ID),
                    i64::from(1),
                ),
                (
                    "bat-flags",
                    i64::from(voxel_world::BAT_FLAGS_DATA_ID),
                    i64::from(0),
                ),
                (
                    "vex-flags",
                    i64::from(voxel_world::VEX_FLAGS_DATA_ID),
                    i64::from(0),
                ),
                (
                    "allay-dancing",
                    i64::from(voxel_world::ALLAY_DANCING_DATA_ID),
                    i64::from(8),
                ),
                (
                    "ghast-charging",
                    i64::from(voxel_world::GHAST_CHARGING_DATA_ID),
                    i64::from(8),
                ),
                (
                    "phantom-size",
                    i64::from(voxel_world::PHANTOM_SIZE_DATA_ID),
                    i64::from(1),
                ),
                (
                    "guardian-moving",
                    i64::from(voxel_world::GUARDIAN_MOVING_DATA_ID),
                    i64::from(8),
                ),
                (
                    "guardian-attack-target",
                    i64::from(voxel_world::GUARDIAN_ATTACK_TARGET_DATA_ID),
                    i64::from(1),
                ),
                (
                    "axolotl-variant",
                    i64::from(voxel_world::AXOLOTL_VARIANT_DATA_ID),
                    i64::from(1),
                ),
                (
                    "axolotl-playing-dead",
                    i64::from(voxel_world::AXOLOTL_PLAYING_DEAD_DATA_ID),
                    i64::from(8),
                ),
                (
                    "turtle-has-egg",
                    i64::from(voxel_world::TURTLE_HAS_EGG_DATA_ID),
                    i64::from(8),
                ),
                (
                    "turtle-laying-egg",
                    i64::from(voxel_world::TURTLE_LAYING_EGG_DATA_ID),
                    i64::from(8),
                ),
                (
                    "salmon-type",
                    i64::from(voxel_world::SALMON_TYPE_DATA_ID),
                    i64::from(1),
                ),
                (
                    "tropical-fish-variant",
                    i64::from(voxel_world::TROPICAL_FISH_VARIANT_DATA_ID),
                    i64::from(1),
                ),
                (
                    "pufferfish-puff-state",
                    i64::from(voxel_world::PUFFERFISH_PUFF_STATE_DATA_ID),
                    i64::from(1),
                ),
            ];
            return Ok(json!({"rows": fields
                .iter()
                .map(|(label, data_id, serializer)| {
                    json!({"label": label, "data_id": data_id, "serializer": serializer})
                })
                .collect::<Vec<_>>()}));
        }
        "entity_data_serializers" => {
            let rows: Vec<Value> = (0..)
                .map_while(|id| voxel_data::entity_type::data_serializer(id).map(|name| (id, name)))
                .map(|(id, name)| json!({"label": name, "id": id}))
                .collect();
            return Ok(json!({"rows": rows}));
        }
        "entity_types" => {
            let rows: Vec<Value> = (0..)
                .map_while(|id| voxel_data::entity_type::name(id).map(|name| (id, name)))
                .map(|(id, name)| json!({"label": name, "id": id}))
                .collect();
            return Ok(json!({"rows": rows}));
        }
        "player_model" => {}
        _ => return Err("unknown entity operation".into()),
    }
    let rows = request["rows"]
        .as_array()
        .ok_or("entity rows must be an array")?;
    let bits = |row: &Value, key: &str| -> Result<f32, String> {
        let text = row[key].as_str().ok_or(format!("{key} must be a string"))?;
        u32::from_str_radix(text, 16)
            .map(f32::from_bits)
            .map_err(|error| error.to_string())
    };
    let flag = |row: &Value, key: &str| row[key].as_bool().unwrap_or(false);
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let mut model = PlayerModel::new(flag(row, "slim"));
        model.setup_anim(&PlayerRenderState {
            head_y_rot: bits(row, "head_y_rot")?,
            x_rot: bits(row, "x_rot")?,
            walk_position: bits(row, "walk_position")?,
            walk_speed: bits(row, "walk_speed")?,
            age_in_ticks: bits(row, "age_in_ticks")?,
            attack_time: bits(row, "attack_time")?,
            attack_arm: if row["attack_arm"].as_str() == Some("left") {
                Arm::Left
            } else {
                Arm::Right
            },
            swim_amount: bits(row, "swim_amount")?,
            crouching: flag(row, "crouching"),
            spectator: flag(row, "spectator"),
            ..Default::default()
        });
        let hex = |value: f32| format!("{:08x}", value.to_bits());
        let mut quads: Vec<String> = model
            .render(&Default::default())
            .into_iter()
            .map(|(_, corners)| {
                corners
                    .iter()
                    .map(|vertex| {
                        vertex
                            .position
                            .iter()
                            .chain(&vertex.uv)
                            .chain(&vertex.normal)
                            .map(|value| hex(*value))
                            .collect::<String>()
                    })
                    .collect::<String>()
            })
            .collect();
        quads.sort();
        out.push(json!({"label": row["label"], "quads": quads}));
    }
    Ok(json!({"rows": out}))
}
