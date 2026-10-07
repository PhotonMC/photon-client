use super::*;

/// The scoreboard packets, by id.
pub(crate) fn parse_scoreboard_packet(
    id: i32,
    input: &[u8],
) -> Result<Option<voxel_game::scoreboard::ScoreboardEvent>, ConnectionError> {
    use voxel_game::scoreboard::{ScoreboardEvent, TeamParameters};
    let mut input = input;
    let input = &mut input;
    let string = |input: &mut &[u8], name| read_chat_string(input, 32767, name);
    Ok(Some(match id {
        PLAY_CLIENTBOUND_SET_OBJECTIVE_PACKET_ID => {
            let name = string(input, "objective name")?;
            let method = read_byte(input, "objective method")?;
            if method == 0 || method == 2 {
                ScoreboardEvent::Objective {
                    name,
                    method,
                    display_name: read_chat_component(input, "objective display name")?,
                    hearts: read_varint(input, "objective render type")? == 1,
                    number_format: read_number_format(input)?,
                }
            } else {
                ScoreboardEvent::Objective {
                    name,
                    method,
                    display_name: voxel_game::text::Component::empty(),
                    hearts: false,
                    number_format: None,
                }
            }
        }
        PLAY_CLIENTBOUND_SET_SCORE_PACKET_ID => ScoreboardEvent::Score {
            owner: string(input, "score owner")?,
            objective: string(input, "score objective")?,
            value: read_varint(input, "score value")?,
            display: if read_bool(input, "score display present")? {
                Some(read_chat_component(input, "score display")?)
            } else {
                None
            },
            number_format: read_number_format(input)?,
        },
        PLAY_CLIENTBOUND_RESET_SCORE_PACKET_ID => ScoreboardEvent::ResetScore {
            owner: string(input, "reset score owner")?,
            objective: if read_bool(input, "reset score objective present")? {
                Some(string(input, "reset score objective")?)
            } else {
                None
            },
        },
        PLAY_CLIENTBOUND_SET_DISPLAY_OBJECTIVE_PACKET_ID => {
            let slot = read_varint(input, "display slot")?;
            let name = string(input, "display objective")?;
            ScoreboardEvent::Display {
                slot: u8::try_from(slot).unwrap_or(u8::MAX),
                objective: (!name.is_empty()).then_some(name),
            }
        }
        PLAY_CLIENTBOUND_SET_PLAYER_TEAM_PACKET_ID => {
            let name = string(input, "team name")?;
            let method = read_byte(input, "team method")?;
            let parameters = if method == 0 || method == 2 {
                let display_name = read_chat_component(input, "team display name")?;
                let prefix = read_chat_component(input, "team prefix")?;
                let suffix = read_chat_component(input, "team suffix")?;
                read_varint(input, "team name tag visibility")?;
                read_varint(input, "team collision rule")?;
                let colour = if read_bool(input, "team colour present")? {
                    Some(read_varint(input, "team colour")?.clamp(0, 15) as u8)
                } else {
                    None
                };
                read_byte(input, "team options")?;
                Some(TeamParameters {
                    display_name,
                    prefix,
                    suffix,
                    colour,
                })
            } else {
                None
            };
            let mut players = Vec::new();
            if matches!(method, 0 | 3 | 4) {
                for _ in 0..read_count(input, "team player count")? {
                    players.push(string(input, "team player")?);
                }
            }
            ScoreboardEvent::Team {
                name,
                method,
                parameters,
                players,
            }
        }
        _ => return Ok(None),
    }))
}

/// `NumberFormatTypes.OPTIONAL_STREAM_CODEC`: absent, or a type (blank,
/// styled, fixed) and what it holds.
pub(super) fn read_number_format(
    input: &mut &[u8],
) -> Result<Option<voxel_game::scoreboard::NumberFormat>, ConnectionError> {
    use voxel_game::scoreboard::NumberFormat;
    if !read_bool(input, "number format present")? {
        return Ok(None);
    }
    Ok(Some(match read_varint(input, "number format type")? {
        0 => NumberFormat::Blank,
        1 => NumberFormat::Styled(
            voxel_core::nbt::read_any_nbt(input)
                .ok()
                .flatten()
                .and_then(|tag| voxel_game::text::Style::from_nbt(&tag))
                .ok_or(ConnectionError::Codec("number format style"))?,
        ),
        2 => NumberFormat::Fixed(read_chat_component(input, "number format value")?),
        _ => return Err(ConnectionError::Codec("number format type")),
    }))
}

