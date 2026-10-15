//! The values of a stack's components: the stack's own patch over the item's
//! defaults, and what is derived from them (a potion's colour).
//!
//! The values are read from the bytes a patch carries, on demand; only the
//! components something reads are decoded.

use super::equippable_allowed::Allowed;
use crate::item_components::{self, Cursor, Patch, PotionContents, ids};
use crate::registries::Registries;
use crate::text::Component;
use photon_core::ids::EntityTypeId;

/// The components of a stack: `ItemStack.get`.
pub struct Values {
    /// The item's default components, in the wire's form.
    defaults: Patch,
    patch: Option<Patch>,
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
        let Some(mut cursor) = self.raw(ids::TOOLTIP_DISPLAY).map(|data| Cursor { data }) else {
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
        let tag = photon_core::nbt::read_any_nbt(&mut data).ok()??;
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

    /// `BookViewScreen.BookAccess.fromItem`: the pages of a written book, or of a book and quill
    /// (its text as it is, the text-filtering option being off); `None` for any other stack.
    pub fn book_pages(&self) -> Option<Vec<Component>> {
        use photon_data::item::component_types::{WRITABLE_BOOK_CONTENT, WRITTEN_BOOK_CONTENT};
        let string = |cursor: &mut Cursor<'_>| -> Option<String> {
            let length = cursor.count().ok()?;
            let bytes = cursor.data.get(..length)?;
            let text = String::from_utf8(bytes.to_vec()).ok()?;
            cursor.skip(length).ok()?;
            Some(text)
        };
        if let Some(data) = self.raw(WRITTEN_BOOK_CONTENT) {
            let mut cursor = Cursor { data };
            // The title (raw, then the filtered one if there is one), the author and the
            // generation.
            cursor.string().ok()?;
            cursor.optional(Cursor::string).ok()?;
            cursor.string().ok()?;
            cursor.varint().ok()?;
            let count = cursor.count().ok()?;
            let mut pages = Vec::new();
            for _ in 0..count {
                let tag = photon_core::nbt::read_any_nbt(&mut cursor.data).ok()??;
                pages.push(Component::from_nbt(&tag).unwrap_or_else(Component::empty));
                cursor.optional(Cursor::tag).ok()?;
            }
            return Some(pages);
        }
        let data = self.raw(WRITABLE_BOOK_CONTENT)?;
        let mut cursor = Cursor { data };
        let count = cursor.count().ok()?;
        let mut pages = Vec::new();
        for _ in 0..count {
            pages.push(Component::literal(&string(&mut cursor)?));
            cursor.optional(Cursor::string).ok()?;
        }
        Some(pages)
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
            let Ok(Some(tag)) = photon_core::nbt::read_any_nbt(&mut data) else {
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

    /// Whether the stack's patch removes a component (so the item's own is gone).
    pub fn removes(&self, component: i32) -> bool {
        self.patch
            .as_ref()
            .is_some_and(|patch| patch.removed.contains(&component))
    }

    /// The `equippable` component a stack's patch adds, as far as the layers read it: the slot's
    /// name and the equipment asset (`Equippable.STREAM_CODEC`, whose sound holders come first).
    fn patched_equippable(&self) -> Option<Equippable> {
        let mut cursor = Cursor {
            data: self.raw(ids::EQUIPPABLE)?,
        };
        let slot = match cursor.varint().ok()? {
            1 => "feet",
            2 => "legs",
            3 => "chest",
            4 => "head",
            5 => "offhand",
            6 => "body",
            7 => "saddle",
            _ => "mainhand",
        };
        // `SoundEvent.STREAM_CODEC`: a registry id plus one, or the event itself (its name, and a
        // fixed range if it has one).
        if cursor.varint().ok()? == 0 {
            cursor.identifier().ok()?;
            cursor.optional(Cursor::float).ok()?;
        }
        let asset = if cursor.boolean().ok()? {
            let length = cursor.count().ok()?;
            let name = cursor.data.get(..length)?;
            let name = String::from_utf8_lossy(name).into_owned();
            cursor.skip(length).ok()?;
            Some(name)
        } else {
            None
        };
        // The camera overlay, then the entities that may wear it: a holder
        // set, which is a tag's name or a count plus one and the types.
        if cursor.boolean().ok()? {
            let length = cursor.count().ok()?;
            cursor.skip(length).ok()?;
        }
        let allowed_entities = if cursor.boolean().ok()? {
            match cursor.count().ok()? {
                0 => {
                    let length = cursor.count().ok()?;
                    let tag = String::from_utf8_lossy(cursor.data.get(..length)?).into_owned();
                    Some(Allowed::Tag(tag))
                }
                count => {
                    let mut types = Vec::new();
                    for _ in 1..count {
                        let id = cursor.varint().ok()?;
                        types.push(photon_data::entity_type::name(EntityTypeId(id))?.to_owned());
                    }
                    Some(Allowed::Types(types))
                }
            }
        } else {
            None
        };
        Some(Equippable {
            slot,
            asset,
            allowed_entities,
        })
    }

    /// `ItemStack.hasFoil`: the glint override if the stack has one, else whether it has any
    /// enchantment (`Item.isFoil`).
    pub fn has_foil(&self) -> bool {
        match self.raw(ids::ENCHANTMENT_GLINT_OVERRIDE) {
            Some(raw) => raw.first().is_some_and(|value| *value != 0),
            None => !self.enchantments(ids::ENCHANTMENTS).is_empty(),
        }
    }

    /// `attribute_modifiers`.
    pub fn attribute_modifiers(&self) -> Vec<AttributeEntry> {
        let Some(data) = self.raw(ids::ATTRIBUTE_MODIFIERS) else {
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
                        let tag = photon_core::nbt::read_any_nbt(&mut cursor.data).ok()??;
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
            data: self.raw(ids::FIREWORKS)?,
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
            data: self.raw(ids::FIREWORK_EXPLOSION)?,
        })
    }

    /// `painting/variant`.
    pub fn painting_variant(&self) -> Option<PaintingVariant> {
        let mut cursor = Cursor {
            data: self.raw(ids::PAINTING_VARIANT)?,
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
            let tag = photon_core::nbt::read_any_nbt(&mut cursor.data).ok()??;
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
        let Some(data) = self.raw(ids::BANNER_PATTERNS) else {
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
            data: self.raw(ids::TRIM)?,
        };
        let material = cursor.varint().ok()?;
        let pattern = cursor.varint().ok()?;
        (material > 0 && pattern > 0).then_some((material - 1, pattern - 1))
    }

    /// `suspicious_stew_effects`: effect registry ids and durations.
    pub fn suspicious_stew(&self) -> Vec<(i32, i32)> {
        let Some(data) = self.raw(ids::SUSPICIOUS_STEW_EFFECTS) else {
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
        self.raw(ids::POTION_DURATION_SCALE)
            .and_then(|data| Some(f32::from_be_bytes(data.get(..4)?.try_into().ok()?)))
            .unwrap_or(1.0)
    }

    /// `block_state`'s properties.
    pub fn block_state(&self) -> Vec<(String, String)> {
        let Some(data) = self.raw(ids::BLOCK_STATE) else {
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

mod color;

#[cfg(test)]
mod tests {
    use super::*;

    fn item(name: &str) -> i32 {
        crate::item::id_of(name).expect("an item")
    }

    #[test]
    fn a_tipped_arrows_colour_is_its_potions() {
        let registries = crate::registries::for_tests();
        let creative = crate::menus::creative::shown_tabs()
            .into_iter()
            .flat_map(|tab| {
                photon_data::item::creative_tabs::CREATIVE_TABS[tab]
                    .items
                    .iter()
            })
            .map(|entry| crate::menus::creative::table_stack(entry, &registries))
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

/// What a worn stack is equipped as (`Equippable`): the slot's name and the equipment asset, if it
/// names one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Equippable {
    pub slot: &'static str,
    pub asset: Option<String>,
    /// `allowedEntities`; none lets every entity wear it.
    pub allowed_entities: Option<Allowed>,
}

/// `ItemStack.get(DataComponents.EQUIPPABLE)`: the stack's patch over the item's own.
pub fn equippable_of(item: i32, patch: &[u8]) -> Option<Equippable> {
    let values = Values::new(item, patch);
    if let Some(equippable) = values.patched_equippable() {
        return Some(equippable);
    }
    if values.removes(ids::EQUIPPABLE) {
        return None;
    }
    let item = usize::try_from(item)
        .ok()
        .and_then(|index| photon_data::item::registry::ITEMS.get(index))?;
    let equip = item.equippable.as_ref()?;
    Some(Equippable {
        slot: equip.slot,
        asset: equip.asset.map(str::to_owned),
        allowed_entities: super::equippable_allowed::allowed_entities(item.name),
    })
}
