//! What a villager or a wandering trader offers (`MerchantOffers`, `MerchantOffer`, `ItemCost`):
//! the prices as the demand and the player's standing move them, and which offer the items in
//! the payment slots buy.

use crate::menus::container::max_stack_size;
use voxel_world::ClientItemStackState as Stack;
use voxel_world::item_components::Patch;

/// `ItemCost`: an item, how many, and the components the payment must carry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemCost {
    /// The item's registry id.
    pub item: i32,
    pub count: i32,
    /// `DataComponentExactPredicate`: each component's type id and its value's bytes.
    pub components: Vec<(i32, Vec<u8>)>,
}

impl ItemCost {
    /// `ItemCost.itemStack` for a count: the item with the components the cost names.
    pub fn stack(&self, count: i32) -> Stack {
        let stack = Stack::new(count, self.item);
        if self.components.is_empty() {
            return stack;
        }
        stack.with_patch_bytes(
            Patch {
                added: self.components.clone(),
                removed: Vec::new(),
            }
            .trusted(),
        )
    }

    /// `ItemCost.test`: the item, and each named component the same on the stack.
    pub fn test(&self, stack: Option<&Stack>) -> bool {
        let Some(stack) = stack else {
            return false;
        };
        if stack.item_registry_id() != self.item {
            return false;
        }
        let values = crate::item::values::Values::new(self.item, stack.patch_bytes());
        self.components
            .iter()
            .all(|(id, value)| values.raw(*id) == Some(value.as_slice()))
    }
}

/// `MerchantOffer`.
#[derive(Debug, Clone, PartialEq)]
pub struct MerchantOffer {
    pub cost_a: ItemCost,
    pub cost_b: Option<ItemCost>,
    pub result: Stack,
    pub uses: i32,
    pub max_uses: i32,
    pub xp: i32,
    pub special_price_diff: i32,
    pub demand: i32,
    pub price_multiplier: f32,
}

impl MerchantOffer {
    /// `getModifiedCostCount`: the base count, raised by the demand and moved by the special price,
    /// at least one and at most a stack.
    fn modified_count(&self, cost: &ItemCost) -> i32 {
        let base = cost.count;
        let demand = (base.wrapping_mul(self.demand) as f32 * self.price_multiplier).floor() as i32;
        let max = max_stack_size(&cost.stack(1));
        (base + demand.max(0) + self.special_price_diff).clamp(1, max.max(1))
    }

    /// `getCostA`: the first payment at the price now.
    pub fn cost_a_now(&self) -> Stack {
        self.cost_a.stack(self.modified_count(&self.cost_a))
    }

    /// `getCostB`: the second payment, if there is one.
    pub fn cost_b_now(&self) -> Option<Stack> {
        self.cost_b.as_ref().map(|cost| cost.stack(cost.count))
    }

    /// `getBaseCostA`: the first payment at its base price.
    pub fn base_cost_a(&self) -> Stack {
        self.cost_a.stack(self.cost_a.count)
    }

    /// `isOutOfStock`.
    pub fn is_out_of_stock(&self) -> bool {
        self.uses >= self.max_uses
    }

    /// `satisfiedBy`.
    pub fn satisfied_by(&self, buy_a: Option<&Stack>, buy_b: Option<&Stack>) -> bool {
        let count = |stack: Option<&Stack>| stack.map_or(0, Stack::item_count);
        if !self.cost_a.test(buy_a) || count(buy_a) < self.modified_count(&self.cost_a) {
            return false;
        }
        match &self.cost_b {
            None => buy_b.is_none(),
            Some(cost) => cost.test(buy_b) && count(buy_b) >= cost.count,
        }
    }

    /// `take`: what the payments lose when the offer is taken, if they satisfy it.
    pub fn take(&self, buy_a: &mut Option<Stack>, buy_b: &mut Option<Stack>) -> bool {
        if !self.satisfied_by(buy_a.as_ref(), buy_b.as_ref()) {
            return false;
        }
        let price = self.modified_count(&self.cost_a);
        if let Some(stack) = buy_a.as_mut() {
            stack.shrink(price);
        }
        if let (Some(cost), Some(stack)) = (&self.cost_b, buy_b.as_mut()) {
            stack.shrink(cost.count);
        }
        for payment in [buy_a, buy_b] {
            if payment
                .as_ref()
                .is_some_and(|stack| stack.item_count() <= 0)
            {
                *payment = None;
            }
        }
        true
    }
}

