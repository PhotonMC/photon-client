use super::*;

pub(super) fn player_abilities_packet_case(request: &Value) -> Result<Value, String> {
    let inputs = request
        .get("inputs")
        .ok_or("inputs must be an array")?
        .as_array()
        .ok_or("inputs must be an array")?;
    let states = inputs
        .iter()
        .map(|value| {
            let input = value
                .as_object()
                .ok_or("player abilities input must be an object")?;
            let bytes = decode_hex(
                input
                    .get("hex")
                    .and_then(Value::as_str)
                    .ok_or("hex must be a string")?,
            )?;
            let packet = parse_player_abilities_packet(&bytes)
                .map_err(|error| format!("player abilities packet: {error:?}"))?;
            let abilities = ClientPlayerAbilities::new(
                packet.invulnerable,
                packet.flying,
                packet.can_fly,
                packet.can_instabuild,
                packet.flying_speed,
                packet.walking_speed,
            );
            Ok(json!({
                "label": input
                    .get("label")
                    .and_then(Value::as_str)
                    .ok_or("label must be a string")?,
                "invulnerable": abilities.invulnerable(),
                "flying": abilities.flying(),
                "may_fly": abilities.may_fly(),
                "instabuild": abilities.instabuild(),
                "flying_speed": format!("{:08x}", abilities.flying_speed().to_bits()),
                "walking_speed": format!("{:08x}", abilities.walking_speed().to_bits()),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"states": states}))
}

pub(super) fn player_game_mode_event_case(request: &Value) -> Result<Value, String> {
    let inputs = request
        .get("inputs")
        .ok_or("inputs must be an array")?
        .as_array()
        .ok_or("inputs must be an array")?;
    let states = inputs
        .iter()
        .map(|value| {
            let input = value
                .as_object()
                .ok_or("player game mode input must be an object")?;
            let bytes = decode_hex(
                input
                    .get("hex")
                    .and_then(Value::as_str)
                    .ok_or("hex must be a string")?,
            )?;
            let packet = parse_game_event_packet(&bytes)
                .map_err(|error| format!("game event packet: {error:?}"))?;
            if packet.event_id != 3 {
                return Err("expected change-game-mode event".into());
            }
            let flying_speed = input
                .get("initial_flying_speed")
                .map(|value| f32_value(value, "initial_flying_speed"))
                .transpose()?
                .unwrap_or(0.05);
            let walking_speed = input
                .get("initial_walking_speed")
                .map(|value| f32_value(value, "initial_walking_speed"))
                .transpose()?
                .unwrap_or(0.1);
            let abilities = ClientPlayerAbilities::new(
                false,
                input
                    .get("initial_flying")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                false,
                false,
                flying_speed,
                walking_speed,
            );
            let mode = ClientPlayerGameMode::from_game_event_param(packet.param);
            let abilities = mode.update_player_abilities(abilities);
            Ok(json!({
                "label": input
                    .get("label")
                    .and_then(Value::as_str)
                    .ok_or("label must be a string")?,
                "param": format!("{:08x}", packet.param.to_bits()),
                "mode_id": mode.id(),
                "mode_name": mode.name(),
                "is_spectator": mode.is_spectator(),
                "is_creative": mode.is_creative(),
                "is_survival": mode.is_survival(),
                "may_build": abilities.may_build(),
                "invulnerable": abilities.invulnerable(),
                "flying": abilities.flying(),
                "may_fly": abilities.may_fly(),
                "instabuild": abilities.instabuild(),
                "flying_speed": format!("{:08x}", abilities.flying_speed().to_bits()),
                "walking_speed": format!("{:08x}", abilities.walking_speed().to_bits()),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"states": states}))
}
