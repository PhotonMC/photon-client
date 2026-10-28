use super::*;

pub(super) fn resource_vanilla_result(request: &Value) -> Result<Value, String> {
    let namespaces = request["namespaces"]
        .as_array()
        .ok_or("namespaces must be an array")?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| "namespace entries must be strings".to_owned())
                .map(str::to_owned)
        })
        .collect::<Result<Vec<_>, String>>()?;
    let resources = VanillaPackResources::new(namespaces, parse_resource_packs(&request["roots"])?);
    let queries = request["queries"]
        .as_array()
        .ok_or("queries must be an array")?
        .iter()
        .map(|query_value| {
            let id = query_value
                .as_str()
                .ok_or("query entries must be strings")?;
            let mut result = serde_json::Map::new();
            result.insert("id".into(), json!(id));
            if let Some((_root_id, bytes)) = resources.resource(id) {
                result.insert("exists".into(), json!(true));
                result.insert("stack".into(), json!(["vanilla"]));
                result.insert("source".into(), json!("vanilla"));
                result.insert("bytes_hex".into(), json!(hex(bytes)));
            } else {
                result.insert("exists".into(), json!(false));
                result.insert("stack".into(), json!([]));
            }
            Ok(Value::Object(result))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({
        "namespaces": resources.namespaces(),
        "queries": queries,
    }))
}

pub(super) fn resource_known_pack_result(request: &Value) -> Result<Value, String> {
    let packs = request["probes"]
        .as_array()
        .ok_or("probes must be an array")?
        .iter()
        .map(|probe| {
            let pack = if probe["kind"].as_str() == Some("vanilla") {
                KnownPack::vanilla(string(probe, "id")?.to_owned())
            } else {
                KnownPack::new(
                    string(probe, "namespace")?.to_owned(),
                    string(probe, "id")?.to_owned(),
                    string(probe, "version")?.to_owned(),
                )
            };
            Ok(json!({
                "namespace": pack.namespace(),
                "id": pack.id(),
                "version": pack.version(),
                "is_vanilla": pack.is_vanilla(),
                "string": pack.to_string(),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"packs": packs}))
}

pub(super) fn known_pack_codec_error(error: KnownPackCodecError) -> &'static str {
    match error {
        KnownPackCodecError::Read(utf8::ReadError::DecoderError) => "decoder_error",
        KnownPackCodecError::Read(utf8::ReadError::BufferUnderflow) => "buffer_underflow",
        KnownPackCodecError::Read(utf8::ReadError::TooBig) => "too_big",
        KnownPackCodecError::Write(utf8::WriteError::EncoderError) => "encoder_error",
        KnownPackCodecError::InvalidUtf16 => "decoder_error",
    }
}

pub(super) fn resource_known_pack_codec_result(request: &Value) -> Result<Value, String> {
    let prefix = decode_hex(string(request, "prefix_hex")?)?;
    match string(request, "mode")? {
        "encode" => {
            let pack = KnownPack::new(
                string(request, "namespace")?.to_owned(),
                string(request, "id")?.to_owned(),
                string(request, "version")?.to_owned(),
            );
            let mut bytes = prefix.clone();
            let result = match pack.encode_stream(&mut bytes) {
                Ok(()) => json!({
                    "outcome": "value",
                    "written": bytes.len() - prefix.len(),
                    "hex": hex(&bytes),
                }),
                Err(error) => json!({
                    "outcome": "error",
                    "error": known_pack_codec_error(error),
                    "written": bytes.len() - prefix.len(),
                    "hex": hex(&bytes),
                }),
            };
            Ok(result)
        }
        "decode" => {
            let framed = decode_hex(string(request, "hex")?)?;
            let mut remaining = framed.as_slice();
            match KnownPack::decode_stream(&mut remaining) {
                Ok(pack) => Ok(json!({
                    "namespace": pack.namespace(),
                    "id": pack.id(),
                    "version": pack.version(),
                    "string": pack.to_string(),
                    "outcome": "value",
                    "consumed": framed.len() - remaining.len(),
                    "remaining_hex": hex(remaining),
                })),
                Err(error) => Ok(json!({
                    "outcome": "error",
                    "error": known_pack_codec_error(error),
                    "consumed": framed.len() - remaining.len(),
                    "remaining_hex": hex(remaining),
                })),
            }
        }
        _ => Err("unknown KnownPack codec operation".into()),
    }
}

pub(super) fn known_pack_list_codec_error(
    error: KnownPackListCodecError,
    encoding: bool,
) -> &'static str {
    match error {
        KnownPackListCodecError::CountRead(error) => match error {
            wire::ReadError::BufferUnderflow => "buffer_underflow",
            wire::ReadError::TooBig => "too_big",
        },
        KnownPackListCodecError::CountTooBig => {
            if encoding {
                "encoder_error"
            } else {
                "decoder_error"
            }
        }
        KnownPackListCodecError::NegativeCount => "invalid_count",
        KnownPackListCodecError::Pack(error) => known_pack_codec_error(error),
    }
}

