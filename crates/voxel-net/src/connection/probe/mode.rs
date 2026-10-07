use super::*;

#[expect(
    clippy::cognitive_complexity,
    clippy::too_many_lines,
    reason = "a live-server probe run as one sequence of scenario steps"
)]
pub(super) fn connect_and_probe_with_mode(
    address: SocketAddr,
    username: &str,
    timeout: Duration,
    probe: PlayProbeConfig,
) -> Result<PlayConnectionTrace, ConnectionError> {
    if username.is_empty() || username.encode_utf16().count() > crate::login::MAX_NAME_LENGTH {
        return Err(ConnectionError::InvalidArgument("username"));
    }
    if timeout.is_zero() {
        return Err(ConnectionError::InvalidArgument("timeout"));
    }

    let mut trace = PlayConnectionTrace::default();
    if matches!(probe.action, ProbeAction::CreativePlace) {
        trace.stone_item_registry_id = Some(VANILLA_STONE_ITEM_REGISTRY_ID);
    }
    if matches!(
        probe.action,
        ProbeAction::CreativeSnowballUse | ProbeAction::WorldSnowballTick
    ) {
        trace.snowball_item_registry_id = Some(VANILLA_SNOWBALL_ITEM_REGISTRY_ID);
    }
    let (mut stream, compression_threshold) = open_play_connection(
        address,
        username,
        timeout,
        probe.select_known_packs,
        &mut trace,
    )?;

    let play_started = Instant::now();
    let mut chunk_batch_started = false;
    let mut player_loaded_sent = false;
    let mut player_position = None;
    let mut creative_place_target_position = None;
    let mut creative_place_neighbor_position = None;
    let mut survival_water_bucket_target_position = None;
    let mut survival_water_bucket_neighbor_position = None;
    let mut world_chunk_unload_last_sent = None;
    let mut world_ready_drain_started = None;
    let mut death_respawn_command_sent = false;
    while play_started.elapsed() <= timeout {
        let body = match read_packet(&mut stream, compression_threshold) {
            Ok(body) => body,
            Err(ConnectionError::Io(error))
                if matches!(probe.action, ProbeAction::WorldPlayerDeathRespawn)
                    && world_ready_drain_started.is_none()
                    && matches!(
                        error.kind(),
                        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
                    ) =>
            {
                continue;
            }
            Err(ConnectionError::Io(error))
                if world_ready_drain_started.is_some()
                    && matches!(
                        error.kind(),
                        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
                    ) =>
            {
                if matches!(
                    probe.action,
                    ProbeAction::WorldChunkUnload | ProbeAction::WorldEntityTicking
                ) && trace.movement_packets_sent < 96
                {
                    send_world_chunk_unload_step(
                        &mut stream,
                        &mut player_position,
                        &mut trace,
                        compression_threshold,
                        "world chunk unload timeout step position",
                    )?;
                    world_chunk_unload_last_sent = Some(Instant::now());
                    if trace.movement_packets_sent >= 96 {
                        stream.set_read_timeout(Some(WORLD_LIFECYCLE_DRAIN_TIMEOUT))?;
                    }
                    continue;
                }
                if matches!(
                    probe.action,
                    ProbeAction::SurvivalInventoryItemComponentConformance(_)
                ) && trace.container_click_packets_sent == 0
                {
                    return Err(ConnectionError::Codec("item component preparation"));
                }
                return Ok(trace);
            }
            Err(error) => return Err(error),
        };
        let (packet_id, payload) = split_packet(&body)?;
        trace.packets.push(PacketObservation {
            phase: ConnectionProtocol::Play,
            id: packet_id,
            body_length: body.len(),
        });

        match packet_id {
            PLAY_CLIENTBOUND_ADD_ENTITY_PACKET_ID => {
                let mut observation = parse_add_entity_packet(payload)?;
                observation.applied_to_world = entities::add(&mut trace.world, &observation);
                observation.applied_rotation = trace
                    .world
                    .entities()
                    .at(observation.entity_id)
                    .map(|entity| entity.rotation());
                let should_tick_world_entity = match probe.action {
                    ProbeAction::WorldSnowballTick => {
                        trace.use_item_packets_sent > 0
                            && observation.entity_type_id == VANILLA_SNOWBALL_ENTITY_TYPE_ID
                    }
                    ProbeAction::WorldArrowTick => {
                        observation.entity_type_id == VANILLA_ARROW_ENTITY_TYPE_ID
                    }
                    ProbeAction::WorldPrimedTntTick => {
                        observation.entity_type_id == VANILLA_PRIMED_TNT_ENTITY_TYPE_ID
                    }
                    ProbeAction::WorldFallingBlockTick => {
                        observation.entity_type_id == VANILLA_FALLING_BLOCK_ENTITY_TYPE_ID
                    }
                    ProbeAction::WorldFireworkTick => {
                        observation.entity_type_id == VANILLA_FIREWORK_ROCKET_ENTITY_TYPE_ID
                    }
                    _ => false,
                } && trace.world_entity_tick_observations.is_empty();
                if should_tick_world_entity {
                    observe_world_entity_tick(&mut trace, &observation)?;
                }
                if matches!(probe.action, ProbeAction::WorldExperienceOrbTick)
                    && observation.entity_type_id == VANILLA_EXPERIENCE_ORB_ENTITY_TYPE_ID
                    && trace.experience_orb_tick_observations.is_empty()
                {
                    observe_experience_orb_tick(&mut trace, &observation)?;
                }
                trace.add_entity_observations.push(observation);
            }
            PLAY_CLIENTBOUND_SET_ENTITY_MOTION_PACKET_ID => {
                let mut observation = parse_set_entity_motion_packet(payload)?;
                let motion_before = if probe.observe_player_motion {
                    trace
                        .world
                        .entities()
                        .at(observation.entity_id)
                        .filter(|entity| entity.type_id == 156)
                        .map(|entity| entity.motion())
                } else {
                    None
                };
                observation.applied_to_world = entities::motion(&mut trace.world, &observation);
                if observation.applied_to_world {
                    observation.applied_motion = trace
                        .world
                        .entities()
                        .at(observation.entity_id)
                        .map(|entity| entity.motion());
                    if let Some(before_motion) = motion_before {
                        observe_player_motion_interpolation(
                            &mut trace,
                            observation.entity_id,
                            before_motion,
                            (observation.x, observation.y, observation.z),
                        );
                    }
                }
                trace.entity_motion_observations.push(observation);
            }
            PLAY_CLIENTBOUND_ROTATE_HEAD_PACKET_ID => {
                let mut observation = parse_rotate_head_packet(payload)?;
                observation.applied_to_world =
                    entities::rotate_head(&mut trace.world, &observation);
                if observation.applied_to_world {
                    observation.applied_head_rotation = trace
                        .world
                        .entities()
                        .at(observation.entity_id)
                        .map(|entity| entity.rotation().2);
                }
                trace.entity_head_rotation_observations.push(observation);
            }
            PLAY_CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID => {
                let mut observation = parse_remove_entities_packet(payload)?;
                observation.applied_entity_count = entities::remove(&mut trace.world, &observation);
                trace.remove_entities_observations.push(observation);
            }
            PLAY_CLIENTBOUND_SET_PASSENGERS_PACKET_ID => {
                let mut observation = parse_set_passengers_packet(payload)?;
                observation.applied_to_world = entities::passengers(&mut trace.world, &observation);
                if observation.applied_to_world {
                    observation.applied_vehicle_passengers = trace
                        .world
                        .entity_passengers(observation.vehicle_id)
                        .map(<[i32]>::to_vec);
                    observation.applied_passenger_vehicles = observation
                        .passenger_ids
                        .iter()
                        .filter_map(|&passenger_id| {
                            trace.world.entity_passengers(passenger_id).map(|_| {
                                (passenger_id, trace.world.entity_vehicle_id(passenger_id))
                            })
                        })
                        .collect();
                }
                if matches!(
                    probe.action,
                    ProbeAction::WorldEntityPassengerTick
                        | ProbeAction::WorldEntityPassengerRotation
                        | ProbeAction::WorldEntityPassengerAnimalRotation
                ) && trace.entity_passenger_tick_observations.is_empty()
                    && observation.applied_to_world
                    && (probe.action != ProbeAction::WorldEntityPassengerAnimalRotation
                        || observation.passenger_ids.len() == 2)
                {
                    let passenger_id =
                        if probe.action == ProbeAction::WorldEntityPassengerAnimalRotation {
                            observation.passenger_ids.iter().copied().find(|&id| {
                                trace.world.entities().at(id).is_some_and(|entity| {
                                    entity.type_id == VANILLA_COW_ENTITY_TYPE_ID
                                })
                            })
                        } else {
                            observation
                                .passenger_ids
                                .iter()
                                .copied()
                                .find(|&id| trace.world.entities().at(id).is_some())
                        };
                    if let Some(passenger_id) = passenger_id
                        && let (Some(vehicle), Some(passenger)) = (
                            trace.world.entities().at(observation.vehicle_id),
                            trace.world.entities().at(passenger_id),
                        )
                    {
                        let before_vehicle_tick_count = vehicle.tick_count();
                        let before_passenger_tick_count = passenger.tick_count();
                        let before_passenger_position = passenger.position();
                        let before_passenger_motion = passenger.motion();
                        let before_passenger_rotation = passenger.rotation();
                        let before_passenger_body_y_rotation = passenger.body_y_rotation();
                        let passenger_type_id = passenger.type_id;
                        let tick_count =
                            if matches!(probe.action, ProbeAction::WorldEntityPassengerRotation) {
                                3
                            } else {
                                1
                            };
                        for _ in 0..tick_count {
                            trace.world.tick();
                        }
                        let vehicle = trace
                            .world
                            .entities()
                            .at(observation.vehicle_id)
                            .ok_or(ConnectionError::Codec("passenger tick vehicle"))?;
                        let passenger = trace
                            .world
                            .entities()
                            .at(passenger_id)
                            .ok_or(ConnectionError::Codec("passenger tick entity"))?;
                        trace.entity_passenger_tick_observations.push(
                            EntityPassengerTickObservation {
                                vehicle_id: observation.vehicle_id,
                                passenger_id,
                                passenger_type_id,
                                before_vehicle_tick_count,
                                after_vehicle_tick_count: vehicle.tick_count(),
                                before_passenger_tick_count,
                                after_passenger_tick_count: passenger.tick_count(),
                                before_passenger_position,
                                after_passenger_position: passenger.position(),
                                before_passenger_motion,
                                after_passenger_motion: passenger.motion(),
                                before_passenger_rotation,
                                after_passenger_rotation: passenger.rotation(),
                                before_passenger_body_y_rotation,
                                after_passenger_body_y_rotation: passenger.body_y_rotation(),
                            },
                        );
                    }
                }
                trace.entity_passengers_observations.push(observation);
            }
            PLAY_CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID => {
                let mut observation = parse_entity_metadata_packet(payload)?;
                observation.applied_to_world = entities::metadata(&mut trace.world, &observation);
                trace.entity_metadata_observations.push(observation);
            }
            PLAY_CLIENTBOUND_ENTITY_POSITION_SYNC_PACKET_ID => {
                let mut observation = parse_entity_position_sync_packet(payload)?;
                let snap_before = if probe.observe_player_position_snap {
                    trace
                        .world
                        .entities()
                        .at(observation.entity_id)
                        .filter(|entity| entity.type_id == 156)
                        .map(|entity| {
                            (
                                entity.position(),
                                (entity.rotation().0, entity.rotation().1),
                            )
                        })
                } else {
                    None
                };
                observation.applied_to_world =
                    entities::position_sync(&mut trace.world, &observation);
                if observation.applied_to_world {
                    observation.applied_position = trace
                        .world
                        .entities()
                        .at(observation.entity_id)
                        .map(|entity| entity.position());
                    observation.applied_rotation = trace
                        .world
                        .entities()
                        .at(observation.entity_id)
                        .map(|entity| entity.rotation());
                    observation.applied_on_ground = trace
                        .world
                        .entities()
                        .at(observation.entity_id)
                        .map(|entity| entity.on_ground());
                    if let Some((before_position, before_rotation)) = snap_before {
                        observe_player_position_snap(
                            &mut trace,
                            "position_sync",
                            observation.entity_id,
                            before_position,
                            before_rotation,
                        );
                    }
                }
                trace.entity_position_sync_observations.push(observation);
            }
            PLAY_CLIENTBOUND_TELEPORT_ENTITY_PACKET_ID => {
                let mut observation = parse_entity_teleport_packet(payload)?;
                let motion_before = trace
                    .world
                    .entities()
                    .at(observation.entity_id)
                    .map(|entity| entity.motion());
                let snap_before = if probe.observe_player_position_snap {
                    trace
                        .world
                        .entities()
                        .at(observation.entity_id)
                        .filter(|entity| entity.type_id == 156)
                        .map(|entity| {
                            (
                                entity.position(),
                                (entity.rotation().0, entity.rotation().1),
                            )
                        })
                } else {
                    None
                };
                observation.applied_to_world = entities::teleport(&mut trace.world, &observation);
                if observation.applied_to_world {
                    observation.motion_before = motion_before;
                    observation.applied_position = trace
                        .world
                        .entities()
                        .at(observation.entity_id)
                        .map(|entity| entity.position());
                    observation.applied_rotation = trace
                        .world
                        .entities()
                        .at(observation.entity_id)
                        .map(|entity| entity.rotation());
                    observation.applied_motion = trace
                        .world
                        .entities()
                        .at(observation.entity_id)
                        .map(|entity| entity.motion());
                    observation.applied_on_ground = trace
                        .world
                        .entities()
                        .at(observation.entity_id)
                        .map(|entity| entity.on_ground());
                    if let Some((before_position, before_rotation)) = snap_before {
                        observe_player_position_snap(
                            &mut trace,
                            "teleport",
                            observation.entity_id,
                            before_position,
                            before_rotation,
                        );
                    }
                }
                trace.entity_teleport_observations.push(observation);
            }
            PLAY_CLIENTBOUND_MOVE_ENTITY_POS_PACKET_ID => {
                let mut observation = parse_move_entity_pos_packet(payload)?;
                let interpolation_before = if probe.observe_player_interpolation {
                    trace
                        .world
                        .entities()
                        .at(observation.entity_id)
                        .filter(|entity| entity.type_id == 156)
                        .map(|entity| {
                            (
                                entity.position(),
                                (entity.rotation().0, entity.rotation().1),
                            )
                        })
                } else {
                    None
                };
                observation.applied_to_world = entities::step(&mut trace.world, &observation);
                if observation.applied_to_world {
                    observation.applied_position = trace
                        .world
                        .entities()
                        .at(observation.entity_id)
                        .map(|entity| entity.position());
                    observation.applied_rotation = trace
                        .world
                        .entities()
                        .at(observation.entity_id)
                        .map(|entity| entity.rotation());
                    observation.applied_on_ground = trace
                        .world
                        .entities()
                        .at(observation.entity_id)
                        .map(|entity| entity.on_ground());
                    if observation.has_position
                        && let Some((before_position, before_rotation)) = interpolation_before
                    {
                        observe_player_interpolation(
                            &mut trace,
                            observation.entity_id,
                            before_position,
                            before_rotation,
                        );
                    }
                }
                trace.entity_move_observations.push(observation);
            }
            PLAY_CLIENTBOUND_MOVE_ENTITY_POS_ROT_PACKET_ID => {
                let mut observation = parse_move_entity_pos_rot_packet(payload)?;
                let interpolation_before = if probe.observe_player_interpolation {
                    trace
                        .world
                        .entities()
                        .at(observation.entity_id)
                        .filter(|entity| entity.type_id == 156)
                        .map(|entity| {
                            (
                                entity.position(),
                                (entity.rotation().0, entity.rotation().1),
                            )
                        })
                } else {
                    None
                };
                observation.applied_to_world = entities::step(&mut trace.world, &observation);
                if observation.applied_to_world {
                    observation.applied_position = trace
                        .world
                        .entities()
                        .at(observation.entity_id)
                        .map(|entity| entity.position());
                    observation.applied_rotation = trace
                        .world
                        .entities()
                        .at(observation.entity_id)
                        .map(|entity| entity.rotation());
                    observation.applied_on_ground = trace
                        .world
                        .entities()
                        .at(observation.entity_id)
                        .map(|entity| entity.on_ground());
                    if observation.has_position
                        && let Some((before_position, before_rotation)) = interpolation_before
                    {
                        observe_player_interpolation(
                            &mut trace,
                            observation.entity_id,
                            before_position,
                            before_rotation,
                        );
                    }
                }
                trace.entity_move_observations.push(observation);
            }
            PLAY_CLIENTBOUND_MOVE_ENTITY_ROT_PACKET_ID => {
                let mut observation = parse_move_entity_rot_packet(payload)?;
                let interpolation_before = if probe.observe_player_interpolation {
                    trace
                        .world
                        .entities()
                        .at(observation.entity_id)
                        .filter(|entity| entity.type_id == 156)
                        .map(|entity| {
                            (
                                entity.position(),
                                (entity.rotation().0, entity.rotation().1),
                            )
                        })
                } else {
                    None
                };
                observation.applied_to_world = entities::step(&mut trace.world, &observation);
                if observation.applied_to_world {
                    observation.applied_rotation = trace
                        .world
                        .entities()
                        .at(observation.entity_id)
                        .map(|entity| entity.rotation());
                    observation.applied_on_ground = trace
                        .world
                        .entities()
                        .at(observation.entity_id)
                        .map(|entity| entity.on_ground());
                    if observation.has_position
                        && let Some((before_position, before_rotation)) = interpolation_before
                    {
                        observe_player_interpolation(
                            &mut trace,
                            observation.entity_id,
                            before_position,
                            before_rotation,
                        );
                    }
                }
                trace.entity_move_observations.push(observation);
            }
            PLAY_CLIENTBOUND_OPEN_SCREEN_PACKET_ID => {
                let mut observation = parse_open_screen_packet(payload)?;
                observation.applied_to_world = trace.world.apply_local_player_open_menu(
                    observation.container_id,
                    observation.menu_type_id,
                    &observation.title_nbt,
                );
                trace.open_screen_observations.push(observation);
            }
            PLAY_CLIENTBOUND_CONTAINER_CLOSE_PACKET_ID => {
                let mut observation = parse_container_close_packet(payload)?;
                observation.applied_to_world = trace
                    .world
                    .apply_local_player_container_close(observation.container_id);
                trace.container_close_observations.push(observation);
            }
            PLAY_CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID => {
                let mut observation = parse_container_set_content_packet(payload)?;
                observation.applied_to_world = trace.world.apply_local_player_container_content(
                    observation.container_id,
                    observation.state_id,
                    &observation.items,
                    observation.carried_item.as_ref(),
                );
                trace.container_content_observations.push(observation);
            }
            PLAY_CLIENTBOUND_CONTAINER_SET_SLOT_PACKET_ID => {
                let mut observation = parse_container_set_slot_packet(payload)?;
                observation.applied_to_world = trace.world.apply_local_player_container_slot(
                    observation.container_id,
                    observation.state_id,
                    observation.slot,
                    observation.item.as_ref(),
                );
                trace.container_slot_observations.push(observation);
            }
            PLAY_CLIENTBOUND_SET_CURSOR_ITEM_PACKET_ID => {
                let mut observation = parse_cursor_item_packet(payload)?;
                observation.applied_to_world = trace
                    .world
                    .apply_local_player_carried_item(observation.item.as_ref());
                trace.cursor_item_observations.push(observation);
            }
            PLAY_CLIENTBOUND_SET_PLAYER_INVENTORY_PACKET_ID => {
                let mut observation = parse_set_player_inventory_packet(payload)?;
                observation.applied_to_world = trace
                    .world
                    .apply_local_player_inventory_slot(observation.slot, observation.item.as_ref());
                trace.player_inventory_observations.push(observation);
            }
            PLAY_CLIENTBOUND_REMOVE_MOB_EFFECT_PACKET_ID => {
                let mut observation = parse_remove_mob_effect_packet(payload)?;
                observation.applied_to_world = trace.world.remove_local_player_mob_effect(
                    observation.entity_id,
                    observation.effect_registry_id,
                );
                trace.mob_effect_remove_observations.push(observation);
            }
            PLAY_CLIENTBOUND_SYSTEM_CHAT_PACKET_ID => {
                trace
                    .system_chat_observations
                    .push(parse_system_chat_packet(payload)?);
            }
            PLAY_CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID => {
                let mut input = payload;
                let sequence = read_varint(&mut input, "block changed ack sequence")?;
                if !input.is_empty() {
                    return Err(ConnectionError::Codec("block changed ack trailing payload"));
                }
                trace.block_changed_ack_sequences.push(sequence);
            }
            PLAY_CLIENTBOUND_BLOCK_DESTRUCTION_PACKET_ID => {
                trace
                    .block_destruction_observations
                    .push(parse_block_destruction_packet(payload)?);
            }
            PLAY_CLIENTBOUND_BLOCK_ENTITY_DATA_PACKET_ID => {
                let mut observation = parse_block_entity_data_packet(payload)?;
                observation.applied_to_world =
                    trace.world.apply_block_entity_update(BlockEntityInput {
                        x: observation.x,
                        y: observation.y,
                        z: observation.z,
                        type_id: observation.type_id,
                        tag: observation.tag.as_deref(),
                    });
                trace.block_entity_observations.push(observation);
            }
            PLAY_CLIENTBOUND_BLOCK_UPDATE_PACKET_ID => {
                let observation = parse_block_update_packet(payload)?;
                trace.world.update_block_state(
                    observation.x,
                    observation.y,
                    observation.z,
                    observation.state_id,
                );
                if creative_place_target_position.is_some_and(|position| {
                    position == (observation.x, observation.y, observation.z)
                }) {
                    trace.creative_place_target_update = Some(observation.clone());
                }
                if creative_place_neighbor_position.is_some_and(|position| {
                    position == (observation.x, observation.y, observation.z)
                }) {
                    trace.creative_place_neighbor_update = Some(observation.clone());
                }
                if survival_water_bucket_target_position.is_some_and(|position| {
                    position == (observation.x, observation.y, observation.z)
                }) {
                    trace.survival_water_bucket_target_update = Some(observation.clone());
                }
                if survival_water_bucket_neighbor_position.is_some_and(|position| {
                    position == (observation.x, observation.y, observation.z)
                }) {
                    trace.survival_water_bucket_neighbor_update = Some(observation.clone());
                }
                trace.block_update_observations.push(observation);
            }
            PLAY_CLIENTBOUND_FORGET_LEVEL_CHUNK_PACKET_ID => {
                let (x, z) = parse_forget_level_chunk_packet(payload)?;
                let removed_from_world = trace.world.unload_chunk(x, z);
                let tracked_entity_count = trace.world.entities().count();
                let ticking_entity_count = trace
                    .world
                    .entities()
                    .all()
                    .filter(|entity| entity.is_ticking())
                    .count();
                trace
                    .chunk_unload_observations
                    .push(ChunkUnloadObservation {
                        x,
                        z,
                        removed_from_world,
                        tracked_entity_count,
                        ticking_entity_count,
                        stopped_entity_count: tracked_entity_count - ticking_entity_count,
                    });
            }
            PLAY_CLIENTBOUND_SET_CHUNK_CACHE_RADIUS_PACKET_ID => {
                let radius = parse_set_chunk_cache_radius_packet(payload)?;
                trace.world.update_chunk_cache_radius(radius);
            }
            PLAY_CLIENTBOUND_SET_CHUNK_CACHE_CENTER_PACKET_ID => {
                let (x, z) = parse_set_chunk_cache_center_packet(payload)?;
                trace.world.update_chunk_cache_center(x, z);
            }
            PLAY_CLIENTBOUND_LIGHT_UPDATE_PACKET_ID => {
                let mut observation = parse_light_update_packet(payload)?;
                let application = trace.world.apply_light_updates(
                    observation.x,
                    observation.z,
                    LightUpdateInput {
                        sky: LightLayerInput {
                            mask: &observation.light_data.sky_mask,
                            empty_mask: &observation.light_data.empty_sky_mask,
                            updates: &observation.light_data.sky_updates,
                        },
                        block: LightLayerInput {
                            mask: &observation.light_data.block_mask,
                            empty_mask: &observation.light_data.empty_block_mask,
                            updates: &observation.light_data.block_updates,
                        },
                    },
                );
                // A probe runs the light after each packet, as the next frame would.
                trace.world.flush_light();
                if application.invalid_sky_updates != 0 || application.invalid_block_updates != 0 {
                    return Err(ConnectionError::Codec("light update data layer"));
                }
                observation.applied_sky_sections = application.applied_sky_sections;
                observation.applied_block_sections = application.applied_block_sections;
                observation.queued_for_chunk = application.queued;
                trace.light_update_observations.push(observation);
            }
            PLAY_CLIENTBOUND_CHUNK_BATCH_START_PACKET_ID => {
                chunk_batch_started = true;
            }
            PLAY_CLIENTBOUND_CHUNK_BATCH_FINISHED_PACKET_ID => {
                let mut input = payload;
                wire::read_varint(&mut input)
                    .map_err(|_| ConnectionError::Codec("chunk batch finished"))?;
                trace.chunk_batch_finished_count += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CHUNK_BATCH_RECEIVED_PACKET_ID,
                    |output| {
                        // This is the initial value used by Java's chunk batch calculator:
                        // 7,000,000 ns / 2,000,000 ns per chunk.
                        output.extend_from_slice(&3.5_f32.to_be_bytes());
                        Ok(())
                    },
                    compression_threshold,
                )?;
            }
            PLAY_CLIENTBOUND_LOGIN_PACKET_ID => {
                let observation = parse_login_packet(payload)?;
                trace.world.replace_dimension_with_environment(
                    DimensionInput {
                        key: &observation.spawn_info.dimension,
                        dimension_type_id: observation.spawn_info.dimension_type_id,
                        seed: observation.spawn_info.seed,
                        is_debug: observation.spawn_info.is_debug,
                        is_flat: observation.spawn_info.is_flat,
                        sea_level: observation.spawn_info.sea_level,
                    },
                    dimension_bounds_for_spawn_info(&trace, &observation.spawn_info),
                    dimension_environment_for_spawn_info(&trace, &observation.spawn_info),
                );
                trace.world.configure_chunk_cache(observation.chunk_radius);
                trace.world.set_local_player_id(observation.player_id);
                trace
                    .world
                    .apply_local_player_game_mode(ClientPlayerGameMode::from_id(i32::from(
                        observation.spawn_info.game_type,
                    )));
                trace.login_observation = Some(observation);
                trace.play_login = true;
            }
            PLAY_CLIENTBOUND_PLAYER_POSITION_PACKET_ID => {
                let packet = parse_player_position_packet(payload)?;
                let absolute = resolve_player_position(packet, player_position);
                let applied_to_world = trace.world.apply_local_player_position(
                    (absolute.0, absolute.1, absolute.2),
                    (packet.delta_x, packet.delta_y, packet.delta_z),
                    (absolute.3, absolute.4),
                );
                trace
                    .player_position_observations
                    .push(PlayerPositionObservation {
                        teleport_id: packet.teleport_id,
                        relative_flags: packet.relative_flags,
                        applied_to_world,
                    });
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_ACCEPT_TELEPORTATION_PACKET_ID,
                    |output| {
                        wire::write_varint(packet.teleport_id, output);
                        Ok(())
                    },
                    compression_threshold,
                )?;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID,
                    |output| encode_move_player_pos_rot(output, absolute, 0),
                    compression_threshold,
                )?;
                player_position = Some(absolute);
            }
            PLAY_CLIENTBOUND_SET_HEALTH_PACKET_ID => {
                let mut observation = parse_set_health_packet(payload)?;
                observation.applied_to_world = trace.world.apply_local_player_health(
                    observation.health,
                    observation.food,
                    observation.saturation,
                );
                let is_dead = observation.health <= 0.0;
                trace.health_observations.push(observation);
                if matches!(probe.action, ProbeAction::WorldPlayerDeathRespawn)
                    && is_dead
                    && !death_respawn_command_sent
                {
                    send_packet(
                        &mut stream,
                        ConnectionProtocol::Play,
                        PLAY_SERVERBOUND_CLIENT_COMMAND_PACKET_ID,
                        encode_perform_respawn,
                        compression_threshold,
                    )?;
                    death_respawn_command_sent = true;
                    trace.respawn_command_packets_sent += 1;
                    world_ready_drain_started = Some(Instant::now());
                    stream.set_read_timeout(Some(DIMENSION_RESPAWN_DRAIN_TIMEOUT))?;
                }
            }
            PLAY_CLIENTBOUND_RESPAWN_PACKET_ID => {
                let (spawn_info, data_to_keep) = parse_respawn_packet(payload)?;
                let cleared_chunk_count = trace.world.loaded_chunk_count();
                let cleared_entity_count = trace.world.entities().count();
                let dimension_changed = trace.world.replace_dimension_with_environment(
                    DimensionInput {
                        key: &spawn_info.dimension,
                        dimension_type_id: spawn_info.dimension_type_id,
                        seed: spawn_info.seed,
                        is_debug: spawn_info.is_debug,
                        is_flat: spawn_info.is_flat,
                        sea_level: spawn_info.sea_level,
                    },
                    dimension_bounds_for_spawn_info(&trace, &spawn_info),
                    dimension_environment_for_spawn_info(&trace, &spawn_info),
                );
                let local_player_recreated = trace.world.recreate_local_player(data_to_keep);
                trace
                    .world
                    .apply_local_player_game_mode(ClientPlayerGameMode::from_id(i32::from(
                        spawn_info.game_type,
                    )));
                let local_player_id = trace.world.local_player().map(|player| player.id());
                let local_player_generation =
                    trace.world.local_player().map(|player| player.generation());
                let local_player_attribute_count = trace
                    .world
                    .local_player()
                    .map(|player| player.attributes().len())
                    .unwrap_or(0);
                let local_player_attribute_modifier_count = trace
                    .world
                    .local_player()
                    .map(|player| {
                        player
                            .attributes()
                            .iter()
                            .map(|attribute| attribute.modifiers().len())
                            .sum()
                    })
                    .unwrap_or(0);
                trace.respawn_observations.push(RespawnObservation {
                    spawn_info,
                    data_to_keep,
                    local_player_id,
                    local_player_recreated,
                    local_player_generation,
                    keep_entity_data: data_to_keep & PLAYER_RESPAWN_KEEP_ENTITY_DATA != 0,
                    keep_attribute_modifiers: data_to_keep
                        & PLAYER_RESPAWN_KEEP_ATTRIBUTE_MODIFIERS
                        != 0,
                    local_player_attribute_count,
                    local_player_attribute_modifier_count,
                    dimension_changed,
                    cleared_chunk_count,
                    cleared_entity_count,
                    applied_to_world: true,
                });
            }
            PLAY_CLIENTBOUND_UPDATE_ATTRIBUTES_PACKET_ID => {
                let mut observation = parse_update_attributes_packet(payload)?;
                observation.applied_to_world = trace
                    .world
                    .apply_local_player_attributes(observation.entity_id, &observation.attributes)
                    || trace
                        .world
                        .entities_mut()
                        .apply_attributes(observation.entity_id, &observation.attributes);
                trace.attribute_update_observations.push(observation);
            }
            PLAY_CLIENTBOUND_PLAYER_ABILITIES_PACKET_ID => {
                let packet = parse_player_abilities_packet(payload)?;
                let abilities = ClientPlayerAbilities::new(
                    packet.invulnerable,
                    packet.flying,
                    packet.can_fly,
                    packet.can_instabuild,
                    packet.flying_speed,
                    packet.walking_speed,
                );
                let applied_to_world = trace.world.apply_local_player_abilities(abilities);
                trace
                    .player_abilities_observations
                    .push(PlayerAbilitiesObservation {
                        packet,
                        applied_to_world,
                    });
            }
            PLAY_CLIENTBOUND_GAME_EVENT_PACKET_ID => {
                let packet = parse_game_event_packet(payload)?;
                let game_mode = (packet.event_id == GAME_EVENT_CHANGE_GAME_MODE_ID)
                    .then(|| ClientPlayerGameMode::from_game_event_param(packet.param));
                let applied_to_world = game_mode
                    .map(|game_mode| trace.world.apply_local_player_game_mode(game_mode))
                    .unwrap_or(false);
                let local_game_mode = trace.world.local_player().map(|player| player.game_mode());
                trace.game_event_observations.push(GameEventObservation {
                    packet,
                    applied_to_world,
                    local_game_mode,
                });
            }
            PLAY_CLIENTBOUND_UPDATE_MOB_EFFECT_PACKET_ID => {
                let mut observation = parse_update_mob_effect_packet(payload)?;
                observation.applied_to_world = trace.world.apply_local_player_mob_effect(
                    observation.entity_id,
                    observation.effect_registry_id,
                    observation.amplifier,
                    observation.duration_ticks,
                    observation.flags,
                );
                trace.mob_effect_update_observations.push(observation);
            }
            PLAY_CLIENTBOUND_CHUNK_PACKET_ID => {
                let mut chunk = parse_chunk_packet(payload)?;
                trace.world.replace_chunk(
                    chunk.x,
                    chunk.z,
                    chunk
                        .section_observations
                        .iter()
                        .map(|section| SectionInput {
                            non_empty_block_count: section.non_empty_block_count,
                            fluid_count: section.fluid_count,
                            block_states: &section.block_state_values,
                            biomes: &section.biome_values,
                        }),
                );
                let applied_block_entities = trace.world.apply_block_entities(
                    chunk.x,
                    chunk.z,
                    chunk.block_entities.iter().map(|entity| BlockEntityInput {
                        x: entity.x,
                        y: entity.y,
                        z: entity.z,
                        type_id: entity.type_id,
                        tag: entity.tag.as_deref(),
                    }),
                );
                if applied_block_entities != chunk.block_entities.len() {
                    return Err(ConnectionError::Codec("chunk block entity application"));
                }
                for entity in &mut chunk.block_entities {
                    entity.applied_to_world = true;
                }
                let light_application = trace.world.apply_chunk_light(
                    chunk.x,
                    chunk.z,
                    LightUpdateInput {
                        sky: LightLayerInput {
                            mask: &chunk.light_data.sky_mask,
                            empty_mask: &chunk.light_data.empty_sky_mask,
                            updates: &chunk.light_data.sky_updates,
                        },
                        block: LightLayerInput {
                            mask: &chunk.light_data.block_mask,
                            empty_mask: &chunk.light_data.empty_block_mask,
                            updates: &chunk.light_data.block_updates,
                        },
                    },
                );
                // A probe runs the light after each packet, as the next frame would.
                trace.world.flush_light();
                if light_application.invalid_sky_updates != 0
                    || light_application.invalid_block_updates != 0
                {
                    return Err(ConnectionError::Codec("chunk light data layer"));
                }
                trace.chunk_observations.push(chunk);
                trace.chunk_packet_count += 1;
            }
            PLAY_CLIENTBOUND_KEEP_ALIVE_PACKET_ID => {
                let mut input = payload;
                let packet = crate::common::ClientboundKeepAlivePacket::decode(&mut input)
                    .map_err(|_| ConnectionError::Codec("play keep alive"))?;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_KEEP_ALIVE_PACKET_ID,
                    |output| {
                        crate::common::ServerboundKeepAlivePacket::new(packet.id).encode(output);
                        Ok(())
                    },
                    compression_threshold,
                )?;
            }
            PLAY_CLIENTBOUND_BUNDLE_DELIMITER_PACKET_ID => {}
            _ => {}
        }

        if trace.play_login
            && trace.chunk_packet_count > 0
            && (!chunk_batch_started || trace.chunk_batch_finished_count > 0)
            && (!matches!(
                probe.action,
                ProbeAction::WorldChunkUnload | ProbeAction::WorldEntityTicking
            ) || player_position.is_some())
            && world_ready_drain_started.is_none()
        {
            if !player_loaded_sent {
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_LOADED_PACKET_ID,
                    |_output| Ok(()),
                    compression_threshold,
                )?;
                player_loaded_sent = true;
                trace.player_loaded_sent = true;
            }
            if matches!(probe.action, ProbeAction::Movement) && trace.movement_packets_sent == 0 {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec("movement probe player position"));
                };
                if let Some(yaws) = probe.movement_yaws {
                    for (index, yaw) in yaws.iter().copied().enumerate() {
                        if !probe.movement_pause.is_zero() {
                            std::thread::sleep(probe.movement_pause);
                        }
                        let moved = (
                            position.0 + 0.25 * (index as f64 + 1.0),
                            position.1,
                            position.2,
                            f32::from(yaw),
                            position.4,
                        );
                        send_packet(
                            &mut stream,
                            ConnectionProtocol::Play,
                            PLAY_SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID,
                            |output| encode_move_player_pos_rot(output, moved, 1),
                            compression_threshold,
                        )?;
                        trace.movement_packets_sent += 1;
                        player_position = Some(moved);
                    }
                    if !probe.movement_hold.is_zero() {
                        std::thread::sleep(probe.movement_hold);
                    }
                } else {
                    for offset in [0.25_f64, 0.5, 0.75] {
                        let moved = (
                            position.0 + offset,
                            position.1,
                            position.2,
                            position.3,
                            position.4,
                        );
                        send_packet(
                            &mut stream,
                            ConnectionProtocol::Play,
                            PLAY_SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID,
                            |output| encode_move_player_pos_rot(output, moved, 1),
                            compression_threshold,
                        )?;
                        trace.movement_packets_sent += 1;
                        player_position = Some(moved);
                    }
                }
            }
            if matches!(
                probe.action,
                ProbeAction::WorldChunkUnload | ProbeAction::WorldEntityTicking
            ) && trace.movement_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(1));
                send_world_chunk_unload_step(
                    &mut stream,
                    &mut player_position,
                    &mut trace,
                    compression_threshold,
                    "world chunk unload initial step position",
                )?;
                world_chunk_unload_last_sent = Some(Instant::now());
            }
            if matches!(probe.action, ProbeAction::BlockAction)
                && trace.player_action_packets_sent == 0
            {
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_abort_destroy_block(output, (0, 0, 0), 1),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
            }
            if matches!(probe.action, ProbeAction::BlockBreak)
                && trace.player_action_packets_sent == 0
            {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec("block break probe player position"));
                };
                let target = support_block_position(position);
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_start_destroy_block(output, target, 2),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
            }
            if matches!(probe.action, ProbeAction::BlockBreakOutOfRange)
                && trace.player_action_packets_sent == 0
            {
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_start_destroy_block(output, (1_000_000, 64, 1_000_000), 3),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
            }
            if matches!(probe.action, ProbeAction::SurvivalBreakAbort)
                && trace.player_action_packets_sent == 0
            {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec(
                        "survival break probe player position",
                    ));
                };
                let target = support_block_position(position);
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_start_destroy_block(output, target, 6),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_abort_destroy_block(output, target, 7),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalBreakStop
                    | ProbeAction::SurvivalStoneBreakStop
                    | ProbeAction::SurvivalOakPlanksBreakStop
            ) && trace.player_action_packets_sent == 0
            {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec(
                        "survival break stop probe player position",
                    ));
                };
                let target = support_block_position(position);
                let (start_sequence, stop_sequence) =
                    if matches!(probe.action, ProbeAction::SurvivalStoneBreakStop) {
                        (12, 13)
                    } else if matches!(probe.action, ProbeAction::SurvivalOakPlanksBreakStop) {
                        (18, 19)
                    } else {
                        (8, 9)
                    };
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_start_destroy_block(output, target, start_sequence),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
                std::thread::sleep(Duration::from_millis(650));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_stop_destroy_block(output, target, stop_sequence),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalStoneWoodenPickaxeBreakStop
            ) && trace.player_action_packets_sent == 0
            {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec(
                        "survival wooden pickaxe probe player position",
                    ));
                };
                std::thread::sleep(Duration::from_secs(12));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;
                let target = support_block_position(position);
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_start_destroy_block(output, target, 20),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
                std::thread::sleep(Duration::from_millis(2_200));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_stop_destroy_block(output, target, 21),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
            }
            if matches!(probe.action, ProbeAction::SurvivalDirtWoodenShovelBreakStop)
                && trace.player_action_packets_sent == 0
            {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec(
                        "survival wooden shovel probe player position",
                    ));
                };
                std::thread::sleep(Duration::from_secs(12));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;
                let target = support_block_position(position);
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_start_destroy_block(output, target, 30),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
                std::thread::sleep(Duration::from_millis(650));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_stop_destroy_block(output, target, 31),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalOakPlanksWoodenAxeBreakStop
            ) && trace.player_action_packets_sent == 0
            {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec(
                        "survival wooden axe probe player position",
                    ));
                };
                std::thread::sleep(Duration::from_secs(12));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;
                let target = support_block_position(position);
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_start_destroy_block(output, target, 32),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
                std::thread::sleep(Duration::from_millis(1_200));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_stop_destroy_block(output, target, 33),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
            }
            if matches!(probe.action, ProbeAction::SurvivalDelayedDestroy)
                && trace.player_action_packets_sent == 0
            {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec(
                        "survival delayed destroy probe player position",
                    ));
                };
                let target = support_block_position(position);
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_start_destroy_block(output, target, 14),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
                std::thread::sleep(Duration::from_millis(100));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_stop_destroy_block(output, target, 15),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
            }
            if matches!(probe.action, ProbeAction::SurvivalSpawnProtection)
                && trace.player_action_packets_sent == 0
            {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec(
                        "survival spawn protection probe player position",
                    ));
                };
                let target = support_block_position(position);
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    |output| encode_start_destroy_block(output, target, 16),
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
            }
            if matches!(probe.action, ProbeAction::UseItemOn) && trace.use_item_on_packets_sent == 0
            {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec("use item on probe player position"));
                };
                std::thread::sleep(probe.use_item_on_delay);
                let target = support_block_position(position);
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_ON_PACKET_ID,
                    |output| encode_use_item_on(output, target, 4),
                    compression_threshold,
                )?;
                trace.use_item_on_packets_sent += 1;
            }
            if matches!(probe.action, ProbeAction::UseItem) && trace.use_item_packets_sent == 0 {
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_PACKET_ID,
                    |output| encode_use_item(output, 10, 0.0, 0.0),
                    compression_threshold,
                )?;
                trace.use_item_packets_sent += 1;
            }
            if matches!(probe.action, ProbeAction::SurvivalSnowballUse)
                && trace.use_item_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(12));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_PACKET_ID,
                    |output| encode_use_item(output, 22, 0.0, 0.0),
                    compression_threshold,
                )?;
                trace.use_item_packets_sent += 1;
            }
            if matches!(probe.action, ProbeAction::SurvivalWaterBucketUseOn)
                && trace.use_item_on_packets_sent == 0
            {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec(
                        "survival water bucket probe player position",
                    ));
                };
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;
                let target = support_block_position(position);
                survival_water_bucket_target_position = Some(target);
                survival_water_bucket_neighbor_position = Some((target.0, target.1 + 1, target.2));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_ON_PACKET_ID,
                    |output| encode_use_item_on_hit(output, target, 1, (0.5, 1.0, 0.5), 23),
                    compression_threshold,
                )?;
                trace.use_item_on_packets_sent += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_PACKET_ID,
                    |output| encode_use_item(output, 24, 0.0, 90.0),
                    compression_threshold,
                )?;
                trace.use_item_packets_sent += 1;
            }
            if matches!(probe.action, ProbeAction::SurvivalEmptyBucketPickup)
                && trace.use_item_on_packets_sent == 0
            {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec(
                        "survival empty bucket pickup probe player position",
                    ));
                };
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;
                let support = support_block_position(position);
                let target = (support.0, support.1 + 1, support.2);
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_ON_PACKET_ID,
                    |output| encode_use_item_on_hit(output, target, 1, (0.5, 1.0, 0.5), 25),
                    compression_threshold,
                )?;
                trace.use_item_on_packets_sent += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_PACKET_ID,
                    |output| encode_use_item(output, 26, 0.0, 90.0),
                    compression_threshold,
                )?;
                trace.use_item_packets_sent += 1;
            }
            if matches!(probe.action, ProbeAction::SurvivalMilkBucketUse)
                && trace.use_item_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_PACKET_ID,
                    |output| encode_use_item(output, 27, 0.0, 0.0),
                    compression_threshold,
                )?;
                trace.use_item_packets_sent += 1;
            }
            if matches!(probe.action, ProbeAction::SurvivalHoneyBottleUse)
                && trace.use_item_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_PACKET_ID,
                    |output| encode_use_item(output, 28, 0.0, 0.0),
                    compression_threshold,
                )?;
                trace.use_item_packets_sent += 1;
            }
            if matches!(probe.action, ProbeAction::SurvivalHoneyBottlePoisonCleanup)
                && trace.use_item_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_PACKET_ID,
                    |output| encode_use_item(output, 29, 0.0, 0.0),
                    compression_threshold,
                )?;
                trace.use_item_packets_sent += 1;
            }
            if matches!(probe.action, ProbeAction::SurvivalHoneyBottleEarlyRelease)
                && trace.use_item_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_PACKET_ID,
                    |output| encode_use_item(output, 34, 0.0, 0.0),
                    compression_threshold,
                )?;
                trace.use_item_packets_sent += 1;
                std::thread::sleep(Duration::from_millis(250));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID,
                    encode_release_use_item,
                    compression_threshold,
                )?;
                trace.player_action_packets_sent += 1;
            }
            let consumable_use_sequence = match probe.action {
                ProbeAction::SurvivalPufferfishUse => Some(35),
                ProbeAction::SurvivalPufferfishFullHungerRejection => Some(38),
                ProbeAction::SurvivalGoldenAppleUse => Some(36),
                ProbeAction::SurvivalEnchantedGoldenAppleUse => Some(37),
                _ => None,
            };
            if let Some(sequence) =
                consumable_use_sequence.filter(|_| trace.use_item_packets_sent == 0)
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_PACKET_ID,
                    move |output| encode_use_item(output, sequence, 0.0, 0.0),
                    compression_threshold,
                )?;
                trace.use_item_packets_sent += 1;
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryPickupStateCorrection
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(12));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| encode_inventory_pickup_state_correction(output, 0),
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(0);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventorySecondaryPickupStateCorrection
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| encode_inventory_secondary_pickup_state_correction(output, 0),
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(0);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryThrowOneStateCorrection
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| encode_inventory_throw_one_state_correction(output, 0),
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(0);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryThrowAllStateCorrection
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(30));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| encode_inventory_throw_all_state_correction(output, 0),
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(0);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryDragEmptyCancelStateCorrection
            ) && trace.container_click_packets_sent == 0
            {
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| encode_inventory_drag_empty_cancel_state_correction(output, 0),
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(0);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryDragDistributionStateCorrection
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                for (slot, button, input_id) in [
                    (36_i16, 0_u8, 0_i32),
                    (-999_i16, 0_u8, 5_i32),
                    (9_i16, 1_u8, 5_i32),
                    (36_i16, 1_u8, 5_i32),
                    (-999_i16, 2_u8, 5_i32),
                ] {
                    send_packet(
                        &mut stream,
                        ConnectionProtocol::Play,
                        PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                        |output| {
                            encode_inventory_click_with_input(output, 0, slot, button, input_id)
                        },
                        compression_threshold,
                    )?;
                    trace.container_click_packets_sent += 1;
                }
                trace.container_click_state_id_sent = Some(0);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryDoubleClickStateCorrection
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                for (slot, button, input_id) in [(36_i16, 0_u8, 0_i32), (10_i16, 0_u8, 6_i32)] {
                    send_packet(
                        &mut stream,
                        ConnectionProtocol::Play,
                        PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                        |output| {
                            encode_inventory_click_with_input(output, 0, slot, button, input_id)
                        },
                        compression_threshold,
                    )?;
                    trace.container_click_packets_sent += 1;
                }
                trace.container_click_state_id_sent = Some(0);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryDoubleClickStackLimitStateCorrection
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                for (slot, button, input_id) in [(36_i16, 0_u8, 0_i32), (10_i16, 0_u8, 6_i32)] {
                    send_packet(
                        &mut stream,
                        ConnectionProtocol::Play,
                        PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                        |output| {
                            encode_inventory_click_with_input(output, 0, slot, button, input_id)
                        },
                        compression_threshold,
                    )?;
                    trace.container_click_packets_sent += 1;
                }
                trace.container_click_state_id_sent = Some(0);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryAheadStateCorrection
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| encode_inventory_click_with_input(output, 99, 36, 0, 0),
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(99);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryCorrectPrediction
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| encode_inventory_pickup_with_prediction(output, 2, 8, 1),
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(2);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryWrongSlotPrediction
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| encode_inventory_pickup_with_wrong_slot_prediction(output, 2, 8, 1),
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(2);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryWrongCarriedPrediction
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| encode_inventory_pickup_with_wrong_carried_prediction(output, 2),
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(2);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryWrongBothPrediction
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| encode_inventory_pickup_with_wrong_both_prediction(output, 2, 8, 1),
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(2);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryComponentHashPrediction
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| {
                        encode_inventory_pickup_with_component_hash_prediction(
                            output,
                            2,
                            1,
                            VANILLA_MILK_BUCKET_ITEM_REGISTRY_ID,
                        )
                    },
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(2);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryPartialChangedSlotPrediction
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| {
                        encode_inventory_pickup_with_partial_changed_slot_prediction(
                            output,
                            2,
                            8,
                            VANILLA_STONE_ITEM_REGISTRY_ID,
                        )
                    },
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(2);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryStaleStateCorrectPrediction
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| encode_inventory_pickup_with_prediction(output, 0, 8, 1),
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(0);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryAheadStateCorrectPrediction
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| encode_inventory_pickup_with_prediction(output, 99, 8, 1),
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(99);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryAheadComponentCorrectPrediction
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(20));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| {
                        let spec = inventory_component_spec("custom_model_data")
                            .expect("custom model data component is in the conformance table");
                        encode_inventory_pickup_with_component_spec_prediction(output, 99, spec)
                    },
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(99);
            }
            if let ProbeAction::SurvivalInventoryItemComponentConformance(spec) = probe.action
                && trace.container_click_packets_sent == 0
            {
                let prepared_state_id = trace
                    .container_slot_observations
                    .iter()
                    .rev()
                    .find(|observation| {
                        observation.container_id == 0
                            && observation.slot == 36
                            && observation.item_count == Some(1)
                            && observation.item_registry_id == Some(spec.item_registry_id)
                            && observation.positive_component_type_ids.as_slice()
                                == spec.expected_positive_component_type_ids
                            && observation.negative_component_type_ids.is_empty()
                    })
                    .map(|observation| observation.state_id);
                if let Some(prepared_state_id) = prepared_state_id {
                    send_packet(
                        &mut stream,
                        ConnectionProtocol::Play,
                        PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                        |output| {
                            encode_inventory_pickup_with_component_spec_prediction(
                                output,
                                prepared_state_id,
                                spec,
                            )
                        },
                        compression_threshold,
                    )?;
                    trace.container_click_packets_sent += 1;
                    trace.container_click_state_id_sent = Some(prepared_state_id);
                }
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryQuickMoveStateCorrection
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(12));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| encode_inventory_quick_move_state_correction(output, 0),
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(0);
            }
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventorySwapStateCorrection
            ) && trace.container_click_packets_sent == 0
            {
                std::thread::sleep(Duration::from_secs(12));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| encode_inventory_swap_state_correction(output, 0),
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(0);
            }
            if matches!(
                probe.action,
                ProbeAction::CreativeSnowballUse | ProbeAction::WorldSnowballTick
            ) && trace.creative_mode_slot_packets_sent == 0
            {
                let Some(snowball_item_registry_id) = trace.snowball_item_registry_id else {
                    return Err(ConnectionError::Codec(
                        "creative snowball use item registry id",
                    ));
                };
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID,
                    |output| encode_creative_item_slot(output, 36, 1, snowball_item_registry_id),
                    compression_threshold,
                )?;
                trace.creative_mode_slot_packets_sent += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_PACKET_ID,
                    |output| encode_use_item(output, 11, 0.0, 0.0),
                    compression_threshold,
                )?;
                trace.use_item_packets_sent += 1;
            }
            if matches!(probe.action, ProbeAction::CreativePlace)
                && trace.creative_mode_slot_packets_sent == 0
            {
                let Some(position) = player_position else {
                    return Err(ConnectionError::Codec(
                        "creative place probe player position",
                    ));
                };
                let Some(stone_item_registry_id) = trace.stone_item_registry_id else {
                    return Err(ConnectionError::Codec(
                        "creative place stone item registry id",
                    ));
                };
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID,
                    |output| encode_creative_stone_slot(output, 36, stone_item_registry_id),
                    compression_threshold,
                )?;
                trace.creative_mode_slot_packets_sent += 1;
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID,
                    |output| encode_set_carried_item(output, 0),
                    compression_threshold,
                )?;
                trace.carried_item_packets_sent += 1;

                let target = support_block_position(position);
                creative_place_target_position = Some(target);
                creative_place_neighbor_position = Some((target.0, target.1, target.2 - 1));
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_USE_ITEM_ON_PACKET_ID,
                    |output| encode_use_item_on_hit(output, target, 2, (0.5, 0.5, 0.0), 5),
                    compression_threshold,
                )?;
                trace.use_item_on_packets_sent += 1;
            }
            if matches!(probe.action, ProbeAction::WorldPlayerDeathRespawn) {
                stream.set_read_timeout(Some(Duration::from_millis(100)))?;
            } else {
                world_ready_drain_started = Some(Instant::now());
                let drain_timeout = probe.drain_timeout_override.unwrap_or(match probe.action {
                    ProbeAction::SurvivalDelayedDestroy => DELAYED_DESTROY_DRAIN_TIMEOUT,
                    ProbeAction::WorldChunkUnload | ProbeAction::WorldEntityTicking => {
                        WORLD_LIFECYCLE_DRAIN_TIMEOUT
                    }
                    ProbeAction::WorldDimensionRespawn | ProbeAction::WorldDimensionTypeBounds => {
                        DIMENSION_RESPAWN_DRAIN_TIMEOUT
                    }
                    ProbeAction::WorldLightUpdate => LIGHT_UPDATE_DRAIN_TIMEOUT,
                    ProbeAction::WorldBlockEntityUpdate => BLOCK_ENTITY_UPDATE_DRAIN_TIMEOUT,
                    ProbeAction::WorldEntityAdd => ENTITY_ADD_DRAIN_TIMEOUT,
                    ProbeAction::WorldEntityRemove => ENTITY_REMOVE_DRAIN_TIMEOUT,
                    ProbeAction::WorldEntityMetadata => ENTITY_METADATA_DRAIN_TIMEOUT,
                    ProbeAction::WorldEntityPosition => ENTITY_POSITION_SYNC_DRAIN_TIMEOUT,
                    ProbeAction::SurvivalWaterBucketUseOn
                    | ProbeAction::SurvivalEmptyBucketPickup => WATER_BUCKET_DRAIN_TIMEOUT,
                    ProbeAction::SurvivalMilkBucketUse => CONSUMABLE_DRAIN_TIMEOUT,
                    ProbeAction::SurvivalHoneyBottleUse => CONSUMABLE_DRAIN_TIMEOUT,
                    ProbeAction::SurvivalHoneyBottlePoisonCleanup => CONSUMABLE_DRAIN_TIMEOUT,
                    ProbeAction::SurvivalPufferfishUse
                    | ProbeAction::SurvivalPufferfishFullHungerRejection
                    | ProbeAction::SurvivalGoldenAppleUse
                    | ProbeAction::SurvivalEnchantedGoldenAppleUse => CONSUMABLE_DRAIN_TIMEOUT,
                    ProbeAction::SurvivalInventoryPickupStateCorrection => CONSUMABLE_DRAIN_TIMEOUT,
                    ProbeAction::SurvivalInventorySecondaryPickupStateCorrection => {
                        CONSUMABLE_DRAIN_TIMEOUT
                    }
                    ProbeAction::SurvivalInventoryThrowOneStateCorrection => {
                        CONSUMABLE_DRAIN_TIMEOUT
                    }
                    ProbeAction::SurvivalInventoryThrowAllStateCorrection => {
                        CONSUMABLE_DRAIN_TIMEOUT
                    }
                    ProbeAction::SurvivalInventoryDragEmptyCancelStateCorrection => {
                        CONSUMABLE_DRAIN_TIMEOUT
                    }
                    ProbeAction::SurvivalInventoryDragDistributionStateCorrection => {
                        CONSUMABLE_DRAIN_TIMEOUT
                    }
                    ProbeAction::SurvivalInventoryDoubleClickStateCorrection => {
                        CONSUMABLE_DRAIN_TIMEOUT
                    }
                    ProbeAction::SurvivalInventoryDoubleClickStackLimitStateCorrection => {
                        CONSUMABLE_DRAIN_TIMEOUT
                    }
                    ProbeAction::SurvivalInventoryAheadStateCorrection => CONSUMABLE_DRAIN_TIMEOUT,
                    ProbeAction::SurvivalInventoryCorrectPrediction => CONSUMABLE_DRAIN_TIMEOUT,
                    ProbeAction::SurvivalInventoryWrongSlotPrediction => CONSUMABLE_DRAIN_TIMEOUT,
                    ProbeAction::SurvivalInventoryWrongCarriedPrediction => {
                        CONSUMABLE_DRAIN_TIMEOUT
                    }
                    ProbeAction::SurvivalInventoryWrongBothPrediction => CONSUMABLE_DRAIN_TIMEOUT,
                    ProbeAction::SurvivalInventoryComponentHashPrediction => {
                        CONSUMABLE_DRAIN_TIMEOUT
                    }
                    ProbeAction::SurvivalInventoryItemComponentConformance(_) => {
                        COMPONENT_PREPARATION_DRAIN_TIMEOUT
                    }
                    ProbeAction::SurvivalInventoryQuickMoveStateCorrection => {
                        CONSUMABLE_DRAIN_TIMEOUT
                    }
                    ProbeAction::SurvivalInventorySwapStateCorrection => CONSUMABLE_DRAIN_TIMEOUT,
                    _ => WORLD_READY_DRAIN_TIMEOUT,
                });
                let read_timeout = if matches!(
                    probe.action,
                    ProbeAction::WorldChunkUnload | ProbeAction::WorldEntityTicking
                ) && trace.movement_packets_sent < 96
                {
                    Duration::from_millis(100)
                } else {
                    drain_timeout
                };
                stream.set_read_timeout(Some(read_timeout))?;
            }
        }

        if let ProbeAction::SurvivalInventoryItemComponentConformance(spec) = probe.action
            && trace.container_click_packets_sent == 0
        {
            let prepared_state_id = trace
                .container_slot_observations
                .iter()
                .rev()
                .find(|observation| {
                    observation.container_id == 0
                        && observation.slot == 36
                        && observation.item_count == Some(1)
                        && observation.item_registry_id == Some(spec.item_registry_id)
                        && observation.positive_component_type_ids.as_slice()
                            == spec.expected_positive_component_type_ids
                        && observation.negative_component_type_ids.is_empty()
                })
                .map(|observation| observation.state_id);
            if let Some(prepared_state_id) = prepared_state_id {
                send_packet(
                    &mut stream,
                    ConnectionProtocol::Play,
                    PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID,
                    |output| {
                        encode_inventory_pickup_with_component_spec_prediction(
                            output,
                            prepared_state_id,
                            spec,
                        )
                    },
                    compression_threshold,
                )?;
                trace.container_click_packets_sent += 1;
                trace.container_click_state_id_sent = Some(prepared_state_id);
            }
        }

        if matches!(
            probe.action,
            ProbeAction::WorldChunkUnload | ProbeAction::WorldEntityTicking
        ) && world_ready_drain_started.is_some()
            && trace.movement_packets_sent < 96
            && world_chunk_unload_last_sent
                .is_none_or(|last_sent| last_sent.elapsed() >= Duration::from_millis(50))
        {
            send_world_chunk_unload_step(
                &mut stream,
                &mut player_position,
                &mut trace,
                compression_threshold,
                "world chunk unload packet step position",
            )?;
            world_chunk_unload_last_sent = Some(Instant::now());
            if trace.movement_packets_sent >= 96 {
                stream.set_read_timeout(Some(WORLD_LIFECYCLE_DRAIN_TIMEOUT))?;
            }
        }

        let drain_timeout = probe.drain_timeout_override.unwrap_or(match probe.action {
            ProbeAction::SurvivalDelayedDestroy => DELAYED_DESTROY_DRAIN_TIMEOUT,
            ProbeAction::WorldChunkUnload | ProbeAction::WorldEntityTicking => {
                WORLD_LIFECYCLE_DRAIN_TIMEOUT
            }
            ProbeAction::WorldDimensionRespawn
            | ProbeAction::WorldPlayerDeathRespawn
            | ProbeAction::WorldDimensionTypeBounds => DIMENSION_RESPAWN_DRAIN_TIMEOUT,
            ProbeAction::WorldLightUpdate => LIGHT_UPDATE_DRAIN_TIMEOUT,
            ProbeAction::WorldBlockEntityUpdate => BLOCK_ENTITY_UPDATE_DRAIN_TIMEOUT,
            ProbeAction::WorldEntityAdd => ENTITY_ADD_DRAIN_TIMEOUT,
            ProbeAction::WorldEntityRemove => ENTITY_REMOVE_DRAIN_TIMEOUT,
            ProbeAction::WorldEntityMetadata => ENTITY_METADATA_DRAIN_TIMEOUT,
            ProbeAction::WorldEntityPosition => ENTITY_POSITION_SYNC_DRAIN_TIMEOUT,
            ProbeAction::SurvivalWaterBucketUseOn | ProbeAction::SurvivalEmptyBucketPickup => {
                WATER_BUCKET_DRAIN_TIMEOUT
            }
            ProbeAction::SurvivalMilkBucketUse => CONSUMABLE_DRAIN_TIMEOUT,
            ProbeAction::SurvivalHoneyBottleUse => CONSUMABLE_DRAIN_TIMEOUT,
            ProbeAction::SurvivalHoneyBottlePoisonCleanup => CONSUMABLE_DRAIN_TIMEOUT,
            ProbeAction::SurvivalPufferfishUse
            | ProbeAction::SurvivalPufferfishFullHungerRejection
            | ProbeAction::SurvivalGoldenAppleUse
            | ProbeAction::SurvivalEnchantedGoldenAppleUse => CONSUMABLE_DRAIN_TIMEOUT,
            ProbeAction::SurvivalInventoryPickupStateCorrection => CONSUMABLE_DRAIN_TIMEOUT,
            ProbeAction::SurvivalInventorySecondaryPickupStateCorrection => {
                CONSUMABLE_DRAIN_TIMEOUT
            }
            ProbeAction::SurvivalInventoryThrowOneStateCorrection => CONSUMABLE_DRAIN_TIMEOUT,
            ProbeAction::SurvivalInventoryThrowAllStateCorrection => CONSUMABLE_DRAIN_TIMEOUT,
            ProbeAction::SurvivalInventoryDragEmptyCancelStateCorrection => {
                CONSUMABLE_DRAIN_TIMEOUT
            }
            ProbeAction::SurvivalInventoryDragDistributionStateCorrection => {
                CONSUMABLE_DRAIN_TIMEOUT
            }
            ProbeAction::SurvivalInventoryDoubleClickStateCorrection => CONSUMABLE_DRAIN_TIMEOUT,
            ProbeAction::SurvivalInventoryDoubleClickStackLimitStateCorrection => {
                CONSUMABLE_DRAIN_TIMEOUT
            }
            ProbeAction::SurvivalInventoryAheadStateCorrection => CONSUMABLE_DRAIN_TIMEOUT,
            ProbeAction::SurvivalInventoryCorrectPrediction => CONSUMABLE_DRAIN_TIMEOUT,
            ProbeAction::SurvivalInventoryWrongSlotPrediction => CONSUMABLE_DRAIN_TIMEOUT,
            ProbeAction::SurvivalInventoryWrongCarriedPrediction => CONSUMABLE_DRAIN_TIMEOUT,
            ProbeAction::SurvivalInventoryWrongBothPrediction => CONSUMABLE_DRAIN_TIMEOUT,
            ProbeAction::SurvivalInventoryComponentHashPrediction => CONSUMABLE_DRAIN_TIMEOUT,
            ProbeAction::SurvivalInventoryItemComponentConformance(_) => {
                COMPONENT_PREPARATION_DRAIN_TIMEOUT
            }
            ProbeAction::SurvivalInventoryQuickMoveStateCorrection => CONSUMABLE_DRAIN_TIMEOUT,
            ProbeAction::SurvivalInventorySwapStateCorrection => CONSUMABLE_DRAIN_TIMEOUT,
            _ => WORLD_READY_DRAIN_TIMEOUT,
        });
        if world_ready_drain_started.is_some_and(|started| started.elapsed() >= drain_timeout) {
            if matches!(
                probe.action,
                ProbeAction::SurvivalInventoryItemComponentConformance(_)
            ) && trace.container_click_packets_sent == 0
            {
                return Err(ConnectionError::Codec("item component preparation"));
            }
            return Ok(trace);
        }
    }

    Err(ConnectionError::Io(io::Error::new(
        io::ErrorKind::TimedOut,
        "play world did not arrive before timeout",
    )))
}
