use super::*;

/// Whether an avatar's main arm is the left (`Avatar.DATA_PLAYER_MAIN_HAND`),
/// when the data holds it.
pub(crate) fn avatar_main_arm_is_left(metadata: &[u8]) -> Option<bool> {
    let (serializer_id, mut value_input) = find_metadata_field(metadata, AVATAR_MAIN_HAND_DATA_ID)?;
    if serializer_id != HUMANOID_ARM_SERIALIZER_ID {
        return None;
    }
    let id = photon_core::wire::read_varint(&mut value_input).ok()?;
    if value_input.is_empty() {
        return None;
    }
    Some(id == 0)
}

pub(crate) fn player_absorption_from_metadata(metadata: &[u8]) -> Option<f32> {
    metadata_float_value(metadata, PLAYER_ABSORPTION_DATA_ID)
}

pub(crate) fn player_model_customisation_from_metadata(metadata: &[u8]) -> Option<u8> {
    metadata_byte_value(metadata, PLAYER_MODE_CUSTOMISATION_DATA_ID)
}

pub(crate) fn player_score_from_metadata(metadata: &[u8]) -> Option<i32> {
    metadata_int_value(metadata, PLAYER_SCORE_DATA_ID)
}

/// The variant of the parrot on a player's left and right shoulder
/// (`Player.DATA_SHOULDER_PARROT_LEFT` and `_RIGHT`), each when the data holds it: none for an
/// empty shoulder.
pub(crate) fn player_shoulder_parrots_from_metadata(metadata: &[u8]) -> [Option<Option<i32>>; 2] {
    [
        metadata_optional_unsigned_int_value(metadata, PLAYER_SHOULDER_PARROT_LEFT_DATA_ID),
        metadata_optional_unsigned_int_value(metadata, PLAYER_SHOULDER_PARROT_RIGHT_DATA_ID),
    ]
}

/// `Player.DATA_SHOULDER_PARROT_LEFT` and `_RIGHT`.
pub(crate) const PLAYER_SHOULDER_PARROT_LEFT_DATA_ID: u8 = 19;
pub(crate) const PLAYER_SHOULDER_PARROT_RIGHT_DATA_ID: u8 = 20;

/// `Avatar.DATA_PLAYER_MODE_CUSTOMISATION`: which skin layers show.
pub(crate) const PLAYER_MODE_CUSTOMISATION_DATA_ID: u8 = 16;

/// `Player.DATA_PLAYER_ABSORPTION_ID`.
pub(crate) const PLAYER_ABSORPTION_DATA_ID: u8 = 17;

/// `Player.DATA_SCORE_ID`: what the death screen shows.
pub(crate) const PLAYER_SCORE_DATA_ID: u8 = 18;

#[cfg(test)]
mod shoulder_tests {
    use super::*;

    #[test]
    fn a_shoulder_holds_the_variant_of_its_parrot() {
        // Left: variant 2 (the wire holds one more); right: empty.
        let data = [19, 19, 3, 20, 19, 0, 0xFF];
        assert_eq!(
            player_shoulder_parrots_from_metadata(&data),
            [Some(Some(2)), Some(None)]
        );
        assert_eq!(player_shoulder_parrots_from_metadata(&[0xFF]), [None, None]);
    }
}
