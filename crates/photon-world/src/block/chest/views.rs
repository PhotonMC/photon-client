use super::*;
use photon_core::ids::{BlockStateId, ItemId};

/// The dyes, in `DyeColor.getId` order.
pub const DYES: [&str; 16] = [
    "white",
    "orange",
    "magenta",
    "light_blue",
    "yellow",
    "lime",
    "pink",
    "gray",
    "light_gray",
    "cyan",
    "purple",
    "blue",
    "brown",
    "green",
    "red",
    "black",
];

/// A table of which blocks are of some classes, by the block's registry id.
pub(super) fn is_of_classes(
    state_id: BlockStateId,
    classes: &[BlockClass],
    table: &'static OnceLock<Vec<bool>>,
) -> bool {
    let blocks = table.get_or_init(|| {
        photon_data::block::registry::BLOCKS
            .iter()
            .map(|block| classes.iter().any(|class| block.class.is(*class)))
            .collect()
    });
    let block = crate::block::state_or_air(state_id).block;
    blocks.get(usize::from(block)).copied().unwrap_or(false)
}

/// Whether a block state is a chest's (the blocks `ChestBlock` and `EnderChestBlock` make).
pub(crate) fn is_chest_state(state_id: BlockStateId) -> bool {
    static BLOCKS: OnceLock<Vec<bool>> = OnceLock::new();
    is_of_classes(
        state_id,
        &[BlockClass::ChestBlock, BlockClass::EnderChestBlock],
        &BLOCKS,
    )
}

/// Whether a block state is a shulker box's.
pub(crate) fn is_shulker_state(state_id: BlockStateId) -> bool {
    static BLOCKS: OnceLock<Vec<bool>> = OnceLock::new();
    is_of_classes(state_id, &[BlockClass::ShulkerBoxBlock], &BLOCKS)
}

/// Whether a block state is a bell's.
pub(crate) fn is_bell_state(state_id: BlockStateId) -> bool {
    static BLOCKS: OnceLock<Vec<bool>> = OnceLock::new();
    is_of_classes(state_id, &[BlockClass::BellBlock], &BLOCKS)
}

/// Whether a block state is an enchanting table's.
pub(crate) fn is_enchanting_table_state(state_id: BlockStateId) -> bool {
    static BLOCKS: OnceLock<Vec<bool>> = OnceLock::new();
    is_of_classes(state_id, &[BlockClass::EnchantingTableBlock], &BLOCKS)
}

/// Whether a block state is a decorated pot's.
pub(crate) fn is_decorated_pot_state(state_id: BlockStateId) -> bool {
    static BLOCKS: OnceLock<Vec<bool>> = OnceLock::new();
    is_of_classes(state_id, &[BlockClass::DecoratedPotBlock], &BLOCKS)
}

/// `PotDecorations.CODEC` of a pot's `sherds`: the items set in its back, left, right and front
/// (a brick is no decoration); none for a tag that has no list of items.
pub(super) fn pot_sherds(tag: &[u8]) -> [Option<String>; 4] {
    use photon_core::nbt::FieldValue;
    let mut input = tag;
    let Ok(Some(fields)) = photon_core::nbt::read_nbt(&mut input) else {
        return Default::default();
    };
    let Some(FieldValue::List(items)) = fields
        .iter()
        .find(|field| field.key == "sherds")
        .map(|field| &field.value)
    else {
        return Default::default();
    };
    let mut sherds: [Option<String>; 4] = Default::default();
    for (slot, item) in sherds.iter_mut().zip(items) {
        if let FieldValue::Str(name) = item
            && name != "minecraft:brick"
        {
            *slot = Some(name.clone());
        }
    }
    sherds
}

/// Whether a block state is a lectern's.
pub(crate) fn is_lectern_state(state_id: BlockStateId) -> bool {
    static BLOCKS: OnceLock<Vec<bool>> = OnceLock::new();
    is_of_classes(state_id, &[BlockClass::LecternBlock], &BLOCKS)
}

/// Whether a block state is a skull's or a head's.
pub(crate) fn is_skull_state(state_id: BlockStateId) -> bool {
    static BLOCKS: OnceLock<Vec<bool>> = OnceLock::new();
    is_of_classes(state_id, &[BlockClass::AbstractSkullBlock], &BLOCKS)
}

/// Whether a block state is a banner's.
pub(crate) fn is_banner_state(state_id: BlockStateId) -> bool {
    static BLOCKS: OnceLock<Vec<bool>> = OnceLock::new();
    is_of_classes(state_id, &[BlockClass::AbstractBannerBlock], &BLOCKS)
}

/// Whether a block state is a sign's or a hanging sign's.
pub(crate) fn is_sign_state(state_id: BlockStateId) -> bool {
    static BLOCKS: OnceLock<Vec<bool>> = OnceLock::new();
    is_of_classes(state_id, &[BlockClass::SignBlock], &BLOCKS)
}

