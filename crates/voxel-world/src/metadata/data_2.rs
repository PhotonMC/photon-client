use super::*;

/// A copper golem's action and weathering (`COPPER_GOLEM_STATE`,
/// `DATA_WEATHER_STATE`: each a varint id), each when the data holds it.
pub(crate) fn copper_golem_data_from_metadata(
    type_id: i32,
    metadata: &[u8],
) -> Option<(Option<i32>, Option<i32>)> {
    if voxel_data::entity_type::name(type_id)? != "minecraft:copper_golem" {
        return None;
    }
    let varint = |data_id: u8, serializer: i32| {
        let (serializer_id, mut value_input) = find_metadata_field(metadata, data_id)?;
        if serializer_id != serializer {
            return None;
        }
        let value = voxel_core::wire::read_varint(&mut value_input).ok()?;
        (!value_input.is_empty()).then_some(value)
    };
    Some((
        varint(COPPER_GOLEM_STATE_DATA_ID, COPPER_GOLEM_STATE_SERIALIZER_ID),
        varint(
            COPPER_GOLEM_WEATHER_DATA_ID,
            COPPER_GOLEM_WEATHER_SERIALIZER_ID,
        ),
    ))
}

/// A glow squid's dark ticks (`GlowSquid.DATA_DARK_TICKS_REMAINING`), when the
/// data holds them.
pub(crate) fn glow_squid_dark_ticks_from_metadata(type_id: i32, metadata: &[u8]) -> Option<i32> {
    if voxel_data::entity_type::name(type_id)? != "minecraft:glow_squid" {
        return None;
    }
    metadata_int_value(metadata, GLOW_SQUID_DARK_TICKS_DATA_ID)
}

/// Whether a ghast is charging a fireball (`Ghast.DATA_IS_CHARGING`), when the
/// data holds it.
pub(crate) fn ghast_charging_from_metadata(type_id: i32, metadata: &[u8]) -> Option<bool> {
    if voxel_data::entity_type::name(type_id)? != "minecraft:ghast" {
        return None;
    }
    metadata_boolean_value(metadata, GHAST_CHARGING_DATA_ID)
}

/// Whether a wither skull is the blue one (`WitherSkull.DATA_DANGEROUS`), when the data holds it.
pub(crate) fn wither_skull_dangerous_from_metadata(type_id: i32, metadata: &[u8]) -> Option<bool> {
    if voxel_data::entity_type::name(type_id)? != "minecraft:wither_skull" {
        return None;
    }
    metadata_boolean_value(metadata, WITHER_SKULL_DANGEROUS_DATA_ID)
}

/// A bat's flags (`Bat.DATA_ID_FLAGS`: resting), a vex's (`Vex.DATA_FLAGS_ID`:
/// charging) and whether an allay dances (`Allay.DATA_DANCING`), each when the
/// data holds it.
pub(crate) fn flier_data_from_metadata(
    type_id: i32,
    metadata: &[u8],
) -> (Option<u8>, Option<u8>, Option<bool>) {
    match voxel_data::entity_type::name(type_id) {
        Some("minecraft:bat") => (metadata_byte_value(metadata, BAT_FLAGS_DATA_ID), None, None),
        Some("minecraft:vex") => (None, metadata_byte_value(metadata, VEX_FLAGS_DATA_ID), None),
        Some("minecraft:allay") => (
            None,
            None,
            metadata_boolean_value(metadata, ALLAY_DANCING_DATA_ID),
        ),
        _ => (None, None, None),
    }
}

/// A boat's data (`VehicleEntity.DATA_ID_*`, `AbstractBoat.DATA_ID_*`), each when
/// the data holds it.
pub(crate) fn boat_data_from_metadata(metadata: &[u8]) -> BoatData {
    BoatData {
        hurt_time: metadata_int_value(metadata, BOAT_HURT_DATA_ID),
        hurt_dir: metadata_int_value(metadata, BOAT_HURT_DIR_DATA_ID),
        damage: metadata_float_value(metadata, BOAT_DAMAGE_DATA_ID),
        paddle_left: metadata_boolean_value(metadata, BOAT_PADDLE_LEFT_DATA_ID),
        paddle_right: metadata_boolean_value(metadata, BOAT_PADDLE_RIGHT_DATA_ID),
        bubble_time: metadata_int_value(metadata, BOAT_BUBBLE_TIME_DATA_ID),
    }
}

