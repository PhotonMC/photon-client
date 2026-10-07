use super::*;

/// `living_entity_ticks`: a living entity through the packets that hurt or
/// kill it, as the net layer hands them to the world, ticked on.
pub(super) fn living_entity_ticks_result(request: &Value) -> Result<Value, String> {
    let bits64 = |row: &Value, key: &str| -> Result<f64, String> {
        u64::from_str_radix(string(row, key)?, 16)
            .map(f64::from_bits)
            .map_err(|error| format!("{key}: {error}"))
    };
    let rows = request["rows"].as_array().ok_or("rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let mut world = movement_world(row)?;
        let entity_id = i32_field(row, "entity_id")?;
        let name = string(row, "entity")?;
        // The client's own player, whose chunk decides whether a death ticks
        // (the scene's simulation distance is eight chunks).
        world.set_local_player_chunk(
            if row.get("player_x").is_some() {
                bits64(row, "player_x")?
            } else {
                6.5
            },
            0.5,
        );
        world.set_server_simulation_distance(8);
        world.set_minecart_improvements(row.get("minecart_improvements").is_some());
        if !world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(0, u64::from(entity_id as u32)),
            type_id: voxel_data::entity_type::id(name).ok_or("unknown entity type")?,
            x: bits64(row, "x")?,
            y: bits64(row, "y")?,
            z: bits64(row, "z")?,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }) {
            return Err("entity was rejected".into());
        }
        // Another entity that stays where it is (what a guardian's beam aims at).
        if let Some(second) = row.get("second") {
            let second_id = i32_field(second, "entity_id")?;
            if !world.apply_entity(EntityInput {
                entity_id: second_id,
                uuid: Uuid::new(0, u64::from(second_id as u32)),
                type_id: voxel_data::entity_type::id(string(second, "entity")?)
                    .ok_or("unknown entity type")?,
                x: bits64(second, "x")?,
                y: bits64(second, "y")?,
                z: bits64(second, "z")?,
                movement: (0.0, 0.0, 0.0),
                rotation: (0.0, 0.0, 0.0),
                data: 0,
            }) {
                return Err("second entity was rejected".into());
            }
        }
        let mut ticks = Vec::new();
        for tick in 0..i32_field(row, "tick_count")? {
            for event in row["events"].as_array().ok_or("events must be an array")? {
                if i32_field(event, "tick")? != tick {
                    continue;
                }
                match string(event, "type")? {
                    "damage" => {
                        world.entities_mut().apply_damage_event(entity_id);
                    }
                    "hurt" => {
                        world.entities_mut().apply_hurt_animation(entity_id);
                    }
                    "death" => {
                        world.entities_mut().apply_death_event(entity_id);
                    }
                    "move" => {
                        world.apply_entity_move(voxel_world::EntityMoveInput {
                            entity_id,
                            delta_x: i32_field(event, "xa")? as i16,
                            delta_y: i32_field(event, "ya")? as i16,
                            delta_z: i32_field(event, "za")? as i16,
                            has_position: true,
                            y_rot: None,
                            x_rot: None,
                        });
                        world.apply_entity_ground_state(
                            entity_id,
                            event["on_ground"].as_bool().unwrap_or(true),
                        );
                    }
                    "creeper_data" => {
                        // `SynchedEntityData` as the packet carries it: id,
                        // serializer, value.
                        let mut data = Vec::new();
                        if let Some(direction) = event["swell_dir"].as_i64() {
                            data.extend([voxel_world::CREEPER_SWELL_DIR_DATA_ID, 1]);
                            voxel_core::wire::write_varint(direction as i32, &mut data);
                        }
                        if let Some(ignited) = event["ignited"].as_bool() {
                            data.extend([
                                voxel_world::CREEPER_IGNITED_DATA_ID,
                                8,
                                u8::from(ignited),
                            ]);
                        }
                        data.push(255);
                        world.apply_entity_metadata(entity_id, &data);
                    }
                    "animate" => {
                        world
                            .entities_mut()
                            .apply_animate(entity_id, i32_field(event, "action")? as u8);
                    }
                    "bee_data" => {
                        let mut data = Vec::new();
                        if let Some(flags) = event["flags"].as_i64() {
                            data.extend([voxel_world::BEE_FLAGS_DATA_ID, 0, flags as u8]);
                        }
                        if let Some(end) = event["anger_end"].as_i64() {
                            data.extend([voxel_world::BEE_ANGER_END_DATA_ID, 2]);
                            voxel_core::wire::write_varlong(end, &mut data);
                        }
                        data.push(255);
                        world.apply_entity_metadata(entity_id, &data);
                    }
                    "sync" => {
                        world.apply_entity_position_sync(
                            entity_id,
                            (
                                bits64(event, "x")?,
                                bits64(event, "y")?,
                                bits64(event, "z")?,
                            ),
                            (0.0, 0.0),
                        );
                        world.apply_entity_ground_state(
                            entity_id,
                            event["on_ground"].as_bool().unwrap_or(false),
                        );
                    }
                    "wolf_data" => {
                        let mut data = Vec::new();
                        if let Some(flags) = event["flags"].as_i64() {
                            data.extend([voxel_world::TAMABLE_FLAGS_DATA_ID, 0, flags as u8]);
                        }
                        if let Some(interested) = event["interested"].as_bool() {
                            data.extend([
                                voxel_world::WOLF_INTERESTED_DATA_ID,
                                8,
                                u8::from(interested),
                            ]);
                        }
                        if let Some(dye) = event["collar"].as_i64() {
                            data.extend([voxel_world::WOLF_COLLAR_DATA_ID, 1]);
                            voxel_core::wire::write_varint(dye as i32, &mut data);
                        }
                        if let Some(end) = event["anger_end"].as_i64() {
                            data.extend([voxel_world::WOLF_ANGER_END_DATA_ID, 2]);
                            voxel_core::wire::write_varlong(end, &mut data);
                        }
                        if let Some(health) = event["health"].as_str() {
                            let health =
                                u32::from_str_radix(health, 16).map_err(|e| e.to_string())?;
                            // The health data is float 3 of the living entity's.
                            data.extend([9, 3]);
                            data.extend(health.to_be_bytes());
                        }
                        data.push(255);
                        world.apply_entity_metadata(entity_id, &data);
                    }
                    "weather" => {
                        let level = |key: &str| -> Result<f32, String> {
                            u32::from_str_radix(string(event, key)?, 16)
                                .map(f32::from_bits)
                                .map_err(|error| format!("{key}: {error}"))
                        };
                        world.set_weather(level("rain")?, level("thunder")?);
                    }
                    "equipment" => {
                        let stack = event["item"].as_str().map(|name| {
                            let id = voxel_data::item::registry::index(name).unwrap_or(0);
                            voxel_world::ClientItemStackState::new(1, id as i32)
                        });
                        world.entities_mut().apply_equipment(
                            entity_id,
                            voxel_world::EquipmentSlot::MainHand,
                            stack,
                        );
                    }
                    "mob_flags" => {
                        let flags = event["flags"].as_i64().ok_or("flags")? as u8;
                        world.apply_entity_metadata(entity_id, &[15, 0, flags, 255]);
                    }
                    "strider_data" => {
                        let suffocating = event["suffocating"].as_bool().ok_or("suffocating")?;
                        let data = [
                            voxel_world::STRIDER_SUFFOCATING_DATA_ID,
                            8,
                            u8::from(suffocating),
                            255,
                        ];
                        world.apply_entity_metadata(entity_id, &data);
                    }
                    "panda_data" => {
                        let mut data = Vec::new();
                        for (key, id) in [
                            ("unhappy", voxel_world::PANDA_UNHAPPY_DATA_ID),
                            ("sneeze", voxel_world::PANDA_SNEEZE_DATA_ID),
                            ("eat", voxel_world::PANDA_EAT_DATA_ID),
                        ] {
                            if let Some(value) = event[key].as_i64() {
                                data.extend([id, 1]);
                                voxel_core::wire::write_varint(value as i32, &mut data);
                            }
                        }
                        for (key, id) in [
                            ("main_gene", voxel_world::PANDA_MAIN_GENE_DATA_ID),
                            ("hidden_gene", voxel_world::PANDA_HIDDEN_GENE_DATA_ID),
                            ("flags", voxel_world::PANDA_FLAGS_DATA_ID),
                        ] {
                            if let Some(value) = event[key].as_i64() {
                                data.extend([id, 0, value as u8]);
                            }
                        }
                        data.push(255);
                        world.apply_entity_metadata(entity_id, &data);
                    }
                    "fox_data" => {
                        let mut data = Vec::new();
                        if let Some(variant) = event["variant"].as_i64() {
                            data.extend([voxel_world::FOX_VARIANT_DATA_ID, 1]);
                            voxel_core::wire::write_varint(variant as i32, &mut data);
                        }
                        if let Some(flags) = event["flags"].as_i64() {
                            data.extend([voxel_world::FOX_FLAGS_DATA_ID, 0, flags as u8]);
                        }
                        data.push(255);
                        world.apply_entity_metadata(entity_id, &data);
                    }
                    "polar_bear_data" => {
                        let standing = event["standing"].as_bool().ok_or("standing")?;
                        let data = [
                            voxel_world::POLAR_BEAR_STANDING_DATA_ID,
                            8,
                            u8::from(standing),
                            255,
                        ];
                        world.apply_entity_metadata(entity_id, &data);
                    }
                    "cat_data" => {
                        let mut data = Vec::new();
                        if let Some(flags) = event["flags"].as_i64() {
                            data.extend([voxel_world::TAMABLE_FLAGS_DATA_ID, 0, flags as u8]);
                        }
                        for (key, id) in [
                            ("lying", voxel_world::CAT_LYING_DATA_ID),
                            ("relaxed", voxel_world::CAT_RELAX_DATA_ID),
                        ] {
                            if let Some(value) = event[key].as_bool() {
                                data.extend([id, 8, u8::from(value)]);
                            }
                        }
                        if let Some(dye) = event["collar"].as_i64() {
                            data.extend([voxel_world::CAT_COLLAR_DATA_ID, 1]);
                            voxel_core::wire::write_varint(dye as i32, &mut data);
                        }
                        if let Some(pose) = event["pose"].as_str() {
                            // `Pose` ids: standing is zero, crouching five.
                            let id = if pose == "CROUCHING" { 5 } else { 0 };
                            data.extend([6, 20, id]);
                        }
                        data.push(255);
                        world.apply_entity_metadata(entity_id, &data);
                    }
                    "horse_data" => {
                        let data = [
                            voxel_world::HORSE_FLAGS_DATA_ID,
                            0,
                            i32_field(event, "flags")? as u8,
                            255,
                        ];
                        world.apply_entity_metadata(entity_id, &data);
                    }
                    "game_time" => world.set_game_time(i64_field(event, "time")? as u64),
                    "camel_data" => {
                        let mut data = Vec::new();
                        if let Some(dash) = event["dash"].as_bool() {
                            data.extend([voxel_world::CAMEL_DASH_DATA_ID, 8, u8::from(dash)]);
                        }
                        if let Some(tick) = event["last_pose"].as_i64() {
                            data.extend([voxel_world::CAMEL_LAST_POSE_CHANGE_DATA_ID, 2]);
                            voxel_core::wire::write_varlong(tick, &mut data);
                        }
                        if let Some(pose) = event["pose"].as_str() {
                            // `Pose` ids: standing is zero, sitting ten.
                            let id = if pose == "SITTING" { 10 } else { 0 };
                            data.extend([6, 20, id]);
                        }
                        data.push(255);
                        world.apply_entity_metadata(entity_id, &data);
                    }
                    "seed" => {
                        let seed = i64_field(event, "seed")?;
                        if name == "minecraft:panda" {
                            world.entities_mut().fix_panda_seed(entity_id, seed);
                        } else {
                            world.entities_mut().fix_horse_seed(entity_id, seed);
                        }
                    }
                    "copper_golem_data" => {
                        let mut data = Vec::new();
                        if let Some(weather) = event["weather"].as_i64() {
                            data.extend([
                                voxel_world::COPPER_GOLEM_WEATHER_DATA_ID,
                                38,
                                weather as u8,
                            ]);
                        }
                        if let Some(state) = event["state"].as_i64() {
                            data.extend([voxel_world::COPPER_GOLEM_STATE_DATA_ID, 37, state as u8]);
                        }
                        data.push(255);
                        world.apply_entity_metadata(entity_id, &data);
                    }
                    "entity_motion" => {
                        world
                            .entities_mut()
                            .apply_motion(voxel_world::EntityMotionInput {
                                entity_id,
                                x: bits64(event, "x")?,
                                y: bits64(event, "y")?,
                                z: bits64(event, "z")?,
                            });
                    }
                    "passengers" => {
                        let riders: Vec<i32> = event["riders"]
                            .as_array()
                            .ok_or("riders must be an array")?
                            .iter()
                            .filter_map(|id| id.as_i64().map(|id| id as i32))
                            .collect();
                        world.apply_entity_passengers(i32_field(event, "vehicle")?, &riders);
                    }
                    "shulker_data" => {
                        let mut data = Vec::new();
                        if let Some(face) = event["face"].as_i64() {
                            data.extend([voxel_world::SHULKER_ATTACH_FACE_DATA_ID, 12, face as u8]);
                        }
                        for (key, id) in [
                            ("peek", voxel_world::SHULKER_PEEK_DATA_ID),
                            ("color", voxel_world::SHULKER_COLOR_DATA_ID),
                        ] {
                            if let Some(value) = event[key].as_i64() {
                                data.extend([id, 0, value as u8]);
                            }
                        }
                        data.push(255);
                        world.apply_entity_metadata(entity_id, &data);
                    }
                    "sulfur_fuse" => {
                        let mut data = vec![voxel_world::SULFUR_CUBE_FUSE_DATA_ID, 1];
                        voxel_core::wire::write_varint(i32_field(event, "fuse")?, &mut data);
                        data.push(255);
                        world.apply_entity_metadata(entity_id, &data);
                    }
                    "head_rotation" => {
                        let angle = i32_field(event, "angle")? as i8;
                        world.apply_entity_head_rotation(
                            entity_id,
                            voxel_core::math::unpack_degrees_i8(angle),
                        );
                    }
                    "cube_size" => {
                        let mut data = vec![voxel_world::CUBE_MOB_SIZE_DATA_ID, 1];
                        voxel_core::wire::write_varint(i32_field(event, "size")?, &mut data);
                        data.push(255);
                        world.apply_entity_metadata(entity_id, &data);
                    }
                    "glow_squid_data" => {
                        let mut data = vec![voxel_world::GLOW_SQUID_DARK_TICKS_DATA_ID, 1];
                        voxel_core::wire::write_varint(i32_field(event, "dark_ticks")?, &mut data);
                        data.push(255);
                        world.apply_entity_metadata(entity_id, &data);
                    }
                    "baby_data" => {
                        let mut data =
                            vec![16, 8, u8::from(event["baby"].as_bool().ok_or("baby")?)];
                        data.push(255);
                        world.apply_entity_metadata(entity_id, &data);
                    }
                    "axolotl_data" => {
                        let mut data = Vec::new();
                        if event.get("variant").is_some() {
                            data.extend([voxel_world::AXOLOTL_VARIANT_DATA_ID, 1]);
                            voxel_core::wire::write_varint(i32_field(event, "variant")?, &mut data);
                        }
                        if let Some(playing_dead) = event["playing_dead"].as_bool() {
                            data.extend([
                                voxel_world::AXOLOTL_PLAYING_DEAD_DATA_ID,
                                8,
                                u8::from(playing_dead),
                            ]);
                        }
                        data.push(255);
                        world.apply_entity_metadata(entity_id, &data);
                    }
                    "boat_data" => {
                        let mut data = Vec::new();
                        for (key, id) in [
                            ("hurt", voxel_world::BOAT_HURT_DATA_ID),
                            ("hurt_dir", voxel_world::BOAT_HURT_DIR_DATA_ID),
                            ("bubble_time", voxel_world::BOAT_BUBBLE_TIME_DATA_ID),
                        ] {
                            if event.get(key).is_some() {
                                data.extend([id, 1]);
                                voxel_core::wire::write_varint(i32_field(event, key)?, &mut data);
                            }
                        }
                        if let Some(damage) = event["damage"].as_f64() {
                            data.extend([voxel_world::BOAT_DAMAGE_DATA_ID, 3]);
                            data.extend((damage as f32).to_be_bytes());
                        }
                        data.push(255);
                        world.apply_entity_metadata(entity_id, &data);
                    }
                    "flier_data" => {
                        let mut data = Vec::new();
                        if event.get("parrot_variant").is_some() {
                            data.extend([voxel_world::PARROT_VARIANT_DATA_ID, 1]);
                            voxel_core::wire::write_varint(
                                i32_field(event, "parrot_variant")?,
                                &mut data,
                            );
                        }
                        if let Some(flags) = event["bat_flags"].as_i64() {
                            data.extend([voxel_world::BAT_FLAGS_DATA_ID, 0, flags as u8]);
                        }
                        if let Some(flags) = event["vex_flags"].as_i64() {
                            data.extend([voxel_world::VEX_FLAGS_DATA_ID, 0, flags as u8]);
                        }
                        if let Some(dancing) = event["allay_dancing"].as_bool() {
                            data.extend([voxel_world::ALLAY_DANCING_DATA_ID, 8, u8::from(dancing)]);
                        }
                        data.push(255);
                        world.apply_entity_metadata(entity_id, &data);
                    }
                    "guardian_random" => {
                        let tail = f32::from_bits(
                            u32::from_str_radix(
                                event["tail"].as_str().ok_or("tail must be a string")?,
                                16,
                            )
                            .map_err(|error| error.to_string())?,
                        );
                        world.entities_mut().fix_guardian_random(
                            entity_id,
                            tail,
                            i64_field(event, "seed")?,
                        );
                    }
                    "guardian_data" => {
                        let mut data = Vec::new();
                        if let Some(moving) = event["moving"].as_bool() {
                            data.extend([
                                voxel_world::GUARDIAN_MOVING_DATA_ID,
                                8,
                                u8::from(moving),
                            ]);
                        }
                        if event.get("target").is_some() {
                            data.extend([voxel_world::GUARDIAN_ATTACK_TARGET_DATA_ID, 1]);
                            voxel_core::wire::write_varint(i32_field(event, "target")?, &mut data);
                        }
                        data.push(255);
                        world.apply_entity_metadata(entity_id, &data);
                    }
                    "frog_data" => {
                        // `Pose` ids: standing is zero, long jumping six, croaking eight, using
                        // the tongue nine, sliding fifteen, shooting sixteen, inhaling seventeen.
                        let id = match event["pose"].as_str().ok_or("pose")? {
                            "LONG_JUMPING" => 6,
                            "CROAKING" => 8,
                            "USING_TONGUE" => 9,
                            "SLIDING" => 15,
                            "SHOOTING" => 16,
                            "INHALING" => 17,
                            "ROARING" => 11,
                            "SNIFFING" => 12,
                            "EMERGING" => 13,
                            "DIGGING" => 14,
                            _ => 0,
                        };
                        world.apply_entity_metadata(entity_id, &[6, 20, id, 255]);
                    }
                    "arrow_data" => {
                        // `AbstractArrow.IN_GROUND` (a boolean), `Arrow.ID_EFFECT_COLOR` (an int).
                        let mut data = Vec::new();
                        if let Some(value) = event["in_ground"].as_bool() {
                            data.extend([
                                voxel_world::objects::arrow::IN_GROUND_DATA_ID,
                                8,
                                u8::from(value),
                            ]);
                        }
                        if let Some(value) = event["color"].as_i64() {
                            data.extend([voxel_world::objects::arrow::EFFECT_COLOR_DATA_ID, 1]);
                            voxel_core::wire::write_varint(value as i32, &mut data);
                        }
                        data.push(255);
                        world.apply_entity_metadata(entity_id, &data);
                    }
                    "minecart_data" => {
                        // `VehicleEntity`'s hit and `AbstractMinecart`'s block, its offset, a furnace's fuel.
                        let mut data = Vec::new();
                        if let Some(value) = event["hurt"].as_i64() {
                            data.extend([voxel_world::objects::minecart::HURT_DATA_ID, 1]);
                            voxel_core::wire::write_varint(value as i32, &mut data);
                        }
                        if let Some(value) = event["hurt_dir"].as_i64() {
                            data.extend([voxel_world::objects::minecart::HURT_DIR_DATA_ID, 1]);
                            voxel_core::wire::write_varint(value as i32, &mut data);
                        }
                        if let Some(value) = event["damage"].as_f64() {
                            data.extend([voxel_world::objects::minecart::DAMAGE_DATA_ID, 3]);
                            data.extend((value as f32).to_be_bytes());
                        }
                        if let Some(value) = event["display"].as_i64() {
                            data.extend([
                                voxel_world::objects::minecart::CUSTOM_DISPLAY_BLOCK_DATA_ID,
                                15,
                            ]);
                            voxel_core::wire::write_varint(value as i32, &mut data);
                        }
                        if event["no_display"].as_bool() == Some(true) {
                            data.extend([
                                voxel_world::objects::minecart::CUSTOM_DISPLAY_BLOCK_DATA_ID,
                                15,
                                0,
                            ]);
                        }
                        if let Some(value) = event["offset"].as_i64() {
                            data.extend([
                                voxel_world::objects::minecart::DISPLAY_OFFSET_DATA_ID,
                                1,
                            ]);
                            voxel_core::wire::write_varint(value as i32, &mut data);
                        }
                        if let Some(value) = event["fuel"].as_bool() {
                            data.extend([
                                voxel_world::objects::minecart::FUEL_DATA_ID,
                                8,
                                u8::from(value),
                            ]);
                        }
                        data.push(255);
                        world.apply_entity_metadata(entity_id, &data);
                    }
                    "crystal_data" => {
                        // `EndCrystal`'s data: the beam's block (optional) and whether the base shows.
                        let mut data = Vec::new();
                        if let Some(target) = event["target"].as_array() {
                            let coordinate =
                                |index: usize| target[index].as_i64().unwrap_or(0) as i32;
                            data.extend([
                                voxel_world::objects::end_crystal::BEAM_TARGET_DATA_ID,
                                11,
                                1,
                            ]);
                            voxel_core::binary::write_blockpos(
                                voxel_core::binary::BlockPos {
                                    x: coordinate(0),
                                    y: coordinate(1),
                                    z: coordinate(2),
                                },
                                &mut data,
                            );
                        }
                        if event["no_target"].as_bool() == Some(true) {
                            data.extend([
                                voxel_world::objects::end_crystal::BEAM_TARGET_DATA_ID,
                                11,
                                0,
                            ]);
                        }
                        if let Some(bottom) = event["bottom"].as_bool() {
                            data.extend([
                                voxel_world::objects::end_crystal::SHOW_BOTTOM_DATA_ID,
                                8,
                                u8::from(bottom),
                            ]);
                        }
                        data.push(255);
                        world.apply_entity_metadata(entity_id, &data);
                    }
                    "dragon_data" => {
                        // `EnderDragon.DATA_PHASE`, an int.
                        let mut data = vec![voxel_world::mobs::ender_dragon::PHASE_DATA_ID, 1];
                        voxel_core::wire::write_varint(i32_field(event, "phase")?, &mut data);
                        data.push(255);
                        world.apply_entity_metadata(entity_id, &data);
                    }
                    "display_data" => {
                        // A display's data in the wire's own form (data id, serializer, value).
                        let float = |value: &Value| -> Result<f32, String> {
                            u32::from_str_radix(value.as_str().ok_or("a float is a string")?, 16)
                                .map(f32::from_bits)
                                .map_err(|error| error.to_string())
                        };
                        let mut data = Vec::new();
                        for (key, id) in [
                            (
                                "start_delta",
                                voxel_world::objects::display::START_DELTA_DATA_ID,
                            ),
                            (
                                "duration",
                                voxel_world::objects::display::INTERPOLATION_DURATION_DATA_ID,
                            ),
                            (
                                "pos_rot",
                                voxel_world::objects::display::POS_ROT_DURATION_DATA_ID,
                            ),
                            (
                                "brightness",
                                voxel_world::objects::display::BRIGHTNESS_DATA_ID,
                            ),
                            ("glow", voxel_world::objects::display::GLOW_COLOR_DATA_ID),
                        ] {
                            if event[key].is_i64() {
                                data.extend([id, 1]);
                                voxel_core::wire::write_varint(i32_field(event, key)?, &mut data);
                            }
                        }
                        for (key, id) in [
                            (
                                "view_range",
                                voxel_world::objects::display::VIEW_RANGE_DATA_ID,
                            ),
                            (
                                "shadow_radius",
                                voxel_world::objects::display::SHADOW_RADIUS_DATA_ID,
                            ),
                            (
                                "shadow_strength",
                                voxel_world::objects::display::SHADOW_STRENGTH_DATA_ID,
                            ),
                            ("width", voxel_world::objects::display::WIDTH_DATA_ID),
                            ("height", voxel_world::objects::display::HEIGHT_DATA_ID),
                        ] {
                            if event[key].is_string() {
                                data.extend([id, 3]);
                                data.extend(float(&event[key])?.to_be_bytes());
                            }
                        }
                        if event["billboard"].is_i64() {
                            data.extend([voxel_world::objects::display::BILLBOARD_DATA_ID, 0]);
                            data.push(i32_field(event, "billboard")? as u8);
                        }
                        for (key, id, serializer, count) in [
                            (
                                "translation",
                                voxel_world::objects::display::TRANSLATION_DATA_ID,
                                39,
                                3,
                            ),
                            ("scale", voxel_world::objects::display::SCALE_DATA_ID, 39, 3),
                            (
                                "left_rotation",
                                voxel_world::objects::display::LEFT_ROTATION_DATA_ID,
                                40,
                                4,
                            ),
                            (
                                "right_rotation",
                                voxel_world::objects::display::RIGHT_ROTATION_DATA_ID,
                                40,
                                4,
                            ),
                        ] {
                            if let Some(parts) = event[key].as_array() {
                                data.extend([id, serializer]);
                                for part in parts.iter().take(count) {
                                    data.extend(float(part)?.to_be_bytes());
                                }
                            }
                        }
                        if event["block_state"].is_i64() {
                            data.extend([voxel_world::objects::display::BLOCK_STATE_DATA_ID, 14]);
                            voxel_core::wire::write_varint(
                                i32_field(event, "block_state")?,
                                &mut data,
                            );
                        }
                        if let Some(text) = event["text"].as_str() {
                            // A component that is only a string: a string tag.
                            data.extend([voxel_world::objects::display::TEXT_DATA_ID, 5, 8]);
                            data.extend((text.len() as u16).to_be_bytes());
                            data.extend(text.as_bytes());
                        }
                        for (key, id) in [
                            (
                                "line_width",
                                voxel_world::objects::display::LINE_WIDTH_DATA_ID,
                            ),
                            (
                                "background",
                                voxel_world::objects::display::BACKGROUND_DATA_ID,
                            ),
                        ] {
                            if event[key].is_i64() {
                                data.extend([id, 1]);
                                voxel_core::wire::write_varint(i32_field(event, key)?, &mut data);
                            }
                        }
                        for (key, id) in [
                            (
                                "text_opacity",
                                voxel_world::objects::display::TEXT_OPACITY_DATA_ID,
                            ),
                            (
                                "style_flags",
                                voxel_world::objects::display::STYLE_FLAGS_DATA_ID,
                            ),
                        ] {
                            if event[key].is_i64() {
                                data.extend([id, 0]);
                                data.push(i32_field(event, key)? as u8);
                            }
                        }
                        if event["item_display"].is_i64() {
                            data.extend([voxel_world::objects::display::ITEM_DISPLAY_DATA_ID, 0]);
                            data.push(i32_field(event, "item_display")? as u8);
                        }
                        data.push(255);
                        world.apply_entity_metadata(entity_id, &data);
                        if let Some(name) = event["item"].as_str() {
                            let id = voxel_data::item::registry::index(name).unwrap_or(0);
                            world.entities_mut().apply_item_stack(
                                entity_id,
                                Some(voxel_world::ClientItemStackState::new(1, id as i32)),
                            );
                        }
                    }
                    "minecart_steps" => {
                        // The steps along the track, with the turns the wire packs in a byte of 256ths.
                        let degrees = |step: &Value, key: &str| -> Result<f32, String> {
                            Ok(i32::from(i32_field(step, key)? as i8) as f32 * 360.0 / 256.0)
                        };
                        let mut steps = Vec::new();
                        for step in event["steps"].as_array().ok_or("steps")? {
                            steps.push(voxel_world::objects::minecart_track::MinecartStep {
                                position: (
                                    bits64(step, "x")?,
                                    bits64(step, "y")?,
                                    bits64(step, "z")?,
                                ),
                                movement: (
                                    bits64(step, "mx")?,
                                    bits64(step, "my")?,
                                    bits64(step, "mz")?,
                                ),
                                y_rot: degrees(step, "y_rot")?,
                                x_rot: degrees(step, "x_rot")?,
                                weight: f32::from_bits(
                                    u32::from_str_radix(string(step, "weight")?, 16)
                                        .map_err(|error| error.to_string())?,
                                ),
                            });
                        }
                        world.apply_minecart_steps(entity_id, &steps);
                    }
                    "move_rot" => {
                        // A relative move with the turn the wire packs in a byte of 256ths.
                        let degrees = |key: &str| -> Result<f32, String> {
                            Ok(i32::from(i32_field(event, key)? as i8) as f32 * 360.0 / 256.0)
                        };
                        world.apply_entity_move(voxel_world::EntityMoveInput {
                            entity_id,
                            delta_x: i32_field(event, "xa")? as i16,
                            delta_y: i32_field(event, "ya")? as i16,
                            delta_z: i32_field(event, "za")? as i16,
                            has_position: true,
                            y_rot: Some(degrees("y_rot")?),
                            x_rot: Some(degrees("x_rot")?),
                        });
                    }
                    "wither_data" => {
                        // `WitherBoss`'s int data: the side heads' targets and the invulnerable ticks.
                        let mut data = Vec::new();
                        for (key, id) in [
                            ("target_b", voxel_world::mobs::wither::TARGET_B_DATA_ID),
                            ("target_c", voxel_world::mobs::wither::TARGET_C_DATA_ID),
                            (
                                "invulnerable",
                                voxel_world::mobs::wither::INVULNERABLE_DATA_ID,
                            ),
                        ] {
                            if event[key].is_i64() {
                                data.extend([id, 1]);
                                voxel_core::wire::write_varint(i32_field(event, key)?, &mut data);
                            }
                        }
                        data.push(255);
                        world.apply_entity_metadata(entity_id, &data);
                    }
                    "sniffer_data" => {
                        // `Sniffer.DATA_STATE`: serializer 35, then the state's id.
                        let mut data = vec![voxel_world::mobs::sniffer::STATE_DATA_ID, 35];
                        voxel_core::wire::write_varint(i32_field(event, "state")?, &mut data);
                        data.push(255);
                        world.apply_entity_metadata(entity_id, &data);
                    }
                    "armadillo_data" => {
                        // `Armadillo.ARMADILLO_STATE`: serializer 36, then the state's id.
                        let mut data = vec![voxel_world::mobs::armadillo::STATE_DATA_ID, 36];
                        voxel_core::wire::write_varint(i32_field(event, "state")?, &mut data);
                        data.push(255);
                        world.apply_entity_metadata(entity_id, &data);
                    }
                    "creaking_data" => {
                        // The booleans `Creaking` defines, each when the event names it.
                        let mut data = Vec::new();
                        for (key, id) in [
                            ("can_move", voxel_world::mobs::creaking::CAN_MOVE_DATA_ID),
                            ("active", voxel_world::mobs::creaking::IS_ACTIVE_DATA_ID),
                            (
                                "tearing_down",
                                voxel_world::mobs::creaking::IS_TEARING_DOWN_DATA_ID,
                            ),
                        ] {
                            if let Some(value) = event[key].as_bool() {
                                data.extend([id, 8, u8::from(value)]);
                            }
                        }
                        if event["home"].as_bool() == Some(true) {
                            // An optional block position: present, then the packed position.
                            data.extend([voxel_world::mobs::creaking::HOME_POS_DATA_ID, 11, 1]);
                            data.extend(0_i64.to_be_bytes());
                        }
                        data.push(255);
                        world.apply_entity_metadata(entity_id, &data);
                    }
                    "warden_data" => {
                        // `Warden.CLIENT_ANGER_LEVEL`, an int.
                        let mut data = vec![voxel_world::mobs::warden::CLIENT_ANGER_DATA_ID, 1];
                        voxel_core::wire::write_varint(i32_field(event, "anger")?, &mut data);
                        data.push(255);
                        world.apply_entity_metadata(entity_id, &data);
                    }
                    "turtle_data" => {
                        let mut data = Vec::new();
                        if let Some(has_egg) = event["has_egg"].as_bool() {
                            data.extend([
                                voxel_world::TURTLE_HAS_EGG_DATA_ID,
                                8,
                                u8::from(has_egg),
                            ]);
                        }
                        if let Some(laying) = event["laying_egg"].as_bool() {
                            data.extend([
                                voxel_world::TURTLE_LAYING_EGG_DATA_ID,
                                8,
                                u8::from(laying),
                            ]);
                        }
                        data.push(255);
                        world.apply_entity_metadata(entity_id, &data);
                    }
                    "fish_data" => {
                        let data_id = match string(event, "fish")? {
                            "salmon" => voxel_world::SALMON_TYPE_DATA_ID,
                            _ => voxel_world::PUFFERFISH_PUFF_STATE_DATA_ID,
                        };
                        let mut data = vec![data_id, 1];
                        voxel_core::wire::write_varint(i32_field(event, "value")?, &mut data);
                        data.push(255);
                        world.apply_entity_metadata(entity_id, &data);
                    }
                    "squid_speed" => {
                        let speed = u32::from_str_radix(string(event, "speed")?, 16)
                            .map(f32::from_bits)
                            .map_err(|error| error.to_string())?;
                        world
                            .entities_mut()
                            .fix_squid_tentacle_speed(entity_id, speed);
                    }
                    "copper_golem_idle" => {
                        world
                            .entities_mut()
                            .fix_copper_golem_seed(entity_id, i64_field(event, "seed")?);
                    }
                    "rabbit_idle" => {
                        world.entities_mut().fix_rabbit_idle(
                            entity_id,
                            i32_field(event, "timeout")?,
                            i64_field(event, "seed")?,
                        );
                    }
                    "attributes" => {
                        let mut changed = Vec::new();
                        for item in event["attributes"].as_array().ok_or("attributes")? {
                            let (id, _) = voxel_data::entity_type::attribute(
                                item["name"].as_str().ok_or("name")?,
                            )
                            .ok_or("no such attribute")?;
                            let modifiers = item["modifiers"]
                                .as_array()
                                .ok_or("modifiers")?
                                .iter()
                                .map(|modifier| {
                                    Ok(voxel_world::ClientAttributeModifierState::new(
                                        modifier["id"].as_str().ok_or("id")?.to_owned(),
                                        f64_field(modifier, "amount")?,
                                        i32_field(modifier, "operation")? as u8,
                                    ))
                                })
                                .collect::<Result<Vec<_>, String>>()?;
                            changed.push(voxel_world::ClientAttributeState::new(
                                id as i32,
                                f64_field(item, "base")?,
                                modifiers,
                            ));
                        }
                        world.entities_mut().apply_attributes(entity_id, &changed);
                    }
                    "entity_event" => {
                        world.apply_entity_event(entity_id, i32_field(event, "event")? as i8);
                    }
                    other => return Err(format!("unknown event {other}")),
                }
            }
            world.tick_entities();
            let entity = world.entities().at(entity_id).ok_or("entity vanished")?;
            let hex = |value: f32| format!("{:08x}", value.to_bits());
            let mut described = json!({
                "hurt_time": entity.hurt_time(),
                "death_time": entity.death_time(),
                "dead": entity.is_dead_or_dying(),
                "walk_speed": hex(entity.walk_animation().speed()),
                "walk_position": hex(entity.walk_animation().position()),
            });
            for extra in row["extras"].as_array().into_iter().flatten() {
                match extra.as_str().ok_or("extras must be strings")? {
                    "swing" => {
                        let swing = entity.swing_state();
                        described["swinging"] = json!(swing.is_swinging());
                        described["swing_time"] = json!(swing.swing_time());
                        described["off_hand"] = json!(swing.is_off_hand());
                        for (key, partial) in [("0.0", 0.0_f32), ("0.5", 0.5), ("1.0", 1.0)] {
                            described[format!("attack_anim_{key}")] =
                                json!(hex(swing.attack_anim(partial)));
                        }
                    }
                    "sheep" => {
                        for (key, partial) in [("0.0", 0.0_f32), ("0.5", 0.5)] {
                            let (position, angle) = entity.sheep_head_eat_scales(partial, 0.0);
                            described[format!("eat_position_{key}")] = json!(hex(position));
                            described[format!("eat_angle_{key}")] = json!(hex(angle));
                        }
                    }
                    "camel" => {
                        if let Some(camel) = entity.horse().map(|horse| &horse.camel) {
                            let tick = entity.tick_count() as f32;
                            for (name, state) in [
                                ("sit", camel.sit),
                                ("sit_pose", camel.sit_pose),
                                ("sit_up", camel.sit_up),
                                ("idle", camel.idle),
                                ("dash", camel.dash),
                            ] {
                                described[format!("{name}_started")] = json!(state.is_started());
                                described[format!("{name}_millis")] =
                                    json!(if state.is_started() {
                                        state.time_in_millis(tick)
                                    } else {
                                        0
                                    });
                            }
                            described["dashing"] = json!(camel.is_dashing());
                        }
                    }
                    "horse" => {
                        if let Some(horse) = entity.horse() {
                            for (key, partial) in [("0.0", 0.0_f32), ("0.5", 0.5)] {
                                described[format!("eat_{key}")] =
                                    json!(hex(horse.eat_animation(partial)));
                                described[format!("stand_{key}")] =
                                    json!(hex(horse.stand_animation(partial)));
                                described[format!("mouth_{key}")] =
                                    json!(hex(horse.mouth_animation(partial)));
                            }
                            described["tail_counter"] = json!(horse.tail_counter());
                        }
                    }
                    "bee" => {
                        described["roll_0.0"] = json!(hex(entity.bee_roll_amount(0.0)));
                        described["roll_0.5"] = json!(hex(entity.bee_roll_amount(0.5)));
                        described["stung"] = json!(entity.bee_has_stung());
                        described["nectar"] = json!(entity.bee_has_nectar());
                        described["angry"] = json!(entity.is_angry(world.game_time()));
                    }
                    "wolf" => {
                        let game_time = world.game_time();
                        described["angry"] = json!(entity.is_angry(game_time));
                        described["sitting"] = json!(entity.is_sitting());
                        described["tame"] = json!(entity.is_tame());
                        described["tail_angle"] = json!(hex(entity.wolf_tail_angle(game_time)));
                        for (key, partial) in [("0.0", 0.0_f32), ("0.5", 0.5)] {
                            let wolf = entity.wolf();
                            described[format!("head_roll_{key}")] =
                                json!(hex(wolf.head_roll_angle(partial)));
                            described[format!("shake_{key}")] =
                                json!(hex(wolf.shake_anim(partial)));
                        }
                    }
                    "strider" => {
                        described["suffocating"] = json!(entity.is_suffocating());
                    }
                    "ravager" => {
                        let ravager = entity.ravager();
                        described["attack_tick"] = json!(ravager.attack_tick());
                        described["stunned_tick"] = json!(ravager.stunned_tick());
                        described["roar_tick"] = json!(ravager.roar_tick());
                    }
                    "panda" => {
                        let panda = entity.panda();
                        let thundering = world.is_thundering();
                        described["variant"] = json!(panda.variant());
                        described["unhappy_counter"] = json!(panda.unhappy_counter());
                        described["sneeze_counter"] = json!(panda.sneeze_counter());
                        described["sneezing"] = json!(panda.is_sneezing());
                        described["eating"] = json!(panda.is_eating());
                        described["sitting"] = json!(panda.is_sitting());
                        described["rolling"] = json!(panda.is_rolling());
                        described["on_back"] = json!(panda.is_on_back());
                        described["roll_counter"] = json!(panda.roll_counter());
                        described["scared"] = json!(panda.is_scared(thundering));
                        described["x_rot"] = json!(hex(entity.rotation().1));
                        for (key, partial) in [("0.0", 0.0_f32), ("0.5", 0.5)] {
                            described[format!("sit_{key}")] = json!(hex(panda.sit_amount(partial)));
                            described[format!("back_{key}")] =
                                json!(hex(panda.lie_on_back_amount(partial)));
                            described[format!("roll_{key}")] =
                                json!(hex(panda.roll_amount(partial)));
                        }
                    }
                    "fox" => {
                        let fox = entity.fox();
                        described["variant"] = json!(fox.variant());
                        described["sitting"] = json!(fox.is_sitting());
                        described["crouching"] = json!(fox.is_crouching());
                        described["sleeping"] = json!(fox.is_sleeping());
                        described["faceplanted"] = json!(fox.is_faceplanted());
                        described["pouncing"] = json!(fox.is_pouncing());
                        for (key, partial) in [("0.0", 0.0_f32), ("0.5", 0.5)] {
                            described[format!("head_roll_{key}")] =
                                json!(hex(fox.head_roll_angle(partial)));
                            described[format!("crouch_{key}")] =
                                json!(hex(fox.crouch_amount(partial)));
                        }
                    }
                    "hoglin" => {
                        described["attack_ticks"] = json!(entity.hoglin_attack_ticks());
                    }
                    "polar_bear" => {
                        let bear = entity.polar_bear();
                        described["standing"] = json!(bear.is_standing());
                        for (key, partial) in [("0.0", 0.0_f32), ("0.5", 0.5)] {
                            described[format!("stand_{key}")] =
                                json!(hex(bear.standing_animation_scale(partial)));
                        }
                    }
                    "cat" => {
                        described["sitting"] = json!(entity.is_sitting());
                        described["tame"] = json!(entity.is_tame());
                        described["crouching"] = json!(entity.is_crouching());
                        let cat = entity.cat();
                        described["on_player"] = json!(cat.is_lying_on_top_of_sleeping_player());
                        for (key, partial) in [("0.0", 0.0_f32), ("0.5", 0.5)] {
                            described[format!("lie_{key}")] =
                                json!(hex(cat.lie_down_amount(partial)));
                            described[format!("lie_tail_{key}")] =
                                json!(hex(cat.lie_down_amount_tail(partial)));
                            described[format!("relax_{key}")] =
                                json!(hex(cat.relax_state_one_amount(partial)));
                        }
                    }
                    "rabbit" => {
                        if let Some(rabbit) = entity.rabbit() {
                            let tick = entity.tick_count() as f32;
                            described["hop_started"] = json!(rabbit.hop.is_started());
                            described["idle_started"] = json!(rabbit.idle_head_tilt.is_started());
                            described["hop_millis"] = json!(rabbit.hop.time_in_millis(tick));
                            described["idle_millis"] =
                                json!(rabbit.idle_head_tilt.time_in_millis(tick));
                            described["jump_completion"] = json!(hex(rabbit.jump_completion(0.5)));
                        }
                    }
                    "turtle" => {
                        described["in_water"] = json!(entity.is_in_water());
                        described["on_ground"] = json!(entity.on_ground());
                        described["has_egg"] = json!(entity.turtle_has_egg());
                        described["laying_egg"] = json!(entity.turtle_laying_egg());
                        described["is_on_land"] =
                            json!(!entity.is_in_water() && entity.on_ground());
                        let area = entity.bounding_box();
                        described["max_y"] = json!(format!("{:016x}", area.max_y.to_bits()));
                        described["max_x"] = json!(format!("{:016x}", area.max_x.to_bits()));
                    }
                    "boat" => {
                        let state = entity.boat().copied().unwrap_or_default();
                        described["under_water"] = json!(state.is_under_water);
                        described["hurt_time"] = json!(state.hurt_time);
                        described["hurt_dir"] = json!(state.hurt_dir);
                        described["damage"] = json!(hex(state.damage));
                        for (key, partial) in [("0.0", 0.0_f32), ("0.5", 0.5), ("1.0", 1.0)] {
                            described[format!("bubble_angle_{key}")] =
                                json!(hex(state.bubble_angle(partial)));
                            for side in 0..2 {
                                described[format!("rowing_{side}_{key}")] = json!(hex(
                                    state.rowing_time(side, entity.boat_paddle_rows(side), partial)
                                ));
                            }
                        }
                    }
                    "bat" => {
                        described["resting"] = json!(entity.is_bat_resting());
                        let (_, y, _) = entity.position();
                        described["y"] = json!(format!("{:016x}", y.to_bits()));
                        let (_, motion_y, _) = entity.motion();
                        described["motion_y"] = json!(format!("{:016x}", motion_y.to_bits()));
                        let state = entity.bat().copied().unwrap_or_default();
                        let age = entity.tick_count() as f32;
                        for (name, animation) in [("fly", state.fly), ("rest", state.rest)] {
                            described[format!("{name}_started")] = json!(animation.is_started());
                            described[format!("{name}_millis")] =
                                json!(if animation.is_started() {
                                    animation.time_in_millis(age)
                                } else {
                                    0
                                });
                        }
                    }
                    "breeze" => {
                        let state = entity.breeze().copied().unwrap_or_default();
                        let age = entity.tick_count() as f32;
                        for (name, animation) in [
                            ("idle", state.idle),
                            ("shoot", state.shoot),
                            ("slide", state.slide),
                            ("slide_back", state.slide_back),
                            ("inhale", state.inhale),
                            ("long_jump", state.long_jump),
                        ] {
                            described[format!("{name}_started")] = json!(animation.is_started());
                            described[format!("{name}_millis")] =
                                json!(if animation.is_started() {
                                    animation.time_in_millis(age)
                                } else {
                                    0
                                });
                        }
                    }
                    "arrow" => {
                        let state = entity.arrow().copied().unwrap_or_default();
                        described["shake_time"] = json!(state.shake_time());
                        described["in_ground"] = json!(state.is_in_ground());
                        described["tipped"] = json!(state.is_tipped());
                    }
                    "minecart" => {
                        let state = entity.minecart().copied().unwrap_or_default();
                        described["hurt_time"] = json!(state.hurt_time);
                        described["hurt_dir"] = json!(state.hurt_dir);
                        described["damage"] = json!(hex(state.damage));
                        described["display_block"] =
                            json!(entity.minecart_display_block().map_or(0, i64::from));
                        described["display_offset"] = json!(entity.minecart_display_offset());
                        described["fuse"] = json!(state.fuse());
                        let position = entity.position();
                        let (y_rot, x_rot, _) = entity.rotation();
                        for (key, value) in
                            [("x", position.0), ("y", position.1), ("z", position.2)]
                        {
                            described[key] = json!(format!("{:016x}", value.to_bits()));
                        }
                        described["y_rot"] = json!(hex(y_rot));
                        described["x_rot"] = json!(hex(x_rot));
                        if let Some(track) = entity.minecart_track() {
                            let motion = entity.motion();
                            for (key, value) in [("x", motion.0), ("y", motion.1), ("z", motion.2)]
                            {
                                described[format!("motion_{key}")] =
                                    json!(format!("{:016x}", value.to_bits()));
                            }
                            described["playing"] = json!(track.is_playing());
                            let mut samples = Vec::new();
                            // Unseen ticks are not asked (a place asked for is kept for the next ask).
                            if track.is_playing()
                                && tick
                                    >= row
                                        .get("sample_from")
                                        .map_or(Ok(0), |_| i32_field(row, "sample_from"))?
                            {
                                for partial in [0.0_f32, 0.5] {
                                    let place = track.place(partial).ok_or("playing")?;
                                    samples.push(json!(format!(
                                        "{:016x}{:016x}{:016x}{}{}",
                                        place.position.0.to_bits(),
                                        place.position.1.to_bits(),
                                        place.position.2.to_bits(),
                                        hex(place.x_rot),
                                        hex(place.y_rot)
                                    )));
                                }
                            }
                            described["lerp"] = json!(samples);
                        }
                        let mut probes = Vec::new();
                        if tick + 1 == i32_field(row, "tick_count")? {
                            for probe in row["rail_probes"].as_array().into_iter().flatten() {
                                let parts = probe.as_array().ok_or("a probe is an array")?;
                                let double = |index: usize| -> Result<f64, String> {
                                    u64::from_str_radix(
                                        parts[index].as_str().ok_or("a double")?,
                                        16,
                                    )
                                    .map(f64::from_bits)
                                    .map_err(|error| error.to_string())
                                };
                                let at = (double(0)?, double(1)?, double(2)?);
                                let text = |place: Option<(f64, f64, f64)>| {
                                    place.map_or_else(
                                        || "none".to_string(),
                                        |place| {
                                            format!(
                                                "{:016x}{:016x}{:016x}",
                                                place.0.to_bits(),
                                                place.1.to_bits(),
                                                place.2.to_bits()
                                            )
                                        },
                                    )
                                };
                                let half = f64::from(0.3_f32);
                                probes.push(json!([
                                    text(world.minecart_rail_pos(at)),
                                    text(world.minecart_rail_pos_offset(at, half)),
                                    text(world.minecart_rail_pos_offset(at, -half)),
                                ]));
                            }
                        }
                        described["rail"] = json!(probes);
                    }
                    "display" => {
                        let (x, y, z) = entity.position();
                        let (y_rot, x_rot, _) = entity.rotation();
                        for (key, value) in [("x", x), ("y", y), ("z", z)] {
                            described[key] = json!(format!("{:016x}", value.to_bits()));
                        }
                        described["y_rot"] = json!(hex(y_rot));
                        described["x_rot"] = json!(hex(x_rot));
                        let display = entity.display().ok_or("a display")?;
                        if let Some(block) = display.block_state() {
                            described["block_state"] = json!(block);
                        }
                        if let Some(context) = display.item_context() {
                            described["item_context"] = json!(context);
                            described["item_empty"] = json!(
                                entity
                                    .item_stack()
                                    .is_none_or(|stack| stack.item_count() <= 0)
                            );
                        }
                        let state = display.render_state();
                        described["has_state"] = json!(state.is_some());
                        let mut samples = Vec::new();
                        if let Some(state) = state {
                            for partial in [0.0_f32, 0.5] {
                                let progress =
                                    display.interpolation_progress(entity.tick_count(), partial);
                                let matrix = voxel_render::objects::display::transformation_matrix(
                                    &state.transformation.get(progress),
                                );
                                let mut sample = hex(progress);
                                for value in matrix {
                                    sample.push_str(&hex(value));
                                }
                                sample.push_str(&hex(state.shadow_radius.get(progress)));
                                sample.push_str(&hex(state.shadow_strength.get(progress)));
                                samples.push(json!(sample));
                            }
                            described["billboard"] = json!(state.billboard as i32);
                            described["brightness"] = json!(state.brightness_override);
                            described["glow"] = json!(state.glow_color_override);
                        }
                        described["samples"] = json!(samples);
                        if let Some((text, _)) = display.text_render_state() {
                            let mut data = text.text.as_slice();
                            let tag = voxel_core::nbt::read_any_nbt(&mut data)
                                .ok()
                                .flatten()
                                .and_then(|tag| voxel_game::text::Component::from_nbt(&tag))
                                .ok_or("a component")?;
                            let words: String = voxel_game::text::Lang::default()
                                .runs(&tag, &voxel_game::text::Style::default())
                                .into_iter()
                                .map(|run| run.text)
                                .collect();
                            described["text"] = json!(words);
                            described["line_width"] = json!(text.line_width);
                            described["style_flags"] = json!(i32::from(text.flags));
                            let mut text_samples = Vec::new();
                            for partial in [0.0_f32, 0.5] {
                                let progress =
                                    display.interpolation_progress(entity.tick_count(), partial);
                                text_samples.push(json!(format!(
                                    "{},{}",
                                    text.text_opacity.get(progress),
                                    text.background.get(progress)
                                )));
                            }
                            described["text_samples"] = json!(text_samples);
                        }
                        described["render_at"] =
                            json!([0.0, 4095.99, 4096.0, 16383.99, 16384.0, 1.0e9].map(
                                |distance| display.should_render_at_sqr_distance(distance, 1.0)
                            ));
                    }
                    "end_crystal" => {
                        let state = entity.end_crystal().copied();
                        described["beam_target"] =
                            json!(state.and_then(|state| state.beam_target()).map_or_else(
                                || "none".to_string(),
                                |target| format!("{},{},{}", target.x, target.y, target.z)
                            ));
                        described["shows_bottom"] =
                            json!(state.is_none_or(|state| state.shows_bottom()));
                    }
                    "ender_dragon" => {
                        let state = entity.ender_dragon().cloned().unwrap_or_else(|| {
                            voxel_world::mobs::ender_dragon::EnderDragonState::new(0)
                        });
                        // Which crystal and when it is found rests on the entity's own random
                        // source (a tick in ten): only the last tick says, by then it is found.
                        described["nearest_crystal"] =
                            json!(if tick + 1 == i32_field(row, "tick_count")? {
                                state.nearest_crystal().unwrap_or(0)
                            } else {
                                -1
                            });
                        described["flap_time"] = json!(hex(state.flap_time()));
                        described["old_flap_time"] = json!(hex(state.old_flap_time()));
                        let samples: Vec<Value> = (0..64)
                            .map(|delay| {
                                let sample = state.history.get(delay);
                                json!({"y": format!("{:016x}", sample.y.to_bits()),
                                    "y_rot": hex(sample.y_rot)})
                            })
                            .collect();
                        described["history"] = json!(samples);
                        described["y_rot"] = json!(hex(entity.rotation().0));
                        described["dragon_death_time"] = json!(state.dragon_death_time());
                        let parts: Vec<String> = state
                            .parts()
                            .iter()
                            .map(|part| {
                                format!(
                                    "{:016x}{:016x}{:016x}",
                                    part.position.0.to_bits(),
                                    part.position.1.to_bits(),
                                    part.position.2.to_bits()
                                )
                            })
                            .collect();
                        described["parts"] = json!(parts);
                    }
                    "wither" => {
                        let state = entity.wither().copied().unwrap_or_default();
                        described["invulnerable"] = json!(state.invulnerable_ticks());
                        for i in 0..2 {
                            described[format!("x_head_{i}")] = json!(hex(state.x_rot_heads()[i]));
                            described[format!("y_head_{i}")] = json!(hex(state.y_rot_heads()[i]));
                        }
                        described["y_rot"] = json!(hex(entity.rotation().0));
                        described["y_body_rot"] = json!(hex(entity.y_body_rot()));
                    }
                    "sniffer" => {
                        let state = entity.sniffer().copied().unwrap_or_default();
                        let age = entity.tick_count() as f32;
                        for (name, animation) in [
                            ("digging", state.digging),
                            ("sniffing", state.sniffing),
                            ("rising", state.rising),
                            ("happy", state.feeling_happy),
                            ("scenting", state.scenting),
                        ] {
                            described[format!("{name}_started")] = json!(animation.is_started());
                            described[format!("{name}_millis")] =
                                json!(if animation.is_started() {
                                    animation.time_in_millis(age)
                                } else {
                                    0
                                });
                        }
                        described["searching"] = json!(state.is_searching());
                    }
                    "armadillo" => {
                        let state = entity.armadillo().copied().unwrap_or_default();
                        let age = entity.tick_count() as f32;
                        for (name, animation) in [
                            ("roll_out", state.roll_out),
                            ("roll_up", state.roll_up),
                            ("peek", state.peek),
                        ] {
                            described[format!("{name}_started")] = json!(animation.is_started());
                            described[format!("{name}_millis")] =
                                json!(if animation.is_started() {
                                    animation.time_in_millis(age)
                                } else {
                                    0
                                });
                        }
                        described["hiding"] = json!(state.should_hide_in_shell());
                        described["scared"] = json!(state.is_scared());
                        described["head_yaw"] = json!(hex(entity.y_head_rot()));
                    }
                    "creaking" => {
                        let state = entity
                            .creaking()
                            .cloned()
                            .unwrap_or_else(|| voxel_world::mobs::creaking::CreakingState::new(0));
                        let age = entity.tick_count() as f32;
                        for (name, animation) in [
                            ("attack", state.attack),
                            ("invulnerable", state.invulnerable),
                            ("death", state.death),
                        ] {
                            described[format!("{name}_started")] = json!(animation.is_started());
                            described[format!("{name}_millis")] =
                                json!(if animation.is_started() {
                                    animation.time_in_millis(age)
                                } else {
                                    0
                                });
                        }
                        described["can_move"] = json!(state.can_move());
                        described["active"] = json!(state.is_active());
                        described["tearing_down"] = json!(state.is_tearing_down());
                        described["heart_bound"] = json!(state.is_heart_bound());
                    }
                    "warden" => {
                        let state = entity.warden().copied().unwrap_or_default();
                        let age = entity.tick_count() as f32;
                        for (name, animation) in [
                            ("roar", state.roar),
                            ("sniff", state.sniff),
                            ("emerge", state.emerge),
                            ("digging", state.digging),
                            ("attack", state.attack),
                            ("sonic_boom", state.sonic_boom),
                        ] {
                            described[format!("{name}_started")] = json!(animation.is_started());
                            described[format!("{name}_millis")] =
                                json!(if animation.is_started() {
                                    animation.time_in_millis(age)
                                } else {
                                    0
                                });
                        }
                        described["tendril_old"] = json!(hex(state.tendril_animation(0.0)));
                        described["tendril"] = json!(hex(state.tendril_animation(1.0)));
                        described["heart_old"] = json!(hex(state.heart_animation(0.0)));
                        described["heart"] = json!(hex(state.heart_animation(1.0)));
                    }
                    "parrot" => {
                        let state = entity.parrot().copied().unwrap_or_default();
                        described["variant"] = json!(entity.parrot_variant());
                        let (_, motion_y, _) = entity.motion();
                        described["motion_y"] = json!(format!("{:016x}", motion_y.to_bits()));
                        for (key, partial) in [("0.0", 0.0_f32), ("0.5", 0.5), ("1.0", 1.0)] {
                            described[format!("flap_angle_{key}")] =
                                json!(hex(state.flap_angle(partial)));
                        }
                    }
                    "vex" => described["charging"] = json!(entity.is_vex_charging()),
                    "allay" => {
                        let state = entity.allay().copied().unwrap_or_default();
                        described["dancing"] = json!(entity.is_allay_dancing());
                        described["spinning"] = json!(state.is_spinning());
                        for (key, partial) in [("0.0", 0.0_f32), ("0.5", 0.5), ("1.0", 1.0)] {
                            described[format!("holding_{key}")] =
                                json!(hex(state.holding_item_progress(partial)));
                            described[format!("spin_{key}")] =
                                json!(hex(state.spinning_progress(partial)));
                        }
                    }
                    "guardian" => {
                        described["in_water"] = json!(entity.is_in_water());
                        let (y_rot, x_rot, head_y) = entity.rotation();
                        described["y_rot"] = json!(hex(y_rot));
                        described["x_rot"] = json!(hex(x_rot));
                        described["head_y"] = json!(hex(head_y));
                        described["body_y"] = json!(hex(entity.body_y_rotation()));
                        described["on_ground"] = json!(entity.on_ground());
                        let (pos_x, pos_y, pos_z) = entity.position();
                        for (key, value) in [("x", pos_x), ("y", pos_y), ("z", pos_z)] {
                            described[format!("pos_{key}")] =
                                json!(format!("{:016x}", value.to_bits()));
                        }
                        let (motion_x, motion_y, motion_z) = entity.motion();
                        for (key, value) in [("x", motion_x), ("y", motion_y), ("z", motion_z)] {
                            described[format!("motion_{key}")] =
                                json!(format!("{:016x}", value.to_bits()));
                        }
                        if let Some(guardian) = entity.guardian() {
                            for (key, partial) in [("0.0", 0.0_f32), ("0.5", 0.5), ("1.0", 1.0)] {
                                described[format!("tail_{key}")] =
                                    json!(hex(guardian.tail_animation(partial)));
                                described[format!("spikes_{key}")] =
                                    json!(hex(guardian.spikes_animation(partial)));
                            }
                            described["attack_time"] = json!(hex(guardian.attack_time() as f32));
                            described["random_state"] = json!(guardian.random_state());
                        }
                    }
                    "frog" => {
                        described["in_water"] = json!(entity.is_in_water());
                        if let Some(frog) = entity.frog() {
                            let age = entity.tick_count() as f32;
                            for (name, animation) in [
                                ("jump", frog.jump),
                                ("croak", frog.croak),
                                ("tongue", frog.tongue),
                                ("swim_idle", frog.swim_idle),
                            ] {
                                described[format!("{name}_started")] =
                                    json!(animation.is_started());
                                described[format!("{name}_millis")] =
                                    json!(if animation.is_started() {
                                        animation.time_in_millis(age)
                                    } else {
                                        0
                                    });
                            }
                        }
                    }
                    "axolotl" => {
                        let state = entity.axolotl().copied().unwrap_or_default();
                        described["in_water"] = json!(entity.is_in_water());
                        described["variant"] = json!(entity.axolotl_variant());
                        for partial in [0.0_f32, 0.5, 1.0] {
                            for (name, animator) in [
                                ("playing_dead", state.playing_dead),
                                ("in_water", state.in_water),
                                ("on_ground", state.on_ground),
                                ("moving", state.moving),
                            ] {
                                described[format!("{name}_{partial:?}")] =
                                    json!(hex(animator.factor(partial)));
                            }
                        }
                        let age = entity.tick_count() as f32;
                        for (name, animation) in [
                            ("swim", state.swim),
                            ("walk", state.walk),
                            ("walk_under_water", state.walk_under_water),
                            ("idle_under_water", state.idle_under_water),
                            (
                                "idle_under_water_on_ground",
                                state.idle_under_water_on_ground,
                            ),
                            ("idle_on_ground", state.idle_on_ground),
                            ("play_dead", state.play_dead),
                        ] {
                            described[format!("{name}_started")] = json!(animation.is_started());
                            described[format!("{name}_millis")] =
                                json!(if animation.is_started() {
                                    animation.time_in_millis(age)
                                } else {
                                    0
                                });
                        }
                    }
                    "dolphin" => {
                        described["max_x"] =
                            json!(format!("{:016x}", entity.bounding_box().max_x.to_bits()));
                        let (x, _, z) = entity.motion();
                        described["is_moving"] = json!(entity.is_moving_horizontally());
                        described["motion_x"] = json!(format!("{:016x}", x.to_bits()));
                        described["motion_z"] = json!(format!("{:016x}", z.to_bits()));
                    }
                    "fish" => {
                        described["in_water"] = json!(entity.is_in_water());
                        let area = entity.bounding_box();
                        for (key, value) in [
                            ("min_x", area.min_x),
                            ("min_y", area.min_y),
                            ("min_z", area.min_z),
                            ("max_x", area.max_x),
                            ("max_y", area.max_y),
                            ("max_z", area.max_z),
                        ] {
                            described[key] = json!(format!("{:016x}", value.to_bits()));
                        }
                    }
                    "shulker" => {
                        let shulker = entity.shulker();
                        described["face"] = json!(entity.shulker_attach_face());
                        described["color"] = json!(entity.shulker_color().map_or(16, i32::from));
                        described["peek_half"] = json!(hex(shulker.peek(0.5)));
                        described["peek_start"] = json!(hex(shulker.peek(0.0)));
                        described["peek_end"] = json!(hex(shulker.peek(1.0)));
                        described["y_rot"] = json!(hex(entity.rotation().0));
                        described["y_body_rot"] = json!(hex(entity.body_y_rotation()));
                        described["y_body_rot_old"] = json!(hex(entity.previous_body_y_rotation()));
                        described["y_head_rot"] = json!(hex(entity.rotation().2));
                        let (x, y, z) = entity.position();
                        let (old_x, old_y, old_z) = entity.previous_position();
                        for (key, value) in [
                            ("pos_x", x),
                            ("pos_y", y),
                            ("pos_z", z),
                            ("old_x", old_x),
                            ("old_y", old_y),
                            ("old_z", old_z),
                        ] {
                            described[key] = json!(format!("{:016x}", value.to_bits()));
                        }
                        described["riding"] = json!(entity.vehicle_id().is_some());
                        let area = entity.bounding_box();
                        for (key, value) in [
                            ("box_min_x", area.min_x),
                            ("box_min_y", area.min_y),
                            ("box_min_z", area.min_z),
                            ("box_max_x", area.max_x),
                            ("box_max_y", area.max_y),
                            ("box_max_z", area.max_z),
                        ] {
                            described[key] = json!(format!("{:016x}", value.to_bits()));
                        }
                        for (key, partial) in [
                            ("render_0.0", 0.0_f32),
                            ("render_0.5", 0.5),
                            ("render_1.0", 1.0),
                        ] {
                            described[key] = json!(match entity.shulker_render_offset(partial) {
                                None => "none".to_owned(),
                                Some(offset) => offset
                                    .map(|value| format!("{:016x}", value.to_bits()))
                                    .join(","),
                            });
                        }
                    }
                    "cube" => {
                        let cube = entity.cube_mob();
                        described["size"] = json!(entity.cube_size());
                        described["squish"] = json!(hex(cube.current_squish()));
                        described["old_squish"] = json!(hex(cube.old_squish()));
                        described["target_squish"] = json!(hex(cube.target_squish()));
                        described["squish_half"] = json!(hex(cube.squish(0.5)));
                        described["y_rot"] = json!(hex(entity.rotation().0));
                        described["y_body_rot"] = json!(hex(entity.body_y_rotation()));
                        described["on_ground"] = json!(entity.on_ground());
                        if voxel_data::entity_type::name(entity.type_id)
                            == Some("minecraft:sulfur_cube")
                        {
                            described["fuse"] = json!(entity.sulfur_fuse());
                        }
                        let area = entity.bounding_box();
                        for (key, value) in [
                            ("min_x", area.min_x),
                            ("min_y", area.min_y),
                            ("min_z", area.min_z),
                            ("max_x", area.max_x),
                            ("max_y", area.max_y),
                            ("max_z", area.max_z),
                        ] {
                            described[format!("box_{key}")] =
                                json!(format!("{:016x}", value.to_bits()));
                        }
                    }
                    "squid" => {
                        if let Some(squid) = entity.squid() {
                            let pose = squid.pose(0.5);
                            described["in_water"] = json!(entity.is_in_water());
                            described["y_rot"] = json!(hex(entity.rotation().0));
                            described["y_body_rot"] = json!(hex(entity.body_y_rotation()));
                            described["tentacle_angle"] = json!(hex(squid.tentacle_angle));
                            described["x_body_rot"] = json!(hex(squid.x_body_rot));
                            described["z_body_rot"] = json!(hex(squid.z_body_rot));
                            described["tentacle_angle_half"] = json!(hex(pose.tentacle_angle));
                            described["x_body_rot_half"] = json!(hex(pose.x_body_rot));
                            described["z_body_rot_half"] = json!(hex(pose.z_body_rot));
                            if voxel_data::entity_type::name(entity.type_id)
                                == Some("minecraft:glow_squid")
                            {
                                described["dark_ticks"] = json!(entity.glow_dark_ticks());
                            }
                            let (x, y, z) = entity.motion();
                            for (key, value) in [("motion_x", x), ("motion_y", y), ("motion_z", z)]
                            {
                                described[key] = json!(format!("{:016x}", value.to_bits()));
                            }
                        }
                    }
                    "copper_golem" => {
                        let tick = entity.tick_count() as f32;
                        if let Some(golem) = entity.copper_golem() {
                            let playing = golem.playing;
                            for (key, state) in [
                                ("idle", playing.idle),
                                ("get_item", playing.get_item),
                                ("get_no_item", playing.get_no_item),
                                ("drop_item", playing.drop_item),
                                ("drop_no_item", playing.drop_no_item),
                            ] {
                                described[format!("{key}_started")] = json!(state.is_started());
                                described[format!("{key}_millis")] =
                                    json!(state.time_in_millis(tick));
                            }
                        }
                        described["state"] = json!(entity.copper_golem_action() as i32);
                        described["weather"] = json!(entity.copper_golem_weather());
                    }
                    "attributes" => {
                        described["max_health"] = json!(hex(entity.max_health().unwrap_or(0.0)));
                        described["scale"] = json!(hex(entity.scale()));
                    }
                    "goat" => {
                        described["ramming"] = json!(hex(entity.goat_ramming_x_head_rot()));
                    }
                    "iron_golem" => {
                        let (attack, offer) = entity.iron_golem_ticks();
                        described["attack_tick"] = json!(attack);
                        described["offer_flower_tick"] = json!(offer);
                    }
                    "creeper" => {
                        for (key, partial) in [("0.0", 0.0_f32), ("0.5", 0.5), ("1.0", 1.0)] {
                            described[format!("swelling_{key}")] =
                                json!(hex(entity.creeper_swelling(partial)));
                        }
                    }
                    "chicken" => {
                        let (flap, flap_speed) = entity.chicken_flap(1.0);
                        let (old_flap, old_flap_speed) = entity.chicken_flap(0.0);
                        described["flap"] = json!(hex(flap));
                        described["flap_speed"] = json!(hex(flap_speed));
                        described["old_flap"] = json!(hex(old_flap));
                        described["old_flap_speed"] = json!(hex(old_flap_speed));
                    }
                    other => return Err(format!("unknown extra {other}")),
                }
            }
            ticks.push(described);
        }
        out.push(json!({"label": row["label"], "outcome": "ticked", "ticks": ticks}));
    }
    Ok(json!({"rows": out}))
}