pub(super) fn parse_known_pack_list(value: &Value) -> Result<Vec<KnownPack>, String> {
    value
        .as_array()
        .ok_or("packs must be an array")?
        .iter()
        .map(|pack| {
            Ok(KnownPack::new(
                string(pack, "namespace")?.to_owned(),
                string(pack, "id")?.to_owned(),
                string(pack, "version")?.to_owned(),
            ))
        })
        .collect()
}

pub(super) fn known_pack_list_value(packs: &[KnownPack]) -> Value {
    Value::Array(
        packs
            .iter()
            .map(|pack| {
                json!({
                    "namespace": pack.namespace(),
                    "id": pack.id(),
                    "version": pack.version(),
                    "string": pack.to_string(),
                })
            })
            .collect(),
    )
}

pub(super) fn resource_known_pack_list_codec_result(request: &Value) -> Result<Value, String> {
    let packet = string(request, "packet")?;
    let max_size = match packet {
        "clientbound" => i32::MAX as usize,
        "serverbound" => 64,
        _ => return Err("unknown KnownPack list packet".into()),
    };
    let prefix = decode_hex(string(request, "prefix_hex")?)?;
    match string(request, "mode")? {
        "encode" => {
            let packs = parse_known_pack_list(&request["packs"])?;
            let mut bytes = prefix.clone();
            let result = match KnownPack::encode_list(&packs, &mut bytes, max_size) {
                Ok(()) => json!({
                    "outcome": "value",
                    "written": bytes.len() - prefix.len(),
                    "hex": hex(&bytes),
                }),
                Err(error) => json!({
                    "outcome": "error",
                    "error": known_pack_list_codec_error(error, true),
                    "written": bytes.len() - prefix.len(),
                    "hex": hex(&bytes),
                }),
            };
            Ok(result)
        }
        "decode" => {
            let framed = decode_hex(string(request, "hex")?)?;
            let mut remaining = framed.as_slice();
            match KnownPack::decode_list(&mut remaining, max_size) {
                Ok(packs) => Ok(json!({
                    "packs": known_pack_list_value(&packs),
                    "outcome": "value",
                    "consumed": framed.len() - remaining.len(),
                    "remaining_hex": hex(remaining),
                })),
                Err(error) => Ok(json!({
                    "outcome": "error",
                    "error": known_pack_list_codec_error(error, false),
                    "consumed": framed.len() - remaining.len(),
                    "remaining_hex": hex(remaining),
                })),
            }
        }
        _ => Err("unknown KnownPack list codec operation".into()),
    }
}

pub(super) fn input_with_modifiers_snapshot<T: InputWithModifiers>(input: &T) -> Value {
    json!({
        "input": input.input(),
        "modifiers": input.modifiers(),
        "is_selection": input.is_selection(),
        "is_confirmation": input.is_confirmation(),
        "is_escape": input.is_escape(),
        "is_left": input.is_left(),
        "is_right": input.is_right(),
        "is_up": input.is_up(),
        "is_down": input.is_down(),
        "is_cycle_focus": input.is_cycle_focus(),
        "digit": input.get_digit(),
        "has_alt_down": input.has_alt_down(),
        "has_shift_down": input.has_shift_down(),
        "has_control_down": input.has_control_down(),
        "has_control_down_with_quirk": input.has_control_down_with_quirk(),
        "is_select_all": input.is_select_all(),
        "is_copy": input.is_copy(),
        "is_paste": input.is_paste(),
        "is_cut": input.is_cut(),
    })
}

