//! The values of a stack's components: the stack's own patch over the item's
//! defaults, and what is derived from them (a potion's colour).
//!
//! The values are read from the bytes a patch carries, on demand; only the
//! components something reads are decoded.

use std::sync::OnceLock;

use crate::item_components::{self, Patch, PotionContents, ids};

/// Every item's default components, parsed from the reference's own patch of
/// them, by registry id.
fn defaults() -> &'static [Option<Patch>] {
    static DEFAULTS: OnceLock<Vec<Option<Patch>>> = OnceLock::new();
    DEFAULTS.get_or_init(|| {
        crate::item_components_table::ITEM_COMPONENT_WIRES
            .iter()
            .map(|wire| {
                let bytes: Vec<u8> = (0..wire.len() / 2)
                    .filter_map(|at| u8::from_str_radix(&wire[at * 2..at * 2 + 2], 16).ok())
                    .collect();
                let mut input = bytes.as_slice();
                item_components::read_patch(&mut input).ok()
            })
            .collect()
    })
}

/// The components of a stack: `ItemStack.get`.
pub struct Values {
    item: i32,
    patch: Option<Patch>,
}

impl Values {
    /// The components of a stack of an item with a patch, as the wire
    /// carried it from the positive component count on; empty for none.
    pub fn new(item: i32, patch: &[u8]) -> Self {
        let mut input = patch;
        Self {
            item,
            patch: item_components::read_patch(&mut input).ok(),
        }
    }

    /// A component's value bytes: the patch's, or the item's default unless
    /// the patch removes it.
    pub fn raw(&self, component: i32) -> Option<&[u8]> {
        if let Some(patch) = &self.patch {
            if let Some((_, value)) = patch.added.iter().find(|(id, _)| *id == component) {
                return Some(value);
            }
            if patch.removed.contains(&component) {
                return None;
            }
        }
        let default = defaults().get(usize::try_from(self.item).ok()?)?.as_ref()?;
        default
            .added
            .iter()
            .find(|(id, _)| *id == component)
            .map(|(_, value)| value.as_slice())
    }

    /// `dyed_color`, `0xRRGGBB`.
    pub fn dyed_color(&self) -> Option<i32> {
        item_components::int_value(self.raw(ids::DYED_COLOR)?)
    }

    /// `map_color`, `0xRRGGBB`.
    pub fn map_color(&self) -> Option<i32> {
        item_components::int_value(self.raw(ids::MAP_COLOR)?)
    }

    /// `firework_explosion`'s colours, none without the component.
    pub fn firework_colors(&self) -> Vec<i32> {
        self.raw(ids::FIREWORK_EXPLOSION)
            .and_then(item_components::firework_colors)
            .unwrap_or_default()
    }

    /// `potion_contents`.
    pub fn potion_contents(&self) -> Option<PotionContents> {
        item_components::potion_contents(self.raw(ids::POTION_CONTENTS)?)
    }

    /// `PotionContents.getColorOr`, `0xRRGGBB`: the custom colour, or the
    /// average of the effects' colours weighted by amplifier.
    pub fn potion_color_or(&self, default: i32) -> i32 {
        let Some(contents) = self.potion_contents() else {
            return default;
        };
        contents.custom_color.unwrap_or_else(|| {
            let effects = contents
                .potion
                .and_then(|id| crate::potion_table::POTIONS.get(usize::try_from(id).ok()?))
                .map(|potion| {
                    potion
                        .effects
                        .iter()
                        .map(|effect| (effect.effect, effect.amplifier, effect.visible))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let (mut red, mut green, mut blue, mut weight) = (0, 0, 0, 0);
            for (effect, amplifier, visible) in effects.into_iter().chain(contents.effects) {
                if !visible {
                    continue;
                }
                let Some(color) = usize::try_from(effect)
                    .ok()
                    .and_then(|id| crate::potion_table::MOB_EFFECTS.get(id))
                    .map(|effect| effect.color)
                else {
                    continue;
                };
                let amplifier = amplifier + 1;
                red += amplifier * (color >> 16 & 0xFF);
                green += amplifier * (color >> 8 & 0xFF);
                blue += amplifier * (color & 0xFF);
                weight += amplifier;
            }
            if weight == 0 {
                default
            } else {
                (red / weight) << 16 | (green / weight) << 8 | (blue / weight)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(name: &str) -> i32 {
        crate::item::id_of(name).expect("an item")
    }

    #[test]
    fn a_tipped_arrows_colour_is_its_potions() {
        let creative = crate::creative_menu::shown_tabs()
            .into_iter()
            .flat_map(|tab| crate::creative_tabs_table::CREATIVE_TABS[tab].items.iter())
            .map(|&index| crate::creative_menu::table_stack(index))
            .filter(|stack| stack.item_registry_id() == item("minecraft:tipped_arrow"))
            .collect::<Vec<_>>();
        assert!(creative.len() > 10);
        let mut colours = std::collections::BTreeSet::new();
        for stack in &creative {
            let values = Values::new(stack.item_registry_id(), stack.patch_bytes());
            colours.insert(values.potion_color_or(-13_083_194 & 0xFF_FFFF));
        }
        // Different potions, different colours.
        assert!(colours.len() > 10, "{colours:?}");
    }

    #[test]
    fn a_stack_without_components_has_its_items_defaults() {
        let values = Values::new(item("minecraft:leather_boots"), &[]);
        assert_eq!(values.dyed_color(), None);
        assert_eq!(values.potion_color_or(7), 7);
    }
}
