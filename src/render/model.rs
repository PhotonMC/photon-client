//! Model texture slots: turning a model's texture variables into sprites.
//!
//! A model names its faces' textures either directly or by pointing at another
//! slot with a leading `#`. Resolving those across a model's parent chain is
//! what decides which sprite every quad ends up sampling, so it is pinned
//! before any geometry is baked.

use std::collections::BTreeMap;

use crate::identifier::Identifier;

/// A resolved texture reference: which sprite, and whether it forces the
/// translucent draw path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Material {
    /// Sprite the slot points at.
    pub sprite: Identifier,
    /// Whether the model asked for the translucent path regardless of content.
    pub force_translucent: bool,
}

/// One slot's contents before resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlotContents {
    /// Points at another slot by name, without the leading `#`.
    Reference(String),
    /// Names a sprite outright.
    Value(Material),
}

impl SlotContents {
    /// Java `TextureSlots.parseEntry` for the plain-string form.
    ///
    /// A leading `#` makes the rest a slot name; anything else is a sprite
    /// identifier.
    pub fn parse_string(value: &str) -> Result<Self, crate::identifier::ParseError> {
        if let Some(target) = value.strip_prefix('#') {
            return Ok(Self::Reference(target.to_owned()));
        }
        Ok(Self::Value(Material {
            sprite: Identifier::parse(value)?,
            force_translucent: false,
        }))
    }
}

/// One model's texture map, before its parents are taken into account.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextureSlotsData {
    values: BTreeMap<String, SlotContents>,
}

impl TextureSlotsData {
    /// Build a texture map from slot name and contents pairs.
    pub fn from_entries<I>(entries: I) -> Self
    where
        I: IntoIterator<Item = (String, SlotContents)>,
    {
        Self {
            values: entries.into_iter().collect(),
        }
    }

    /// Whether this model named no textures at all.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

/// Collects a model's texture maps and resolves them together.
#[derive(Debug, Default)]
pub struct TextureSlotsResolver {
    entries: Vec<TextureSlotsData>,
}

impl TextureSlotsResolver {
    /// Start with nothing collected.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append one level, walking from the model towards its root.
    pub fn add_last(&mut self, data: TextureSlotsData) -> &mut Self {
        self.entries.push(data);
        self
    }

    /// Prepend one level.
    pub fn add_first(&mut self, data: TextureSlotsData) -> &mut Self {
        self.entries.insert(0, data);
        self
    }

    /// Java `Resolver.resolve`.
    ///
    /// Levels are applied root first so a child overrides its parent. A value
    /// clears an inherited reference for that slot and a reference clears an
    /// inherited value, which is what lets a child swap between the two.
    /// References then resolve to a fixed point; whatever still dangles is
    /// dropped, because the reference does not treat it as an error.
    pub fn resolve(&self) -> TextureSlots {
        if self.entries.is_empty() {
            return TextureSlots::default();
        }
        let mut resolved: BTreeMap<String, Material> = BTreeMap::new();
        let mut unresolved: BTreeMap<String, String> = BTreeMap::new();

        for data in self.entries.iter().rev() {
            for (slot, contents) in &data.values {
                match contents {
                    SlotContents::Value(material) => {
                        unresolved.remove(slot);
                        resolved.insert(slot.clone(), material.clone());
                    }
                    SlotContents::Reference(target) => {
                        resolved.remove(slot);
                        unresolved.insert(slot.clone(), target.clone());
                    }
                }
            }
        }

        let mut changed = true;
        while changed {
            changed = false;
            let ready: Vec<(String, Material)> = unresolved
                .iter()
                .filter_map(|(slot, target)| {
                    resolved
                        .get(target)
                        .map(|material| (slot.clone(), material.clone()))
                })
                .collect();
            for (slot, material) in ready {
                unresolved.remove(&slot);
                resolved.insert(slot, material);
                changed = true;
            }
        }

        TextureSlots { resolved }
    }
}

/// A model's fully resolved texture slots.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextureSlots {
    resolved: BTreeMap<String, Material>,
}

impl TextureSlots {
    /// Java `getMaterial`: a leading `#` is stripped before lookup, so a face
    /// may name its slot either way.
    pub fn material(&self, reference: &str) -> Option<&Material> {
        let key = reference.strip_prefix('#').unwrap_or(reference);
        self.resolved.get(key)
    }

    /// Every resolved slot, ordered by name.
    pub fn entries(&self) -> impl Iterator<Item = (&str, &Material)> {
        self.resolved
            .iter()
            .map(|(slot, material)| (slot.as_str(), material))
    }
}
