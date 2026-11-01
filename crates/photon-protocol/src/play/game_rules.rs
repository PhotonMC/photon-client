//! The game rule packets of the in-world rule screen.

use super::*;

/// `ClientboundGameRuleValuesPacket`: each rule's key and its value as the server writes it.
pub struct GameRuleValues(pub Vec<(String, String)>);

impl Decode for GameRuleValues {
    fn decode(
        payload: &[u8],
        _registries: &photon_data::version_registries::VersionRegistries,
    ) -> Result<Self, ConnectionError> {
        let mut input = payload;
        let count = read_count(&mut input, "game rule count")?;
        let mut values = Vec::with_capacity(count.min(256));
        for _ in 0..count {
            let key = read_identifier_string(&mut input, "game rule key")?;
            let units = photon_core::utf8::read_utf8(&mut input, 32_767)
                .map_err(|_| ConnectionError::Codec("game rule value"))?;
            let value = String::from_utf16(&units)
                .map_err(|_| ConnectionError::Codec("game rule value"))?;
            values.push((key, value));
        }
        if !input.is_empty() {
            return Err(ConnectionError::Codec("game rule values trailing payload"));
        }
        Ok(Self(values))
    }
}

/// `ServerboundClientCommandPacket` with `REQUEST_GAMERULE_VALUES`.
pub fn encode_request_game_rule_values(output: &mut Vec<u8>) -> Result<(), ConnectionError> {
    output.push(2);
    Ok(())
}

/// `ServerboundSetGameRulePacket`: each changed rule's key and its new value as text.
pub fn encode_set_game_rules(
    output: &mut Vec<u8>,
    entries: &[(String, String)],
) -> Result<(), ConnectionError> {
    photon_core::wire::write_varint(entries.len() as i32, output);
    for (key, value) in entries {
        let identifier = photon_core::identifier::Identifier::parse(key)
            .map_err(|_| ConnectionError::Codec("game rule key"))?;
        photon_core::identifier::write_identifier(&identifier, output)
            .map_err(|_| ConnectionError::Codec("game rule key"))?;
        let units: Vec<u16> = value.encode_utf16().collect();
        photon_core::utf8::write_utf8(output, &units, 32_767)
            .map_err(|_| ConnectionError::Codec("game rule value"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_round_trip_through_the_map_layout() {
        let mut bytes = Vec::new();
        encode_set_game_rules(
            &mut bytes,
            &[
                ("minecraft:pvp".into(), "false".into()),
                ("minecraft:random_tick_speed".into(), "7".into()),
            ],
        )
        .unwrap();
        // The values packet is the same layout: a count, then key and text per entry.
        let decoded = GameRuleValues::decode(
            &bytes,
            &photon_data::version_registries::VersionRegistries::without_gaps(),
        )
        .unwrap();
        assert_eq!(decoded.0.len(), 2);
        assert_eq!(
            decoded.0[1],
            ("minecraft:random_tick_speed".into(), "7".into())
        );
    }

    #[test]
    fn the_request_is_the_third_client_command() {
        let mut bytes = Vec::new();
        encode_request_game_rule_values(&mut bytes).unwrap();
        assert_eq!(bytes, [2]);
    }
}
