//! What `VillagerProfessionLayer` draws over a villager: the type's texture on
//! the hatted or the hatless model, then the profession's and its level's.

use voxel_data::villager::{self, Hat};

/// One pass of the layer: which model it draws and which texture (a path under
/// `textures/`, without the extension). Every pass is uncoloured.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pass {
    /// The hatless model replaces the villager's own when a hat is in the way.
    pub hatless: bool,
    pub texture: String,
}

/// The texture of each level's badge (`LEVEL_LOCATIONS`), level one first.
const LEVEL_BADGES: [&str; 5] = ["stone", "iron", "gold", "emerald", "diamond"];

fn hat_of(table: &[(&str, Hat)], name: &str) -> Hat {
    table
        .iter()
        .find(|(candidate, _)| *candidate == name)
        .map_or(Hat::None, |(_, hat)| *hat)
}

/// The layer's passes for a villager of `folder` (`villager` or
/// `zombie_villager`, the folder the textures sit in) of this type and
/// profession (registry paths) and level.
pub fn profession_passes(
    folder: &str,
    type_name: &str,
    profession: &str,
    level: i32,
    baby: bool,
) -> Vec<Pass> {
    let type_hat = hat_of(villager::types(), type_name);
    let profession_hat = hat_of(villager::professions(), profession);
    let type_hat_visible =
        profession_hat == Hat::None || (profession_hat == Hat::Partial && type_hat != Hat::Full);
    let texture = |kind: &str, name: &str| format!("entity/{folder}/{kind}/{name}");
    let mut passes = vec![Pass {
        hatless: !type_hat_visible,
        texture: texture(if baby { "baby" } else { "type" }, type_name),
    }];
    if profession != "none" && !baby {
        passes.push(Pass {
            hatless: false,
            texture: texture("profession", profession),
        });
        if profession != "nitwit" {
            let badge = LEVEL_BADGES[(level.clamp(1, LEVEL_BADGES.len() as i32) - 1) as usize];
            passes.push(Pass {
                hatless: false,
                texture: texture("profession_level", badge),
            });
        }
    }
    passes
}

/// The paths (under `textures/`) of every texture the layer can draw for the
/// villagers' folder.
pub fn profession_textures(folder: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (name, _) in villager::types() {
        out.push(format!("entity/{folder}/type/{name}"));
        out.push(format!("entity/{folder}/baby/{name}"));
    }
    for (name, _) in villager::professions() {
        if *name != "none" {
            out.push(format!("entity/{folder}/profession/{name}"));
        }
    }
    for badge in LEVEL_BADGES {
        out.push(format!("entity/{folder}/profession_level/{badge}"));
    }
    out
}