/// Whether a block state is a hanging sign's (on a ceiling or a wall).
pub(super) fn is_hanging_sign_state(state_id: BlockStateId) -> bool {
    static BLOCKS: OnceLock<Vec<bool>> = OnceLock::new();
    is_of_classes(
        state_id,
        &[
            BlockClass::CeilingHangingSignBlock,
            BlockClass::WallHangingSignBlock,
        ],
        &BLOCKS,
    )
}

/// `BannerBlockEntity.loadAdditional`: the layers of a `patterns` list, a layer without a pattern
/// id or a colour left out.
pub(super) fn banner_patterns(tag: &[u8]) -> Vec<(String, String)> {
    use photon_core::nbt::FieldValue;
    let mut input = tag;
    let Ok(Some(fields)) = photon_core::nbt::read_nbt(&mut input) else {
        return Vec::new();
    };
    let Some(FieldValue::List(layers)) = fields
        .iter()
        .find(|field| field.key == "patterns")
        .map(|field| &field.value)
    else {
        return Vec::new();
    };
    let text = |fields: &[photon_core::nbt::Field], key: &str| {
        fields.iter().find_map(|field| match &field.value {
            FieldValue::Str(value) if field.key == key => Some(value.clone()),
            _ => None,
        })
    };
    layers
        .iter()
        .filter_map(|layer| {
            let FieldValue::Compound(layer) = layer else {
                return None;
            };
            Some((text(layer, "pattern")?, text(layer, "color")?))
        })
        .collect()
}

/// Whether a block state has a block entity the client draws.
pub(crate) fn is_drawn_state(state_id: BlockStateId) -> bool {
    is_chest_state(state_id)
        || is_shulker_state(state_id)
        || is_bell_state(state_id)
        || is_skull_state(state_id)
        || is_banner_state(state_id)
        || is_enchanting_table_state(state_id)
        || is_lectern_state(state_id)
        || is_decorated_pot_state(state_id)
        || is_sign_state(state_id)
        || is_other_drawn_state(state_id)
}

/// Whether a block state is a suspicious block's.
pub(crate) fn is_brushable_state(state_id: BlockStateId) -> bool {
    static BLOCKS: OnceLock<Vec<bool>> = OnceLock::new();
    is_of_classes(state_id, &[BlockClass::BrushableBlock], &BLOCKS)
}

/// Whether a block state is one of the block entities that draw what the model renderers do not:
/// the items some hold, the copper golem statues and the end portal surfaces.
fn is_other_drawn_state(state_id: BlockStateId) -> bool {
    is_brushable_state(state_id)
        || is_shelf_state(state_id)
        || is_campfire_state(state_id)
        || is_vault_state(state_id)
        || is_copper_statue_state(state_id)
        || is_portal_state(state_id)
}

/// Whether a block state is an end portal's or an end gateway's.
pub(crate) fn is_portal_state(state_id: BlockStateId) -> bool {
    static BLOCKS: OnceLock<Vec<bool>> = OnceLock::new();
    is_of_classes(
        state_id,
        &[BlockClass::EndPortalBlock, BlockClass::EndGatewayBlock],
        &BLOCKS,
    )
}

/// What an end portal or end gateway block shows (`AbstractEndPortalRenderer`): the faces of its
/// cube that are drawn.
#[derive(Debug, Clone, PartialEq)]
pub struct PortalView {
    /// Whether it is a gateway, which is drawn out to a longer distance and with its whole cube.
    pub gateway: bool,
    /// The faces `shouldRenderFace` lets through.
    pub faces: Vec<Direction>,
}

/// Whether a block state is a copper golem statue's, waxed or weathered.
pub(crate) fn is_copper_statue_state(state_id: BlockStateId) -> bool {
    static BLOCKS: OnceLock<Vec<bool>> = OnceLock::new();
    is_of_classes(
        state_id,
        &[
            BlockClass::CopperGolemStatueBlock,
            BlockClass::WeatheringCopperGolemStatueBlock,
        ],
        &BLOCKS,
    )
}

/// What a copper golem statue is, for drawing it.
#[derive(Debug, Clone, PartialEq)]
pub struct CopperStatueView {
    /// `CopperGolemStatueBlock.FACING`.
    pub facing: Direction,
    /// `CopperGolemStatueBlock.POSE`: `standing`, `sitting`, `running` or `star`.
    pub pose: String,
    /// `getWeatheringState`: 0 for none to 3 for oxidized, waxed or not.
    pub weathering: u8,
}

/// Whether a block state is a vault's.
pub(crate) fn is_vault_state(state_id: BlockStateId) -> bool {
    static BLOCKS: OnceLock<Vec<bool>> = OnceLock::new();
    is_of_classes(state_id, &[BlockClass::VaultBlock], &BLOCKS)
}

