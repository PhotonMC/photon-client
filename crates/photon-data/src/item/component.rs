//! The default components an item carries that the client reads, typed the
//! way the reference's component values are: an item declares only those
//! that differ from what its absence would mean, and the rest (`item_name`
//! from its name, an empty `lore`) follow from the item.

/// `Rarity`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
}

/// `AttributeModifier.Operation`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Operation {
    AddValue,
    AddMultipliedBase,
    AddMultipliedTotal,
}

/// `EquipmentSlotGroup`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SlotGroup {
    Any,
    MainHand,
    OffHand,
    Hand,
    Feet,
    Legs,
    Chest,
    Head,
    Armor,
    Body,
    Saddle,
}

/// `ItemAttributeModifiers.Display`, for the two that need no text.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AttributeDisplay {
    Default,
    Hidden,
}

/// One entry of `attribute_modifiers`.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct AttributeModifier {
    pub attribute: &'static str,
    /// The modifier's id, `minecraft:armor.helmet` and the like.
    pub id: &'static str,
    pub amount: f64,
    pub operation: Operation,
    pub slot: SlotGroup,
    pub display: AttributeDisplay,
}

/// A default component value the client reads.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Component {
    /// `item_name`, when it is not the translation of the item's own name.
    ItemName(&'static str),
    Rarity(Rarity),
    AttributeModifiers(&'static [AttributeModifier]),
    EnchantmentGlintOverride(bool),
    MapColor(i32),
    /// `potion_contents` with a potion (by registry name) or none, and no
    /// colour, effects or name.
    PotionContents(Option<&'static str>),
    PotionDurationScale(f32),
    /// `use_cooldown` of some seconds, in the item's own group.
    UseCooldown(f32),
    /// `use_remainder`: one of this item (by registry name) with no components.
    UseRemainder(&'static str),
    Fireworks {
        flight_duration: i32,
    },
    /// An instrument, by registry name.
    Instrument(&'static str),
    /// A jukebox song, by registry name.
    JukeboxPlayable(&'static str),
    BlockState(&'static [(&'static str, &'static str)]),
    OminousBottleAmplifier(i32),
    /// Enchantments by registry name and level.
    StoredEnchantments(&'static [(&'static str, i32)]),
    /// A painting variant, by registry name.
    PaintingVariant(&'static str),
    /// Mob effects by registry name, each with its duration in ticks.
    SuspiciousStewEffects(&'static [(&'static str, i32)]),
    /// Banner layers: a pattern by registry name and a dye colour by name.
    BannerPatterns(&'static [(&'static str, &'static str)]),
    /// A tooltip that hides none but the named components.
    TooltipDisplay(&'static [&'static str]),
}
