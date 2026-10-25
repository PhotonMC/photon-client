//! What a mob effect and a potion declare (`MobEffects`, `Potions`): names,
//! colours, the attributes an effect changes and the effects a potion gives.
//! An effect's description key and tooltip colour follow from its name and
//! category; a reference to an effect or an attribute is by name, and its
//! registry id is its place in [`MOB_EFFECTS`] or `ATTRIBUTES`.

use super::attributes::ATTRIBUTES;
use super::mob_effects::MOB_EFFECTS;

/// `MobEffectCategory`: which row of the HUD the effect sits in and its tooltip colour.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EffectCategory {
    Beneficial,
    Neutral,
    Harmful,
}

/// What an effect does to an attribute at amplifier zero; a higher amplifier
/// multiplies the amount by amplifier plus one.
#[derive(Debug)]
pub struct EffectModifier {
    pub attribute: &'static str,
    pub amount: f64,
    /// `AttributeModifier.Operation.id`.
    pub operation: i32,
}

/// One mob effect of the registry.
#[derive(Debug)]
pub struct MobEffect {
    pub name: &'static str,
    pub category: EffectCategory,
    /// `MobEffect.getColor`, `0xRRGGBB`.
    pub color: i32,
    pub modifiers: &'static [EffectModifier],
}

impl MobEffect {
    pub const fn beneficial(name: &'static str, color: i32) -> Self {
        Self::new(name, EffectCategory::Beneficial, color)
    }

    pub const fn harmful(name: &'static str, color: i32) -> Self {
        Self::new(name, EffectCategory::Harmful, color)
    }

    pub const fn neutral(name: &'static str, color: i32) -> Self {
        Self::new(name, EffectCategory::Neutral, color)
    }

    const fn new(name: &'static str, category: EffectCategory, color: i32) -> Self {
        Self {
            name,
            category,
            color,
            modifiers: &[],
        }
    }

    /// The attributes the effect changes.
    pub const fn modifying(mut self, modifiers: &'static [EffectModifier]) -> Self {
        self.modifiers = modifiers;
        self
    }

    /// The translation key of its name: `effect.`, its namespace and path.
    pub fn description_id(&self) -> String {
        let (namespace, path) = self
            .name
            .split_once(':')
            .unwrap_or(("minecraft", self.name));
        format!("effect.{namespace}.{path}")
    }

    /// The formatting its tooltip line takes, by its category.
    pub fn style(&self) -> &'static str {
        match self.category {
            // A neutral effect reads as blue as a beneficial one (`MobEffectCategory`).
            EffectCategory::Beneficial | EffectCategory::Neutral => "BLUE",
            EffectCategory::Harmful => "RED",
        }
    }
}

/// One effect a potion gives.
#[derive(Debug)]
pub struct PotionEffect {
    pub effect: &'static str,
    pub amplifier: i32,
    pub duration: i32,
    pub visible: bool,
}

/// One potion of the registry.
#[derive(Debug)]
pub struct Potion {
    pub name: &'static str,
    /// `Potion.name` when it is not the registry path (the long and strong
    /// potions take their base's).
    pub base_name: Option<&'static str>,
    pub effects: &'static [PotionEffect],
}

impl Potion {
    pub const fn of(name: &'static str, effects: &'static [PotionEffect]) -> Self {
        Self {
            name,
            base_name: None,
            effects,
        }
    }

    /// The potion this one is a variant of, by the name its item uses.
    pub const fn variant_of(mut self, base_name: &'static str) -> Self {
        self.base_name = Some(base_name);
        self
    }

    /// `Potion.name`, which names the potion's item translation.
    pub fn potion_name(&self) -> &'static str {
        self.base_name.unwrap_or_else(|| {
            self.name
                .split_once(':')
                .map_or(self.name, |(_, path)| path)
        })
    }
}

/// An effect's registry id.
pub fn effect_id(name: &str) -> Option<i32> {
    MOB_EFFECTS
        .iter()
        .position(|effect| effect.name == name)
        .map(|id| id as i32)
}

/// An attribute's registry id.
pub fn attribute_id(name: &str) -> Option<i32> {
    ATTRIBUTES
        .iter()
        .position(|attribute| attribute.name == name)
        .map(|id| id as i32)
}

/// An attribute's registry id, resolved while compiling: a name the registry lacks stops the
/// build.
pub const fn attribute_named(name: &str) -> i32 {
    let mut id = 0;
    while id < ATTRIBUTES.len() {
        if crate::name::same(ATTRIBUTES[id].name, name) {
            return id as i32;
        }
        id += 1;
    }
    panic!("the attribute registry has no such name");
}
