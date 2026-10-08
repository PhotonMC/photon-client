use super::*;

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

pub(crate) fn minecart_data_from_metadata(metadata: &[u8]) -> MinecartData {
    use crate::objects::minecart::{
        CUSTOM_DISPLAY_BLOCK_DATA_ID, DISPLAY_OFFSET_DATA_ID, FUEL_DATA_ID,
    };
    // `OPTIONAL_BLOCK_STATE`: the state's number, or zero for none.
    let custom_display = find_metadata_field(metadata, CUSTOM_DISPLAY_BLOCK_DATA_ID)
        .filter(|(serializer, _)| *serializer == OPTIONAL_BLOCK_STATE_SERIALIZER_ID)
        .and_then(|(_, mut value)| photon_core::wire::read_varint(&mut value).ok())
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
) -> (Option<Option<photon_core::binary::BlockPos>>, Option<bool>) {
    use crate::objects::end_crystal::{BEAM_TARGET_DATA_ID, SHOW_BOTTOM_DATA_ID};
    let beam_target = find_metadata_field(metadata, BEAM_TARGET_DATA_ID).and_then(
        |(serializer_id, value_input)| {
            if serializer_id != ENTITY_DATA_SERIALIZER_OPTIONAL_BLOCK_POS_ID {
                return None;
            }
            let (&present, mut rest) = value_input.split_first()?;
            if present == 0 {
                return Some(None);
            }
            let value = photon_core::binary::read_blockpos(&mut rest).ok()?;
            Some(Some(value))
        },
    );
    (
        beam_target,
        metadata_boolean_value(metadata, SHOW_BOTTOM_DATA_ID),
    )
}

/// An abstract horse's flags (`AbstractHorse.DATA_ID_FLAGS`), a horse's
/// variant data (`Horse.DATA_ID_TYPE_VARIANT`) and a chested horse's chest
/// (`AbstractChestedHorse.DATA_ID_CHEST`), when the data holds them.
pub(crate) fn horse_data_from_metadata(
    type_id: i32,
    metadata: &[u8],
) -> (Option<u8>, Option<i32>, Option<bool>) {
    let Some(name) = photon_data::entity_type::name(type_id) else {
        return (None, None, None);
    };
    let horselike = matches!(
        name,
        "minecraft:horse"
            | "minecraft:donkey"
            | "minecraft:mule"
            | "minecraft:skeleton_horse"
            | "minecraft:zombie_horse"
            | "minecraft:llama"
            | "minecraft:trader_llama"
            | "minecraft:camel"
            | "minecraft:camel_husk"
    );
    if !horselike {
        return (None, None, None);
    }
    let variant = match name {
        "minecraft:horse" => metadata_int_value(metadata, HORSE_VARIANT_DATA_ID),
        "minecraft:llama" | "minecraft:trader_llama" => {
            metadata_int_value(metadata, LLAMA_VARIANT_DATA_ID)
        }
        _ => None,
    };
    let chest = matches!(
        name,
        "minecraft:donkey" | "minecraft:mule" | "minecraft:llama" | "minecraft:trader_llama"
    )
    .then(|| metadata_boolean_value(metadata, HORSE_CHEST_DATA_ID))
    .flatten();
    (
        metadata_byte_value(metadata, HORSE_FLAGS_DATA_ID),
        variant,
        chest,
    )
}

/// A camel's dash flag (`Camel.DASH`) and the tick its pose last changed
/// (`LAST_POSE_CHANGE_TICK`), when the data holds them.
pub(crate) fn camel_data_from_metadata(
    type_id: i32,
    metadata: &[u8],
) -> (Option<bool>, Option<i64>) {
    if !matches!(
        photon_data::entity_type::name(type_id),
        Some("minecraft:camel" | "minecraft:camel_husk")
    ) {
        return (None, None);
    }
    let last_pose = find_metadata_field(metadata, CAMEL_LAST_POSE_CHANGE_DATA_ID)
        .filter(|(serializer, _)| *serializer == LONG_SERIALIZER_ID)
        .and_then(|(_, mut value)| {
            let tick = photon_core::wire::read_varlong(&mut value).ok()?;
            (!value.is_empty()).then_some(tick)
        });
    (
        metadata_boolean_value(metadata, CAMEL_DASH_DATA_ID),
        last_pose,
    )
}

/// A strider's suffocating flag (`Strider.DATA_SUFFOCATING`), when the data
/// holds it.
pub(crate) fn strider_suffocating_from_metadata(type_id: i32, metadata: &[u8]) -> Option<bool> {
    if photon_data::entity_type::name(type_id)? != "minecraft:strider" {
        return None;
    }
    metadata_boolean_value(metadata, STRIDER_SUFFOCATING_DATA_ID)
}

/// What a boat's data holds of its hit, paddles and bubbles.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct BoatData {
    pub(crate) hurt_time: Option<i32>,
    pub(crate) hurt_dir: Option<i32>,
    pub(crate) damage: Option<f32>,
    pub(crate) paddle_left: Option<bool>,
    pub(crate) paddle_right: Option<bool>,
    pub(crate) bubble_time: Option<i32>,
}

/// A minecart's own synced data (`AbstractMinecart.DATA_ID_CUSTOM_DISPLAY_BLOCK`,
/// `DATA_ID_DISPLAY_OFFSET`, `MinecartFurnace.DATA_ID_FUEL`), each when the data holds it.
/// The custom block is a block state number, none inside when the server names none.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct MinecartData {
    pub(super) custom_display: Option<Option<u32>>,
    pub(super) display_offset: Option<i32>,
    pub(super) fuel: Option<bool>,
}

impl MinecartData {
    pub(crate) fn apply(self, state: &mut crate::objects::minecart::MinecartState) {
        state.custom_display = self.custom_display.unwrap_or(state.custom_display);
        state.display_offset = self.display_offset.or(state.display_offset);
        state.fuel = self.fuel.unwrap_or(state.fuel);
    }
}