/// A parrot's variant id (`Parrot.DATA_VARIANT_ID`), when the data holds it.
pub(crate) fn parrot_variant_from_metadata(type_id: i32, metadata: &[u8]) -> Option<i32> {
    if voxel_data::entity_type::name(type_id)? != "minecraft:parrot" {
        return None;
    }
    metadata_int_value(metadata, PARROT_VARIANT_DATA_ID)
}

/// A slime's or a magma cube's size (`AbstractCubeMob.ID_SIZE`), when the data holds it.
pub(crate) fn cube_mob_size_from_metadata(type_id: i32, metadata: &[u8]) -> Option<i32> {
    if !is_cube_mob_type(type_id) {
        return None;
    }
    metadata_int_value(metadata, CUBE_MOB_SIZE_DATA_ID)
}

/// Whether the type is a slime or a magma cube.
pub(crate) fn is_cube_mob_type(type_id: i32) -> bool {
    matches!(
        voxel_data::entity_type::name(type_id),
        Some("minecraft:slime" | "minecraft:magma_cube" | "minecraft:sulfur_cube")
    )
}

/// A sulfur cube's fuse (`SulfurCube.MAX_FUSE`), when the data holds it.
pub(crate) fn sulfur_cube_fuse_from_metadata(type_id: i32, metadata: &[u8]) -> Option<i32> {
    if voxel_data::entity_type::name(type_id)? != "minecraft:sulfur_cube" {
        return None;
    }
    metadata_int_value(metadata, SULFUR_CUBE_FUSE_DATA_ID)
}

/// A shulker's attach face (a direction's data value), raw peek and colour, each
/// when the data holds it.
pub(crate) fn shulker_data_from_metadata(
    type_id: i32,
    metadata: &[u8],
) -> (Option<i32>, Option<u8>, Option<u8>) {
    if voxel_data::entity_type::name(type_id) != Some("minecraft:shulker") {
        return (None, None, None);
    }
    let face = find_metadata_field(metadata, SHULKER_ATTACH_FACE_DATA_ID)
        .filter(|(serializer, _)| *serializer == DIRECTION_SERIALIZER_ID)
        .and_then(|(_, mut value)| voxel_core::wire::read_varint(&mut value).ok());
    (
        face,
        metadata_byte_value(metadata, SHULKER_PEEK_DATA_ID),
        metadata_byte_value(metadata, SHULKER_COLOR_DATA_ID),
    )
}

/// A phantom's size (`Phantom.ID_SIZE`), when the data holds it.
pub(crate) fn phantom_size_from_metadata(type_id: i32, metadata: &[u8]) -> Option<i32> {
    if voxel_data::entity_type::name(type_id)? != "minecraft:phantom" {
        return None;
    }
    metadata_int_value(metadata, PHANTOM_SIZE_DATA_ID)
}

/// A warden's client anger level (`Warden.CLIENT_ANGER_LEVEL`), when the data holds it.
pub(crate) fn warden_anger_from_metadata(type_id: i32, metadata: &[u8]) -> Option<i32> {
    if voxel_data::entity_type::name(type_id)? != "minecraft:warden" {
        return None;
    }
    metadata_int_value(metadata, crate::warden::CLIENT_ANGER_DATA_ID)
}

