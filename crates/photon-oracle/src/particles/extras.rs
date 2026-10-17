//! What a row puts in the level besides its blocks: players, the players its vaults share as
//! connected, and parrots.

use photon_core::binary::Uuid;
use photon_core::ids::EntityId;
use photon_core::nbt::{Field, FieldValue};
use photon_world::{BlockEntityInput, ClientWorld, EntityInput};

use super::*;

/// A UUID of four ints, the high half first (`UUIDUtil.uuidFromIntArray`).
fn uuid_of(value: &Value) -> Result<(Uuid, [i32; 4]), String> {
    let ints = value.as_array().ok_or("a uuid is four integers")?;
    let mut parts = [0_i32; 4];
    if ints.len() != 4 {
        return Err("a uuid is four integers".into());
    }
    for (part, int) in parts.iter_mut().zip(ints) {
        *part = int
            .as_i64()
            .and_then(|int| i32::try_from(int).ok())
            .ok_or("a uuid is four integers")?;
    }
    let half = |high: i32, low: i32| (u64::from(high as u32) << 32) | u64::from(low as u32);
    Ok((
        Uuid::new(half(parts[0], parts[1]), half(parts[2], parts[3])),
        parts,
    ))
}

/// Put a row's `players` and `parrots` in the level, and give each of its `vaults` the shared
/// data the server would have sent with the chunk.
pub(super) fn place(world: &mut ClientWorld, row: &Value) -> Result<(), String> {
    if row["light"].as_bool() == Some(true) {
        world.light_from_sources();
        world.run_light_updates();
    }
    if let Some(time) = row["game_time"].as_u64() {
        world.set_game_time(time);
    }
    if let Some([rain, thunder]) = row["weather"].as_array().map(Vec::as_slice).map(|bits| {
        let level = |index: usize| {
            bits[index]
                .as_str()
                .and_then(|text| u32::from_str_radix(text, 16).ok())
                .map_or(0.0, f32::from_bits)
        };
        [level(0), level(1)]
    }) {
        world.set_weather(rain, thunder);
    }
    for (index, player) in row["players"].as_array().into_iter().flatten().enumerate() {
        let (uuid, _) = uuid_of(&player["uuid"])?;
        put_entity(
            world,
            ("minecraft:player", 100 + index as i32, uuid),
            player,
        )?;
    }
    for (index, parrot) in row["parrots"].as_array().into_iter().flatten().enumerate() {
        let uuid = Uuid::new(7, 200 + index as u64);
        put_entity(
            world,
            ("minecraft:parrot", 200 + index as i32, uuid),
            parrot,
        )?;
    }
    for gateway in row["gateways"].as_array().into_iter().flatten() {
        place_gateway(world, gateway)?;
    }
    for vault in row["vaults"].as_array().into_iter().flatten() {
        share_vault(world, vault)?;
    }
    if let Some(seed) = row["book_seed"].as_i64() {
        world.seed_book_random(seed);
    }
    place_block_entities(world, row)?;
    Ok(())
}

/// An entity of a type at the place a row gives, standing still.
fn put_entity(
    world: &mut ClientWorld,
    (kind, id, uuid): (&str, i32, Uuid),
    at: &Value,
) -> Result<(), String> {
    let type_id = photon_data::entity_type::id(kind).ok_or("an unknown entity type")?;
    let placed = world.apply_entity(EntityInput {
        entity_id: EntityId(id),
        uuid,
        type_id,
        x: parsed64(at, "x")?,
        y: parsed64(at, "y")?,
        z: parsed64(at, "z")?,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    });
    if placed {
        Ok(())
    } else {
        Err(format!("a {kind} was rejected"))
    }
}

/// An end gateway's block entity as the chunk packet carries it: with its age.
fn place_gateway(world: &mut ClientWorld, gateway: &Value) -> Result<(), String> {
    let type_id = photon_data::block_entity_type::id("minecraft:end_gateway")
        .ok_or("no end gateway block entity")?;
    let tag = [Field {
        key: "Age".into(),
        value: FieldValue::Long(gateway["age"].as_i64().ok_or("a gateway has an age")?),
    }];
    let mut bytes = Vec::new();
    photon_core::nbt::write_nbt(Some(&tag), &mut bytes);
    let (x, y, z) = (
        i32_field(gateway, "x")?,
        i32_field(gateway, "y")?,
        i32_field(gateway, "z")?,
    );
    let input = BlockEntityInput {
        x,
        y,
        z,
        type_id,
        tag: Some(&bytes),
    };
    world.apply_block_entities(x.div_euclid(16), z.div_euclid(16), [input]);
    Ok(())
}

