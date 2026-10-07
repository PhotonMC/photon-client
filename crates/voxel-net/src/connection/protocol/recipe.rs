use super::*;

pub(crate) fn encode_set_carried_item(
    output: &mut Vec<u8>,
    slot: i16,
) -> Result<(), ConnectionError> {
    output.extend_from_slice(&slot.to_be_bytes());
    Ok(())
}

pub(crate) fn encode_player_action(
    output: &mut Vec<u8>,
    action: i32,
    position: (i32, i32, i32),
    direction: u8,
    sequence: i32,
) -> Result<(), ConnectionError> {
    wire::write_varint(action, output);
    output.extend_from_slice(&pack_block_position(position).to_be_bytes());
    output.push(direction);
    wire::write_varint(sequence, output);
    Ok(())
}

pub(crate) fn pack_block_position(position: (i32, i32, i32)) -> i64 {
    const HORIZONTAL_MASK: i64 = (1_i64 << 26) - 1;
    const Y_MASK: i64 = (1_i64 << 12) - 1;

    ((i64::from(position.0) & HORIZONTAL_MASK) << 38)
        | ((i64::from(position.2) & HORIZONTAL_MASK) << 12)
        | (i64::from(position.1) & Y_MASK)
}

pub(crate) fn support_block_position(position: (f64, f64, f64, f32, f32)) -> (i32, i32, i32) {
    let block = voxel_core::binary::BlockPos::containing(position.0, position.1 - 1.0, position.2);
    (block.x, block.y, block.z)
}

pub(crate) fn encode_packet_frame(
    body: &[u8],
    compression_threshold: Option<usize>,
) -> Result<Vec<u8>, ConnectionError> {
    if body.len() > MAX_PACKET_UNCOMPRESSED_LENGTH {
        return Err(ConnectionError::Frame(FrameError::PacketTooLarge));
    }

    let mut payload = Vec::new();
    match compression_threshold {
        Some(threshold) if body.len() < threshold => {
            wire::write_varint(0, &mut payload);
            payload.extend_from_slice(body);
        }
        Some(_) => {
            wire::write_varint(
                i32::try_from(body.len())
                    .map_err(|_| ConnectionError::Frame(FrameError::PacketTooLarge))?,
                &mut payload,
            );
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(body)?;
            payload.extend_from_slice(
                &encoder
                    .finish()
                    .map_err(|error| ConnectionError::Compression(error.to_string()))?,
            );
        }
        None => payload.extend_from_slice(body),
    }
    frame::prepend(&payload).map_err(ConnectionError::from)
}

pub(crate) fn decode_packet_frame(
    frame: &[u8],
    compression_threshold: Option<usize>,
) -> Result<Vec<u8>, ConnectionError> {
    let Some(_) = compression_threshold else {
        return Ok(frame.to_vec());
    };

    let mut input = frame;
    let uncompressed_length = wire::read_varint(&mut input)?;
    if uncompressed_length < 0 {
        return Err(ConnectionError::Compression(
            "negative uncompressed length".to_owned(),
        ));
    }
    if uncompressed_length == 0 {
        return Ok(input.to_vec());
    }

    let uncompressed_length = usize::try_from(uncompressed_length)
        .map_err(|_| ConnectionError::Compression("invalid uncompressed length".to_owned()))?;
    if uncompressed_length > MAX_PACKET_UNCOMPRESSED_LENGTH {
        return Err(ConnectionError::Compression(
            "uncompressed packet exceeds 8 MiB".to_owned(),
        ));
    }

    let decoder = ZlibDecoder::new(input);
    let mut output = Vec::with_capacity(uncompressed_length);
    decoder
        .take((uncompressed_length as u64) + 1)
        .read_to_end(&mut output)?;
    if output.len() != uncompressed_length {
        return Err(ConnectionError::Compression(format!(
            "declared {uncompressed_length} bytes, decoded {}",
            output.len()
        )));
    }
    Ok(output)
}

pub(crate) fn split_packet(body: &[u8]) -> Result<(i32, &[u8]), ConnectionError> {
    let mut input = body;
    let packet_id = wire::read_varint(&mut input)?;
    Ok((packet_id, input))
}

pub(crate) fn parse_login_packet(input: &[u8]) -> Result<LoginObservation, ConnectionError> {
    let mut input = input;
    let packet = LoginObservation {
        player_id: read_i32(&mut input, "login player id")?,
        hardcore: read_bool(&mut input, "login hardcore")?,
        level_count: {
            let count = read_count(&mut input, "login level count")?;
            for _ in 0..count {
                read_identifier_string(&mut input, "login level identifier")?;
            }
            count
        },
        max_players: read_varint(&mut input, "login max players")?,
        chunk_radius: read_varint(&mut input, "login chunk radius")?,
        simulation_distance: read_varint(&mut input, "login simulation distance")?,
        reduced_debug_info: read_bool(&mut input, "login reduced debug info")?,
        show_death_screen: read_bool(&mut input, "login show death screen")?,
        do_limited_crafting: read_bool(&mut input, "login limited crafting")?,
        spawn_info: parse_spawn_info(&mut input)?,
        online_mode: read_bool(&mut input, "login online mode")?,
        enforces_secure_chat: read_bool(&mut input, "login secure chat")?,
    };
    if !input.is_empty() {
        return Err(ConnectionError::Codec("login trailing payload"));
    }
    Ok(packet)
}

