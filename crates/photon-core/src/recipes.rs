//! What a recipe update gives the client besides its property sets: the
//! stonecutter's recipes, each an ingredient and what the screen shows for
//! it (`SelectableRecipe.SingleInputSet`, without the recipes themselves,
//! which the server keeps).

use crate::ids::ItemId;

/// An `Ingredient` as the network writes it: items, or a tag of them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ingredient {
    Items(Vec<ItemId>),
    Tag(String),
}

/// A `SlotDisplay`: what a slot shows for a recipe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlotDisplay {
    Empty,
    AnyFuel,
    WithAnyPotion(Box<SlotDisplay>),
    OnlyWithComponent(Box<SlotDisplay>, i32),
    Item(ItemId),
    /// An item stack: its item and count; the components are not kept.
    ItemStack(ItemId, i32),
    Tag(String),
    Dyed(Box<SlotDisplay>, Box<SlotDisplay>),
    SmithingTrim(Box<SlotDisplay>, Box<SlotDisplay>, i32),
    WithRemainder(Box<SlotDisplay>, Box<SlotDisplay>),
    Composite(Vec<SlotDisplay>),
}

impl SlotDisplay {
    /// `resolveForFirstStack`'s item: the first thing the display shows, for
    /// the displays that name an item.
    pub fn first_item(&self) -> Option<ItemId> {
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

/// A `RecipeDisplay`: what the recipe book shows for a recipe. The crafting
/// station is kept beside it, in the entry: only the recipe toast draws it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecipeDisplay {
    Shapeless {
        ingredients: Vec<SlotDisplay>,
        result: SlotDisplay,
    },
    Shaped {
        width: i32,
        height: i32,
        ingredients: Vec<SlotDisplay>,
        result: SlotDisplay,
    },
    Furnace {
        ingredient: SlotDisplay,
        fuel: SlotDisplay,
        result: SlotDisplay,
    },
    Stonecutter {
        result: SlotDisplay,
    },
    Smithing {
        result: SlotDisplay,
    },
}

impl RecipeDisplay {
    pub fn result(&self) -> &SlotDisplay {
        match self {
            Self::Shapeless { result, .. }
            | Self::Shaped { result, .. }
            | Self::Furnace { result, .. }
            | Self::Stonecutter { result }
            | Self::Smithing { result } => result,
        }
    }
}

/// A `RecipeDisplayEntry`: a recipe the player knows, by its display id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeDisplayEntry {
    pub id: i32,
    pub display: RecipeDisplay,
    /// `RecipeDisplay.craftingStation`.
    pub station: SlotDisplay,
    /// `OptionalInt group`: entries of one category and group share a button.
    pub group: Option<i32>,
    /// A `RecipeBookCategories` registry id.
    pub category: i32,
    /// What the recipe takes, for the craftable check; none when the recipe
    /// cannot be crafted in a menu.
    pub requirements: Option<Vec<Ingredient>>,
}

/// `RecipeBookSettings` of one `RecipeBookType`: whether its book is open and
/// whether it lists only what can be crafted.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BookTypeSettings {
    pub open: bool,
    pub filtering: bool,
}

/// `ClientRecipeBook`'s state the server sends: the known recipes, the ones
/// to highlight as new, the settings of the four book types (crafting,
/// furnace, blast furnace, smoker) and the ghost recipe the server placed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecipeBook {
    pub known: std::collections::BTreeMap<i32, RecipeDisplayEntry>,
    pub highlight: std::collections::BTreeSet<i32>,
    pub settings: [BookTypeSettings; 4],
    /// Bumped on every change, so a view knows when to rebuild.
    pub revision: u64,
    /// `PlaceGhostRecipe`: the container it is for and what it shows.
    pub ghost: Option<(i32, RecipeDisplay)>,
}

impl RecipeBook {
    /// `handleRecipeBookAdd`: `replace` empties the book first; a flag byte's
    /// bit 2 marks the entry new.
    pub fn add(&mut self, entries: Vec<(RecipeDisplayEntry, u8)>, replace: bool) {
        if replace {
            self.known.clear();
            self.highlight.clear();
        }
        for (entry, flags) in entries {
            if flags & 2 != 0 {
                self.highlight.insert(entry.id);
            }
            self.known.insert(entry.id, entry);
        }
        self.revision += 1;
    }

    /// `handleRecipeBookRemove`.
    pub fn remove(&mut self, ids: &[i32]) {
        for id in ids {
            self.known.remove(id);
            self.highlight.remove(id);
        }
        self.revision += 1;
    }

    /// `handleRecipeBookSettings`.
    pub fn set_settings(&mut self, settings: [BookTypeSettings; 4]) {
        self.settings = settings;
        self.revision += 1;
    }
}
