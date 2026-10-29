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
        let Some(mut entity) = self.entities.at_mut(entity_id) else {
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

/// `Mannequin.DATA_DESCRIPTION`: the line under a mannequin's name (`ClientMannequin.belowNameDisplay`).
#[derive(Debug, Clone, PartialEq)]
pub enum MannequinDescription {
    /// `DEFAULT_DESCRIPTION`, `entity.minecraft.mannequin.label`: the data was never changed.
    Default,
    /// `hide_description`: none.
    Hidden,
    /// A text component of the server's.
    Text(photon_core::nbt::FieldValue),
}

impl crate::ClientEntity {
    /// A mannequin's description; none for any other entity.
    pub fn mannequin_description(&self) -> Option<MannequinDescription> {
        if photon_data::entity_type::name(self.type_id) != Some("minecraft:mannequin") {
            return None;
        }
        let field = self.metadata.as_deref().and_then(|metadata| {
            crate::entity_metadata_field(metadata, crate::MANNEQUIN_DESCRIPTION_DATA_ID)
        });
        let Some((serializer, value)) = field else {
            return Some(MannequinDescription::Default);
        };
        // An optional component: a presence flag, then the component as a tag.
        if serializer != 6 {
            return Some(MannequinDescription::Default);
        }
        let (&present, mut rest) = value.split_first()?;
        if present == 0 {
            return Some(MannequinDescription::Hidden);
        }
        let tag = photon_core::nbt::read_any_nbt(&mut rest).ok()??;
        Some(MannequinDescription::Text(tag))
    }
}
