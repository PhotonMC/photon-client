//! `AnvilMenu.createResult`, which the client runs too: the result of the
//! two inputs, what it costs in levels, and what `onTake` uses up.

use super::components::{
    damage, enchantment_type, enchantments_for_crafting, has, int_or, is_damageable, level,
    max_damage, remove, set, set_damage, set_int, update_enchantments,
};
use super::*;
use photon_world::item_components::ids;

/// What `AnvilMenu` keeps between `createResult` calls.
#[derive(Debug, Clone, Default)]
pub struct AnvilState {
    /// `repairItemCountCost`: how many of the additional item a repair uses.
    pub(super) repair_item_count_cost: i32,
    /// `itemName`: the text of the name box.
    pub(super) item_name: Option<String>,
    /// `onlyRenaming`.
    pub(super) only_renaming: bool,
}

/// `AnvilMenu.MAX_NAME_LENGTH`.
const MAX_NAME_LENGTH: usize = 50;
/// The most a result may cost outside creative.
const TOO_EXPENSIVE: i64 = 40;

/// `StringUtil.isBlank`.
fn is_blank(text: Option<&str>) -> bool {
    text.is_none_or(|text| text.chars().all(char::is_whitespace))
}

/// A literal text component as a component's network NBT: a string tag.
fn literal_name(name: &str) -> Vec<u8> {
    let mut out = Vec::new();
    photon_core::nbt::write_any_nbt(
        Some(&photon_core::nbt::FieldValue::Str(name.to_owned())),
        &mut out,
    );
    out
}

/// `AnvilMenu.calculateIncreasedRepairCost`.
fn increased_repair_cost(base: i32) -> i32 {
    (i64::from(base) * 2 + 1).min(i64::from(i32::MAX)) as i32
}

impl ContainerMenu {
    /// `ItemStack.isValidRepairItem`: the `repairable` component's items hold
    /// the other stack's item.
    fn is_valid_repair_item(&self, stack: &Stack, repair: &Stack) -> bool {
        let Some(value) = super::components::get(stack, ids::REPAIRABLE) else {
            return false;
        };
        let mut input = value.as_slice();
        let Ok(count) = photon_core::wire::read_varint(&mut input) else {
            return false;
        };
        if count == 0 {
            // A tag, named by a string.
            let Ok(length) = photon_core::wire::read_varint(&mut input) else {
                return false;
            };
            let name = input.get(..length as usize).unwrap_or_default();
            return std::str::from_utf8(name).is_ok_and(|tag| self.in_tag(repair, tag));
        }
        (1..count).any(|_| {
            photon_core::wire::read_varint(&mut input)
                .is_ok_and(|item| ItemId(item) == repair.item_registry_id())
        })
    }

    /// `Enchantment.canEnchant`: the item is in the enchantment's supported tag.
    fn can_enchant(&self, enchantment: i32, stack: &Stack) -> bool {
        self.in_tag(stack, &self.traits(enchantment).supported_items)
    }

    /// `Enchantment.areCompatible`.
    fn are_compatible(&self, a: i32, b: i32) -> bool {
        a != b
            && !self.traits(a).exclusive_set.contains(&b)
            && !self.traits(b).exclusive_set.contains(&a)
    }

    /// `ItemStack.getHoverName().getString()`.
    fn hover_name_string(&self, stack: &Stack) -> String {
        let values =
            crate::item::values::Values::new(stack.item_registry_id(), stack.patch_bytes());
        self.lang.plain(&crate::tooltip::hover_name(stack, &values))
    }

    fn set_cost(&mut self, cost: i64) {
        if self.data.is_empty() {
            self.data.push(0);
        }
        self.data[0] = i16::try_from(cost).unwrap_or(i16::MAX);
    }

