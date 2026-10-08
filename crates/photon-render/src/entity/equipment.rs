//! What a wearer is drawn in over its model: the equipment assets
//! (`equipment/<name>.json`, `EquipmentClientInfo`) and how their layers are
//! coloured.

use std::collections::BTreeMap;

use photon_core::json::{JsonObject, JsonValue};

/// What an equipment asset's layer is multiplied by when the stack is not
/// dyed: nothing (the layer has no `dyeable`), a colour, or none, which
/// leaves the layer unseen until it is dyed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dyeable {
    /// The layer is not dyeable: drawn as it is.
    No,
    /// Dyeable, and this colour (`0xRRGGBB`) while it is not dyed.
    Undyed(Option<u32>),
}

/// One layer of an equipment asset (`EquipmentClientInfo.Layer`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquipmentLayer {
    /// The texture, as an identifier path under the layer type's folder
    /// (`minecraft:iron` is `iron`).
    pub texture: String,
    pub dyeable: Dyeable,
    pub use_player_texture: bool,
}

/// The layers of an equipment asset, by layer type name (`humanoid`,
/// `humanoid_leggings`, `humanoid_baby`...).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EquipmentInfo {
    pub layers: BTreeMap<String, Vec<EquipmentLayer>>,
}

impl EquipmentInfo {
    /// `EquipmentClientInfo.getLayers`.
    pub fn layers(&self, layer_type: &str) -> &[EquipmentLayer] {
        self.layers.get(layer_type).map_or(&[], Vec::as_slice)
    }
}

impl EquipmentLayer {
    /// `Layer.getTextureLocation`: the texture's path under `textures/`.
    pub fn texture_path(&self, layer_type: &str) -> String {
        format!("entity/equipment/{layer_type}/{}", self.texture)
    }

    /// `EquipmentLayerRenderer.getColorForLayer`: what the layer is multiplied
    /// by, as ARGB, given the stack's dye (zero when it is not dyed); zero
    /// for a layer that is not drawn (dyeable only once dyed).
    pub fn colour(&self, dye: u32) -> u32 {
        match self.dyeable {
            Dyeable::No => 0xFFFF_FFFF,
            Dyeable::Undyed(undyed) => {
                let undyed = undyed.map_or(0, |colour| colour | 0xFF00_0000);
                if dye != 0 { dye } else { undyed }
            }
        }
    }
}

/// Parse an equipment asset (`EquipmentClientInfo.CODEC`).
pub fn parse_equipment_info(text: &str) -> Option<EquipmentInfo> {
    let JsonValue::Object(root) = photon_core::json::parse(text).ok()? else {
        return None;
    };
    let mut info = EquipmentInfo::default();
    for (name, value) in root.object("layers")?.entries() {
        let name = String::from_utf16(name).ok()?;
        let JsonValue::Array(layers) = value else {
            return None;
        };
        let layers = layers
            .iter()
            .map(|layer| match layer {
                JsonValue::Object(layer) => parse_layer(layer),
                _ => None,
            })
            .collect::<Option<Vec<_>>>()?;
        if layers.is_empty() {
            return None;
        }
        info.layers.insert(name, layers);
    }
    Some(info)
}

fn parse_layer(layer: &JsonObject) -> Option<EquipmentLayer> {
    let texture = layer.text("texture")?;
    let dyeable = match layer.object("dyeable") {
        None => Dyeable::No,
        Some(dyeable) => Dyeable::Undyed(
            dyeable
                .i32("color_when_undyed")
                .map(|colour| colour as u32 & 0x00FF_FFFF),
        ),
    };
    Some(EquipmentLayer {
        texture: texture
            .strip_prefix("minecraft:")
            .unwrap_or(&texture)
            .to_owned(),
        dyeable,
        use_player_texture: layer.bool("use_player_texture").unwrap_or(false),
    })
}