pub(super) fn input_event_result(request: &Value) -> Result<Value, String> {
    let key_event = InputKeyEvent::new(
        i32_field(request, "key")?,
        i32_field(request, "scancode")?,
        i32_field(request, "modifiers")?,
    );
    let button_info = MouseButtonInfo::new(
        i32_field(request, "mouse_button")?,
        i32_field(request, "modifiers")?,
    );
    let mouse_event = MouseButtonEvent::new(
        f64_field(request, "mouse_x")?,
        f64_field(request, "mouse_y")?,
        button_info,
    );
    let mut key = input_with_modifiers_snapshot(&key_event);
    key["key"] = json!(key_event.key);
    key["scancode"] = json!(key_event.scancode);
    let mut info = input_with_modifiers_snapshot(&button_info);
    info["button"] = json!(button_info.button);
    let mut mouse = input_with_modifiers_snapshot(&mouse_event);
    mouse["x"] = json!(format!("{:016x}", mouse_event.x.to_bits()));
    mouse["y"] = json!(format!("{:016x}", mouse_event.y.to_bits()));
    mouse["button"] = json!(mouse_event.button());
    Ok(json!({"key_event": key, "mouse_button_info": info, "mouse_button_event": mouse}))
}

pub(super) fn input_key(value: &Value) -> Result<InputKey, String> {
    let kind = string(value, "type")?;
    let number = i32_field(value, "value")?;
    match kind {
        "keysym" => Ok(InputKey::key_sym(number)),
        "scancode" => Ok(InputKey::scan_code(number)),
        "mouse" => Ok(InputKey::mouse(number)),
        _ => Err("unknown input key type".into()),
    }
}

pub(super) fn mapping_id(
    target: &str,
    primary: usize,
    secondary: usize,
    other: usize,
) -> Result<usize, String> {
    match target {
        "primary" => Ok(primary),
        "secondary" => Ok(secondary),
        "other" => Ok(other),
        _ => Err("unknown key mapping target".into()),
    }
}

pub(super) fn action_key(
    target: &str,
    primary: InputKey,
    primary_default: InputKey,
    secondary: InputKey,
    other: InputKey,
    rebind: InputKey,
) -> Result<InputKey, String> {
    match target {
        "primary" => Ok(primary),
        "primary_default" => Ok(primary_default),
        "secondary" => Ok(secondary),
        "other" => Ok(other),
        "rebind" => Ok(rebind),
        _ => Err("unknown action key".into()),
    }
}

pub(super) fn key_mapping_state(
    registry: &KeyMappingRegistry,
    primary: usize,
    secondary: usize,
    other: usize,
) -> Result<Value, String> {
    let primary_down = registry
        .mapping(primary)
        .ok_or("missing primary key mapping")?
        .is_down();
    let secondary_down = registry
        .mapping(secondary)
        .ok_or("missing secondary key mapping")?
        .is_down();
    let other_down = registry
        .mapping(other)
        .ok_or("missing other key mapping")?
        .is_down();
    Ok(json!({
        "primary_down": primary_down,
        "secondary_down": secondary_down,
        "other_down": other_down,
    }))
}

pub(super) fn key_mapping_property(
    mapping: &KeyMapping,
    key_event: (i32, i32),
    unknown_key_event: (i32, i32),
    mouse_button: i32,
) -> Value {
    json!({
        "name": mapping.name(),
        "key": mapping.key().name(),
        "default_key": mapping.default_key().name(),
        "is_toggle": mapping.is_toggle(),
        "is_default": mapping.is_default(),
        "is_unbound": mapping.is_unbound(),
        "save": mapping.save_string(),
        "matches_key_event": mapping.matches_key_event(key_event.0, key_event.1),
        "matches_unknown_key_event": mapping.matches_key_event(unknown_key_event.0, unknown_key_event.1),
        "matches_mouse": mapping.matches_mouse(mouse_button),
    })
}

pub(super) fn key_mapping_properties(
    registry: &KeyMappingRegistry,
    [primary, secondary, other]: [usize; 3],
    key_event: (i32, i32),
    unknown_key_event: (i32, i32),
    mouse_button: i32,
) -> Result<Value, String> {
    let primary_mapping = registry
        .mapping(primary)
        .ok_or("missing primary key mapping")?;
    let secondary_mapping = registry
        .mapping(secondary)
        .ok_or("missing secondary key mapping")?;
    let other_mapping = registry.mapping(other).ok_or("missing other key mapping")?;
    Ok(json!({
        "primary": key_mapping_property(primary_mapping, key_event, unknown_key_event, mouse_button),
        "secondary": key_mapping_property(secondary_mapping, key_event, unknown_key_event, mouse_button),
        "other": key_mapping_property(other_mapping, key_event, unknown_key_event, mouse_button),
        "primary_same_secondary": primary_mapping.same(secondary_mapping),
        "primary_same_other": primary_mapping.same(other_mapping),
    }))
}
