use super::*;

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
    state_id: i32,
    classes: &[BlockClass],
    table: &'static OnceLock<Vec<bool>>,
) -> bool {
    let blocks = table.get_or_init(|| {
        photon_data::block::registry::BLOCKS
            .iter()
            .map(|block| classes.iter().any(|class| block.class.is(*class)))
            .collect()
    });
    let block = crate::block::state_or_air(i64::from(state_id)).block;
    blocks.get(usize::from(block)).copied().unwrap_or(false)
}

/// Whether a block state is a chest's (the blocks `ChestBlock` and `EnderChestBlock` make).
pub(crate) fn is_chest_state(state_id: i32) -> bool {
    static BLOCKS: OnceLock<Vec<bool>> = OnceLock::new();
    is_of_classes(
        state_id,
        &[BlockClass::ChestBlock, BlockClass::EnderChestBlock],
        &BLOCKS,
    )
}

/// Whether a block state is a shulker box's.
pub(crate) fn is_shulker_state(state_id: i32) -> bool {
    static BLOCKS: OnceLock<Vec<bool>> = OnceLock::new();
    is_of_classes(state_id, &[BlockClass::ShulkerBoxBlock], &BLOCKS)
}

/// Whether a block state is a bell's.
pub(crate) fn is_bell_state(state_id: i32) -> bool {
    static BLOCKS: OnceLock<Vec<bool>> = OnceLock::new();
    is_of_classes(state_id, &[BlockClass::BellBlock], &BLOCKS)
}

/// Whether a block state is a skull's or a head's.
pub(crate) fn is_skull_state(state_id: i32) -> bool {
    static BLOCKS: OnceLock<Vec<bool>> = OnceLock::new();
    is_of_classes(state_id, &[BlockClass::AbstractSkullBlock], &BLOCKS)
}

/// Whether a block state is a banner's.
pub(crate) fn is_banner_state(state_id: i32) -> bool {
    static BLOCKS: OnceLock<Vec<bool>> = OnceLock::new();
    is_of_classes(state_id, &[BlockClass::AbstractBannerBlock], &BLOCKS)
}

/// Whether a block state is a sign's or a hanging sign's.
pub(crate) fn is_sign_state(state_id: i32) -> bool {
    static BLOCKS: OnceLock<Vec<bool>> = OnceLock::new();
    is_of_classes(state_id, &[BlockClass::SignBlock], &BLOCKS)
}

/// Whether a block state is a hanging sign's (on a ceiling or a wall).
pub(super) fn is_hanging_sign_state(state_id: i32) -> bool {
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
pub(crate) fn is_drawn_state(state_id: i32) -> bool {
    is_chest_state(state_id)
        || is_shulker_state(state_id)
        || is_bell_state(state_id)
        || is_skull_state(state_id)
        || is_banner_state(state_id)
        || is_sign_state(state_id)
}

/// The chest blocks of a chunk's sections.
pub(crate) fn chests_of(
    chunk_x: i32,
    chunk_z: i32,
    sections: &std::collections::BTreeMap<i32, crate::chunk::ClientSection>,
) -> std::collections::BTreeSet<(i32, i32, i32)> {
    let mut found = std::collections::BTreeSet::new();
    for (&section_y, section) in sections {
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
    /// The block's name (`minecraft:chest`, `minecraft:ender_chest`, a copper one…).
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
    /// The block's name (`minecraft:zombie_head`, `minecraft:dragon_wall_head`…).
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