/// `ClientboundBossEventPacket(RegistryFriendlyByteBuf)`.
pub(super) fn parse_boss_event(input: &mut &[u8]) -> Result<HudEvent, ConnectionError> {
    let id = wire::take::<16>(input)
        .map(u128::from_be_bytes)
        .ok_or(ConnectionError::Codec("boss event id"))?;
    let operation = match read_varint(input, "boss event operation")? {
        0 => BossOperation::Add {
            name: read_chat_component(input, "boss bar name")?,
            progress: read_f32(input, "boss bar progress")?,
            color: boss_ordinal(input, 7, "boss bar color")?,
            overlay: boss_ordinal(input, 5, "boss bar overlay")?,
            flags: read_byte(input, "boss bar flags")?,
        },
        1 => BossOperation::Remove,
        2 => BossOperation::Progress(read_f32(input, "boss bar progress")?),
        3 => BossOperation::Name(read_chat_component(input, "boss bar name")?),
        4 => BossOperation::Style {
            color: boss_ordinal(input, 7, "boss bar color")?,
            overlay: boss_ordinal(input, 5, "boss bar overlay")?,
        },
        5 => BossOperation::Properties(read_byte(input, "boss bar flags")?),
        _ => return Err(ConnectionError::Codec("boss event operation")),
    };
    Ok(HudEvent::Boss { id, operation })
}

/// `ClientboundEntityEventPacket.STREAM_CODEC`: the entity's id and the
/// event (a byte).
/// `ClientboundAnimatePacket`: the entity and the action.
pub(crate) fn parse_animate_packet(input: &[u8]) -> Result<(i32, u8), ConnectionError> {
    let mut input = input;
    let id = read_varint(&mut input, "animate entity id")?;
    let action = read_byte(&mut input, "animate action")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("animate trailing payload"));
    }
    Ok((id, action))
}

pub(crate) fn parse_entity_event_packet(input: &[u8]) -> Result<(i32, i8), ConnectionError> {
    let mut input = input;
    let id = read_i32(&mut input, "entity event id")?;
    let event = *input
        .first()
        .ok_or(ConnectionError::Codec("entity event"))? as i8;
    Ok((id, event))
}

/// The entity a `ClientboundDamageEventPacket` or a
/// `ClientboundHurtAnimationPacket` names: both begin with its id as a
/// var int. The rest (the source, the yaw) is not read.
pub(crate) fn parse_entity_id_prefix(input: &[u8]) -> Result<i32, ConnectionError> {
    let mut input = input;
    voxel_core::wire::read_varint(&mut input).map_err(|_| ConnectionError::Codec("entity id"))
}

/// `ItemCost.STREAM_CODEC`: an item, a count and the components the payment must carry.
pub(super) fn parse_item_cost(
    input: &mut &[u8],
) -> Result<voxel_game::merchant::ItemCost, ConnectionError> {
    const NAME: &str = "merchant item cost";
    let item = read_varint(input, NAME)?;
    let count = read_varint(input, NAME)?;
    let mut cursor = voxel_game::item_components::Cursor { data: input };
    let components = cursor.count().map_err(|_| ConnectionError::Codec(NAME))?;
    let mut expected = Vec::new();
    for _ in 0..components {
        let id = cursor.varint().map_err(|_| ConnectionError::Codec(NAME))?;
        let start = cursor.data;
        cursor
            .component(id)
            .map_err(|_| ConnectionError::Codec(NAME))?;
        let length = start.len() - cursor.data.len();
        expected.push((id, start[..length].to_vec()));
    }
    *input = cursor.data;
    Ok(voxel_game::merchant::ItemCost {
        item,
        count,
        components: expected,
    })
}

