use super::*;

/// The block state an enderman carries (`DATA_CARRY_STATE`, zero for none)
/// and whether it is creepy (`DATA_CREEPY`), when the data holds them.
pub(crate) fn enderman_flags_from_metadata(
    type_id: EntityTypeId,
    metadata: &[u8],
) -> (Option<i32>, Option<bool>) {
    if type_id != photon_data::entity_type::ids::ENDERMAN {
        return (None, None);
    }
    let carrying = find_metadata_field(metadata, ENDERMAN_CARRY_DATA_ID)
        .filter(|(serializer, _)| *serializer == OPTIONAL_BLOCK_STATE_SERIALIZER_ID)
        .and_then(|(_, mut value)| photon_core::wire::read_varint(&mut value).ok());
    (
        carrying,
        metadata_boolean_value(metadata, ENDERMAN_CREEPY_DATA_ID),
    )
}

/// Whether an enderman has been stared at (`DATA_STARED_AT`), when the data holds it.
pub(crate) fn enderman_stared_at_from_metadata(
    type_id: EntityTypeId,
    metadata: &[u8],
) -> Option<bool> {
    (type_id == photon_data::entity_type::ids::ENDERMAN)
        .then(|| metadata_boolean_value(metadata, ENDERMAN_STARED_AT_DATA_ID))
        .flatten()
}

impl crate::entity::EntityViewMut<'_> {
    /// What an enderman's data says: the block it carries, and the two flags. The data is
    /// assigned in the order of its ids, so the callback of the creepy flag sees the stared-at
    /// flag as it was before this packet.
    pub(crate) fn apply_enderman_data(&mut self, metadata: &[u8]) {
        let (carrying, creepy) = enderman_flags_from_metadata(self.type_id, metadata);
        if let Some(carried) = carrying {
            self.living.carried_state = carried;
        }
        if let Some(creepy) = creepy {
            self.set_creepy(creepy);
        }
        if let Some(stared) = enderman_stared_at_from_metadata(self.type_id, metadata) {
            self.living.stared_at = stared;
        }
    }
}
