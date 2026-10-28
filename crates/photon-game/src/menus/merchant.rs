//! What a villager or a wandering trader offers (`MerchantOffers`, `MerchantOffer`, `ItemCost`):
//! the prices as the demand and the player's standing move them, and which offer the items in
//! the payment slots buy.

use crate::menus::container::max_stack_size;
pub use photon_data::merchant::{ItemCost, MerchantOffer, MerchantState, Trader};
use photon_world::ClientItemStackState as Stack;

/// `ItemCost.test`, which reads the item's own components.
pub trait ItemCostExt {
    /// `ItemCost.test`: the item, and each named component the same on the stack.
    fn test(&self, stack: Option<&Stack>) -> bool;
}

impl ItemCostExt for ItemCost {
    fn test(&self, stack: Option<&Stack>) -> bool {
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

/// The offer's prices and trades, which need the item's stack size and components.
pub trait MerchantOfferExt {
    /// `getCostA`: the first payment at the price now.
    fn cost_a_now(&self) -> Stack;
    /// `satisfiedBy`.
    fn satisfied_by(&self, buy_a: Option<&Stack>, buy_b: Option<&Stack>) -> bool;
    /// `take`: what the payments lose when the offer is taken, if they satisfy it.
    fn take(&self, buy_a: &mut Option<Stack>, buy_b: &mut Option<Stack>) -> bool;
}

/// `getModifiedCostCount`: the base count, raised by the demand and moved by the special price,
/// at least one and at most a stack.
fn modified_count(offer: &MerchantOffer, cost: &ItemCost) -> i32 {
    let base = cost.count;
    let demand = (base.wrapping_mul(offer.demand) as f32 * offer.price_multiplier).floor() as i32;
    let max = max_stack_size(&cost.stack(1));
    (base + demand.max(0) + offer.special_price_diff).clamp(1, max.max(1))
}

impl MerchantOfferExt for MerchantOffer {
    fn cost_a_now(&self) -> Stack {
        self.cost_a.stack(modified_count(self, &self.cost_a))
    }

    fn satisfied_by(&self, buy_a: Option<&Stack>, buy_b: Option<&Stack>) -> bool {
        let count = |stack: Option<&Stack>| stack.map_or(0, Stack::item_count);
        if !self.cost_a.test(buy_a) || count(buy_a) < modified_count(self, &self.cost_a) {
            return false;
        }
        match &self.cost_b {
            None => buy_b.is_none(),
            Some(cost) => cost.test(buy_b) && count(buy_b) >= cost.count,
        }
    }

    fn take(&self, buy_a: &mut Option<Stack>, buy_b: &mut Option<Stack>) -> bool {
        if !self.satisfied_by(buy_a.as_ref(), buy_b.as_ref()) {
            return false;
        }
        let price = modified_count(self, &self.cost_a);
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
