use super::*;

/// The `bidi` family: a line of text as it is drawn, as code points.
pub(super) fn bidi_result(request: &Value) -> Result<Value, String> {
    if string(request, "operation")? != "reorder" {
        return Err("unknown bidi operation".into());
    }
    let hex = string(request, "utf16_hex")?;
    let units = hex
        .as_bytes()
        .chunks(4)
        .map(|digits| {
            let digits = core::str::from_utf8(digits).map_err(|error| error.to_string())?;
            u16::from_str_radix(digits, 16).map_err(|error| error.to_string())
        })
        .collect::<Result<Vec<u16>, String>>()?;
    let text = String::from_utf16(&units).map_err(|error| error.to_string())?;
    let right_to_left = request["right_to_left"]
        .as_bool()
        .ok_or("right_to_left must be a boolean")?;
    // `FormattedText.of` goes through `StringDecomposer.iterateFormatted`: the codes take no room.
    let plain: String = photon_gui::runs::legacy_runs(&text)
        .iter()
        .map(|run| run.text.as_str())
        .collect();
    let drawn = photon_core::bidi::reorder(&plain, right_to_left);
    Ok(json!({
        "codepoints_hex": drawn.chars().map(|c| format!("{:06x}", c as u32)).collect::<String>(),
    }))
}
