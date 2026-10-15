use super::*;
use photon_core::ids::ItemId;

impl Values {
    /// The components of a stack of an item with a patch, as the wire
    /// carried it from the positive component count on; empty for none.
    /// A default that names an entry of a registry (an instrument) is left
    /// out, for there is no registry to find its id in.
    pub fn new(item: ItemId, patch: &[u8]) -> Self {
        Self::build(item, patch, None)
    }

    /// Like [`Values::new`], with the registries the defaults' entries are
    /// found in.
    pub fn with_registries(item: ItemId, patch: &[u8], registries: &Registries) -> Self {
        Self::build(item, patch, Some(registries))
    }

    pub(super) fn build(item: ItemId, patch: &[u8], registries: Option<&Registries>) -> Self {
        let mut input = patch;
        let defaults = usize::try_from(item.0)
            .ok()
            .and_then(|index| photon_data::item::registry::ITEMS.get(index))
            .map_or_else(
                || Patch {
                    added: Vec::new(),
                    removed: Vec::new(),
                },
                |item| crate::item::default_components::defaults(item, registries),
            );
        Self {
            defaults,
            patch: item_components::read_patch(&mut input).ok(),
        }
    }

    /// A component's value bytes: the patch's, or the item's default unless
    /// the patch removes it.
    /// How many components the stack has (`PatchedDataComponentMap.size`):
    /// the item's own, given as a set of type ids (`DEFAULT_COMPONENT_SETS`),
    /// with what the patch adds and less what it removes.
    pub fn count(&self, own: u128) -> usize {
        let bit = |id: &i32| {
            u32::try_from(*id)
                .ok()
                .and_then(|id| 1_u128.checked_shl(id))
                .unwrap_or(0)
        };
        let mut present = own;
        if let Some(patch) = &self.patch {
            present |= patch
                .added
                .iter()
                .map(|(id, _)| bit(id))
                .fold(0, |set, id| set | id);
            present &= !patch.removed.iter().map(bit).fold(0, |set, id| set | id);
        }
        present.count_ones() as usize
    }

    pub fn raw(&self, component: i32) -> Option<&[u8]> {
        if let Some(patch) = &self.patch {
            if let Some((_, value)) = patch.added.iter().find(|(id, _)| *id == component) {
                return Some(value);
            }
            if patch.removed.contains(&component) {
                return None;
            }
        }
        self.defaults
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

    /// `block_state`: the properties a block item sets on its block.
    pub fn block_state_properties(&self) -> Vec<(String, String)> {
        self.raw(ids::BLOCK_STATE)
            .and_then(item_components::block_state_properties)
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
                .and_then(|id| photon_data::item::potions::POTIONS.get(usize::try_from(id).ok()?))
                .map(|potion| {
                    potion
                        .effects
                        .iter()
                        .map(|effect| {
                            (
                                photon_data::item::effect::effect_id(effect.effect).unwrap_or(-1),
                                effect.amplifier,
                                effect.visible,
                            )
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let (mut red, mut green, mut blue, mut weight) = (0, 0, 0, 0);
            let custom = contents
                .effects
                .into_iter()
                .map(|(effect, amplifier, _, visible)| (effect, amplifier, visible));
            for (effect, amplifier, visible) in effects.into_iter().chain(custom) {
                if !visible {
                    continue;
                }
                let Some(color) = usize::try_from(effect)
                    .ok()
                    .and_then(|id| photon_data::item::mob_effects::MOB_EFFECTS.get(id))
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
