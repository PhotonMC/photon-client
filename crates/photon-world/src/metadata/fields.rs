use super::*;

/// An experience orb's value (`ExperienceOrb.DATA_VALUE`).
pub(crate) fn orb_value_from_metadata(type_id: i32, metadata: &[u8]) -> Option<i32> {
    if photon_data::entity_type::name(type_id)? != "minecraft:experience_orb" {
        return None;
    }
    metadata_int_value(metadata, EXPERIENCE_ORB_VALUE_DATA_ID)
}

/// Position the cursor at one entity-metadata field value.
///
/// The reference writes `data_id`, serializer id and value per field and ends
/// the list with `255`. This walks that layout once and hands the caller the
/// serializer id the reference recorded plus the bytes starting at the value,
/// so each typed reader below keeps its own trailing-byte rule.
pub fn find_metadata_field(metadata: &[u8], target_data_id: u8) -> Option<(i32, &[u8])> {
    if metadata.last() != Some(&255) {
        return None;
    }
    let mut metadata = metadata;
    loop {
        let (&data_id, rest) = metadata.split_first()?;
        if data_id == 255 {
            return None;
        }
        metadata = rest;
        let serializer_id = photon_core::wire::read_varint(&mut metadata).ok()?;
        if data_id == target_data_id {
            return Some((serializer_id, metadata));
        }
        skip_metadata_value(&mut metadata, serializer_id)?;
    }
}

pub(crate) fn metadata_byte_value(metadata: &[u8], target_data_id: u8) -> Option<u8> {
    let (serializer_id, value_input) = find_metadata_field(metadata, target_data_id)?;
    if serializer_id != ENTITY_DATA_SERIALIZER_BYTE_ID {
        return None;
    }
    let (&value, rest) = value_input.split_first()?;
    if rest.is_empty() {
        return None;
    }
    Some(value)
}

pub(crate) fn metadata_boolean_value(metadata: &[u8], target_data_id: u8) -> Option<bool> {
    let (serializer_id, value_input) = find_metadata_field(metadata, target_data_id)?;
    if serializer_id != ENTITY_DATA_SERIALIZER_BOOLEAN_ID {
        return None;
    }
    let (&value, rest) = value_input.split_first()?;
    if rest.is_empty() {
        return None;
    }
    Some(value != 0)
}

