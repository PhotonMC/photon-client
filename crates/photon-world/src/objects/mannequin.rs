//! What the client keeps of a mannequin beyond an avatar's data: the profile its skin comes from
//! (`Mannequin.DATA_PROFILE`, a `ResolvableProfile` as the wire carries it).

use photon_core::ids::EntityId;

use crate::{ClientWorld, MANNEQUIN_PROFILE_DATA_ID, RESOLVABLE_PROFILE_SERIALIZER_ID};

/// A mannequin's profile, as the bytes that follow its serializer (the profile's stream codec
/// reads from their start and leaves the rest).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MannequinState {
    pub profile: Vec<u8>,
}

impl ClientWorld {
    /// `Mannequin.onSyncedDataUpdated` of the profile: the new profile is kept for the skin.
    pub fn apply_mannequin_metadata(&mut self, entity_id: EntityId, metadata: &[u8]) {
        let Some((serializer, value)) =
            crate::entity_metadata_field(metadata, MANNEQUIN_PROFILE_DATA_ID)
        else {
            return;
        };
        let Some(entity) = self.entities.at_mut(entity_id) else {
            return;
        };
        if serializer != RESOLVABLE_PROFILE_SERIALIZER_ID
            || photon_data::entity_type::name(entity.type_id) != Some("minecraft:mannequin")
        {
            return;
        }
        let state = entity
            .kind
            .mannequin_or_insert_with(|| Box::new(MannequinState::default()));
        state.profile = value.to_vec();
    }
}

impl crate::ClientEntity {
    /// The bytes of a mannequin's profile, once the server has sent one.
    pub fn mannequin_profile(&self) -> Option<&[u8]> {
        self.kind.mannequin().map(|state| state.profile.as_slice())
    }
}
