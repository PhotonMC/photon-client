use super::*;

impl ClientEntity {
    /// `Strider.isSuffocating`.
    pub fn is_suffocating(&self) -> bool {
        self.living.strider_suffocating
    }

    /// A horse's animations and tail, if it has ticked.
    pub fn horse(&self) -> Option<&crate::mobs::horse::HorseState> {
        self.kind.horse()
    }

    /// `Horse.getVariant` and `getMarkings`: ids, each wrapped into its enum
    /// (seven variants, five markings).
    pub fn horse_look(&self) -> (i32, i32) {
        let data = self.living.horse_variant;
        (
            (data & 0xFF).rem_euclid(7),
            ((data & 0xFF00) >> 8).rem_euclid(5),
        )
    }

    /// `Llama.getVariant`'s id, kept within the four variants (an id past the
    /// last takes the last).
    pub fn llama_variant(&self) -> i32 {
        self.living.horse_variant.clamp(0, 3)
    }

    /// `AbstractChestedHorse.hasChest`.
    pub fn horse_has_chest(&self) -> bool {
        self.living.horse_chest
    }

    /// Fixes the source a horse's tail chance is drawn from (see
    /// [`crate::mobs::horse::HorseState::fix_seed`]).
    pub fn fix_horse_seed(&mut self, seed: i64) {
        self.kind
            .horse_or_insert_with(|| Box::new(crate::mobs::horse::HorseState::new()))
            .fix_seed(seed);
    }

    /// Whether this is a camel or a camel husk.
    pub fn is_camel(&self) -> bool {
        matches!(
            photon_data::entity_type::name(self.type_id),
            Some("minecraft:camel" | "minecraft:camel_husk")
        )
    }

    /// Whether this is a horse, donkey, mule or undead horse, saddled or not.
    pub fn is_horse(&self) -> bool {
        crate::ride::mount::is_horse_type(self.type_id)
    }

    /// Whether this is a pig.
    pub fn is_pig(&self) -> bool {
        photon_data::entity_type::name(self.type_id) == Some("minecraft:pig")
    }

    /// `Strider.setSuffocating`: the flag, and the cold's modifier on the movement speed (a third
    /// less of the base).
    pub(crate) fn set_strider_suffocating(&mut self, suffocating: bool) {
        self.living.strider_suffocating = suffocating;
        let Some((id, _)) = photon_data::entity_type::attribute("minecraft:movement_speed") else {
            return;
        };
        let Ok(attribute_id) = i32::try_from(id) else {
            return;
        };
        let position = self
            .attributes
            .iter()
            .position(|state| state.attribute_id == attribute_id);
        let modifier_id = "minecraft:suffocating";
        if suffocating {
            let modifier =
                ClientAttributeModifierState::new(modifier_id.to_string(), f64::from(-0.34_f32), 1);
            match position {
                Some(index) => {
                    let modifiers = &mut self.attributes[index].modifiers;
                    modifiers.retain(|existing| existing.id != modifier_id);
                    modifiers.push(modifier);
                }
                None => {
                    let base = photon_data::entity_type::name(self.type_id)
                        .and_then(|name| {
                            photon_data::entity_type::default_attribute(
                                name,
                                "minecraft:movement_speed",
                            )
                        })
                        .unwrap_or(0.0);
                    self.attributes.push(ClientAttributeState::new(
                        attribute_id,
                        base,
                        vec![modifier],
                    ));
                }
            }
        } else if let Some(index) = position {
            self.attributes[index]
                .modifiers
                .retain(|existing| existing.id != modifier_id);
        }
    }

    /// Whether this is a strider in a saddle (which a warped fungus on a stick steers).
    pub fn is_saddled_strider(&self) -> bool {
        photon_data::entity_type::name(self.type_id) == Some("minecraft:strider")
            && self.equipment(EquipmentSlot::Saddle).is_some()
    }

    /// Whether this is a pig in a saddle (which a carrot on a stick steers).
    pub fn is_saddled_pig(&self) -> bool {
        photon_data::entity_type::name(self.type_id) == Some("minecraft:pig")
            && self.equipment(EquipmentSlot::Saddle).is_some()
    }

    /// Whether this is a horse, donkey, mule or undead horse in a saddle, which the player on it
    /// steers (`AbstractHorse.getControllingPassenger`).
    pub fn is_controllable_horse(&self) -> bool {
        crate::ride::mount::is_horse_type(self.type_id)
            && self.equipment(EquipmentSlot::Saddle).is_some()
    }
}
