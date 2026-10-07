use super::*;

pub(super) fn clamping_lazy_max_int_range_result(request: &Value) -> Result<Value, String> {
    let max_values = request["max_values"]
        .as_array()
        .ok_or("max_values must be an array")?
        .iter()
        .map(|value| {
            value
                .as_i64()
                .ok_or("max_values entries must be integers")?
                .try_into()
                .map_err(|_| "max_values entry must fit in i32".to_owned())
        })
        .collect::<Result<Vec<i32>, String>>()?;
    if max_values.is_empty() {
        return Err("max_values must not be empty".into());
    }
    let supplied_maxima = max_values.clone();
    let max_calls = Rc::new(RefCell::new(0_usize));
    let supplier_calls = Rc::clone(&max_calls);
    let range = ClampingLazyMaxIntRange::new(
        i32_field(request, "min")?,
        move || {
            let mut call = supplier_calls.borrow_mut();
            let index = (*call).min(supplied_maxima.len() - 1);
            *call += 1;
            supplied_maxima[index]
        },
        i32_field(request, "encodable_max")?,
    );
    let maxima = vec![range.max_inclusive(), range.max_inclusive()];
    let probes = request["probes"]
        .as_array()
        .ok_or("probes must be an array")?
        .iter()
        .map(|entry| {
            let value = entry
                .as_i64()
                .ok_or("probe entries must be integers")?
                .try_into()
                .map_err(|_| "probe entry must fit in i32".to_owned())?;
            Ok(json!({
                "value": value,
                "validated": range.validate_value(value),
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
        .map(|entry| {
            let slider = f64_value(entry, "slider")?;
            Ok(json!({
                "slider": format!("{:016x}", slider.to_bits()),
                "from_slider": range.from_slider_value(slider),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let values = range.values();
    Ok(json!({
        "min": range.min_inclusive(),
        "encodable_max": range.encodable_max_inclusive(),
        "maxima": maxima,
        "create_cycle_button": range.create_cycle_button(),
        "values": values.clone(),
        "selected_values": values,
        "probes": probes,
        "sliders": sliders,
    }))
}

pub(super) fn mapped_int_range_result(request: &Value) -> Result<Value, String> {
    let range = IntRange::new(i32_field(request, "min")?, i32_field(request, "max")?);
    let discrete = request["discrete"]
        .as_bool()
        .ok_or("discrete must be a boolean")?;
    let mapped = range.xmap(
        |value| value.to_string(),
        |value: &String| {
            value
                .parse::<i32>()
                .expect("mapped test values must be signed integers")
        },
        discrete,
    );
    let values = request["values"]
        .as_array()
        .ok_or("values must be an array")?
        .iter()
        .map(|value| string_value(value, "mapped value"))
        .collect::<Result<Vec<_>, String>>()?;
    let value_results = values
        .iter()
        .map(|value| {
            let mut state = serde_json::Map::new();
            state.insert("value".into(), json!(value));
            let validated = mapped.validate_value(value);
            state.insert("valid".into(), json!(validated.is_some()));
            if let Some(validated) = validated {
                state.insert("validated".into(), json!(validated));
            }
            state.insert(
                "to_slider".into(),
                json!(format!("{:016x}", mapped.to_slider_value(value).to_bits())),
            );
            if let Some(next) = mapped.next(value) {
                state.insert("next".into(), json!(next));
            }
            if let Some(previous) = mapped.previous(value) {
                state.insert("previous".into(), json!(previous));
            }
            Value::Object(state)
        })
        .collect::<Vec<_>>();
    let sliders = request["sliders"]
        .as_array()
        .ok_or("sliders must be an array")?
        .iter()
        .map(|entry| {
            let slider = f64_value(entry, "slider")?;
            Ok(json!({
                "slider": format!("{:016x}", slider.to_bits()),
                "from_slider": mapped.from_slider_value(slider),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({
        "values": value_results,
        "sliders": sliders,
    }))
}

pub(super) fn mapped_unit_double_result(request: &Value) -> Result<Value, String> {
    let mapped = UnitDouble::xmap(|value| value * value, |value: &f64| value.sqrt());
    let values = request["values"]
        .as_array()
        .ok_or("values must be an array")?
        .iter()
        .map(|value| f64_value(value, "mapped value"))
        .collect::<Result<Vec<_>, String>>()?;
    let value_results = values
        .iter()
        .map(|value| {
            let mut state = serde_json::Map::new();
            state.insert("value".into(), json!(format!("{:016x}", value.to_bits())));
            let validated = mapped.validate_value(value);
            state.insert("valid".into(), json!(validated.is_some()));
            if let Some(validated) = validated {
                state.insert(
                    "validated".into(),
                    json!(format!("{:016x}", validated.to_bits())),
                );
            }
            state.insert(
                "to_slider".into(),
                json!(format!("{:016x}", mapped.to_slider_value(value).to_bits())),
            );
            Value::Object(state)
        })
        .collect::<Vec<_>>();
    let sliders = request["sliders"]
        .as_array()
        .ok_or("sliders must be an array")?
        .iter()
        .map(|entry| {
            let slider = f64_value(entry, "slider")?;
            Ok(json!({
                "slider": format!("{:016x}", slider.to_bits()),
                "from_slider": format!("{:016x}", mapped.from_slider_value(slider).to_bits()),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({
        "values": value_results,
        "sliders": sliders,
    }))
}

pub(super) fn scroll_wheel_result(request: &Value) -> Result<Value, String> {
    let mut handler = ScrollWheelHandler::new();
    let events = request["scrolls"]
        .as_array()
        .ok_or("scrolls must be an array")?
        .iter()
        .map(|step| {
            let (x, y) = handler.on_mouse_scroll(f64_field(step, "x")?, f64_field(step, "y")?);
            Ok(json!({"x": x, "y": y}))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let selections = request["selections"]
        .as_array()
        .ok_or("selections must be an array")?
        .iter()
        .map(|selection| {
            Ok(json!({
                "wheel": format!("{:016x}", f64_field(selection, "wheel")?.to_bits()),
                "current": i32_field(selection, "current")?,
                "limit": i32_field(selection, "limit")?,
                "selected": ScrollWheelHandler::next_scroll_wheel_selection(
                    f64_field(selection, "wheel")?,
                    i32_field(selection, "current")?,
                    i32_field(selection, "limit")?,
                ),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"scrolls": events, "selections": selections}))
}

pub(super) fn smooth_double_result(request: &Value) -> Result<Value, String> {
    let mut smooth = SmoothDouble::new();
    let events = request["steps"]
        .as_array()
        .ok_or("steps must be an array")?
        .iter()
        .map(|step| {
            let operation = string(step, "op")?;
            if operation == "reset" {
                smooth.reset();
                Ok(json!({"op": "reset"}))
            } else if operation == "delta" {
                let value = smooth.get_new_delta_value(
                    f64_field(step, "target_delta")?,
                    f64_field(step, "time")?,
                );
                Ok(json!({"op": "delta", "value": format!("{:016x}", value.to_bits())}))
            } else {
                Err("unknown SmoothDouble operation".into())
            }
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"events": events}))
}

pub(super) fn resources_result(request: &Value) -> Result<Value, String> {
    if request["operation"].as_str() == Some("packs") {
        return resource_packs_result(request);
    }
    if request["operation"].as_str() == Some("reload") {
        return resource_reload_result(request);
    }
    if request["operation"].as_str() == Some("compatibility") {
        return resource_compatibility_result(request);
    }
    if request["operation"].as_str() == Some("vanilla") {
        return resource_vanilla_result(request);
    }
    if request["operation"].as_str() == Some("known_pack") {
        return resource_known_pack_result(request);
    }
    if request["operation"].as_str() == Some("known_pack_codec") {
        return resource_known_pack_codec_result(request);
    }
    if request["operation"].as_str() == Some("known_pack_list_codec") {
        return resource_known_pack_list_codec_result(request);
    }

    let entries = request["entries"]
        .as_array()
        .ok_or("entries must be an array")?
        .iter()
        .map(|entry| {
            Ok((
                string(entry, "path")?.to_owned(),
                string(entry, "hash")?.to_owned(),
            ))
        })
        .collect::<Result<Vec<(String, String)>, String>>()?;
    let index = AssetIndex::from_entries(entries)
        .map_err(|error| format!("invalid asset index: {error:?}"))?;
    let objects = request["objects"]
        .as_array()
        .ok_or("objects must be an array")?
        .iter()
        .map(|entry| {
            Ok((
                string(entry, "hash")?.to_owned(),
                decode_hex(string(entry, "hex")?)?,
            ))
        })
        .collect::<Result<std::collections::HashMap<String, Vec<u8>>, String>>()?;
    let queries = request["queries"]
        .as_array()
        .ok_or("queries must be an array")?
        .iter()
        .map(|entry| {
            let path = entry
                .as_str()
                .ok_or("query entries must be strings")?
                .to_owned();
            let mut result = serde_json::Map::new();
            result.insert("path".into(), json!(&path));
            match index.resolve(&path) {
                AssetResolution::Missing => {
                    result.insert("exists".into(), json!(false));
                    result.insert("directory".into(), json!(false));
                    result.insert("readable".into(), json!(false));
                }
                AssetResolution::Directory => {
                    result.insert("exists".into(), json!(true));
                    result.insert("directory".into(), json!(true));
                    result.insert("readable".into(), json!(false));
                }
                AssetResolution::File { hash } => {
                    result.insert("exists".into(), json!(true));
                    result.insert("directory".into(), json!(false));
                    result.insert("object_path".into(), json!(resources::object_path(hash)));
                    if let Some(bytes) = objects.get(hash) {
                        result.insert("readable".into(), json!(true));
                        result.insert("bytes_hex".into(), json!(hex(bytes)));
                    } else {
                        result.insert("readable".into(), json!(false));
                    }
                }
            }
            Ok(Value::Object(result))
        })
        .collect::<Result<Vec<Value>, String>>()?;
    Ok(json!({"queries": queries}))
}

pub(super) fn resource_packs_result(request: &Value) -> Result<Value, String> {
    let pack_stack = ResourcePackStack::new(parse_resource_packs(&request["packs"])?);
    let queries = resource_queries_result(&pack_stack, &request["queries"])?;
    Ok(json!({"queries": queries}))
}

pub(super) fn parse_resource_packs(value: &Value) -> Result<Vec<ResourcePack>, String> {
    value
        .as_array()
        .ok_or("packs must be an array")?
        .iter()
        .map(|pack_value| {
            let mut pack = ResourcePack::new(string(pack_value, "id")?.to_owned());
            for resource_value in pack_value["resources"]
                .as_array()
                .ok_or("pack resources must be an array")?
            {
                pack.insert(
                    string(resource_value, "id")?.to_owned(),
                    decode_hex(string(resource_value, "hex")?)?,
                );
            }
            Ok(pack)
        })
        .collect::<Result<Vec<_>, String>>()
}

pub(super) fn resource_queries_result(
    pack_stack: &ResourcePackStack,
    query_values: &Value,
) -> Result<Value, String> {
    let queries = query_values
        .as_array()
        .ok_or("queries must be an array")?
        .iter()
        .map(|query_value| {
            let id = query_value
                .as_str()
                .ok_or("query entries must be strings")?;
            let stack = pack_stack.resource_stack(id);
            let mut result = serde_json::Map::new();
            result.insert("id".into(), json!(id));
            result.insert("exists".into(), json!(pack_stack.resource(id).is_some()));
            result.insert(
                "stack".into(),
                json!(
                    stack
                        .iter()
                        .map(|(pack_id, _)| *pack_id)
                        .collect::<Vec<_>>()
                ),
            );
            if let Some((source, bytes)) = pack_stack.resource(id) {
                result.insert("source".into(), json!(source));
                result.insert("bytes_hex".into(), json!(hex(bytes)));
            }
            Ok(Value::Object(result))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!(queries))
}

pub(super) fn resource_reload_result(request: &Value) -> Result<Value, String> {
    let mut manager = ReloadableResourceManager::new();
    let initial = json!({
        "packs": manager.resources().pack_ids(),
        "namespaces": manager.resources().namespaces(),
        "queries": resource_queries_result(manager.resources(), &request["initial_queries"])?,
    });
    let rounds = request["rounds"]
        .as_array()
        .ok_or("rounds must be an array")?
        .iter()
        .enumerate()
        .map(|(round, round_value)| {
            manager.reload(parse_resource_packs(&round_value["packs"])?);
            Ok(json!({
                "packs": manager.resources().pack_ids(),
                "namespaces": manager.resources().namespaces(),
                "queries": resource_queries_result(manager.resources(), &round_value["queries"])?,
                "round": round,
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"initial": initial, "rounds": rounds}))
}

pub(super) fn resource_compatibility_result(request: &Value) -> Result<Value, String> {
    let game = PackFormat::new(88, 0);
    let probes = request["probes"]
        .as_array()
        .ok_or("probes must be an array")?
        .iter()
        .map(|probe| {
            let declared_min = PackFormat::new(
                i32_field(probe, "min_major")?,
                i32_field(probe, "min_minor")?,
            );
            let declared_max = PackFormat::new(
                i32_field(probe, "max_major")?,
                i32_field(probe, "max_minor")?,
            );
            let compatibility = PackCompatibility::for_version(declared_min, declared_max, game);
            Ok(json!({
                "compatibility": compatibility.name(),
                "is_compatible": compatibility.is_compatible(),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({
        "game_major": game.major(),
        "game_minor": game.minor(),
        "probes": probes,
    }))
}
