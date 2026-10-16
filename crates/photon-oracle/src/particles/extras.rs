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
    for vault in row["vaults"].as_array().into_iter().flatten() {
        share_vault(world, vault)?;
    }
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