    /// `AnvilMenu.createResult`.
    pub(super) fn anvil_create_result(&mut self) {
        self.anvil.only_renaming = false;
        let Some(input) = self.slots[0]
            .clone()
            .filter(|input| has(input, enchantment_type(input)))
        else {
            self.slots[2] = None;
            self.set_cost(0);
            return;
        };
        let addition = self.slots[1].clone();
        let mut result = input.copy_with_count(input.item_count());
        let mut enchantments = enchantments_for_crafting(&result);
        let mut price: i64 = 0;
        let mut naming_cost: i64 = 0;
        let tax = i64::from(int_or(Some(&input), ids::REPAIR_COST, 0))
            + i64::from(int_or(addition.as_ref(), ids::REPAIR_COST, 0));
        self.anvil.repair_item_count_cost = 0;
        if let Some(addition) = &addition {
            let using_book = has(addition, ids::STORED_ENCHANTMENTS);
            if is_damageable(&result) && self.is_valid_repair_item(&input, addition) {
                let mut repair = damage(&result).min(max_damage(&result) / 4);
                if repair <= 0 {
                    self.slots[2] = None;
                    self.set_cost(0);
                    return;
                }
                let mut count = 0;
                while repair > 0 && count < addition.item_count() {
                    let repaired = damage(&result) - repair;
                    set_damage(&mut result, repaired);
                    price += 1;
                    repair = damage(&result).min(max_damage(&result) / 4);
                    count += 1;
                }
                self.anvil.repair_item_count_cost = count;
            } else {
                if !using_book
                    && (result.item_registry_id() != addition.item_registry_id()
                        || !is_damageable(&result))
                {
                    self.slots[2] = None;
                    self.set_cost(0);
                    return;
                }
                if is_damageable(&result) && !using_book {
                    let remaining1 = max_damage(&input) - damage(&input);
                    let remaining2 = max_damage(addition) - damage(addition);
                    let additional = remaining2 + max_damage(&result) * 12 / 100;
                    let result_damage = (max_damage(&result) - (remaining1 + additional)).max(0);
                    if result_damage < damage(&result) {
                        set_damage(&mut result, result_damage);
                        price += 2;
                    }
                }
                let mut any_compatible = false;
                let mut any_incompatible = false;
                for (id, added) in enchantments_for_crafting(addition) {
                    let current = level(&enchantments, id);
                    let mut level_now = if current == added {
                        added + 1
                    } else {
                        added.max(current)
                    };
                    let mut compatible = self.can_enchant(id, &input)
                        || self.infinite_materials
                        || Some(input.item_registry_id())
                            == crate::item::id_of("minecraft:enchanted_book");
                    for &(other, _) in &enchantments {
                        if other != id && !self.are_compatible(id, other) {
                            compatible = false;
                            price += 1;
                        }
                    }
                    if compatible {
                        any_compatible = true;
                        let traits = self.traits(id);
                        level_now = level_now.min(traits.max_level);
                        match enchantments.iter_mut().find(|(entry, _)| *entry == id) {
                            Some(entry) => entry.1 = level_now,
                            None => enchantments.push((id, level_now)),
                        }
                        let mut fee = traits.anvil_cost;
                        if using_book {
                            fee = (fee / 2).max(1);
                        }
                        price += i64::from(fee) * i64::from(level_now);
                        if input.item_count() > 1 {
                            price = TOO_EXPENSIVE;
                        }
                    } else {
                        any_incompatible = true;
                    }
                }
                if any_incompatible && !any_compatible {
                    self.slots[2] = None;
                    self.set_cost(0);
                    return;
                }
            }
        }

        let name = self.anvil.item_name.clone();
        if !is_blank(name.as_deref()) {
            let name = name.unwrap_or_default();
            if name != self.hover_name_string(&input) {
                naming_cost = 1;
                price += naming_cost;
                set(&mut result, ids::CUSTOM_NAME, literal_name(&name));
            }
        } else if has(&input, ids::CUSTOM_NAME) {
            naming_cost = 1;
            price += naming_cost;
            remove(&mut result, ids::CUSTOM_NAME);
        }

        let mut cost = if price <= 0 {
            0
        } else {
            (tax + price).clamp(0, i64::from(i32::MAX))
        };
        let mut made = (price > 0).then_some(result);
        if naming_cost == price && naming_cost > 0 {
            if cost >= TOO_EXPENSIVE {
                cost = TOO_EXPENSIVE - 1;
            }
            self.anvil.only_renaming = true;
        }
        if cost >= TOO_EXPENSIVE && !self.infinite_materials {
            made = None;
        }
        if let Some(result) = made.as_mut() {
            let mut base = int_or(Some(result), ids::REPAIR_COST, 0);
            base = base.max(int_or(addition.as_ref(), ids::REPAIR_COST, 0));
            if naming_cost != price || naming_cost == 0 {
                base = increased_repair_cost(base);
            }
            set_int(result, ids::REPAIR_COST, base);
            let kind = enchantment_type(result);
            if has(result, kind) {
                update_enchantments(result, |current| *current = enchantments);
            }
        }
        self.slots[2] = made;
        self.set_cost(cost);
    }

    /// `AnvilMenu.setItemName`: whether the name was news.
    pub fn set_item_name(&mut self, name: &str) -> bool {
        let name: String = name
            .chars()
            .filter(|&c| c != '\u{a7}' && c >= ' ' && c != '\u{7f}')
            .collect();
        if name.encode_utf16().count() > MAX_NAME_LENGTH
            || self.anvil.item_name.as_deref() == Some(name.as_str())
        {
            return false;
        }
        self.anvil.item_name = Some(name.clone());
        if let Some(result) = self.slots[2].as_mut() {
            if is_blank(Some(&name)) {
                remove(result, ids::CUSTOM_NAME);
            } else {
                set(result, ids::CUSTOM_NAME, literal_name(&name));
            }
        }
        self.anvil_create_result();
        true
    }

    /// `AnvilMenu.onTake`: the additional item a repair used up, or all of
    /// it; the input goes.
    pub(super) fn anvil_taken(&mut self) {
        let used = self.anvil.repair_item_count_cost;
        if used > 0 {
            match self.slots[1].as_mut() {
                Some(addition) if addition.item_count() > used => addition.shrink(used),
                _ => self.slots[1] = None,
            }
        } else if !self.anvil.only_renaming {
            self.slots[1] = None;
        }
        self.set_cost(0);
        self.slots[0] = None;
    }
}
