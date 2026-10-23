//! What a block declares, the way `Blocks` builds it: a name, a class, the
//! state properties and a `BlockBehaviour.Properties` chain. Only what a
//! block sets differently from `Properties.of()` is written in
//! [`super::blocks`]; everything a state works out from these and from its
//! shapes is computed, not stored.

use super::classes::{BlockClass, BlockInterface, Method};

/// What kind of values a property takes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PropertyKind {
    Boolean,
    Integer,
    Enum,
}

/// A state property (`BlockStateProperties`), with its values in the order
/// the reference lists them.
#[derive(Debug, PartialEq, Eq)]
pub struct Property {
    pub name: &'static str,
    pub kind: PropertyKind,
    pub values: &'static [&'static str],
}

/// `PushReaction`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PushReaction {
    Normal,
    Destroy,
    Block,
    Ignore,
    PushOnly,
}

/// A fluid type (`Fluids`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fluid {
    Empty,
    Water,
    FlowingWater,
    Lava,
    FlowingLava,
}

/// `BlockBehaviour.OffsetType`: how far off the grid a block is nudged.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OffsetType {
    None,
    Xz,
    Xyz,
}

/// A `StatePredicate` a block may replace: the one `Properties.of()` starts
/// with, or the block's own, as its value in each state.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Rule {
    Default,
    Never,
    Always,
    /// One value per state, in state order.
    States(&'static [bool]),
}

impl Rule {
    /// The block's own value for the state at `offset`, or `None` where the
    /// default predicate decides.
    pub fn value(self, offset: u32) -> Option<bool> {
        match self {
            Self::Default => None,
            Self::Never => Some(false),
            Self::Always => Some(true),
            Self::States(values) => values.get(offset as usize).copied(),
        }
    }
}

/// `Properties.lightLevel`: one level for every state, or one per state.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Emission {
    Constant(u8),
    States(&'static [u8]),
}

impl Emission {
    pub fn value(self, offset: u32) -> u8 {
        match self {
            Self::Constant(level) => level,
            Self::States(levels) => levels.get(offset as usize).copied().unwrap_or(0),
        }
    }
}

/// One block of the registry.
#[derive(Debug)]
pub struct Block {
    pub name: &'static str,
    pub class: BlockClass,
    /// The state properties, in the block's own order.
    pub properties: &'static [&'static Property],
    /// The default state's value of each property.
    pub default: &'static [&'static str],
    pub destroy_time: f32,
    pub explosion_resistance: f32,
    pub friction: f32,
    pub speed_factor: f32,
    pub jump_factor: f32,
    pub bounce_restitution: f32,
    pub has_collision: bool,
    pub requires_correct_tool_for_drops: bool,
    pub randomly_ticking: bool,
    pub can_occlude: bool,
    pub air: bool,
    pub ignited_by_lava: bool,
    pub liquid: bool,
    pub force_solid_off: bool,
    pub force_solid_on: bool,
    pub spawn_terrain_particles: bool,
    pub replaceable: bool,
    pub dynamic_shape: bool,
    pub push_reaction: PushReaction,
    /// `Properties.instrument`: the note block instrument the block plays under a note block.
    pub instrument: &'static str,
    pub offset_type: OffsetType,
    pub light_emission: Emission,
    pub emissive_rendering: Rule,
    pub suffocating: Rule,
    pub view_blocking: Rule,
    pub redstone_conductor: Rule,
    /// The fluid a `LiquidBlock` is made of.
    pub fluid: Fluid,
    /// `asItem`, when it is not the item named like the block.
    pub item: Option<&'static str>,
}

impl Block {
    /// `Properties.of()` for a block of `class`.
    pub const fn of(name: &'static str, class: BlockClass) -> Self {
        Self {
            name,
            class,
            properties: &[],
            default: &[],
            destroy_time: 0.0,
            explosion_resistance: 0.0,
            friction: 0.6,
            speed_factor: 1.0,
            jump_factor: 1.0,
            bounce_restitution: 0.0,
            has_collision: true,
            requires_correct_tool_for_drops: false,
            randomly_ticking: false,
            can_occlude: true,
            air: false,
            ignited_by_lava: false,
            liquid: false,
            force_solid_off: false,
            force_solid_on: false,
            spawn_terrain_particles: true,
            replaceable: false,
            dynamic_shape: false,
            push_reaction: PushReaction::Normal,
            instrument: "harp",
            offset_type: OffsetType::None,
            light_emission: Emission::Constant(0),
            emissive_rendering: Rule::Default,
            suffocating: Rule::Default,
            view_blocking: Rule::Default,
            redstone_conductor: Rule::Default,
            fluid: Fluid::Empty,
            item: None,
        }
    }