/// What a vault shows, for drawing it.
#[derive(Debug, Clone, PartialEq)]
pub struct VaultView {
    /// The item on show and how many (`VaultSharedData.getDisplayItem`).
    pub display_item: (ItemId, i32),
    /// The components its tag holds, as the tag writes them.
    pub display_components: Vec<photon_core::nbt::Field>,
    /// `VaultClientData.previousSpin` and `currentSpin`, in degrees.
    pub spin: (i32, i32),
}

/// Whether a block state is a shelf's.
pub(crate) fn is_shelf_state(state_id: BlockStateId) -> bool {
    static BLOCKS: OnceLock<Vec<bool>> = OnceLock::new();
    is_of_classes(state_id, &[BlockClass::ShelfBlock], &BLOCKS)
}

/// What a shelf holds, for drawing it.
#[derive(Debug, Clone, PartialEq)]
pub struct ShelfView {
    /// `ShelfBlock.FACING`.
    pub facing: Direction,
    /// `ShelfBlockEntity.getAlignItemsToBottom`.
    pub align_to_bottom: bool,
    /// The item in each slot and how many, none for an empty one.
    pub items: [Option<(ItemId, i32)>; 3],
}

/// `ContainerHelper.loadAllItems`: the stacks of a tag's `Items`, each in its `Slot`, in an array
/// of any length; a slot the tag does not name, or names past the end, is left as it is.
fn fill_slots(fields: &[photon_core::nbt::Field], items: &mut [Option<(ItemId, i32)>]) {
    use photon_core::nbt::FieldValue;
    let Some(FieldValue::List(held)) = fields
        .iter()
        .find(|field| field.key == "Items")
        .map(|field| &field.value)
    else {
        return;
    };
    for entry in held {
        let FieldValue::Compound(stack) = entry else {
            continue;
        };
        let slot = stack
            .iter()
            .find(|field| field.key == "Slot")
            .and_then(|field| crate::block::entity::numeric_nbt_value(&field.value))
            .unwrap_or(0);
        if let Some(place) = usize::try_from(slot)
            .ok()
            .and_then(|slot| items.get_mut(slot))
        {
            *place = tag_stack(stack);
        }
    }
}

/// `CampfireBlockEntity.loadAdditional` as the client's update tag has it: what cooks in each of
/// its four slots.
pub(super) fn campfire_tag(tag: &[u8]) -> [Option<(ItemId, i32)>; 4] {
    let mut input = tag;
    let mut items = [None; 4];
    if let Ok(Some(fields)) = photon_core::nbt::read_nbt(&mut input) {
        fill_slots(&fields, &mut items);
    }
    items
}

/// What a campfire holds, for drawing it.
#[derive(Debug, Clone, PartialEq)]
pub struct CampfireView {
    /// `CampfireBlock.FACING`.
    pub facing: Direction,
    /// What cooks in each slot and how many.
    pub items: [Option<(ItemId, i32)>; 4],
}

/// Whether a block state is a campfire's (soul campfires are the same class).
pub(crate) fn is_campfire_state(state_id: BlockStateId) -> bool {
    static BLOCKS: OnceLock<Vec<bool>> = OnceLock::new();
    is_of_classes(state_id, &[BlockClass::CampfireBlock], &BLOCKS)
}

/// `ShelfBlockEntity.loadAdditional` as the client's update tag has it: the items by slot, and
/// whether they stand on the shelf.
pub(super) fn shelf_tag(tag: &[u8]) -> (bool, [Option<(ItemId, i32)>; 3]) {
    let mut input = tag;
    let mut items = [None; 3];
    let Ok(Some(fields)) = photon_core::nbt::read_nbt(&mut input) else {
        return (false, items);
    };
    let align = fields
        .iter()
        .find(|field| field.key == "align_items_to_bottom")
        .and_then(|field| crate::block::entity::numeric_nbt_value(&field.value))
        .is_some_and(|value| value != 0);
    fill_slots(&fields, &mut items);
    (align, items)
}

/// What a suspicious block holds, for drawing it.
#[derive(Debug, Clone, PartialEq)]
pub struct BrushableView {
    /// The face that was brushed (`BrushableBlockEntity.getHitDirection`).
    pub hit_direction: Direction,
    /// `BlockStateProperties.DUSTED`.
    pub dusted: i32,
    /// The item buried in it and how many (`BrushableBlockEntity.getItem`).
    pub item: (ItemId, i32),
}

