use super::*;

/// Whether a goat has its left and its right horn (`Goat.DATA_HAS_LEFT_HORN`,
/// `DATA_HAS_RIGHT_HORN`), when the data holds them.
pub(crate) fn goat_horns_from_metadata(
    type_id: EntityTypeId,
    metadata: &[u8],
) -> (Option<bool>, Option<bool>) {
    if photon_data::entity_type::name(type_id) != Some("minecraft:goat") {
        return (None, None);
    }
    (
        metadata_boolean_value(metadata, GOAT_LEFT_HORN_DATA_ID),
        metadata_boolean_value(metadata, GOAT_RIGHT_HORN_DATA_ID),
    )
}

/// Whether a goat screams (`Goat.DATA_IS_SCREAMING_GOAT`), when the data holds it.
pub(crate) fn goat_screaming_from_metadata(type_id: EntityTypeId, metadata: &[u8]) -> Option<bool> {
    if photon_data::entity_type::name(type_id)? != "minecraft:goat" {
        return None;
    }
    metadata_boolean_value(metadata, GOAT_SCREAMING_DATA_ID)
}

/// A goat's horns and screaming flag in its data, applied to the entity.
pub(crate) fn apply_goat_data(entity: &mut crate::entity::EntityViewMut<'_>, metadata: &[u8]) {
    let (left, right) = goat_horns_from_metadata(entity.type_id, metadata);
    if let Some(left) = left {
        entity.living.lost_left_horn = !left;
    }
    if let Some(right) = right {
        entity.living.lost_right_horn = !right;
    }
    if let Some(screaming) = goat_screaming_from_metadata(entity.type_id, metadata) {
        entity.living.screaming_goat = screaming;
    }
}

/// An ocelot's trusting flag (`Ocelot.DATA_TRUSTING`), applied to the entity when the data holds
/// it. The flag is read only for ocelots, so another species' boolean at the same place is inert.
pub(crate) fn apply_ocelot_data(entity: &mut crate::entity::EntityViewMut<'_>, metadata: &[u8]) {
    if let Some(trusting) = metadata_boolean_value(metadata, OCELOT_TRUSTING_DATA_ID) {
        entity.living.trusting_ocelot = trusting;
    }
}
