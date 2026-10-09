//! `HorseInventoryMenu` and `NautilusInventoryMenu` (`AbstractMountInventoryMenu`): the saddle,
//! the body armour, the mount's chest (three rows of the columns the server names) and then the
//! player's 36. The mount's own slots carry the rules of `ArmorSlot`: the entity decides, by
//! `canUseSlot` and the item's `equippable`, what may be put in.

use super::*;
use crate::item::equippable_allowed::Allowed;

/// Which screen and rules a mount has.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MountFamily {
    /// `AbstractHorse` but a llama: the horse, donkey, mule, skeleton and zombie horse, and the
    /// camel and the camel husk.
    Horse,
    /// `Llama` and `TraderLlama`.
    Llama,
    /// `AbstractNautilus`.
    Nautilus,
}

impl MountFamily {
    /// The family an entity type belongs to; none for one with no mount screen.
    pub fn of_entity(name: &str) -> Option<Self> {
        Some(match name {
            "minecraft:horse"
            | "minecraft:donkey"
            | "minecraft:mule"
            | "minecraft:skeleton_horse"
            | "minecraft:zombie_horse"
            | "minecraft:camel"
            | "minecraft:camel_husk" => Self::Horse,
            "minecraft:llama" | "minecraft:trader_llama" => Self::Llama,
            "minecraft:nautilus" | "minecraft:zombie_nautilus" => Self::Nautilus,
            _ => return None,
        })
    }
}

/// A mount's menu: its family and the columns of its chest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mount {
    pub family: MountFamily,
    pub columns: u8,
}

/// What the menu reads of the mount entity when it opens.
#[derive(Debug, Clone, Default)]
pub struct MountEntity {
    /// The entity type's registry id.
    pub entity_type: i32,
    /// `canUseSlot(SADDLE)` and `canUseSlot(BODY)`.
    pub can_use: [bool; 2],
    /// The entity type tags the server sent.
    pub entity_tags: std::sync::Arc<photon_world::block::tags::ItemTags>,
}

/// The id `ClientContainerState` gives the menu a `ClientboundMountScreenOpenPacket` opens, which
/// has no menu type of its own.
pub const MOUNT_MENU_TYPE: i32 = -2;

impl MountEntity {
    /// `canUseSlot(SADDLE)` and `canUseSlot(BODY)` of a mount that is alive: the horse, the
    /// skeleton and zombie horse and the llamas use every slot, the donkey, mule and camels
    /// need a tame adult for the saddle, the nautilus for both.
    pub fn can_use_slots(entity: &str, baby: bool, tame: bool) -> [bool; 2] {
        match entity {
            "minecraft:horse"
            | "minecraft:skeleton_horse"
            | "minecraft:zombie_horse"
            | "minecraft:llama"
            | "minecraft:trader_llama" => [true, true],
            "minecraft:nautilus" | "minecraft:zombie_nautilus" => [!baby && tame; 2],
            _ => [!baby && tame, true],
        }
    }

    /// `Equippable.canBeEquippedBy`.
    fn allows(&self, allowed: Option<&Allowed>) -> bool {
        match allowed {
            None => true,
            Some(Allowed::Tag(tag)) => self.entity_tags.contains(self.entity_type, tag),
            Some(Allowed::Types(types)) => photon_data::entity_type::name(self.entity_type)
                .is_some_and(|name| types.iter().any(|other| other == name)),
        }
    }
}

impl ContainerMenu {
    /// The mount slot's equipment slot, for the saddle (0) and the body armour (1).
    fn mount_slot_name(index: usize) -> Option<&'static str> {
        match index {
            0 => Some("saddle"),
            1 => Some("body"),
            _ => None,
        }
    }

    /// `ArmorSlot.mayPlace`: `LivingEntity.isEquippableInSlot`.
    pub(super) fn mount_may_place(&self, index: usize, stack: &Stack) -> bool {
        let Some(slot) = Self::mount_slot_name(index) else {
            return true;
        };
        crate::item::values::equippable_of(stack.item_registry_id(), stack.patch_bytes())
            .is_some_and(|equippable| {
                equippable.slot == slot
                    && self.mount.can_use[index]
                    && self.mount.allows(equippable.allowed_entities.as_ref())
            })
    }

    /// `AbstractMountInventoryMenu.quickMoveStack`.
    pub(super) fn quick_move_mount(&mut self, index: usize, player: Player) -> Option<Stack> {
        let MenuKind::Mount(mount) = self.kind else {
            return None;
        };
        let original = self.slots[index].as_ref()?.clone();
        let clicked = original.copy_with_count(original.item_count());
        let mut stack = self.slots[index].take().expect("checked above");
        let size = self.slots.len();
        let player_start = 2 + usize::from(mount.columns) * 3;
        let moved = if index < player_start {
            self.move_item_stack_to(&mut stack, player_start, size, true)
        } else if self.may_place(1, &stack) && self.slots[1].is_none() {
            self.move_item_stack_to(&mut stack, 1, 2, false)
        } else if self.may_place(0, &stack) && self.slots[0].is_none() {
            self.move_item_stack_to(&mut stack, 0, 1, false)
        } else if player_start == 2 || !self.move_item_stack_to(&mut stack, 2, player_start, false)
        {
            // The reference shuffles the player's own rows and answers nothing either way.
            let (main_end, hotbar_end) = (player_start + 27, player_start + 36);
            let _shuffled = if (main_end..hotbar_end).contains(&index) {
                self.move_item_stack_to(&mut stack, player_start, main_end, false)
            } else if (player_start..main_end).contains(&index) {
                self.move_item_stack_to(&mut stack, main_end, hotbar_end, false)
            } else {
                false
            };
            false
        } else {
            true
        };
        if !moved {
            self.slots[index] = Some(stack);
            settle(&mut self.slots[index]);
            return None;
        }
        self.finish_quick_move(index, stack, &clicked, player)
    }

    /// `ArmorSlot.mayPickup`: armour that stays on its wearer is kept but in creative.
    pub(super) fn mount_keeps(&self, index: usize) -> bool {
        matches!(self.kind, MenuKind::Mount(_))
            && index < 2
            && !self.infinite_materials
            && self.slots[index].as_ref().is_some_and(|stack| {
                super::components::enchantments_for_crafting(stack)
                    .iter()
                    .any(|&(id, _)| self.traits(id).prevents_armor_change)
            })
    }
}