pub(crate) fn parse_respawn_packet(
    input: &[u8],
) -> Result<(SpawnInfoObservation, u8), ConnectionError> {
    let mut input = input;
    let spawn_info = parse_spawn_info(&mut input)?;
    let data_to_keep = read_byte(&mut input, "respawn data to keep")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("respawn trailing payload"));
    }
    Ok((spawn_info, data_to_keep))
}

pub(crate) fn parse_open_screen_packet(
    input: &[u8],
) -> Result<OpenScreenObservation, ConnectionError> {
    let mut input = input;
    let container_id = read_varint(&mut input, "open screen container id")?;
    let menu_type_id = read_varint(&mut input, "open screen menu type")?;
    let title_input = input;
    let title = voxel_core::nbt::read_any_nbt(&mut input)
        .map_err(|_| ConnectionError::Codec("open screen title"))?;
    if title.is_none() {
        return Err(ConnectionError::Codec("open screen title"));
    }
    let title_length = title_input.len() - input.len();
    if !input.is_empty() {
        return Err(ConnectionError::Codec("open screen trailing payload"));
    }
    Ok(OpenScreenObservation {
        container_id,
        menu_type_id,
        title_nbt: title_input[..title_length].to_vec(),
        applied_to_world: false,
    })
}

pub(crate) fn parse_container_close_packet(
    input: &[u8],
) -> Result<ContainerCloseObservation, ConnectionError> {
    let mut input = input;
    let container_id = read_varint(&mut input, "container close id")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("container close trailing payload"));
    }
    Ok(ContainerCloseObservation {
        container_id,
        applied_to_world: false,
    })
}

pub(crate) fn parse_container_set_data_packet(
    input: &[u8],
) -> Result<ContainerDataObservation, ConnectionError> {
    let mut input = input;
    let container_id = read_varint(&mut input, "container data id")?;
    let id = read_i16(&mut input, "container data slot")?;
    let value = read_i16(&mut input, "container data value")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("container data trailing payload"));
    }
    Ok(ContainerDataObservation {
        container_id,
        id,
        value,
    })
}

/// `SlotDisplay.STREAM_CODEC`.
pub(super) fn read_slot_display(
    input: &mut &[u8],
    depth: usize,
) -> Result<voxel_world::SlotDisplay, ConnectionError> {
    use voxel_world::SlotDisplay as D;
    if depth > 16 {
        return Err(ConnectionError::Codec("slot display too deep"));
    }
    let next = |input: &mut &[u8]| read_slot_display(input, depth + 1);
    Ok(match read_varint(input, "slot display type")? {
        0 => D::Empty,
        1 => D::AnyFuel,
        2 => D::WithAnyPotion(Box::new(next(input)?)),
        3 => {
            let source = next(input)?;
            D::OnlyWithComponent(Box::new(source), read_varint(input, "display component")?)
        }
        4 => D::Item(read_varint(input, "display item")?),
        5 => {
            let item = read_varint(input, "display stack item")?;
            let count = read_varint(input, "display stack count")?;
            voxel_game::item_components::read_patch(input)
                .map_err(|_| ConnectionError::Codec("display stack components"))?;
            D::ItemStack(item, count)
        }
        6 => D::Tag(read_identifier_string(input, "display tag")?),
        7 => {
            let dye = next(input)?;
            D::Dyed(Box::new(dye), Box::new(next(input)?))
        }
        8 => {
            let base = next(input)?;
            let material = next(input)?;
            // `Holder<TrimPattern>`: a registry id plus one, or zero and the
            // pattern written out (its asset, its name and whether it is a decal).
            let pattern = match read_varint(input, "display trim pattern")? {
                0 => {
                    read_identifier_string(input, "trim pattern asset")?;
                    voxel_core::nbt::read_any_nbt(input)
                        .map_err(|_| ConnectionError::Codec("trim pattern name"))?;
                    read_bool(input, "trim pattern decal")?;
                    -1
                }
                id => id - 1,
            };
            D::SmithingTrim(Box::new(base), Box::new(material), pattern)
        }
        9 => {
            let source = next(input)?;
            D::WithRemainder(Box::new(source), Box::new(next(input)?))
        }
        10 => {
            let count = read_varint(input, "display count")?;
            let mut all = Vec::new();
            for _ in 0..count {
                all.push(next(input)?);
            }
            D::Composite(all)
        }
        _ => return Err(ConnectionError::Codec("unknown slot display")),
    })
}

/// `Ingredient.CONTENTS_STREAM_CODEC`: a tag, or the items it lists.
pub(super) fn read_ingredient(
    input: &mut &[u8],
) -> Result<voxel_world::Ingredient, ConnectionError> {
    let marker = read_varint(input, "ingredient marker")?;
    if marker == 0 {
        return Ok(voxel_world::Ingredient::Tag(read_identifier_string(
            input,
            "ingredient tag",
        )?));
    }
    let mut items = Vec::new();
    for _ in 0..marker - 1 {
        items.push(read_varint(input, "ingredient item")?);
    }
    Ok(voxel_world::Ingredient::Items(items))
}

