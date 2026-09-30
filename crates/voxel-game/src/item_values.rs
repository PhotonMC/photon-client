//! The values of a stack's components: the stack's own patch over the item's
//! defaults, and what is derived from them (a potion's colour).
//!
//! The values are read from the bytes a patch carries, on demand; only the
//! components something reads are decoded.

use std::sync::OnceLock;

use crate::item_components::{self, Cursor, Patch, PotionContents, ids};
use crate::text::Component;

/// Every item's default components, parsed from the reference's own patch of
/// them, by registry id.
fn defaults() -> &'static [Option<Patch>] {
    static DEFAULTS: OnceLock<Vec<Option<Patch>>> = OnceLock::new();
    DEFAULTS.get_or_init(|| {
        voxel_data::item::components::DEFAULT_PATCHES
            .iter()
            .map(|&patch| {
                let mut input = patch;
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
                .and_then(|id| voxel_data::item::potions::POTIONS.get(usize::try_from(id).ok()?))
                .map(|potion| {
                    potion
                        .effects
                        .iter()
                        .map(|effect| {
                            (
                                voxel_data::item::effect::effect_id(effect.effect).unwrap_or(-1),
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
                    .and_then(|id| voxel_data::item::mob_effects::MOB_EFFECTS.get(id))
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

/// One entry of `attribute_modifiers`.
#[derive(Debug, Clone, PartialEq)]
pub struct AttributeEntry {
    /// The attribute's registry id.
    pub attribute: i32,
    /// The modifier's id, `minecraft:base_attack_damage` and the like.
    pub id: String,
    pub amount: f64,
    /// `AttributeModifier.Operation.id`.
    pub operation: i32,
    /// `EquipmentSlotGroup.id`.
    pub slot: i32,
    pub display: AttributeDisplay,
}

/// `ItemAttributeModifiers.Display`.
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeDisplay {
    Default,
    Hidden,
    Override(Component),
}

/// `FireworkExplosion`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Explosion {
    pub shape: i32,
    pub colors: Vec<i32>,
    pub fade_colors: Vec<i32>,
    pub trail: bool,
    pub twinkle: bool,
}

/// A painting's variant: by registry id, or written out in full.
#[derive(Debug, Clone, PartialEq)]
#[expect(
    clippy::large_enum_variant,
    reason = "the inline variant is the one nearly every painting uses"
)]
pub enum PaintingVariant {
    Registry(i32),
    Direct {
        width: i32,
        height: i32,
        title: Option<Component>,
        author: Option<Component>,
    },
}

impl Values {
    /// `tooltip_display`: whether the tooltip is hidden, and which
    /// components it leaves out.
    pub fn tooltip_display(&self) -> (bool, Vec<i32>) {
        let Some(mut cursor) = self.raw(18).map(|data| Cursor { data }) else {
            return (false, Vec::new());
        };
        let hide = cursor.boolean().unwrap_or(false);
        let mut hidden = Vec::new();
        for _ in 0..cursor.count().unwrap_or(0) {
            let Ok(id) = cursor.varint() else { break };
            hidden.push(id);
        }
        (hide, hidden)
    }

    /// A text component value.
    fn text(&self, id: i32) -> Option<Component> {
        let mut data = self.raw(id)?;
        let tag = voxel_core::nbt::read_any_nbt(&mut data).ok()??;
        Component::from_nbt(&tag)
    }

    pub fn custom_name(&self) -> Option<Component> {
        self.text(ids::CUSTOM_NAME)
    }

    pub fn item_name(&self) -> Option<Component> {
        self.text(ids::ITEM_NAME)
    }

    pub fn has(&self, component: i32) -> bool {
        self.raw(component).is_some()
    }

    /// `lore`.
    pub fn lore(&self) -> Vec<Component> {
        let Some(mut data) = self.raw(ids::LORE) else {
            return Vec::new();
        };
        let mut lines = Vec::new();
        let count = Cursor { data }.count().unwrap_or(0);
        // Skip the count, then read each tag.
        let mut cursor = Cursor { data };
        let _ = cursor.count();
        data = cursor.data;
        for _ in 0..count {
            let Ok(Some(tag)) = voxel_core::nbt::read_any_nbt(&mut data) else {
                break;
            };
            lines.extend(Component::from_nbt(&tag));
        }
        lines
    }

    /// `rarity`: 0 common, 1 uncommon, 2 rare, 3 epic; common without.
    pub fn rarity(&self) -> i32 {
        self.raw(ids::RARITY)
            .and_then(item_components::varint_value)
            .unwrap_or(0)
    }

    /// `enchantments` or `stored_enchantments`: enchantment registry ids and
    /// levels, in the wire's order.
    pub fn enchantments(&self, component: i32) -> Vec<(i32, i32)> {
        let Some(data) = self.raw(component) else {
            return Vec::new();
        };
        let mut cursor = Cursor { data };
        let mut entries = Vec::new();
        for _ in 0..cursor.count().unwrap_or(0) {
            let (Ok(id), Ok(level)) = (cursor.varint(), cursor.varint()) else {
                break;
            };
            entries.push((id, level));
        }
        entries
    }

    /// `attribute_modifiers`.
    pub fn attribute_modifiers(&self) -> Vec<AttributeEntry> {
        let Some(data) = self.raw(16) else {
            return Vec::new();
        };
        let mut cursor = Cursor { data };
        let mut entries = Vec::new();
        for _ in 0..cursor.count().unwrap_or(0) {
            let read = |cursor: &mut Cursor| -> Option<AttributeEntry> {
                let attribute = cursor.varint().ok()?;
                let length = cursor.count().ok()?;
                let id = String::from_utf8_lossy(cursor.data.get(..length)?).into_owned();
                cursor.skip(length).ok()?;
                let amount = f64::from_be_bytes(cursor.data.get(..8)?.try_into().ok()?);
                cursor.skip(8).ok()?;
                let operation = cursor.varint().ok()?;
                let slot = cursor.varint().ok()?;
                let display = match cursor.varint().ok()? {
                    0 => AttributeDisplay::Default,
                    1 => AttributeDisplay::Hidden,
                    2 => {
                        let tag = voxel_core::nbt::read_any_nbt(&mut cursor.data).ok()??;
                        AttributeDisplay::Override(Component::from_nbt(&tag)?)
                    }
                    _ => return None,
                };
                Some(AttributeEntry {
                    attribute,
                    id,
                    amount,
                    operation,
                    slot,
                    display,
                })
            };
            match read(&mut cursor) {
                Some(entry) => entries.push(entry),
                None => break,
            }
        }
        entries
    }

    /// `fireworks`: the flight duration and the explosions.
    pub fn fireworks(&self) -> Option<(i32, Vec<Explosion>)> {
        let mut cursor = Cursor {
            data: self.raw(69)?,
        };
        let flight = cursor.varint().ok()?;
        let mut explosions = Vec::new();
        for _ in 0..cursor.count().ok()? {
            explosions.push(read_explosion(&mut cursor)?);
        }
        Some((flight, explosions))
    }

    /// `firework_explosion`.
    pub fn firework_explosion(&self) -> Option<Explosion> {
        read_explosion(&mut Cursor {
            data: self.raw(68)?,
        })
    }

    /// `painting/variant`.
    pub fn painting_variant(&self) -> Option<PaintingVariant> {
        let mut cursor = Cursor {
            data: self.raw(103)?,
        };
        let id = cursor.varint().ok()?;
        if id > 0 {
            return Some(PaintingVariant::Registry(id - 1));
        }
        let width = cursor.varint().ok()?;
        let height = cursor.varint().ok()?;
        cursor.identifier().ok()?;
        let optional_text = |cursor: &mut Cursor| -> Option<Option<Component>> {
            if !cursor.boolean().ok()? {
                return Some(None);
            }
            let tag = voxel_core::nbt::read_any_nbt(&mut cursor.data).ok()??;
            Some(Component::from_nbt(&tag))
        };
        let title = optional_text(&mut cursor)?;
        let author = optional_text(&mut cursor)?;
        Some(PaintingVariant::Direct {
            width,
            height,
            title,
            author,
        })
    }

    /// A component that is a holder of a registry entry by its id: the
    /// registry id, when it is not written out.
    pub fn holder_id(&self, component: i32) -> Option<i32> {
        let id = Cursor {
            data: self.raw(component)?,
        }
        .varint()
        .ok()?;
        (id > 0).then_some(id - 1)
    }

    /// `banner_patterns`: each layer's pattern registry id and dye.
    pub fn banner_layers(&self) -> Vec<(i32, i32)> {
        let Some(data) = self.raw(72) else {
            return Vec::new();
        };
        let mut cursor = Cursor { data };
        let mut layers = Vec::new();
        for _ in 0..cursor.count().unwrap_or(0) {
            let (Ok(pattern), true) = (cursor.varint(), true) else {
                break;
            };
            if pattern == 0 {
                break;
            }
            let Ok(dye) = cursor.varint() else { break };
            layers.push((pattern - 1, dye));
        }
        layers
    }

    /// `trim`: the material's and the pattern's registry ids.
    pub fn trim(&self) -> Option<(i32, i32)> {
        let mut cursor = Cursor {
            data: self.raw(56)?,
        };
        let material = cursor.varint().ok()?;
        let pattern = cursor.varint().ok()?;
        (material > 0 && pattern > 0).then_some((material - 1, pattern - 1))
    }

    /// `suspicious_stew_effects`: effect registry ids and durations.
    pub fn suspicious_stew(&self) -> Vec<(i32, i32)> {
        let Some(data) = self.raw(53) else {
            return Vec::new();
        };
        let mut cursor = Cursor { data };
        let mut effects = Vec::new();
        for _ in 0..cursor.count().unwrap_or(0) {
            let (Ok(effect), Ok(duration)) = (cursor.varint(), cursor.varint()) else {
                break;
            };
            effects.push((effect, duration));
        }
        effects
    }

    /// `potion_duration_scale`.
    pub fn potion_duration_scale(&self) -> f32 {
        self.raw(52)
            .and_then(|data| Some(f32::from_be_bytes(data.get(..4)?.try_into().ok()?)))
            .unwrap_or(1.0)
    }

    /// `block_state`'s properties.
    pub fn block_state(&self) -> Vec<(String, String)> {
        let Some(data) = self.raw(76) else {
            return Vec::new();
        };
        let mut cursor = Cursor { data };
        let mut properties = Vec::new();
        let string = |cursor: &mut Cursor| -> Option<String> {
            let length = cursor.count().ok()?;
            let text = String::from_utf8_lossy(cursor.data.get(..length)?).into_owned();
            cursor.skip(length).ok()?;
            Some(text)
        };
        for _ in 0..cursor.count().unwrap_or(0) {
            let (Some(key), Some(value)) = (string(&mut cursor), string(&mut cursor)) else {
                break;
            };
            properties.push((key, value));
        }
        properties
    }
}

fn read_explosion(cursor: &mut Cursor) -> Option<Explosion> {
    let shape = cursor.varint().ok()?;
    let colors = |cursor: &mut Cursor| -> Option<Vec<i32>> {
        let mut colors = Vec::new();
        for _ in 0..cursor.count().ok()? {
            colors.push(i32::from_be_bytes(cursor.data.get(..4)?.try_into().ok()?));
            cursor.skip(4).ok()?;
        }
        Some(colors)
    };
    let colours = colors(cursor)?;
    let fade_colors = colors(cursor)?;
    let trail = cursor.boolean().ok()?;
    let twinkle = cursor.boolean().ok()?;
    Some(Explosion {
        shape,
        colors: colours,
        fade_colors,
        trail,
        twinkle,
    })
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
            .flat_map(|tab| {
                voxel_data::item::creative_tabs::CREATIVE_TABS[tab]
                    .items
                    .iter()
            })
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
