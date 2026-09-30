//! What a recipe update gives the client besides its property sets: the
//! stonecutter's recipes, each an ingredient and what the screen shows for
//! it (`SelectableRecipe.SingleInputSet`, without the recipes themselves,
//! which the server keeps).

/// An `Ingredient` as the network writes it: items, or a tag of them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ingredient {
    Items(Vec<i32>),
    Tag(String),
}

/// A `SlotDisplay`: what a slot shows for a recipe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlotDisplay {
    Empty,
    AnyFuel,
    WithAnyPotion(Box<SlotDisplay>),
    OnlyWithComponent(Box<SlotDisplay>, i32),
    Item(i32),
    /// An item stack: its item and count; the components are not kept.
    ItemStack(i32, i32),
    Tag(String),
    Dyed(Box<SlotDisplay>, Box<SlotDisplay>),
    SmithingTrim(Box<SlotDisplay>, Box<SlotDisplay>, i32),
    WithRemainder(Box<SlotDisplay>, Box<SlotDisplay>),
    Composite(Vec<SlotDisplay>),
}

impl SlotDisplay {
    /// `resolveForFirstStack`'s item: the first thing the display shows, for
    /// the displays that name an item.
    pub fn first_item(&self) -> Option<i32> {
        match self {
            Self::Item(item) | Self::ItemStack(item, _) => Some(*item),
            Self::WithAnyPotion(inner)
            | Self::OnlyWithComponent(inner, _)
            | Self::WithRemainder(inner, _) => inner.first_item(),
            Self::Dyed(_, target) => target.first_item(),
            Self::SmithingTrim(base, _, _) => base.first_item(),
            Self::Composite(all) => all.iter().find_map(Self::first_item),
            Self::Empty | Self::AnyFuel | Self::Tag(_) => None,
        }
    }
}

/// One stonecutter entry: what it takes and what it shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StonecutterEntry {
    pub input: Ingredient,
    pub display: SlotDisplay,
}