pub(crate) fn parse_update_recipes_packet(input: &[u8]) -> Result<RecipeUpdate, ConnectionError> {
    let mut input = input;
    let sets = read_varint(&mut input, "recipe property set count")?;
    let mut out = Vec::new();
    for _ in 0..sets {
        let name = read_identifier_string(&mut input, "recipe property set name")?;
        let count = read_varint(&mut input, "recipe property set size")?;
        let mut items = Vec::new();
        for _ in 0..count {
            items.push(read_varint(&mut input, "recipe property set item")?);
        }
        out.push((name, items));
    }
    let entries = read_varint(&mut input, "stonecutter recipe count")?;
    let mut stonecutter = Vec::new();
    for _ in 0..entries {
        let ingredient = read_ingredient(&mut input)?;
        let display = read_slot_display(&mut input, 0)?;
        stonecutter.push(voxel_world::StonecutterEntry {
            input: ingredient,
            display,
        });
    }
    Ok(RecipeUpdate {
        sets: out,
        stonecutter,
    })
}

pub(super) fn read_slot_displays(
    input: &mut &[u8],
) -> Result<Vec<voxel_world::SlotDisplay>, ConnectionError> {
    let count = read_varint(input, "recipe display count")?;
    let mut all = Vec::new();
    for _ in 0..count {
        all.push(read_slot_display(input, 0)?);
    }
    Ok(all)
}

/// `RecipeDisplay.STREAM_CODEC`: the crafting station each one ends with is
/// read and dropped.
pub(crate) fn read_recipe_display(
    input: &mut &[u8],
) -> Result<voxel_world::RecipeDisplay, ConnectionError> {
    use voxel_world::RecipeDisplay as R;
    let slot = |input: &mut &[u8]| read_slot_display(input, 0);
    Ok(match read_varint(input, "recipe display type")? {
        0 => {
            let ingredients = read_slot_displays(input)?;
            let result = slot(input)?;
            slot(input)?;
            R::Shapeless {
                ingredients,
                result,
            }
        }
        1 => {
            let width = read_varint(input, "shaped width")?;
            let height = read_varint(input, "shaped height")?;
            let ingredients = read_slot_displays(input)?;
            if i64::from(width) * i64::from(height) != ingredients.len() as i64 {
                return Err(ConnectionError::Codec("shaped recipe size"));
            }
            let result = slot(input)?;
            slot(input)?;
            R::Shaped {
                width,
                height,
                ingredients,
                result,
            }
        }
        2 => {
            let ingredient = slot(input)?;
            let fuel = slot(input)?;
            let result = slot(input)?;
            slot(input)?;
            read_varint(input, "furnace duration")?;
            read_f32(input, "furnace experience")?;
            R::Furnace {
                ingredient,
                fuel,
                result,
            }
        }
        3 => {
            slot(input)?;
            let result = slot(input)?;
            slot(input)?;
            R::Stonecutter { result }
        }
        4 => {
            slot(input)?;
            slot(input)?;
            slot(input)?;
            let result = slot(input)?;
            slot(input)?;
            R::Smithing { result }
        }
        _ => return Err(ConnectionError::Codec("unknown recipe display")),
    })
}

/// `ClientboundRecipeBookAddPacket`: the entries with their flag bytes, and
/// whether they replace the book.
pub(crate) fn parse_recipe_book_add_packet(
    input: &[u8],
) -> Result<(Vec<(voxel_world::RecipeDisplayEntry, u8)>, bool), ConnectionError> {
    let mut input = input;
    let count = read_varint(&mut input, "recipe book add count")?;
    let mut entries = Vec::new();
    for _ in 0..count {
        let id = read_varint(&mut input, "recipe id")?;
        let display = read_recipe_display(&mut input)?;
        let group = match read_varint(&mut input, "recipe group")? {
            0 => None,
            value => Some(value - 1),
        };
        let category = read_varint(&mut input, "recipe category")?;
        let requirements = if read_byte(&mut input, "recipe requirements flag")? != 0 {
            let count = read_varint(&mut input, "recipe requirement count")?;
            let mut all = Vec::new();
            for _ in 0..count {
                all.push(read_ingredient(&mut input)?);
            }
            Some(all)
        } else {
            None
        };
        let flags = read_byte(&mut input, "recipe flags")?;
        entries.push((
            voxel_world::RecipeDisplayEntry {
                id,
                display,
                group,
                category,
                requirements,
            },
            flags,
        ));
    }
    let replace = read_byte(&mut input, "recipe book replace")? != 0;
    Ok((entries, replace))
}

pub(crate) fn parse_recipe_book_remove_packet(input: &[u8]) -> Result<Vec<i32>, ConnectionError> {
    let mut input = input;
    let count = read_varint(&mut input, "recipe book remove count")?;
    let mut ids = Vec::new();
    for _ in 0..count {
        ids.push(read_varint(&mut input, "recipe book remove id")?);
    }
    Ok(ids)
}
