use super::*;

pub(super) fn f32_value(value: &Value, name: &str) -> Result<f32, String> {
    let text = value
        .as_str()
        .ok_or_else(|| format!("{name} must be a string"))?;
    if text.len() != 8 || !text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{name} must be eight hexadecimal digits"));
    }
    u32::from_str_radix(text, 16)
        .map(f32::from_bits)
        .map_err(|error| error.to_string())
}

pub(super) fn millis(value: &Value, name: &str) -> Result<i64, String> {
    string(value, name)?
        .parse::<i64>()
        .map_err(|error| error.to_string())
}

pub(super) fn f64_field(value: &Value, name: &str) -> Result<f64, String> {
    let text = string(value, name)?;
    if text.len() != 16 || !text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{name} must be sixteen hexadecimal digits"));
    }
    u64::from_str_radix(text, 16)
        .map(f64::from_bits)
        .map_err(|error| error.to_string())
}

pub(super) fn u64_hex_field(value: &Value, name: &str) -> Result<u64, String> {
    let text = string(value, name)?;
    if text.len() != 16 || !text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{name} must be sixteen hexadecimal digits"));
    }
    u64::from_str_radix(text, 16).map_err(|error| error.to_string())
}

pub(super) fn f64_value(value: &Value, name: &str) -> Result<f64, String> {
    let text = value
        .as_str()
        .ok_or_else(|| format!("{name} must be a string"))?;
    if text.len() != 16 || !text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{name} must be sixteen hexadecimal digits"));
    }
    u64::from_str_radix(text, 16)
        .map(f64::from_bits)
        .map_err(|error| error.to_string())
}

pub(super) fn vec2_snapshot(vector: Vec2) -> Value {
    json!({
        "x": format!("{:08x}", vector.x.to_bits()),
        "y": format!("{:08x}", vector.y.to_bits()),
        "length": format!("{:08x}", vector.length().to_bits()),
        "length_squared": format!("{:08x}", vector.length_squared().to_bits()),
    })
}

pub(super) fn input_snapshot(input: Input) -> Value {
    json!({
        "forward": input.forward,
        "backward": input.backward,
        "left": input.left,
        "right": input.right,
        "jump": input.jump,
        "shift": input.shift,
        "sprint": input.sprint,
    })
}

pub(super) fn utf16_snapshot(units: &[u16]) -> Value {
    Value::Array(
        units
            .iter()
            .map(|unit| json!(format!("{unit:04x}")))
            .collect(),
    )
}

pub(super) fn character_event_result(request: &Value) -> Result<Value, String> {
    let event = CharacterEvent::new(i32_field(request, "codepoint")?);
    let units = event
        .codepoint_as_utf16()
        .map_err(|error| format!("invalid codepoint: {}", error.0))?;
    Ok(json!({
        "codepoint": event.codepoint,
        "utf16": utf16_snapshot(&units),
        "is_allowed_chat_character": event.is_allowed_chat_character(),
    }))
}

pub(super) fn i32_array_field(value: &Value, name: &str) -> Result<Vec<i32>, String> {
    value[name]
        .as_array()
        .ok_or_else(|| format!("{name} must be an array"))?
        .iter()
        .map(|entry| {
            entry
                .as_i64()
                .ok_or_else(|| format!("{name} entries must be integers"))?
                .try_into()
                .map_err(|_| format!("{name} entry must fit in i32"))
        })
        .collect()
}

pub(super) fn preedit_snapshot(event: &PreeditEvent) -> Value {
    json!({
        "full_text": utf16_snapshot(&event.full_text),
        "caret_position": event.caret_position,
        "blocks": event.blocks.iter().map(|block| utf16_snapshot(block)).collect::<Vec<_>>(),
        "focused_block": event.focused_block,
    })
}

pub(super) fn preedit_event_result(request: &Value) -> Result<Value, String> {
    let codepoints = i32_array_field(request, "codepoints")?;
    let block_sizes = i32_array_field(request, "block_sizes")?;
    let preedit_size = i32::try_from(codepoints.len())
        .map_err(|_| "codepoint array is too large for Java int".to_owned())?;
    let event = PreeditEvent::create_from_callback(
        preedit_size,
        &codepoints,
        &block_sizes,
        i32_field(request, "focused_block")?,
        i32_field(request, "caret")?,
    );
    Ok(match event {
        None => json!({}),
        Some(Ok(event)) => json!({"event": preedit_snapshot(&event)}),
        Some(Err(error)) => return Err(format!("preedit event error: {error:?}")),
    })
}