/// An item stack's id and count in a block entity's tag, which its components are left out of.
pub(crate) fn tag_stack(fields: &[photon_core::nbt::Field]) -> Option<(ItemId, i32)> {
    use photon_core::nbt::FieldValue;
    let FieldValue::Str(name) = &fields.iter().find(|field| field.key == "id")?.value else {
        return None;
    };
    let count = fields
        .iter()
        .find(|field| field.key == "count")
        .and_then(|field| crate::block::entity::numeric_nbt_value(&field.value))
        .unwrap_or(1);
    let index = photon_data::item::items::ITEMS
        .iter()
        .position(|item| item.name == name)?;
    Some((ItemId(index as i32), count)).filter(|(_, count)| *count > 0)
}

/// `BrushableBlockEntity.loadAdditional` as the client's update tag has it: the item and the face
/// brushed, if it holds an item.
pub(super) fn brushable_tag(tag: &[u8]) -> Option<(Direction, (ItemId, i32))> {
    use photon_core::nbt::FieldValue;
    let mut input = tag;
    let fields = photon_core::nbt::read_nbt(&mut input).ok()??;
    let FieldValue::Compound(held) = &fields.iter().find(|field| field.key == "item")?.value else {
        return None;
    };
    let item = tag_stack(held)?;
    let hit = fields.iter().find(|field| field.key == "hit_direction")?;
    let FieldValue::Byte(hit) = hit.value else {
        return None;
    };
    Some((Direction::from_3d_data_value(i32::from(hit)), item))
}

/// The chest blocks of a chunk's sections.
pub(crate) fn chests_of(
    chunk_x: i32,
    chunk_z: i32,
    sections: &crate::chunk::Sections,
) -> std::collections::BTreeSet<(i32, i32, i32)> {
    let mut found = std::collections::BTreeSet::new();
    for (section_y, section) in sections.iter() {
        if section.has_only_air() {
            continue;
        }
        for index in 0..4096 {
            if section.block_states.get(index).is_some_and(is_drawn_state) {
                found.insert((
                    chunk_x * 16 + (index % 16) as i32,
                    section_y * 16 + (index / 256) as i32,
                    chunk_z * 16 + (index / 16 % 16) as i32,
                ));
            }
        }
    }
    found
}

/// What a chest block is, for drawing it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChestView {
    /// The block's name (`minecraft:chest`, `minecraft:ender_chest`, a copper one...).
    pub name: &'static str,
    pub facing: Direction,
    /// `ChestType`: 0 single, 1 left, 2 right.
    pub kind: u8,
    /// The other half of a double chest.
    pub partner: Option<(i32, i32, i32)>,
}

/// What a shulker box block is, for drawing it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShulkerBoxView {
    /// The dye of the box (`red`), none for the undyed one.
    pub color: Option<&'static str>,
    pub facing: Direction,
}

/// What a decorated pot is, for drawing it.
#[derive(Debug, Clone, PartialEq)]
pub struct PotView {
    /// The way the pot faces (`DecoratedPotBlock.HORIZONTAL_FACING`).
    pub facing: Direction,
    /// The items set in its back, left, right and front, none for a side of plain clay.
    pub sherds: [Option<String>; 4],
    /// How far through its wobble it is, a value past 0 to 1 once the wobble is over.
    pub wobble: f32,
}

/// What a banner block is, for drawing it.
#[derive(Debug, Clone, PartialEq)]
pub struct BannerView {
    /// The dye of the banner's cloth (`red`).
    pub color: &'static str,
    /// The wall a wall banner hangs on's facing, none for a standing one.
    pub wall: Option<Direction>,
    /// `BannerBlock.ROTATION` of a standing one.
    pub rotation: i32,
    /// `BannerBlockEntity.getPatterns`: each layer's pattern (a registry id) and dye, bottom first.
    pub patterns: Vec<(String, String)>,
}

/// What a skull or head block is, for drawing it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkullBlockView {
    /// The block's name (`minecraft:zombie_head`, `minecraft:dragon_wall_head`...).
    pub name: &'static str,
    /// The wall a wall skull hangs on's facing, none for a standing one.
    pub wall: Option<Direction>,
    /// `SkullBlock.ROTATION` of a standing one.
    pub rotation: i32,
    /// `SkullBlockEntity.getAnimation`.
    pub animation: f32,
}

/// What a sign block is, for drawing its text.
#[derive(Debug, Clone, PartialEq)]
pub struct SignView<'a> {
    /// The block's name (`minecraft:oak_wall_sign`), which names its wood.
    pub block: &'static str,
    /// A hanging sign's text is drawn by `HangingSignRenderer`, another's by `StandingSignRenderer`.
    pub hanging: bool,
    /// The facing of a sign on a wall, none for one standing or hanging from a ceiling.
    pub wall: Option<Direction>,
    /// The `rotation` of one that is not on a wall.
    pub rotation: i32,
    /// The block entity's tag, which carries the text.
    pub tag: Option<&'a [u8]>,
}
