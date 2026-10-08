use super::*;
use photon_core::java_float::JavaF64;

impl ClientEntity {
    /// `LivingEntity.getAttributeValue`: the attribute's value, from what the
    /// type starts with and what the server has changed (`AttributeInstance.getValue`:
    /// the base, the modifiers in their order, kept in the attribute's range).
    /// Nothing for an attribute the type does not have.
    pub fn attribute_value(&self, attribute: &str) -> Option<f64> {
        let (id, range) = photon_data::entity_type::attribute(attribute)?;
        let default = photon_data::entity_type::default_attribute(
            photon_data::entity_type::name(self.type_id)?,
            attribute,
        )?;
        let (low, high) = range.range.unwrap_or((f64::NEG_INFINITY, f64::INFINITY));
        let changed = self
            .attributes
            .iter()
            .find(|state| usize::try_from(state.attribute_id) == Ok(id));
        Some(changed.map_or_else(
            || ClientAttributeState::new(0, default, Vec::new()).effective_value(low, high),
            |state| state.effective_value(low, high),
        ))
    }

    /// `LivingEntity.getMaxHealth`.
    pub fn max_health(&self) -> Option<f32> {
        self.attribute_value("minecraft:max_health")
            .map(|value| value as f32)
    }

    /// `LivingEntity.getScale`: one for an entity that has no scale attribute.
    pub fn scale(&self) -> f32 {
        self.attribute_value("minecraft:scale")
            .map_or(1.0, |value| value as f32)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientAttributeModifierState {
    pub(crate) id: String,
    pub(crate) amount: JavaF64,
    pub(crate) operation: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientAttributeState {
    pub(crate) attribute_id: i32,
    pub(crate) base_value: JavaF64,
    pub(crate) modifiers: Vec<ClientAttributeModifierState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientMobEffectState {
    pub(crate) effect_id: i32,
    pub(crate) amplifier: i32,
    pub(crate) duration_ticks: i32,
    pub(crate) flags: u8,
}

impl ClientAttributeModifierState {
    pub fn new(id: String, amount: f64, operation: u8) -> Self {
        Self {
            id,
            amount: JavaF64::new(amount),
            operation,
        }
    }

    pub fn from_wire(id: String, amount: f64, operation: u8) -> Self {
        Self::new(id, amount, operation)
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn amount(&self) -> f64 {
        self.amount.get()
    }

    pub fn operation(&self) -> u8 {
        self.operation
    }
}

impl ClientMobEffectState {
    pub fn effect_id(&self) -> i32 {
        self.effect_id
    }

    pub fn amplifier(&self) -> i32 {
        self.amplifier
    }

    pub fn duration_ticks(&self) -> i32 {
        self.duration_ticks
    }

    pub fn has_remaining_duration(&self) -> bool {
        self.duration_ticks == -1 || self.duration_ticks > 0
    }

    pub fn flags(&self) -> u8 {
        self.flags
    }

    pub(crate) fn tick_client(&mut self) {
        if self.has_remaining_duration() && self.duration_ticks != -1 {
            self.duration_ticks -= 1;
        }
    }
}

impl ClientAttributeState {
    pub fn new(
        attribute_id: i32,
        base_value: f64,
        modifiers: Vec<ClientAttributeModifierState>,
    ) -> Self {
        Self {
            attribute_id,
            base_value: JavaF64::new(base_value),
            modifiers,
        }
    }

    pub fn from_wire(
        attribute_id: i32,
        base_value: f64,
        modifiers: Vec<ClientAttributeModifierState>,
    ) -> Self {
        Self::new(attribute_id, base_value, modifiers)
    }

    pub fn attribute_id(&self) -> i32 {
        self.attribute_id
    }

    pub fn base_value(&self) -> f64 {
        self.base_value.get()
    }

    pub fn modifiers(&self) -> &[ClientAttributeModifierState] {
        &self.modifiers
    }

    /// Calculate the effective value using Java's AttributeInstance operation order.
    pub fn effective_value(&self, min_value: f64, max_value: f64) -> f64 {
        assert!(min_value <= max_value, "attribute range must be ordered");

        let mut base = self.base_value();
        for modifier in self.modifiers.iter().filter(|modifier| {
            modifier.operation() == 0 || (modifier.operation() != 1 && modifier.operation() != 2)
        }) {
            base += modifier.amount();
        }

        let mut result = base;
        for modifier in self
            .modifiers
            .iter()
            .filter(|modifier| modifier.operation() == 1)
        {
            result += base * modifier.amount();
        }
        for modifier in self
            .modifiers
            .iter()
            .filter(|modifier| modifier.operation() == 2)
        {
            result *= 1.0 + modifier.amount();
        }

        if result.is_nan() {
            min_value
        } else {
            result.clamp(min_value, max_value)
        }
    }
}