/// The block entities of a row, set up in the order it lists them (the level makes one the first
/// time something asks for the entity at the block).
fn place_block_entities(world: &mut ClientWorld, row: &Value) -> Result<(), String> {
    for entity in row["block_entities"].as_array().into_iter().flatten() {
        let kind = string(entity, "type")?;
        let type_id = photon_data::block_entity_type::id(kind)
            .ok_or_else(|| format!("no block entity {kind}"))?;
        let (x, y, z) = (
            i32_field(entity, "x")?,
            i32_field(entity, "y")?,
            i32_field(entity, "z")?,
        );
        let input = BlockEntityInput {
            x,
            y,
            z,
            type_id,
            tag: None,
        };
        world.apply_block_entities(x.div_euclid(16), z.div_euclid(16), [input]);
    }
    Ok(())
}

/// The shared data of a vault: the players connected to it and, if the row says, its reach.
fn share_vault(world: &mut ClientWorld, vault: &Value) -> Result<(), String> {
    let vault_type =
        photon_data::block_entity_type::id("minecraft:vault").ok_or("no vault block entity")?;
    let mut connected = Vec::new();
    for player in vault["players"].as_array().ok_or("vault players")? {
        connected.push(FieldValue::IntArray(uuid_of(player)?.1.to_vec()));
    }
    let mut shared = vec![Field {
        key: "connected_players".into(),
        value: FieldValue::List(connected),
    }];
    if vault.get("range").is_some() {
        shared.push(Field {
            key: "connected_particles_range".into(),
            value: FieldValue::Double(parsed64(vault, "range")?),
        });
    }
    let tag = [Field {
        key: "shared_data".into(),
        value: FieldValue::Compound(shared),
    }];
    let mut bytes = Vec::new();
    photon_core::nbt::write_nbt(Some(&tag), &mut bytes);
    let placed = world.apply_block_entity_update(BlockEntityInput {
        x: i32_field(vault, "x")?,
        y: i32_field(vault, "y")?,
        z: i32_field(vault, "z")?,
        type_id: vault_type,
        tag: Some(&bytes),
    });
    if placed {
        Ok(())
    } else {
        Err("a vault was rejected".into())
    }
}

/// `party,<id>` for each parrot: whether it dances, as the reference's `isPartyParrot` says.
pub(super) fn party_lines(world: &ClientWorld) -> Vec<String> {
    let mut parrots: Vec<_> = world
        .entities()
        .values()
        .filter(|entity| {
            entity.id().0 >= 200
                && photon_data::entity_type::name(entity.type_id) == Some("minecraft:parrot")
        })
        .map(|entity| {
            (
                entity.id().0,
                entity.parrot().is_some_and(|own| own.is_party()),
            )
        })
        .collect();
    parrots.sort_unstable();
    parrots
        .into_iter()
        .map(|(id, party)| format!("party,{id},{party}"))
        .collect()
}

/// `be,x,y,z,...` for each place a row watches: what its block entity holds of what block events
/// set. A place whose block does not keep an entity reads as `none`.
pub(super) fn watch_lines(world: &ClientWorld, row: &Value) -> Vec<String> {
    let mut lines = Vec::new();
    for watched in row["watch"].as_array().into_iter().flatten() {
        let (Ok(x), Ok(y), Ok(z)) = (
            i32_field(watched, "x"),
            i32_field(watched, "y"),
            i32_field(watched, "z"),
        ) else {
            continue;
        };
        lines.push(format!("be,{x},{y},{z},{}", held_at(world, (x, y, z))));
    }
    lines
}

/// The type of block entity a block keeps, were it asked for one.
fn entity_type_of(block_name: &str) -> Option<&'static str> {
    match block_name {
        "minecraft:decorated_pot" => Some("minecraft:decorated_pot"),
        "minecraft:end_gateway" => Some("minecraft:end_gateway"),
        "minecraft:spawner" => Some("minecraft:mob_spawner"),
        "minecraft:potent_sulfur" => Some("minecraft:potent_sulfur"),
        "minecraft:enchanting_table" => Some("minecraft:enchanting_table"),
        "minecraft:beacon" => Some("minecraft:beacon"),
        "minecraft:conduit" => Some("minecraft:conduit"),
        _ => None,
    }
}