/// `MerchantOffers.getRecipeFor`: the index of the offer the payments buy; the hinted one is
/// the only one tried when the hint is inside the list and past the first.
pub fn recipe_for(
    offers: &[MerchantOffer],
    buy_a: Option<&Stack>,
    buy_b: Option<&Stack>,
    hint: usize,
) -> Option<usize> {
    if hint > 0 && hint < offers.len() {
        return offers[hint].satisfied_by(buy_a, buy_b).then_some(hint);
    }
    offers
        .iter()
        .position(|offer| offer.satisfied_by(buy_a, buy_b))
}

/// What the server tells of the merchant it opened a menu on besides its offers.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Trader {
    pub level: i32,
    pub xp: i32,
    pub show_progress: bool,
    pub can_restock: bool,
}

/// What `ClientboundMerchantOffersPacket` carries.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MerchantState {
    pub offers: Vec<MerchantOffer>,
    pub trader: Trader,
}

/// `VillagerData.getMinXpPerLevel` and `getMaxXpPerLevel`: the experience a level starts and
/// ends at (none for a level that cannot be left).
pub fn level_xp_range(level: i32) -> (i32, i32) {
    const THRESHOLDS: [i32; 5] = [0, 10, 70, 150, 250];
    if can_level_up(level) {
        (THRESHOLDS[(level - 1) as usize], THRESHOLDS[level as usize])
    } else {
        (0, 0)
    }
}

/// `VillagerData.canLevelUp`.
pub fn can_level_up(level: i32) -> bool {
    (1..5).contains(&level)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cost(name: &str, count: i32) -> ItemCost {
        ItemCost {
            item: crate::item::id_of(name).unwrap(),
            count,
            components: Vec::new(),
        }
    }

    fn stack(name: &str, count: i32) -> Stack {
        Stack::new(count, crate::item::id_of(name).unwrap())
    }

    fn offer() -> MerchantOffer {
        MerchantOffer {
            cost_a: cost("minecraft:wheat", 20),
            cost_b: None,
            result: stack("minecraft:emerald", 1),
            uses: 0,
            max_uses: 16,
            xp: 2,
            special_price_diff: 0,
            demand: 0,
            price_multiplier: 0.05,
        }
    }

    #[test]
    fn demand_and_the_special_price_move_the_price_within_one_stack() {
        let mut offer = offer();
        assert_eq!(offer.cost_a_now().item_count(), 20);
        // Twenty and four more for a demand of four at five percent.
        offer.demand = 4;
        assert_eq!(offer.cost_a_now().item_count(), 24);
        offer.special_price_diff = -30;
        assert_eq!(offer.cost_a_now().item_count(), 1);
        offer.special_price_diff = 100;
        assert_eq!(offer.cost_a_now().item_count(), 64);
    }

    #[test]
    fn taking_an_offer_needs_the_price_and_leaves_the_rest() {
        let offer = offer();
        let mut short = Some(stack("minecraft:wheat", 19));
        assert!(!offer.take(&mut short, &mut None));
        let mut paid = Some(stack("minecraft:wheat", 25));
        assert!(offer.take(&mut paid, &mut None));
        assert_eq!(paid.map(|stack| stack.item_count()), Some(5));
        let mut exact = Some(stack("minecraft:wheat", 20));
        assert!(offer.take(&mut exact, &mut None));
        assert!(exact.is_none());
        // A second payment the offer does not ask for does not satisfy it.
        assert!(!offer.satisfied_by(
            Some(&stack("minecraft:wheat", 20)),
            Some(&stack("minecraft:paper", 1))
        ));
    }

    #[test]
    fn a_chosen_offer_past_the_first_is_the_only_one_tried() {
        let mut second = offer();
        second.cost_a = cost("minecraft:paper", 1);
        let offers = vec![offer(), second];
        let paper = stack("minecraft:paper", 1);
        assert_eq!(recipe_for(&offers, Some(&paper), None, 0), Some(1));
        assert_eq!(recipe_for(&offers, Some(&paper), None, 1), Some(1));
        let wheat = stack("minecraft:wheat", 20);
        assert_eq!(recipe_for(&offers, Some(&wheat), None, 0), Some(0));
        assert_eq!(recipe_for(&offers, Some(&wheat), None, 1), None);
    }

    #[test]
    fn the_levels_experience_runs_from_one_threshold_to_the_next() {
        assert_eq!(level_xp_range(1), (0, 10));
        assert_eq!(level_xp_range(4), (150, 250));
        assert_eq!(level_xp_range(5), (0, 0));
        assert!(can_level_up(4) && !can_level_up(5));
    }
}
