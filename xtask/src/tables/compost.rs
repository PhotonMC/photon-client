//! The items a composter takes, with the chance each fills a layer.

use super::{Versions, capture, text};

pub fn render(versions: &Versions) -> Result<String, String> {
    let rows = capture::rows(&versions.newest, "movement", "movement/compostables")?;
    let items = rows
        .first()
        .and_then(|row| row["items"].as_array())
        .ok_or("compostables: no items")?;
    let mut out = text::header(
        "The items a composter takes, with the chance each fills a layer.",
        "compost",
    );
    out.push_str("#[rustfmt::skip]\n");
    out.push_str(&format!(
        "pub static COMPOSTABLES: [(&str, f32); {}] = [\n",
        items.len()
    ));
    for item in items {
        let name = item["name"].as_str().ok_or("compostables: no name")?;
        let chance =
            text::f32_from_bits(item["chance"].as_str().ok_or("compostables: no chance")?)?;
        out.push_str(&format!("    ({}, {chance}),\n", text::string(name)));
    }
    out.push_str("];\n");
    Ok(out)
}