/// A beacon's beam: its levels, how high up its column it has looked, and each stretch of the beam
/// it holds, as its colour and height.
fn beacon_line(beacon: &photon_world::BeaconState) -> String {
    let sections: Vec<String> = beacon
        .sections
        .iter()
        .map(|section| format!("{:08x}:{}", section.color, section.height))
        .collect();
    format!(
        "beacon,{},{},{}",
        beacon.levels,
        beacon.last_check_y(),
        sections.join(";")
    )
}

/// An enchanting table's book, the numbers of its animation as bits.
fn book_line(book: &photon_world::BookState) -> String {
    let floats = [
        book.flip,
        book.o_flip,
        book.flip_t,
        book.flip_a,
        book.open,
        book.o_open,
        book.rot,
        book.o_rot,
        book.t_rot,
    ];
    let bits: Vec<String> = floats
        .iter()
        .map(|value| format!("{:08x}", value.to_bits()))
        .collect();
    format!("book,{},{}", book.time, bits.join(","))
}

/// What the block entity at a place holds, as the reference reads it: one it has not made yet is
/// made as it is asked for, with the defaults.
fn held_at(world: &ClientWorld, (x, y, z): (i32, i32, i32)) -> String {
    let block = photon_world::block::state_or_air(world.block_state_or_air(x, y, z)).block();
    let Some(kind) = entity_type_of(block.name) else {
        return "none".to_owned();
    };
    let type_id = photon_data::block_entity_type::id(kind).unwrap_or_default();
    let made = photon_world::ClientBlockEntity::from_wire(type_id, None)
        .made_at(world.game_time() as i64, world.dimension().min_y);
    let entity = world.block_entity_at(x, y, z).unwrap_or(&made);
    let events = entity.events();
    match kind {
        "minecraft:decorated_pot" => {
            let (start, style) = events.wobble.unwrap_or((0, -1));
            format!("pot,{start},{style}")
        }
        "minecraft:end_gateway" => format!("gateway,{},{}", events.teleport_cooldown, events.age),
        "minecraft:mob_spawner" => {
            let delay = entity.spawner_state().map_or(20, |spawner| spawner.delay());
            format!("spawner,{delay}")
        }
        "minecraft:enchanting_table" => book_line(&events.book),
        "minecraft:beacon" => beacon_line(&events.beacon),
        "minecraft:conduit" => {
            let conduit = &events.conduit;
            format!(
                "conduit,{},{:08x},{},{}",
                conduit.tick_count,
                (conduit.rotation as f32).to_bits(),
                conduit.active,
                conduit.hunting
            )
        }
        _ => format!("sulfur,{}", events.eruption_tick),
    }
}

/// `blk,x,y,z,id` for each block that is not air in the box a row watches, and `mov,...` and
/// `shape,...` for the moving pistons among them.
pub(super) fn region_lines(world: &ClientWorld, row: &Value) -> Vec<String> {
    let Some(corners) = row["region"].as_array() else {
        return Vec::new();
    };
    let corner = |index: usize| corners[index].as_i64().unwrap_or_default() as i32;
    let mut lines = Vec::new();
    for x in corner(0)..=corner(3) {
        for y in corner(1)..=corner(4) {
            for z in corner(2)..=corner(5) {
                lines.extend(block_lines(world, (x, y, z)));
            }
        }
    }
    lines
}

/// The lines of one place of a watched box: its block and, for a moving piston, its move and shape.
fn block_lines(world: &ClientWorld, (x, y, z): (i32, i32, i32)) -> Vec<String> {
    let state = world.block_state_or_air(x, y, z);
    if photon_world::block::state_or_air(state).block().air {
        return Vec::new();
    }
    let mut lines = vec![format!("blk,{x},{y},{z},{}", state.0)];
    let Some(moving) = world.moving_piston_at((x, y, z)) else {
        return lines;
    };
    lines.push(format!(
        "mov,{x},{y},{z},{},{},{},{},{:08x},{:08x}",
        moving.moved_state.0,
        moving.direction as usize,
        moving.extending,
        moving.source,
        moving.progress.to_bits(),
        moving.progress_old.to_bits()
    ));
    let mut boxes = format!("shape,{x},{y},{z}");
    for aabb in world
        .moving_piston_collision_shape((x, y, z), None)
        .to_aabbs()
    {
        let edges = [
            aabb.min_x, aabb.min_y, aabb.min_z, aabb.max_x, aabb.max_y, aabb.max_z,
        ];
        for edge in edges {
            boxes.push_str(&format!(",{:016x}", edge.to_bits()));
        }
    }
    lines.push(boxes);
    lines
}