/// `ClientboundMerchantOffersPacket`: the container, the offers, and the merchant's level,
/// experience and what its screen shows.
pub(crate) fn parse_merchant_offers_packet(
    input: &[u8],
) -> Result<(i32, voxel_game::merchant::MerchantState), ConnectionError> {
    use voxel_game::merchant::{MerchantOffer, MerchantState, Trader};
    const NAME: &str = "merchant offers";
    let mut input = input;
    let input = &mut input;
    let container_id = read_varint(input, NAME)?;
    let count = read_varint(input, NAME)?;
    let mut offers = Vec::new();
    for _ in 0..count.max(0) {
        let cost_a = parse_item_cost(input)?;
        let result = parse_optional_item_stack(input, NAME)?;
        let result = voxel_world::ClientItemStackState::from_wire(
            result.item_count.unwrap_or(0),
            result.item_registry_id.unwrap_or(0),
            result.positive_component_type_ids,
            result.negative_component_type_ids,
        )
        .with_patch_bytes(result.patch_bytes);
        let cost_b = if read_bool(input, NAME)? {
            Some(parse_item_cost(input)?)
        } else {
            None
        };
        let out_of_stock = read_bool(input, NAME)?;
        let uses = read_i32(input, NAME)?;
        let max_uses = read_i32(input, NAME)?;
        let xp = read_i32(input, NAME)?;
        let special_price_diff = read_i32(input, NAME)?;
        let price_multiplier = f32::from_bits(read_i32(input, NAME)? as u32);
        let demand = read_i32(input, NAME)?;
        offers.push(MerchantOffer {
            cost_a,
            cost_b,
            result,
            uses: if out_of_stock { max_uses } else { uses },
            max_uses,
            xp,
            special_price_diff,
            demand,
            price_multiplier,
        });
    }
    let level = read_varint(input, NAME)?;
    let xp = read_varint(input, NAME)?;
    let show_progress = read_bool(input, NAME)?;
    let can_restock = read_bool(input, NAME)?;
    Ok((
        container_id,
        MerchantState {
            offers,
            trader: Trader {
                level,
                xp,
                show_progress,
                can_restock,
            },
        },
    ))
}

/// `ServerboundSelectTradePacket`: the offer's place in the list.
pub fn encode_select_trade(trade: i32) -> Vec<u8> {
    let mut output = Vec::new();
    voxel_core::wire::write_varint(trade, &mut output);
    output
}

/// `ServerboundContainerSlotStateChangedPacket`: the slot, the container and the new state.
pub fn encode_container_slot_state(slot: i32, container_id: i32, enabled: bool) -> Vec<u8> {
    let mut output = Vec::new();
    voxel_core::wire::write_varint(slot, &mut output);
    voxel_core::wire::write_varint(container_id, &mut output);
    output.push(u8::from(enabled));
    output
}

/// `ServerboundSetBeaconPacket`: each effect as a present flag and, when present, its registry id.
pub fn encode_set_beacon(primary: Option<i32>, secondary: Option<i32>) -> Vec<u8> {
    let mut output = Vec::new();
    for effect in [primary, secondary] {
        output.push(u8::from(effect.is_some()));
        if let Some(id) = effect {
            voxel_core::wire::write_varint(id, &mut output);
        }
    }
    output
}

/// `ClientboundDamageEventPacket`: the entity hurt, and the registry id of the damage type that
/// hurt it (`None` when the packet carries the type itself instead of naming one, which a server
/// that syncs its registries does not do). The rest (the causes, the place) is not read.
pub(crate) fn parse_damage_event_packet(
    input: &[u8],
) -> Result<(i32, Option<usize>), ConnectionError> {
    let mut input = input;
    let id = read_varint(&mut input, "damage event entity id")?;
    // `ByteBufCodecs.holder`: the registry id plus one, or zero for a type written out.
    let holder = read_varint(&mut input, "damage event type")?;
    Ok((
        id,
        usize::try_from(holder)
            .ok()
            .and_then(|value| value.checked_sub(1)),
    ))
}

/// `ClientboundBlockEventPacket`: the block, the event and its data (the block's registry id
/// is left unread).
pub(crate) fn parse_block_event_packet(input: &[u8]) -> Result<BlockEvent, ConnectionError> {
    const NAME: &str = "block event";
    let mut input = input;
    let input = &mut input;
    let pos = voxel_core::binary::BlockPos::unpack(read_i64(input, NAME)? as u64);
    let event = read_byte(input, NAME)?;
    let data = read_byte(input, NAME)?;
    Ok(((pos.x, pos.y, pos.z), event, data))
}