/// The entity data serializers by id, in the order `EntityDataSerializers`
/// registers them, and the bytes each one's value takes. A value this port
/// cannot walk leaves the fields after it unread: a particle with data of its
/// own, a resolvable profile and a painting variant sent whole.
pub(crate) fn skip_metadata_value(metadata: &mut &[u8], serializer_id: i32) -> Option<()> {
    let varint = |metadata: &mut &[u8]| photon_core::wire::read_varint(metadata).ok().map(drop);
    let bytes = |metadata: &mut &[u8], count: usize| {
        *metadata = metadata.get(count..)?;
        Some(())
    };
    match serializer_id {
        // Byte, boolean.
        ENTITY_DATA_SERIALIZER_BYTE_ID | ENTITY_DATA_SERIALIZER_BOOLEAN_ID => bytes(metadata, 1)?,
        // Int, block state, optional block state, optional unsigned int,
        // direction, pose, humanoid arm, and the enums and registry holders:
        // each a var int.
        ENTITY_DATA_SERIALIZER_INT_ID
        | ENTITY_DATA_SERIALIZER_POSE_ID
        | 12
        | 14
        | 15
        | ENTITY_DATA_SERIALIZER_OPTIONAL_UNSIGNED_INT_ID
        | 21..=32
        | 35..=38
        | 42 => varint(metadata)?,
        2 => {
            photon_core::wire::read_varlong(metadata).ok()?;
        }
        ENTITY_DATA_SERIALIZER_FLOAT_ID => bytes(metadata, 4)?,
        4 => {
            let length = usize::try_from(photon_core::wire::read_varint(metadata).ok()?).ok()?;
            bytes(metadata, length)?;
        }
        5 => {
            photon_core::nbt::read_any_nbt(metadata).ok()?.as_ref()?;
        }
        ENTITY_DATA_SERIALIZER_OPTIONAL_COMPONENT_ID => {
            let (&present, rest) = metadata.split_first()?;
            *metadata = rest;
            if present != 0 {
                let component = photon_core::nbt::read_any_nbt(metadata).ok()?;
                component.as_ref()?;
            }
        }
        // Item stack (`ItemStack.OPTIONAL_STREAM_CODEC`).
        7 => {
            if photon_core::wire::read_varint(metadata).ok()? > 0 {
                varint(metadata)?;
                crate::item_components::read_patch(metadata).ok()?;
            }
        }
        // Rotations, vector3.
        9 | 39 => bytes(metadata, 12)?,
        // Block position.
        10 => bytes(metadata, 8)?,
        ENTITY_DATA_SERIALIZER_OPTIONAL_BLOCK_POS_ID => {
            let (&present, rest) = metadata.split_first()?;
            *metadata = rest;
            if present != 0 {
                photon_core::binary::read_blockpos(metadata).ok()?;
            }
        }
        // Optional living entity reference: a flag and a UUID.
        13 => {
            let (&present, rest) = metadata.split_first()?;
            *metadata = rest;
            if present != 0 {
                bytes(metadata, 16)?;
            }
        }
        // One particle, which only the entity effect's colour is known for.
        16 => {
            if photon_core::wire::read_varint(metadata).ok()? != ENTITY_EFFECT_PARTICLE_TYPE_ID {
                return None;
            }
            bytes(metadata, 4)?;
        }
        ENTITY_DATA_SERIALIZER_PARTICLES_ID => {
            read_effect_particle_type_ids(metadata)?;
        }
        // Villager data: type, profession, level.
        18 => {
            varint(metadata)?;
            varint(metadata)?;
            varint(metadata)?;
        }
        // Optional global position: a flag, a dimension, a block position.
        33 => {
            let (&present, rest) = metadata.split_first()?;
            *metadata = rest;
            if present != 0 {
                let length =
                    usize::try_from(photon_core::wire::read_varint(metadata).ok()?).ok()?;
                bytes(metadata, length)?;
                bytes(metadata, 8)?;
            }
        }
        // Painting variant: a registry holder, or the variant itself.
        34 => {
            if photon_core::wire::read_varint(metadata).ok()? == 0 {
                return None;
            }
        }
        // Quaternion.
        40 => bytes(metadata, 16)?,
        _ => return None,
    }
    Some(())
}

/// The registry id of the variant a pig, cow, frog or other of this type wears, if the
/// data holds one (the default is not sent).
pub(crate) fn variant_from_metadata(type_id: i32, metadata: &[u8]) -> Option<i32> {
    let (data_id, serializer) = match photon_data::entity_type::name(type_id)? {
        // A painting's variant is a holder: its registry id plus one, or zero for one sent whole.
        "minecraft:painting" => {
            let (found, mut value) = find_metadata_field(metadata, PAINTING_VARIANT_DATA_ID)?;
            if found != PAINTING_VARIANT_SERIALIZER_ID {
                return None;
            }
            let holder = photon_core::wire::read_varint(&mut value).ok()?;
            return holder.checked_sub(1).filter(|id| *id >= 0);
        }
        "minecraft:pig" => (PIG_VARIANT_DATA_ID, PIG_VARIANT_SERIALIZER_ID),
        "minecraft:cow" => (COW_VARIANT_DATA_ID, COW_VARIANT_SERIALIZER_ID),
        "minecraft:chicken" => (CHICKEN_VARIANT_DATA_ID, CHICKEN_VARIANT_SERIALIZER_ID),
        "minecraft:wolf" => (WOLF_VARIANT_DATA_ID, WOLF_VARIANT_SERIALIZER_ID),
        "minecraft:cat" => (CAT_VARIANT_DATA_ID, CAT_VARIANT_SERIALIZER_ID),
        "minecraft:frog" => (FROG_VARIANT_DATA_ID, FROG_VARIANT_SERIALIZER_ID),
        "minecraft:zombie_nautilus" => (
            ZOMBIE_NAUTILUS_VARIANT_DATA_ID,
            ZOMBIE_NAUTILUS_VARIANT_SERIALIZER_ID,
        ),
        _ => return None,
    };
    let (found, mut value) = find_metadata_field(metadata, data_id)?;
    if found != serializer {
        return None;
    }
    photon_core::wire::read_varint(&mut value).ok()
}

