//! Data ids a variant-fields row checks beyond the tables: the bogged's and the mannequin's.

/// The sheared flag of a bogged: a boolean on its own id.
pub(super) fn bogged_field() -> (&'static str, i64, i64) {
    (
        "bogged-sheared",
        i64::from(photon_world::BOGGED_SHEARED_DATA_ID),
        8,
    )
}

/// An avatar's model layers (a byte) and a mannequin's profile, immovable flag (a boolean) and
/// description (an optional component).
pub(super) fn mannequin_fields() -> [(&'static str, i64, i64); 4] {
    use photon_world::{
        AVATAR_CUSTOMISATION_DATA_ID, MANNEQUIN_DESCRIPTION_DATA_ID, MANNEQUIN_IMMOVABLE_DATA_ID,
        MANNEQUIN_PROFILE_DATA_ID, RESOLVABLE_PROFILE_SERIALIZER_ID,
    };
    [
        (
            "avatar-customisation",
            i64::from(AVATAR_CUSTOMISATION_DATA_ID),
            0,
        ),
        (
            "mannequin-profile",
            i64::from(MANNEQUIN_PROFILE_DATA_ID),
            i64::from(RESOLVABLE_PROFILE_SERIALIZER_ID),
        ),
        (
            "mannequin-immovable",
            i64::from(MANNEQUIN_IMMOVABLE_DATA_ID),
            8,
        ),
        (
            "mannequin-description",
            i64::from(MANNEQUIN_DESCRIPTION_DATA_ID),
            6,
        ),
    ]
}
