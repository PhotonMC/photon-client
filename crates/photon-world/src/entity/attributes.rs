use super::*;
use photon_core::java_float::{JavaF32, JavaF64};

impl ClientEntity {
    /// `LivingEntity.getAttributeValue`: the attribute's value, from what the
    /// type starts with and what the server has changed (`AttributeInstance.getValue`:
    /// the base, the modifiers in their order, kept in the attribute's range).
    /// Nothing for an attribute the type does not have.
    pub fn attribute_value(&self, attribute: &str) -> Option<f64> {
        effective_attribute(
            &self.attributes,
            photon_data::entity_type::name(self.type_id)?,
            attribute,
        )
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

/// `AttributeMap.getValue` of an entity of type `type_name` that the server has changed
/// `changed` of: the type's base value or the changed one with its modifiers, kept in range.
pub(crate) fn effective_attribute(
    changed: &[ClientAttributeState],
    type_name: &str,
    attribute: &str,
) -> Option<f64> {
    let (id, range) = photon_data::entity_type::attribute(attribute)?;
    let default = photon_data::entity_type::default_attribute(type_name, attribute)?;
    let (low, high) = range.range.unwrap_or((f64::NEG_INFINITY, f64::INFINITY));
    let state = changed
        .iter()
        .find(|state| usize::try_from(state.attribute_id) == Ok(id));
    Some(state.map_or_else(
        || ClientAttributeState::new(0, default, Vec::new()).effective_value(low, high),
        |state| state.effective_value(low, high),
    ))
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
    /// `MobEffectInstance.BlendState`: how far the effect has faded in, this tick and the last.
    pub(crate) blend: (JavaF32, JavaF32),
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

    /// An effect as a client first has it from a packet that does (`blends`) or does not ask for it
    /// to fade in.
    pub fn new(effect_id: i32, amplifier: i32, duration_ticks: i32, blends: bool) -> Self {
        let mut effect = Self {
            effect_id,
            amplifier,
            duration_ticks,
            flags: if blends { 0b1000 } else { 0 },
            blend: (JavaF32::new(0.0), JavaF32::new(0.0)),
        };
        if !blends {
            effect.skip_blending();
        }
        effect
    }

    /// `GameRenderer.nightVisionScale` of a night vision effect: full, then a flicker over its last
    /// ten seconds.
    pub fn night_vision_scale(&self, partial: f32) -> f32 {
        if !self.ends_within(200) {
            return 1.0;
        }
        let angle = (self.duration_ticks as f32 - partial) * std::f32::consts::PI * 0.2;
        0.7 + photon_core::math::mth_sin_f64(f64::from(angle)) * 0.3
    }

    /// `MobEffectInstance.tickClient`: the duration counts down, and the blend moves a step.
    pub fn tick_client(&mut self) {
        if self.has_remaining_duration() && self.duration_ticks != -1 {
            self.duration_ticks -= 1;
        }
        self.tick_blend();
    }

    /// `MobEffectInstance.endsWithin`: not infinite, and no more than `ticks` left.
    pub fn ends_within(&self, ticks: i32) -> bool {
        self.duration_ticks != -1 && self.duration_ticks <= ticks
    }

    /// `MobEffect.setBlendDuration`: how many ticks the effect fades in and out over, and how
    /// many ticks before its end it starts to fade out. Only nausea and darkness blend.
    fn blend_durations(&self) -> (i32, i32, i32) {
        match photon_data::item::mob_effects::MOB_EFFECTS
            .get(usize::try_from(self.effect_id).unwrap_or(usize::MAX))
            .map(|effect| effect.name)
        {
            Some("minecraft:nausea") => (150, 20, 60),
            Some("minecraft:darkness") => (22, 22, 22),
            _ => (0, 0, 0),
        }
    }

    /// `MobEffectInstance.BlendState.hasEffect`.
    fn blends_in(&self) -> bool {
        !self.ends_within(self.blend_durations().2)
    }

    /// `MobEffectInstance.skipBlending`: the effect is as faded in as it will be at once.
    pub(crate) fn skip_blending(&mut self) {
        let factor = JavaF32::new(if self.blends_in() { 1.0 } else { 0.0 });
        self.blend = (factor, factor);
    }

    /// `MobEffectInstance.BlendState.tick`.
    fn tick_blend(&mut self) {
        let (factor, _) = self.blend;
        let (fade_in, fade_out, _) = self.blend_durations();
        let has_effect = self.blends_in();
        let target = if has_effect { 1.0_f32 } else { 0.0_f32 };
        let mut next = factor.get();
        if next != target {
            let duration = if has_effect { fade_in } else { fade_out };
            if duration == 0 {
                next = target;
            } else {
                let step = 1.0_f32 / duration as f32;
                next += (target - next).clamp(-step, step);
            }
        }
        self.blend = (JavaF32::new(next), factor);
    }

    /// The blend's factor this tick and the last.
    pub fn blend_state(&self) -> (f32, f32) {
        (self.blend.0.get(), self.blend.1.get())
    }

    /// `MobEffectInstance.getBlendFactor`: between the last tick's and this one's.
    pub fn blend_factor(&self, partial: f32) -> f32 {
        photon_core::math::mth_lerp_f32(partial, self.blend.1.get(), self.blend.0.get())
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
