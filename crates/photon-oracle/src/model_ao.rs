use super::*;

/// An in-memory set of model files, by the name a `parent` field uses.
struct Models(std::collections::HashMap<String, String>);

impl photon_render::AssetSource for Models {
    fn model(&self, name: &Identifier) -> Option<String> {
        self.0.get(&name.as_string()).cloned()
    }

    fn texture(&self, _name: &Identifier) -> Option<Vec<u8>> {
        None
    }
}

/// The ambient occlusion flag of a leaf model, through the real resolver.
pub(super) fn model_ambient_occlusion_result(request: &Value) -> Result<Value, String> {
    let rows = request["rows"]
        .as_array()
        .ok_or("model rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let mut files = std::collections::HashMap::new();
        for model in row["models"].as_array().ok_or("models must be an array")? {
            let hex = string(model, "text")?;
            let bytes: Vec<u8> = (0..hex.len() / 2)
                .map(|index| u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16))
                .collect::<Result<_, _>>()
                .map_err(|error| format!("text hex: {error}"))?;
            let text = String::from_utf8(bytes).map_err(|error| format!("text: {error}"))?;
            files.insert(string(model, "name")?.to_owned(), text);
        }
        let leaf =
            Identifier::parse(string(row, "leaf")?).map_err(|error| format!("leaf: {error:?}"))?;
        let resolved = photon_render::resolve_model(&Models(files), &leaf)
            .map_err(|error| format!("resolve: {error:?}"))?;
        out.push(json!({
            "label": string(row, "label")?,
            "outcome": "resolved",
            "ambient_occlusion": resolved.ambient_occlusion,
        }));
    }
    Ok(json!({"rows": out}))
}