pub(crate) fn metadata_int_value(metadata: &[u8], target_data_id: u8) -> Option<i32> {
    let (serializer_id, mut value_input) = find_metadata_field(metadata, target_data_id)?;
    if serializer_id != ENTITY_DATA_SERIALIZER_INT_ID {
        return None;
    }
    let value = photon_core::wire::read_varint(&mut value_input).ok()?;
    if value_input.is_empty() {
        return None;
    }
    Some(value)
}

/// An optional unsigned int: zero for none, else the value plus one.
pub(crate) fn metadata_optional_unsigned_int_value(
    metadata: &[u8],
    target_data_id: u8,
) -> Option<Option<i32>> {
    let (serializer_id, mut value_input) = find_metadata_field(metadata, target_data_id)?;
    if serializer_id != ENTITY_DATA_SERIALIZER_OPTIONAL_UNSIGNED_INT_ID {
        return None;
    }
    let value = photon_core::wire::read_varint(&mut value_input).ok()?;
    if value_input.is_empty() {
        return None;
    }
    Some(value.checked_sub(1))
}

pub(crate) fn metadata_float_value(metadata: &[u8], target_data_id: u8) -> Option<f32> {
    let (serializer_id, value_input) = find_metadata_field(metadata, target_data_id)?;
    if serializer_id != ENTITY_DATA_SERIALIZER_FLOAT_ID {
        return None;
    }
    let value = u32::from_be_bytes(value_input.get(..4)?.try_into().ok()?);
    if value_input.get(4..)?.is_empty() {
        return None;
    }
    Some(f32::from_bits(value))
}

