use super::*;
use photon_core::identifier::Identifier;
use photon_core::resources::{FolderResources, Resources};

/// The packs the fonts are read from: the asset index's own files (the unifont definition and its
/// hex archives) above the jar's, as the game's stack has them.
pub(super) struct FontResources(Vec<FolderResources>);

impl FontResources {
    /// The jar's assets under `root`, with the index's files beside them in `named`.
    pub(super) fn new(root: &std::path::Path) -> Self {
        let named = root.join("../../named");
        Self(vec![
            FolderResources::new(named),
            FolderResources::new(root),
        ])
    }
}

impl Resources for FontResources {
    fn read(&self, id: &Identifier) -> Option<Vec<u8>> {
        self.0.iter().find_map(|pack| pack.read(id))
    }

    fn read_stack(&self, id: &Identifier) -> Vec<Vec<u8>> {
        self.0
            .iter()
            .rev()
            .flat_map(|pack| pack.read_stack(id))
            .collect()
    }

    fn list(&self, directory: &str, extension: &str) -> Vec<Identifier> {
        let mut found: Vec<Identifier> = self
            .0
            .iter()
            .flat_map(|pack| pack.list(directory, extension))
            .collect();
        found.sort();
        found.dedup();
        found
    }

    fn namespaces(&self) -> Vec<String> {
        let mut names: Vec<String> = self.0.iter().flat_map(|pack| pack.namespaces()).collect();
        names.sort();
        names.dedup();
        names
    }
}

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