pub(super) fn option_values_result(request: &Value) -> Result<Value, String> {
    let range = IntRange::with_apply_value_immediately(
        i32_field(request, "min")?,
        i32_field(request, "max")?,
        request["apply_value_immediately"]
            .as_bool()
            .ok_or("apply_value_immediately must be a boolean")?,
    );
    let int_values = request["int_values"]
        .as_array()
        .ok_or("int_values must be an array")?
        .iter()
        .map(|value| {
            let value = value
                .as_i64()
                .ok_or("int_values entries must be integers")?
                .try_into()
                .map_err(|_| "int_values entry must fit in i32")?;
            Ok(json!({
                "value": value,
                "valid": range.validate_value(value).is_some(),
                "next": range.next(value),
                "previous": range.previous(value),
                "to_slider": format!("{:016x}", range.to_slider_value(value).to_bits()),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let sliders = request["sliders"]
        .as_array()
        .ok_or("sliders must be an array")?
        .iter()
        .map(|value| {
            let slider = f64_value(value, "slider")?;
            Ok(json!({
                "slider": format!("{:016x}", slider.to_bits()),
                "from_slider": range.from_slider_value(slider),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let unit_values = request["unit_values"]
        .as_array()
        .ok_or("unit_values must be an array")?
        .iter()
        .map(|value| {
            let value = f64_value(value, "unit_value")?;
            let mut state = serde_json::Map::new();
            state.insert("value".into(), json!(format!("{:016x}", value.to_bits())));
            state.insert(
                "valid".into(),
                json!(UnitDouble::validate_value(value).is_some()),
            );
            if let Some(validated) = UnitDouble::validate_value(value) {
                state.insert(
                    "validated".into(),
                    json!(format!("{:016x}", validated.to_bits())),
                );
            }
            state.insert(
                "to_slider".into(),
                json!(format!(
                    "{:016x}",
                    UnitDouble::to_slider_value(value).to_bits()
                )),
            );
            state.insert(
                "from_slider".into(),
                json!(format!(
                    "{:016x}",
                    UnitDouble::from_slider_value(value).to_bits()
                )),
            );
            Ok(Value::Object(state))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({
        "min": range.min_inclusive,
        "max": range.max_inclusive,
        "apply_value_immediately": range.apply_value_immediately,
        "int_values": int_values,
        "sliders": sliders,
        "unit_values": unit_values,
    }))
}

pub(super) fn sliderable_enum_result(request: &Value) -> Result<Value, String> {
    const GRAPHICS_PRESETS: [&str; 4] = ["FAST", "FANCY", "FABULOUS", "CUSTOM"];
    let value_set = SliderableEnum::new(
        GRAPHICS_PRESETS
            .iter()
            .map(|value| (*value).to_owned())
            .collect(),
    );
    let probes = request["probes"]
        .as_array()
        .ok_or("probes must be an array")?
        .iter()
        .map(|probe| {
            let value = string(probe, "value")?.to_owned();
            let next = value_set.next(&value);
            let previous = value_set.previous(&value);
            Ok(json!({
                "value": value,
                "valid": value_set.validate_value(&value).is_some(),
                "to_slider": format!("{:016x}", value_set.to_slider_value(&value).to_bits()),
                "next": next,
                "previous": previous,
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let sliders = request["sliders"]
        .as_array()
        .ok_or("sliders must be an array")?
        .iter()
        .map(|entry| {
            let slider = f64_value(entry, "slider")?;
            Ok(json!({
                "slider": format!("{:016x}", slider.to_bits()),
                "from_slider": value_set.from_slider_value(slider),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({
        "values": value_set.values(),
        "probes": probes,
        "sliders": sliders,
    }))
}

pub(super) fn cycleable_enum_result(request: &Value) -> Result<Value, String> {
    let values = request["values"]
        .as_array()
        .ok_or("values must be an array")?
        .iter()
        .map(|value| string_value(value, "values entry"))
        .collect::<Result<Vec<_>, String>>()?;
    let value_set = CycleableEnum::new(values);
    let probes = request["probes"]
        .as_array()
        .ok_or("probes must be an array")?
        .iter()
        .map(|probe| {
            let value = string_value(probe, "probe")?;
            let mut state = serde_json::Map::new();
            state.insert("value".into(), json!(&value));
            state.insert(
                "valid".into(),
                json!(value_set.validate_value(&value).is_some()),
            );
            if let Some(validated) = value_set.validate_value(&value) {
                state.insert("validated".into(), json!(validated));
            }
            Ok(Value::Object(state))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({
        "values": value_set.values(),
        "selected_values": value_set.values(),
        "probes": probes,
    }))
}

pub(super) fn alt_enum_result(request: &Value) -> Result<Value, String> {
    let values = request["values"]
        .as_array()
        .ok_or("values must be an array")?
        .iter()
        .map(|value| string_value(value, "values entry"))
        .collect::<Result<Vec<_>, String>>()?;
    let alt_values = request["alt_values"]
        .as_array()
        .ok_or("alt_values must be an array")?
        .iter()
        .map(|value| string_value(value, "alt_values entry"))
        .collect::<Result<Vec<_>, String>>()?;
    let conditions = request["conditions"]
        .as_array()
        .ok_or("conditions must be an array")?
        .iter()
        .map(|value| {
            value
                .as_bool()
                .ok_or_else(|| "condition entries must be booleans".to_owned())
        })
        .collect::<Result<Vec<_>, String>>()?;
    if conditions.is_empty() {
        return Err("conditions must not be empty".into());
    }
    let supplied_conditions = conditions.clone();
    let condition_calls = Rc::new(RefCell::new(0_usize));
    let condition_counter = Rc::clone(&condition_calls);
    let value_set = AltEnum::new(values, alt_values, move || {
        let mut call = condition_counter.borrow_mut();
        let index = (*call).min(supplied_conditions.len() - 1);
        *call += 1;
        supplied_conditions[index]
    });
    let mut selected_values = Vec::with_capacity(conditions.len());
    for _ in &conditions {
        selected_values.push(value_set.selected_values());
    }
    let probes = request["probes"]
        .as_array()
        .ok_or("probes must be an array")?
        .iter()
        .map(|probe| {
            let value = string_value(probe, "probe")?;
            let mut state = serde_json::Map::new();
            state.insert("value".into(), json!(&value));
            let validated = value_set.validate_value(&value);
            state.insert("valid".into(), json!(validated.is_some()));
            if let Some(validated) = validated {
                state.insert("validated".into(), json!(validated));
            }
            Ok(Value::Object(state))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({
        "default_values": value_set.values(),
        "alt_values": value_set.alt_values(),
        "selected_values": selected_values,
        "probes": probes,
    }))
}

pub(super) fn lazy_enum_result(request: &Value) -> Result<Value, String> {
    let value_lists = request["value_lists"]
        .as_array()
        .ok_or("value_lists must be an array")?
        .iter()
        .map(|values| {
            values
                .as_array()
                .ok_or("value list must be an array")?
                .iter()
                .map(|value| string_value(value, "value list entry"))
                .collect::<Result<Vec<_>, String>>()
        })
        .collect::<Result<Vec<_>, String>>()?;
    if value_lists.is_empty() {
        return Err("value_lists must not be empty".into());
    }
    let valid_values = request["valid_values"]
        .as_array()
        .ok_or("valid_values must be an array")?
        .iter()
        .map(|value| string_value(value, "valid value"))
        .collect::<Result<Vec<_>, String>>()?;
    let calls = Rc::new(RefCell::new(0_usize));
    let supplier_calls = Rc::clone(&calls);
    let supplied_lists = value_lists.clone();
    let value_set = LazyEnum::new(
        move || {
            let mut call = supplier_calls.borrow_mut();
            let index = (*call).min(supplied_lists.len() - 1);
            *call += 1;
            supplied_lists[index].clone()
        },
        move |value: &String| {
            valid_values
                .iter()
                .any(|candidate| candidate == value)
                .then(|| value.clone())
        },
    );
    let first_values = value_set.values();
    let second_values = value_set.values();
    let probes = request["probes"]
        .as_array()
        .ok_or("probes must be an array")?
        .iter()
        .map(|probe| {
            let value = string_value(probe, "probe")?;
            let mut state = serde_json::Map::new();
            state.insert("value".into(), json!(&value));
            state.insert(
                "valid".into(),
                json!(value_set.validate_value(&value).is_some()),
            );
            if let Some(validated) = value_set.validate_value(&value) {
                state.insert("validated".into(), json!(validated));
            }
            Ok(Value::Object(state))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({
        "first_default": first_values,
        "first_selected": first_values.clone(),
        "second_default": second_values,
        "second_selected": second_values.clone(),
        "probes": probes,
    }))
}
