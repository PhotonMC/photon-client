use super::*;

/// `Font.width` of a row's text, plain and bold, and each glyph's advances and shadow offset.
pub(super) fn font_width_case(ctx: &GuiContext, row: &Value) -> Result<Value, String> {
    let text = row["text"].as_str().ok_or("text")?;
    let bits = |value: f32| format!("{:08x}", value.to_bits());
    let mut advances = Vec::new();
    for codepoint in text.chars() {
        let glyph = ctx.font.glyph(codepoint).ok_or("no glyph")?;
        advances.push(bits(glyph.advance(false)));
        advances.push(bits(glyph.advance(true)));
        advances.push(bits(glyph.shadow_offset()));
    }
    let bold = [photon_render::gui::TextRun {
        text: text.into(),
        colour: None,
        bold: true,
        italic: false,
        underlined: false,
        strikethrough: false,
        obfuscated: false,
        piece_start: true,
        shadow: None,
        joins_previous: false,
    }];
    Ok(json!({
        "width": ctx.font.width(text),
        "bold_width": ctx.font.width_of_text_runs(&bold),
        "advances": advances,
    }))
}