pub(crate) fn minecart_data_from_metadata(metadata: &[u8]) -> MinecartData {
    use crate::minecart::{CUSTOM_DISPLAY_BLOCK_DATA_ID, DISPLAY_OFFSET_DATA_ID, FUEL_DATA_ID};
    // `OPTIONAL_BLOCK_STATE`: the state's number, or zero for none.
    let custom_display = find_metadata_field(metadata, CUSTOM_DISPLAY_BLOCK_DATA_ID)
        .filter(|(serializer, _)| *serializer == OPTIONAL_BLOCK_STATE_SERIALIZER_ID)
        .and_then(|(_, mut value)| voxel_core::wire::read_varint(&mut value).ok())
        .map(|id| u32::try_from(id).ok().filter(|id| *id != 0));
    MinecartData {
        custom_display,
        display_offset: metadata_int_value(metadata, DISPLAY_OFFSET_DATA_ID),
        fuel: metadata_boolean_value(metadata, FUEL_DATA_ID),
    }
}

/// An end crystal's synced data (`EndCrystal.DATA_BEAM_TARGET`, `DATA_SHOW_BOTTOM`), each
/// when the data holds it: the block its beam reaches (none inside), and whether it shows its base.
pub(crate) fn end_crystal_data_from_metadata(
    metadata: &[u8],
) -> (Option<Option<voxel_core::binary::BlockPos>>, Option<bool>) {
    use crate::end_crystal::{BEAM_TARGET_DATA_ID, SHOW_BOTTOM_DATA_ID};
    let beam_target = find_metadata_field(metadata, BEAM_TARGET_DATA_ID).and_then(
        |(serializer_id, value_input)| {
            if serializer_id != ENTITY_DATA_SERIALIZER_OPTIONAL_BLOCK_POS_ID {
                return None;
            }
            let (&present, mut rest) = value_input.split_first()?;
            if present == 0 {
                return Some(None);
            }
            let value = voxel_core::binary::read_blockpos(&mut rest).ok()?;
            Some(Some(value))
        },
    );
    (
        beam_target,
        metadata_boolean_value(metadata, SHOW_BOTTOM_DATA_ID),
    )
}

/// `WitherBoss.DATA_TARGET_B`, `DATA_TARGET_C` and `DATA_ID_INV`.
pub(crate) fn wither_data_from_metadata(metadata: &[u8]) -> WitherData {
    use crate::wither::{INVULNERABLE_DATA_ID, TARGET_B_DATA_ID, TARGET_C_DATA_ID};
    WitherData {
        targets: [
            metadata_int_value(metadata, TARGET_B_DATA_ID),
            metadata_int_value(metadata, TARGET_C_DATA_ID),
        ],
        invulnerable_ticks: metadata_int_value(metadata, INVULNERABLE_DATA_ID),
    }
}

/// A sniffer's synced state (`Sniffer.DATA_STATE`), when the data holds it.
pub(crate) fn sniffer_state_from_metadata(
    type_id: i32,
    metadata: &[u8],
) -> Option<crate::sniffer::SnifferMode> {
    /// `EntityDataSerializers.SNIFFER_STATE`'s place in the registry of serializers.
    const SNIFFER_STATE_SERIALIZER_ID: i32 = 35;
    if voxel_data::entity_type::name(type_id)? != "minecraft:sniffer" {
        return None;
    }
    let (serializer_id, mut value_input) =
        find_metadata_field(metadata, crate::sniffer::STATE_DATA_ID)?;
    if serializer_id != SNIFFER_STATE_SERIALIZER_ID {
        return None;
    }
    let id = voxel_core::wire::read_varint(&mut value_input).ok()?;
    if value_input.is_empty() {
        return None;
    }
    Some(crate::sniffer::SnifferMode::from_id(id))
}

/// An armadillo's synced state (`Armadillo.ARMADILLO_STATE`), when the data holds it.
pub(crate) fn armadillo_state_from_metadata(
    type_id: i32,
    metadata: &[u8],
) -> Option<crate::armadillo::ArmadilloMode> {
    /// `EntityDataSerializers.ARMADILLO_STATE`'s place in the registry of serializers.
    const ARMADILLO_STATE_SERIALIZER_ID: i32 = 36;
    if voxel_data::entity_type::name(type_id)? != "minecraft:armadillo" {
        return None;
    }
    let (serializer_id, mut value_input) =
        find_metadata_field(metadata, crate::armadillo::STATE_DATA_ID)?;
    if serializer_id != ARMADILLO_STATE_SERIALIZER_ID {
        return None;
    }
    let id = voxel_core::wire::read_varint(&mut value_input).ok()?;
    if value_input.is_empty() {
        return None;
    }
    Some(crate::armadillo::ArmadilloMode::from_id(id))
}