/// `ClientboundInitializeBorderPacket`: the whole border state of a level.
pub(crate) fn parse_initialize_border_packet(
    input: &[u8],
) -> Result<voxel_world::ClientWorldBorderInitialization, ConnectionError> {
    const NAME: &str = "initialize border";
    let mut input = input;
    let input = &mut input;
    let center_x = read_f64(input, NAME)?;
    let center_z = read_f64(input, NAME)?;
    let old_size = read_f64(input, NAME)?;
    let new_size = read_f64(input, NAME)?;
    let lerp_time =
        voxel_core::wire::read_varlong(input).map_err(|_| ConnectionError::Codec(NAME))?;
    let absolute_max_size = read_varint(input, NAME)?;
    let warning_blocks = read_varint(input, NAME)?;
    let warning_time = read_varint(input, NAME)?;
    Ok(voxel_world::ClientWorldBorderInitialization {
        center_x,
        center_z,
        old_size,
        new_size,
        lerp_time,
        absolute_max_size,
        warning_blocks,
        warning_time,
    })
}

/// `ClientboundSetBorderLerpSizePacket`: the size it moves from and to, and over how many ticks.
pub(crate) fn parse_set_border_lerp_size_packet(
    input: &[u8],
) -> Result<(f64, f64, i64), ConnectionError> {
    const NAME: &str = "set border lerp size";
    let mut input = input;
    let input = &mut input;
    let old_size = read_f64(input, NAME)?;
    let new_size = read_f64(input, NAME)?;
    let time = voxel_core::wire::read_varlong(input).map_err(|_| ConnectionError::Codec(NAME))?;
    Ok((old_size, new_size, time))
}

/// `ClientboundHurtAnimationPacket`: the entity and the yaw the blow came from.
pub(crate) fn parse_hurt_animation_packet(input: &[u8]) -> Result<(i32, f32), ConnectionError> {
    let mut input = input;
    let id = read_varint(&mut input, "hurt animation entity")?;
    let yaw = f32::from_bits(read_i32(&mut input, "hurt animation yaw")? as u32);
    Ok((id, yaw))
}

/// `ClientboundTrackedWaypointPacket.STREAM_CODEC`: the operation, then the
/// waypoint (`TrackedWaypoint.read`).
pub fn parse_waypoint_packet(
    input: &[u8],
) -> Result<
    (
        voxel_game::waypoint::WaypointOperation,
        voxel_game::waypoint::TrackedWaypoint,
    ),
    ConnectionError,
> {
    use voxel_game::waypoint::{
        TrackedWaypoint, WaypointIcon, WaypointId, WaypointKind, WaypointOperation,
    };
    let mut input = input;
    let input = &mut input;
    // `ByIdMap.continuous` with `WRAP`: an id past the end comes round.
    let operation = match read_varint(input, "waypoint operation")?.rem_euclid(3) {
        0 => WaypointOperation::Track,
        1 => WaypointOperation::Untrack,
        _ => WaypointOperation::Update,
    };
    let id = if read_bool(input, "waypoint id side")? {
        WaypointId::Uuid(
            wire::take::<16>(input)
                .map(u128::from_be_bytes)
                .ok_or(ConnectionError::Codec("waypoint uuid"))?,
        )
    } else {
        WaypointId::Name(read_chat_string(input, 32767, "waypoint name")?)
    };
    let style = read_identifier_string(input, "waypoint style")?;
    let colour = if read_bool(input, "waypoint colour present")? {
        let red = i32::from(read_byte(input, "waypoint red")?);
        let green = i32::from(read_byte(input, "waypoint green")?);
        let blue = i32::from(read_byte(input, "waypoint blue")?);
        Some(red << 16 | green << 8 | blue)
    } else {
        None
    };
    // `readEnum`: the ordinal, which must exist.
    let kind = match read_varint(input, "waypoint type")? {
        0 => WaypointKind::Empty,
        1 => WaypointKind::Position([
            read_varint(input, "waypoint x")?,
            read_varint(input, "waypoint y")?,
            read_varint(input, "waypoint z")?,
        ]),
        2 => WaypointKind::Chunk([
            read_varint(input, "waypoint chunk x")?,
            read_varint(input, "waypoint chunk z")?,
        ]),
        3 => WaypointKind::Azimuth(read_f32(input, "waypoint angle")?),
        _ => return Err(ConnectionError::Codec("waypoint type")),
    };
    let icon = WaypointIcon { style, colour };
    Ok((operation, TrackedWaypoint { id, icon, kind }))
}

/// An enum on the wire: its ordinal as a varint, below `count`.
pub(super) fn boss_ordinal(
    input: &mut &[u8],
    count: i32,
    name: &'static str,
) -> Result<u8, ConnectionError> {
    let ordinal = read_varint(input, name)?;
    if (0..count).contains(&ordinal) {
        Ok(ordinal as u8)
    } else {
        Err(ConnectionError::Codec(name))
    }
}
