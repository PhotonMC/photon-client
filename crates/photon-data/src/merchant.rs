//! What a villager or a wandering trader offers: the value types of `MerchantOffers`.

use crate::item_components::Patch;
use crate::item_stack::ClientItemStackState as Stack;
use photon_core::ids::ItemId;

/// `ItemCost`: an item, how many, and the components the payment must carry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemCost {
    /// The item's registry id.
    pub item: ItemId,
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
