//! Model texture slots: turning a model's texture variables into sprites.
//!
//! A model names its faces' textures either directly or by pointing at another
//! slot with a leading `#`. Resolving those across a model's parent chain is
//! what decides which sprite every quad ends up sampling, so it is pinned
//! before any geometry is baked.

use std::collections::BTreeMap;

use photon_core::identifier::Identifier;

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
    pub fn parse_string(value: &str) -> Result<Self, photon_core::identifier::ParseError> {
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

    /// Slots in name order, as they were written.
    pub fn entries(&self) -> impl Iterator<Item = (&str, &SlotContents)> {
        self.values
            .iter()
            .map(|(slot, contents)| (slot.as_str(), contents))
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

use crate::block::geometry::{FaceUvs, Quadrant};
use crate::block::rotation::{Matrix4, RotationAxis, element_transform, euler_element_transform};
use photon_core::direction::Direction;
use photon_core::json::{JsonObject, JsonValue};

/// How the reference lights a model in a GUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuiLight {
    /// Lit from the front.
    Front,
    /// Lit from the side.
    Side,
}

/// One face of one element.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelFace {
    /// The neighbour that hides this face, if any.
    pub cull_for_direction: Option<Direction>,
    /// Tint slot, or `-1` for no tint.
    pub tint_index: i32,
    /// Texture slot name, usually a `#reference`.
    pub texture: String,
    /// Explicit texture rectangle, or `None` to take the default.
    pub uvs: Option<FaceUvs>,
    /// Texture rotation.
    pub rotation: Quadrant,
}

/// One element's rotation block, already turned into a transform.
#[derive(Debug, Clone, PartialEq)]
pub struct ElementRotation {
    /// Pivot in block units, already scaled from the written sixteenths.
    pub origin: [f32; 3],
    /// Whether the element grows to keep touching its original bounds.
    pub rescale: bool,
    /// The transform the rotation produces.
    pub transform: Matrix4,
}

/// One cuboid element of a model.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelElement {
    /// Low corner in sixteenths.
    pub from: [f32; 3],
    /// High corner in sixteenths.
    pub to: [f32; 3],
    /// Rotation block, if the element carries one.
    pub rotation: Option<ElementRotation>,
    /// Whether the element takes directional shading.
    pub shade: bool,
    /// Light the element emits, `0` to `15`.
    pub light_emission: i32,
    /// Faces, keyed by the direction they face.
    pub faces: Vec<(Direction, ModelFace)>,
}

/// A parsed model file, before its parents are resolved.
#[derive(Debug, Clone, PartialEq)]
pub struct CuboidModel {
    /// The model this one inherits from.
    pub parent: Option<photon_core::identifier::Identifier>,
    /// Ambient occlusion override, or `None` to inherit.
    pub ambient_occlusion: Option<bool>,
    /// GUI lighting override, or `None` to inherit.
    pub gui_light: Option<GuiLight>,
    /// This model's own texture map.
    pub texture_slots: TextureSlotsData,
    /// Geometry, or `None` when the model declares none.
    pub elements: Option<Vec<ModelElement>>,
    /// The `display` transforms, or `None` when the model has no `display`.
    pub display: Option<ItemTransforms>,
}

/// Where an item is shown, as `ItemDisplayContext` orders the contexts a
/// model's `display` can name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayContext {
    ThirdPersonLeftHand,
    ThirdPersonRightHand,
    FirstPersonLeftHand,
    FirstPersonRightHand,
    Head,
    Gui,
    Ground,
    Fixed,
    OnShelf,
    /// `ItemDisplayContext.NONE`: no transform of the model's (not one of [`Self::ALL`]).
    None,
}

impl DisplayContext {
    pub const ALL: [DisplayContext; 9] = [
        DisplayContext::ThirdPersonLeftHand,
        DisplayContext::ThirdPersonRightHand,
        DisplayContext::FirstPersonLeftHand,
        DisplayContext::FirstPersonRightHand,
        DisplayContext::Head,
        DisplayContext::Gui,
        DisplayContext::Ground,
        DisplayContext::Fixed,
        DisplayContext::OnShelf,
    ];

    /// The name a model's `display` uses.
    pub fn name(self) -> &'static str {
        match self {
            DisplayContext::ThirdPersonLeftHand => "thirdperson_lefthand",
            DisplayContext::ThirdPersonRightHand => "thirdperson_righthand",
            DisplayContext::FirstPersonLeftHand => "firstperson_lefthand",
            DisplayContext::FirstPersonRightHand => "firstperson_righthand",
            DisplayContext::Head => "head",
            DisplayContext::Gui => "gui",
            DisplayContext::Ground => "ground",
            DisplayContext::Fixed => "fixed",
            DisplayContext::OnShelf => "on_shelf",
            DisplayContext::None => "none",
        }
    }

    /// Whether the item is shown in a left hand.
    pub fn left_hand(self) -> bool {
        matches!(
            self,
            DisplayContext::ThirdPersonLeftHand | DisplayContext::FirstPersonLeftHand
        )
    }
}

/// `ItemTransform`: degrees, then block units, then scale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ItemTransform {
    pub rotation: [f32; 3],
    pub translation: [f32; 3],
    pub scale: [f32; 3],
}

/// `ItemTransforms`: each context's transform; `None` is the reference's
/// `NO_TRANSFORM`, which a model left unwritten keeps.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ItemTransforms {
    pub transforms: [Option<ItemTransform>; 9],
}

impl ItemTransforms {
    pub fn get(&self, context: DisplayContext) -> Option<ItemTransform> {
        self.transforms.get(context as usize).copied().flatten()
    }
}

/// Why a model file was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelParseError {
    /// What the reference would have complained about.
    pub reason: &'static str,
}

/// The bounds an element may reach, which the reference checks explicitly.
const MIN_EXTENT: f32 = -16.0;
const MAX_EXTENT: f32 = 32.0;

mod parse;
pub use parse::*;
