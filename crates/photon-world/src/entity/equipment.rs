use super::*;
use photon_core::ids::BlockStateId;
use photon_core::ids::EntityTypeId;

impl ClientEntity {
    /// `Piglin.isChargingCrossbow`.
    pub fn is_charging_crossbow(&self) -> bool {
        self.living.charging_crossbow
    }

    /// The stack a dropped item holds.
    pub fn item_stack(&self) -> Option<&ClientItemStackState> {
        self.item_stack.as_ref()
    }

    /// `FireworkRocketEntity.getExplosions`: the explosions of the rocket's `fireworks` component;
    /// none for a stack without it (the rocket's default holds none).
    pub fn firework_explosions(&self) -> Vec<crate::particles::FireworkExplosion> {
        let Some(stack) = self.item_stack.as_ref() else {
            return Vec::new();
        };
        let mut input = stack.patch_bytes();
        crate::item_components::read_patch(&mut input)
            .ok()
            .and_then(|patch| {
                let (_, value) = patch
                    .added
                    .iter()
                    .find(|(id, _)| *id == photon_data::item::component_types::FIREWORKS)?;
                crate::item_components::fireworks(value)
            })
            .unwrap_or_default()
    }

    /// What the entity holds or wears in a slot (`getItemBySlot`); none for
    /// an empty one.
    pub fn equipment(&self, slot: EquipmentSlot) -> Option<&ClientItemStackState> {
        self.equipment[slot as usize].as_ref()
    }

    /// The block state an enderman carries (`getCarriedBlock`), by its id.
    pub fn carried_block_state(&self) -> Option<BlockStateId> {
        Some(self.living.carried_state)
            .filter(|state| *state > 0)
            .map(BlockStateId)
    }

    /// An armor stand's flags and poses.
    pub fn armor_stand(&self) -> &crate::objects::armor_stand::ArmorStandState {
        &self.living.armor_stand
    }

    /// `getItemHeldByArm`.
    pub fn item_held_by_arm(&self, left: bool) -> Option<&ClientItemStackState> {
        self.equipment(if left == self.main_arm_is_left() {
            EquipmentSlot::MainHand
        } else {
            EquipmentSlot::OffHand
        })
    }
}

pub(super) fn armor_stand_type_id() -> EntityTypeId {
    photon_data::entity_type::id("minecraft:armor_stand").unwrap_or(EntityTypeId(-1))
}

/// `EquipmentSlot`, in the order the wire counts them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentSlot {
    MainHand,
    OffHand,
    Feet,
    Legs,
    Chest,
    Head,
    Body,
    Saddle,
}

impl EquipmentSlot {
    pub const COUNT: usize = 8;

    pub fn from_ordinal(ordinal: usize) -> Option<Self> {
        Some(match ordinal {
            0 => Self::MainHand,
            1 => Self::OffHand,
            2 => Self::Feet,
            3 => Self::Legs,
            4 => Self::Chest,
            5 => Self::Head,
            6 => Self::Body,
            7 => Self::Saddle,
            _ => return None,
        })
    }
}