    /// The state definition: the properties and the default state's values.
    pub const fn properties(
        mut self,
        properties: &'static [&'static Property],
        default: &'static [&'static str],
    ) -> Self {
        self.properties = properties;
        self.default = default;
        self
    }

    pub const fn strength(mut self, destroy_time: f32, explosion_resistance: f32) -> Self {
        self.destroy_time = destroy_time;
        self.explosion_resistance = explosion_resistance;
        self
    }

    pub const fn friction(mut self, friction: f32) -> Self {
        self.friction = friction;
        self
    }

    pub const fn speed_factor(mut self, factor: f32) -> Self {
        self.speed_factor = factor;
        self
    }

    pub const fn jump_factor(mut self, factor: f32) -> Self {
        self.jump_factor = factor;
        self
    }

    pub const fn bounce_restitution(mut self, restitution: f32) -> Self {
        self.bounce_restitution = restitution;
        self
    }

    /// `noCollision`, which also stops the block occluding.
    pub const fn no_collision(mut self) -> Self {
        self.has_collision = false;
        self.can_occlude = false;
        self
    }

    pub const fn no_occlusion(mut self) -> Self {
        self.can_occlude = false;
        self
    }

    pub const fn requires_correct_tool_for_drops(mut self) -> Self {
        self.requires_correct_tool_for_drops = true;
        self
    }

    pub const fn random_ticks(mut self) -> Self {
        self.randomly_ticking = true;
        self
    }

    pub const fn air(mut self) -> Self {
        self.air = true;
        self
    }

    pub const fn ignited_by_lava(mut self) -> Self {
        self.ignited_by_lava = true;
        self
    }

    pub const fn liquid(mut self) -> Self {
        self.liquid = true;
        self
    }

    pub const fn force_solid_off(mut self) -> Self {
        self.force_solid_off = true;
        self
    }

    pub const fn force_solid_on(mut self) -> Self {
        self.force_solid_on = true;
        self
    }

    pub const fn no_terrain_particles(mut self) -> Self {
        self.spawn_terrain_particles = false;
        self
    }

    pub const fn replaceable(mut self) -> Self {
        self.replaceable = true;
        self
    }

    pub const fn dynamic_shape(mut self) -> Self {
        self.dynamic_shape = true;
        self
    }

    pub const fn push_reaction(mut self, reaction: PushReaction) -> Self {
        self.push_reaction = reaction;
        self
    }

    pub const fn offset_type(mut self, offset: OffsetType) -> Self {
        self.offset_type = offset;
        self
    }

    pub const fn instrument(mut self, instrument: &'static str) -> Self {
        self.instrument = instrument;
        self
    }

    pub const fn light_level(mut self, emission: Emission) -> Self {
        self.light_emission = emission;
        self
    }

    pub const fn emissive_rendering(mut self, rule: Rule) -> Self {
        self.emissive_rendering = rule;
        self
    }

    pub const fn is_suffocating(mut self, rule: Rule) -> Self {
        self.suffocating = rule;
        self
    }

    pub const fn is_view_blocking(mut self, rule: Rule) -> Self {
        self.view_blocking = rule;
        self
    }

    pub const fn is_redstone_conductor(mut self, rule: Rule) -> Self {
        self.redstone_conductor = rule;
        self
    }

    /// The fluid a `LiquidBlock` is made with.
    pub const fn fluid(mut self, fluid: Fluid) -> Self {
        self.fluid = fluid;
        self
    }

    /// The item the block stands for (`Item.BY_BLOCK`), when it is not the
    /// one named like the block: air for a block with no item.
    pub const fn item(mut self, item: &'static str) -> Self {
        self.item = Some(item);
        self
    }

    /// `asItem`, as an item identifier.
    pub fn item_name(&self) -> &'static str {
        self.item.unwrap_or(self.name)
    }
}

impl BlockClass {
    /// The class and every class it extends, nearest first: the order the
    /// reference looks a method up in.
    pub fn lineage(self) -> impl Iterator<Item = Self> {
        std::iter::successors(Some(self), |class| class.parent())
    }

    /// `instanceof` a class.
    pub fn is(self, other: Self) -> bool {
        self.lineage().any(|class| class == other)
    }

    /// `BonemealableBlock.getType()` is `NEIGHBOR_SPREADER`, not `GROWER`: the four classes that
    /// override it all answer so.
    pub fn bone_meal_spreads(self) -> bool {
        self.lineage()
            .any(|class| class.declares().contains(&Method::GetType))
    }

    /// `BonemealableBlock.getParticlePos` is the block below: the two classes that override it
    /// answer so.
    pub fn particles_below(self) -> bool {
        self.lineage()
            .any(|class| class.declares().contains(&Method::GetParticlePos))
    }

    /// `instanceof` an interface: one the class or a class it extends
    /// implements, or one those extend.
    pub fn implements(self, interface: BlockInterface) -> bool {
        fn reaches(from: BlockInterface, interface: BlockInterface) -> bool {
            from == interface
                || from
                    .parents()
                    .iter()
                    .any(|parent| reaches(*parent, interface))
        }
        self.lineage().any(|class| {
            class
                .interfaces()
                .iter()
                .any(|own| reaches(*own, interface))
        })
    }
}