pub(crate) fn metadata_pose_value(metadata: &[u8]) -> Option<ClientEntityPose> {
    let (serializer_id, mut value_input) = find_metadata_field(metadata, ENTITY_POSE_DATA_ID)?;
    if serializer_id != ENTITY_DATA_SERIALIZER_POSE_ID {
        return None;
    }
    let value = photon_core::wire::read_varint(&mut value_input).ok()?;
    if value_input.is_empty() {
        return None;
    }
    Some(ClientEntityPose::from_wire_id(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Entity data as the wire writes it: fields of `(data id, serializer id,
    /// value bytes)`, ended by 255.
    fn data(fields: &[(u8, i32, &[u8])]) -> Vec<u8> {
        let mut out = Vec::new();
        for (id, serializer, value) in fields {
            out.push(*id);
            photon_core::wire::write_varint(*serializer, &mut out);
            out.extend_from_slice(value);
        }
        out.push(255);
        out
    }

    #[test]
    fn a_wolfs_flags_collar_and_anger_are_read_and_its_variant_is_a_registry_id() {
        let wolf = photon_data::entity_type::id("minecraft:wolf").unwrap();
        let pig = photon_data::entity_type::id("minecraft:pig").unwrap();
        let metadata = data(&[
            (TAMABLE_FLAGS_DATA_ID, 0, &[5]),
            (WOLF_INTERESTED_DATA_ID, 8, &[1]),
            (WOLF_COLLAR_DATA_ID, 1, &[3]),
            (WOLF_ANGER_END_DATA_ID, 2, &[0x90, 0x4e]),
            (WOLF_VARIANT_DATA_ID, WOLF_VARIANT_SERIALIZER_ID, &[4]),
        ]);
        let read = wolf_data_from_metadata(wolf, &metadata);
        assert_eq!(read.flags, Some(5));
        assert_eq!(read.interested, Some(true));
        assert_eq!(read.collar, Some(3));
        assert_eq!(read.anger_end, Some(10_000));
        assert_eq!(variant_from_metadata(wolf, &metadata), Some(4));
        assert_eq!(wolf_data_from_metadata(pig, &metadata), WolfData::default());
    }

    #[test]
    fn a_cats_flags_lying_relax_and_collar_are_read_and_its_variant_is_a_registry_id() {
        let cat = photon_data::entity_type::id("minecraft:cat").unwrap();
        let wolf = photon_data::entity_type::id("minecraft:wolf").unwrap();
        let metadata = data(&[
            (TAMABLE_FLAGS_DATA_ID, 0, &[5]),
            (CAT_VARIANT_DATA_ID, CAT_VARIANT_SERIALIZER_ID, &[7]),
            (CAT_LYING_DATA_ID, 8, &[1]),
            (CAT_RELAX_DATA_ID, 8, &[0]),
            (CAT_COLLAR_DATA_ID, 1, &[3]),
        ]);
        let read = cat_data_from_metadata(cat, &metadata);
        assert_eq!(read.flags, Some(5));
        assert_eq!(read.lying, Some(true));
        assert_eq!(read.relaxed, Some(false));
        assert_eq!(read.collar, Some(3));
        assert_eq!(variant_from_metadata(cat, &metadata), Some(7));
        assert_eq!(cat_data_from_metadata(wolf, &metadata), CatData::default());
    }

    #[test]
    fn a_villagers_data_and_unhappy_counter_are_read_from_their_fields() {
        let villager = photon_data::entity_type::id("minecraft:villager").expect("a villager");
        let cow = photon_data::entity_type::id("minecraft:cow").expect("a cow");
        let metadata = data(&[
            (VILLAGER_UNHAPPY_DATA_ID, 1, &[40]),
            (
                VILLAGER_DATA_DATA_ID,
                VILLAGER_DATA_SERIALIZER_ID,
                &[4, 5, 3],
            ),
        ]);
        assert_eq!(
            villager_data_from_metadata(villager, &metadata),
            Some([4, 5, 3])
        );
        assert_eq!(unhappy_counter_from_metadata(villager, &metadata), Some(40));
        assert_eq!(villager_data_from_metadata(cow, &metadata), None);
        assert_eq!(unhappy_counter_from_metadata(cow, &metadata), None);
    }

    #[test]
    fn an_endermans_block_and_a_zombie_villagers_data_are_read_from_their_fields() {
        let id = |name| photon_data::entity_type::id(name).expect("a type");
        let metadata = data(&[
            (
                ENDERMAN_CARRY_DATA_ID,
                OPTIONAL_BLOCK_STATE_SERIALIZER_ID,
                &[0x81, 0x01],
            ),
            (
                ENDERMAN_CREEPY_DATA_ID,
                ENTITY_DATA_SERIALIZER_BOOLEAN_ID,
                &[1],
            ),
            (
                ZOMBIE_VILLAGER_DATA_DATA_ID,
                VILLAGER_DATA_SERIALIZER_ID,
                &[1, 2, 3],
            ),
        ]);
        assert_eq!(
            enderman_flags_from_metadata(id("minecraft:enderman"), &metadata),
            (Some(129), Some(true))
        );
        let empty = data(&[(
            ENDERMAN_CARRY_DATA_ID,
            OPTIONAL_BLOCK_STATE_SERIALIZER_ID,
            &[0],
        )]);
        assert_eq!(
            enderman_flags_from_metadata(id("minecraft:enderman"), &empty),
            (Some(0), None)
        );
        let pumpkin = data(&[(
            SNOW_GOLEM_PUMPKIN_DATA_ID,
            ENTITY_DATA_SERIALIZER_BYTE_ID,
            &[16],
        )]);
        let bare = data(&[(
            SNOW_GOLEM_PUMPKIN_DATA_ID,
            ENTITY_DATA_SERIALIZER_BYTE_ID,
            &[0],
        )]);
        let golem = id("minecraft:snow_golem");
        assert_eq!(
            snow_golem_pumpkin_from_metadata(golem, &pumpkin),
            Some(true)
        );
        assert_eq!(snow_golem_pumpkin_from_metadata(golem, &bare), Some(false));
        assert_eq!(
            snow_golem_pumpkin_from_metadata(id("minecraft:cow"), &pumpkin),
            None
        );
        assert_eq!(
            villager_data_from_metadata(id("minecraft:zombie_villager"), &metadata),
            Some([1, 2, 3])
        );
    }

    #[test]
    fn a_bees_flags_and_anger_are_read_from_their_fields() {
        let id = |name| photon_data::entity_type::id(name).expect("a type");
        let mut anger = Vec::new();
        photon_core::wire::write_varlong(1_234_567_890_123, &mut anger);
        let metadata = data(&[
            (BEE_FLAGS_DATA_ID, ENTITY_DATA_SERIALIZER_BYTE_ID, &[12]),
            (BEE_ANGER_END_DATA_ID, LONG_SERIALIZER_ID, &anger),
        ]);
        assert_eq!(
            bee_data_from_metadata(id("minecraft:bee"), &metadata),
            (Some(12), Some(1_234_567_890_123))
        );
        assert_eq!(
            bee_data_from_metadata(id("minecraft:cow"), &metadata),
            (None, None)
        );
    }

    #[test]
    fn a_goats_horns_and_a_mooshrooms_variant_are_read_from_their_fields() {
        let id = |name| photon_data::entity_type::id(name).expect("a type");
        let metadata = data(&[
            (MOOSHROOM_TYPE_DATA_ID, ENTITY_DATA_SERIALIZER_INT_ID, &[1]),
            (
                GOAT_LEFT_HORN_DATA_ID,
                ENTITY_DATA_SERIALIZER_BOOLEAN_ID,
                &[0],
            ),
            (
                GOAT_RIGHT_HORN_DATA_ID,
                ENTITY_DATA_SERIALIZER_BOOLEAN_ID,
                &[1],
            ),
        ]);
        assert_eq!(
            goat_horns_from_metadata(id("minecraft:goat"), &metadata),
            (Some(false), Some(true))
        );
        assert_eq!(
            mooshroom_type_from_metadata(id("minecraft:mooshroom"), &metadata),
            Some(1)
        );
        assert_eq!(
            mooshroom_type_from_metadata(id("minecraft:cow"), &metadata),
            None
        );
    }

    #[test]
    fn an_illagers_celebrating_and_casting_are_read_from_their_fields() {
        let id = |name| photon_data::entity_type::id(name).expect("a type");
        let metadata = data(&[
            (
                RAIDER_CELEBRATING_DATA_ID,
                ENTITY_DATA_SERIALIZER_BOOLEAN_ID,
                &[1],
            ),
            (
                SPELLCASTER_CASTING_DATA_ID,
                ENTITY_DATA_SERIALIZER_BYTE_ID,
                &[2],
            ),
        ]);
        assert_eq!(
            illager_flags_from_metadata(id("minecraft:evoker"), &metadata),
            (Some(true), Some(true))
        );
        // A vindicator does not cast, and a cow is no raider.
        assert_eq!(
            illager_flags_from_metadata(id("minecraft:vindicator"), &metadata),
            (Some(true), None)
        );
        assert_eq!(
            illager_flags_from_metadata(id("minecraft:cow"), &metadata),
            (None, None)
        );
    }

    #[test]
    fn the_serializer_ids_are_the_names_this_walk_assumes() {
        let name = |id| photon_data::entity_type::data_serializer(id).expect("a serializer");
        assert_eq!(name(ENTITY_DATA_SERIALIZER_PARTICLES_ID), "PARTICLES");
        assert_eq!(name(PIG_VARIANT_SERIALIZER_ID), "PIG_VARIANT");
        assert_eq!(name(COW_VARIANT_SERIALIZER_ID), "COW_VARIANT");
        // Registry holders and enums are one var int each.
        for id in 21..=32 {
            assert!(name(id).ends_with("_VARIANT"), "{id}");
        }
        for id in 35..=38 {
            assert!(name(id).contains("STATE"), "{id}");
        }
        assert_eq!(name(42), "HUMANOID_ARM");
        assert_eq!(name(7), "ITEM_STACK");
        assert_eq!(name(41), "RESOLVABLE_PROFILE");
    }

    #[test]
    fn every_serializer_the_walk_knows_is_stepped_over() {
        // The value of each serializer, so the field after it is still found.
        let values: [(i32, &[u8]); 41] = [
            (0, &[7]),
            (1, &[0x81, 0x01]),
            (2, &[0xFF, 0xFF, 0x03]),
            (3, &[0, 0, 0x80, 0x3F]),
            (4, &[2, b'h', b'i']),
            (5, &[8, 0, 2, b'h', b'i']),
            (6, &[0]),
            (7, &[0]),
            (8, &[1]),
            (9, &[0; 12]),
            (10, &[0; 8]),
            (11, &[0]),
            (12, &[3]),
            (
                13,
                &[1, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            ),
            (14, &[0x81, 0x01]),
            (15, &[0]),
            (17, &[0]),
            (18, &[1, 2, 3]),
            (19, &[0]),
            (20, &[5]),
            (21, &[1]),
            (22, &[1]),
            (23, &[1]),
            (24, &[1]),
            (25, &[1]),
            (26, &[1]),
            (27, &[1]),
            (28, &[1]),
            (29, &[1]),
            (30, &[1]),
            (31, &[1]),
            (32, &[1]),
            (33, &[1, 3, b'a', b'b', b'c', 0, 0, 0, 0, 0, 0, 0, 0]),
            (34, &[4]),
            (35, &[1]),
            (36, &[1]),
            (37, &[1]),
            (38, &[1]),
            (39, &[0; 12]),
            (40, &[0; 16]),
            (42, &[1]),
        ];
        for (serializer, value) in values {
            let metadata = data(&[(1, serializer, value), (2, 8, &[1])]);
            assert_eq!(
                metadata_boolean_value(&metadata, 2),
                Some(true),
                "serializer {serializer}"
            );
        }
        // A particle with data of its own, a profile and a painting sent
        // whole are not walked, and the fields after them are not read.
        for serializer in [16, 41] {
            let metadata = data(&[(1, serializer, &[1, 1]), (2, 8, &[1])]);
            assert_eq!(metadata_boolean_value(&metadata, 2), None, "{serializer}");
        }
        let direct_painting = data(&[(1, 34, &[0, 1, 1]), (2, 8, &[1])]);
        assert_eq!(metadata_boolean_value(&direct_painting, 2), None);
    }

    #[test]
    fn a_pigs_and_a_cows_variant_is_read_after_the_fields_before_it() {
        let pig = photon_data::entity_type::id("minecraft:pig").expect("pig");
        let cow = photon_data::entity_type::id("minecraft:cow").expect("cow");
        let zombie = photon_data::entity_type::id("minecraft:zombie").expect("zombie");
        let metadata = data(&[
            (6, 20, &[0]),
            (PIG_VARIANT_DATA_ID, PIG_VARIANT_SERIALIZER_ID, &[2]),
        ]);
        assert_eq!(variant_from_metadata(pig, &metadata), Some(2));
        // A cow's data is told apart by the entity's type, not the id alone.
        assert_eq!(variant_from_metadata(cow, &metadata), None);
        assert_eq!(variant_from_metadata(zombie, &metadata), None);
        let metadata = data(&[(COW_VARIANT_DATA_ID, COW_VARIANT_SERIALIZER_ID, &[1])]);
        assert_eq!(variant_from_metadata(cow, &metadata), Some(1));
        // The variant not sent: the default.
        assert_eq!(variant_from_metadata(pig, &data(&[(6, 20, &[0])])), None);
    }
}