/// `Creaking.CAN_MOVE`, `IS_ACTIVE`, `IS_TEARING_DOWN` and whether `HOME_POS` holds a position.
pub(crate) fn creaking_data_from_metadata(metadata: &[u8]) -> CreakingData {
    use crate::creaking::{
        CAN_MOVE_DATA_ID, HOME_POS_DATA_ID, IS_ACTIVE_DATA_ID, IS_TEARING_DOWN_DATA_ID,
    };
    let heart_bound =
        find_metadata_field(metadata, HOME_POS_DATA_ID).and_then(|(serializer_id, value_input)| {
            (serializer_id == ENTITY_DATA_SERIALIZER_OPTIONAL_BLOCK_POS_ID)
                .then(|| value_input.first().map(|&present| present != 0))
                .flatten()
        });
    CreakingData {
        can_move: metadata_boolean_value(metadata, CAN_MOVE_DATA_ID),
        active: metadata_boolean_value(metadata, IS_ACTIVE_DATA_ID),
        tearing_down: metadata_boolean_value(metadata, IS_TEARING_DOWN_DATA_ID),
        heart_bound,
    }
}

/// Whether a guardian moves and the id of what its beam aims at
/// (`Guardian.DATA_ID_MOVING`, `DATA_ID_ATTACK_TARGET`), each when the data holds it.
pub(crate) fn guardian_data_from_metadata(
    type_id: i32,
    metadata: &[u8],
) -> (Option<bool>, Option<i32>) {
    if !matches!(
        voxel_data::entity_type::name(type_id),
        Some("minecraft:guardian" | "minecraft:elder_guardian")
    ) {
        return (None, None);
    }
    (
        metadata_boolean_value(metadata, GUARDIAN_MOVING_DATA_ID),
        metadata_int_value(metadata, GUARDIAN_ATTACK_TARGET_DATA_ID),
    )
}

/// What a salmon, a tropical fish or a pufferfish keeps of its looks in one int
/// (`Salmon.DATA_TYPE`, `TropicalFish.DATA_ID_TYPE_VARIANT`, `Pufferfish.PUFF_STATE`),
/// when the data holds it.
pub(crate) fn fish_variant_from_metadata(type_id: i32, metadata: &[u8]) -> Option<i32> {
    let data_id = match voxel_data::entity_type::name(type_id)? {
        "minecraft:salmon" => SALMON_TYPE_DATA_ID,
        "minecraft:tropical_fish" => TROPICAL_FISH_VARIANT_DATA_ID,
        "minecraft:pufferfish" => PUFFERFISH_PUFF_STATE_DATA_ID,
        _ => return None,
    };
    metadata_int_value(metadata, data_id)
}

/// An axolotl's variant and whether it plays dead (`Axolotl.DATA_VARIANT`,
/// `Axolotl.DATA_PLAYING_DEAD`), each when the data holds it.
pub(crate) fn axolotl_data_from_metadata(
    type_id: i32,
    metadata: &[u8],
) -> (Option<i32>, Option<bool>) {
    if voxel_data::entity_type::name(type_id) != Some("minecraft:axolotl") {
        return (None, None);
    }
    (
        metadata_int_value(metadata, AXOLOTL_VARIANT_DATA_ID),
        metadata_boolean_value(metadata, AXOLOTL_PLAYING_DEAD_DATA_ID),
    )
}

