//! What an item declares, the way `Items` builds it: a name, a class and the
//! `Item.Properties` the client reads. Only what an item sets differently
//! from `Item.Properties` defaults is written in [`super::items`].

pub use super::classes::{ItemClass, ItemMethod};
use super::component::Component;

/// The blocks a tool rule covers.
#[derive(Debug)]
pub enum RuleBlocks {
    /// A block tag, without the leading `#`.
    Tag(&'static str),
    Blocks(&'static [&'static str]),
}

#[derive(Debug)]
pub struct ToolRule {
    pub blocks: RuleBlocks,
    pub speed: Option<f32>,
    pub correct_for_drops: Option<bool>,
}

/// The `minecraft:tool` component.
#[derive(Debug)]
pub struct Tool {
    pub rules: &'static [ToolRule],
    pub default_mining_speed: f32,
    pub damage_per_block: i32,
    pub can_destroy_blocks_in_creative: bool,
}

/// An `equippable` component.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Equip {
    /// The equipment slot's name (`head`, `chest`...).
    pub slot: &'static str,
    /// A player may wear it.
    pub player: bool,
    /// Using it swaps it with what is worn.
    pub swappable: bool,
    /// The equipment asset it is worn as (`Equippable.assetId`): which
    /// layers and textures draw it on a wearer.
    pub asset: Option<&'static str>,
}

/// The item crafting leaves behind, and how many.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Remainder {
    pub item: &'static str,
    pub count: i32,
}

/// One item of the registry.
#[derive(Debug)]
pub struct Item {
    pub name: &'static str,
    pub class: ItemClass,
    pub max_stack_size: i32,
    /// Zero when the item has no durability.
    pub max_damage: i32,
    pub equippable: Option<Equip>,
    pub crafting_remainder: Option<Remainder>,
    pub tool: Option<&'static Tool>,
    pub consumable: bool,
    pub blocks_attacks: bool,
    pub kinetic_weapon: bool,
    /// Ticks a furnace burns it for; zero when it is no fuel.
    pub burn_time: i32,
    /// The block a block item places.
    pub block: Option<&'static str>,
    /// The other default components the client reads.
    pub components: &'static [Component],
}

impl Item {
    /// `new Item.Properties()` for an item of `class`: stacks to 64.
    pub const fn of(name: &'static str, class: ItemClass) -> Self {
        Self {
            name,
            class,
            max_stack_size: 64,
            max_damage: 0,
            equippable: None,
            crafting_remainder: None,
            tool: None,
            consumable: false,
            blocks_attacks: false,
            kinetic_weapon: false,
            burn_time: 0,
            block: None,
            components: &[],
        }
    }

    /// What a furnace burns the item for, in ticks (`FuelValues`).
    pub const fn burns(mut self, ticks: i32) -> Self {
        self.burn_time = ticks;
        self
    }

    pub const fn stacks_to(mut self, size: i32) -> Self {
        self.max_stack_size = size;
        self
    }

    pub const fn durability(mut self, damage: i32) -> Self {
        self.max_damage = damage;
        self
    }

    pub const fn equippable(mut self, equip: Equip) -> Self {
        self.equippable = Some(equip);
        self
    }

    pub const fn crafting_remainder(mut self, item: &'static str, count: i32) -> Self {
        self.crafting_remainder = Some(Remainder { item, count });
        self
    }

    pub const fn tool(mut self, tool: &'static Tool) -> Self {
        self.tool = Some(tool);
        self
    }

    pub const fn consumable(mut self) -> Self {
        self.consumable = true;
        self
    }

    pub const fn blocks_attacks(mut self) -> Self {
        self.blocks_attacks = true;
        self
    }

    pub const fn kinetic_weapon(mut self) -> Self {
        self.kinetic_weapon = true;
        self
    }

    /// The default components the client reads beyond the above.
    pub const fn components(mut self, components: &'static [Component]) -> Self {
        self.components = components;
        self
    }

    /// The block a `BlockItem` places.
    pub const fn places(mut self, block: &'static str) -> Self {
        self.block = Some(block);
        self
    }

    /// Whether the default `Item.use` does nothing for the default stack:
    /// it has no consumable, swappable equippable, blocking or kinetic
    /// component to act on.
    pub fn use_does_nothing(&self) -> bool {
        !self.consumable
            && !self.equippable.is_some_and(|equip| equip.swappable)
            && !self.blocks_attacks
            && !self.kinetic_weapon
    }
}

impl ItemClass {
    /// The class and every class it extends, nearest first: the order the
    /// reference looks a method up in.
    pub fn lineage(self) -> impl Iterator<Item = Self> {
        std::iter::successors(Some(self), |class| class.parent())
    }

    /// `instanceof`.
    pub fn is(self, other: Self) -> bool {
        self.lineage().any(|class| class == other)
    }

    /// The nearest class in the lineage that declares `method` itself.
    pub fn owner_of(self, method: ItemMethod) -> Self {
        self.lineage()
            .find(|class| class.declares().contains(&method))
            .unwrap_or(Self::Item)
    }
}