/// Whether a turtle carries an egg and is laying it (`Turtle.HAS_EGG`,
/// `Turtle.LAYING_EGG`), each when the data holds it.
pub(crate) fn turtle_eggs_from_metadata(
    type_id: i32,
    metadata: &[u8],
) -> (Option<bool>, Option<bool>) {
    if voxel_data::entity_type::name(type_id) != Some("minecraft:turtle") {
        return (None, None);
    }
    (
        metadata_boolean_value(metadata, TURTLE_HAS_EGG_DATA_ID),
        metadata_boolean_value(metadata, TURTLE_LAYING_EGG_DATA_ID),
    )
}

/// A hoglin's immunity to zombification (`Hoglin.DATA_IMMUNE_TO_ZOMBIFICATION`),
/// when the data holds it.
pub(crate) fn hoglin_immune_from_metadata(type_id: i32, metadata: &[u8]) -> Option<bool> {
    if voxel_data::entity_type::name(type_id)? != "minecraft:hoglin" {
        return None;
    }
    metadata_boolean_value(metadata, HOGLIN_IMMUNE_DATA_ID)
}

/// What a panda's data holds, each when it is there.
pub(crate) fn panda_data_from_metadata(type_id: i32, metadata: &[u8]) -> crate::panda::PandaData {
    if voxel_data::entity_type::name(type_id) != Some("minecraft:panda") {
        return crate::panda::PandaData::default();
    }
    crate::panda::PandaData {
        unhappy_counter: metadata_int_value(metadata, PANDA_UNHAPPY_DATA_ID),
        sneeze_counter: metadata_int_value(metadata, PANDA_SNEEZE_DATA_ID),
        eat_counter: metadata_int_value(metadata, PANDA_EAT_DATA_ID),
        main_gene: metadata_byte_value(metadata, PANDA_MAIN_GENE_DATA_ID),
        hidden_gene: metadata_byte_value(metadata, PANDA_HIDDEN_GENE_DATA_ID),
        flags: metadata_byte_value(metadata, PANDA_FLAGS_DATA_ID),
    }
}

/// A polar bear's standing flag (`PolarBear.DATA_STANDING_ID`), when the data holds it.
pub(crate) fn polar_bear_standing_from_metadata(type_id: i32, metadata: &[u8]) -> Option<bool> {
    if voxel_data::entity_type::name(type_id)? != "minecraft:polar_bear" {
        return None;
    }
    metadata_boolean_value(metadata, POLAR_BEAR_STANDING_DATA_ID)
}

/// A rabbit's variant (`Rabbit.DATA_TYPE_ID`), when the data holds it.
pub(crate) fn rabbit_type_from_metadata(type_id: i32, metadata: &[u8]) -> Option<i32> {
    if voxel_data::entity_type::name(type_id)? != "minecraft:rabbit" {
        return None;
    }
    metadata_int_value(metadata, RABBIT_TYPE_DATA_ID)
}

/// A mooshroom's variant (`MushroomCow.DATA_TYPE`), when the data holds it.
pub(crate) fn mooshroom_type_from_metadata(type_id: i32, metadata: &[u8]) -> Option<i32> {
    if voxel_data::entity_type::name(type_id)? != "minecraft:mooshroom" {
        return None;
    }
    metadata_int_value(metadata, MOOSHROOM_TYPE_DATA_ID)
}

/// Whether a snow golem wears its pumpkin (`SnowGolem.DATA_PUMPKIN_ID`, bit
/// sixteen), when the data holds it.
pub(crate) fn snow_golem_pumpkin_from_metadata(type_id: i32, metadata: &[u8]) -> Option<bool> {
    if voxel_data::entity_type::name(type_id)? != "minecraft:snow_golem" {
        return None;
    }
    metadata_byte_value(metadata, SNOW_GOLEM_PUMPKIN_DATA_ID).map(|flags| flags & 16 != 0)
}

/// An experience orb's value (`ExperienceOrb.DATA_VALUE`).
pub(crate) fn orb_value_from_metadata(type_id: i32, metadata: &[u8]) -> Option<i32> {
    if voxel_data::entity_type::name(type_id)? != "minecraft:experience_orb" {
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
        let serializer_id = voxel_core::wire::read_varint(&mut metadata).ok()?;
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
