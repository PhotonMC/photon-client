//! The ids a case carries are the reference's own: wire ids of the version whose Java answers
//! the case. The client's code numbers content by canonical id, so every id that enters from a
//! case goes through here, and every id that leaves for a comparison with the reference's
//! answer goes back.

use std::sync::{Arc, OnceLock};

use photon_core::ids::{BlockStateId, EntityTypeId, ItemId};
use photon_core::version::Version;
use photon_data::version_registries::VersionRegistries;

/// The version the oracle's Java is (`PHOTON_ORACLE_VERSION`, else the newest version's), `tests/behavior/java/v26_3`.
pub(crate) fn oracle_version() -> Version {
    static VERSION: OnceLock<Version> = OnceLock::new();
    *VERSION.get_or_init(|| {
        let version = std::env::var("PHOTON_ORACLE_VERSION")
            .ok()
            .and_then(|name| Version::from_name(&name))
            .unwrap_or(Version::NEWEST);
        Version::set_active(version);
        version
    })
}

/// The numbering of [`oracle_version`].
pub(crate) fn registries() -> Arc<VersionRegistries> {
    static REGISTRIES: OnceLock<Arc<VersionRegistries>> = OnceLock::new();
    REGISTRIES
        .get_or_init(|| Arc::new(VersionRegistries::for_version(oracle_version())))
        .clone()
}

/// The entity type a case names by wire id.
pub(crate) fn entity_type(wire: i32) -> EntityTypeId {
    EntityTypeId(registries().entity_types.canonical_or_unknown(wire))
}

/// The item a case names by wire id.
pub(crate) fn item(wire: i32) -> ItemId {
    ItemId(registries().items.canonical_or_unknown(wire))
}

/// The block state a case names by wire id.
pub(crate) fn block_state(wire: i32) -> BlockStateId {
    BlockStateId(registries().block_states.canonical_or_unknown(wire))
}

/// The blocks of [`oracle_version`] in registry order: the position in this list is the
/// block's place in the reference's registry.
pub(crate) fn blocks() -> impl Iterator<Item = &'static photon_data::block::registry::Block> {
    in_wire_order(&photon_data::block::registry::BLOCKS, &registries().blocks).into_iter()
}

/// The items of [`oracle_version`] in registry order.
pub(crate) fn items() -> impl Iterator<Item = &'static photon_data::item::registry::Item> {
    in_wire_order(&photon_data::item::registry::ITEMS, &registries().items).into_iter()
}

/// The entries of a canonical list that the version has, in the order it numbers them.
fn in_wire_order<T>(
    list: &'static [T],
    map: &photon_data::version_registries::IdMap,
) -> Vec<&'static T> {
    let mut present: Vec<(u32, &'static T)> = list
        .iter()
        .enumerate()
        .filter_map(|(canonical, entry)| Some((map.to_wire(canonical as u32)?, entry)))
        .collect();
    present.sort_by_key(|(wire, _)| *wire);
    present.into_iter().map(|(_, entry)| entry).collect()
}

/// How many block states [`oracle_version`] has.
pub(crate) fn state_count() -> u32 {
    registries().block_states.wire_len() as u32
}

/// The chunk sections a payload holds, read with the numbering of [`oracle_version`].
pub(crate) fn sections(
    bytes: &[u8],
) -> Result<Vec<photon_protocol::play::ChunkSectionObservation>, String> {
    photon_protocol::play::decode_chunk_section_data(bytes, &registries())
}

/// A world that speaks the oracle's version, so the wire ids a case names reach it as they would
/// from a server of that version.
pub(crate) fn world(dimension: photon_world::DimensionBounds) -> photon_world::ClientWorld {
    let mut world = photon_world::ClientWorld::new(dimension);
    world.set_versions(registries());
    world
}

/// The wire id of an item, as the oracle's Java numbers it.
pub(crate) fn wire_item(item: ItemId) -> i32 {
    registries().items.wire_or_unknown(item.0)
}

/// The wire id of a block state, as the oracle's Java numbers it.
pub(crate) fn wire_state(state: BlockStateId) -> i32 {
    registries().block_states.wire_or_unknown(state.0)
}

/// The wire id of a sound event, as the oracle's Java numbers it.
pub(crate) fn wire_sound(sound: i32) -> i32 {
    registries().sound_events.wire_or_unknown(sound)
}

/// The class [`oracle_version`]'s Java gives an item. The newest version moved what the axe,
/// hoe, shovel, sign and bed classes did into item components, so its items are plain ones;
/// the client keeps the classes as the kinds of behaviour they name.
pub(crate) fn reference_class(
    class: photon_data::item::classes::ItemClass,
) -> photon_data::item::classes::ItemClass {
    use photon_data::item::classes::ItemClass as C;
    if Version::active_before_26_3() {
        return class;
    }
    match class {
        C::AxeItem | C::HoeItem | C::ShovelItem => C::Item,
        C::SignItem => C::StandingAndWallBlockItem,
        C::BedItem => C::BlockItem,
        other => other,
    }
}

/// Whether [`oracle_version`]'s Java has the class.
pub(crate) fn reference_has_class(class: photon_data::item::classes::ItemClass) -> bool {
    if Version::active_before_26_3() {
        return !matches!(class, photon_data::item::classes::ItemClass::CushionItem);
    }
    reference_class(class) == class
}

/// Checks that the entity a case describes (a JSON object with a `type_id`) is of the named type.
pub(crate) fn require_type(
    entity: &serde_json::Value,
    name: &str,
    message: &str,
) -> Result<(), String> {
    let wire = entity["type_id"]
        .as_i64()
        .ok_or("entity type_id must be a number")?;
    if entity_type(wire as i32) == photon_data::entity_type::named(name) {
        Ok(())
    } else {
        Err(message.into())
    }
}

/// `require_type` for an entity held as a JSON map.
pub(crate) fn require_type_of(
    entity: &serde_json::Map<String, serde_json::Value>,
    name: &str,
) -> Result<(), String> {
    let message = format!("the fixture requires {name}");
    require_type(&serde_json::Value::Object(entity.clone()), name, &message)
}

/// The class [`oracle_version`]'s Java gives a block: before 26.3 the spruce let tinted leaves
/// fall, and the client keeps the plain leaves class for it.
pub(crate) fn reference_block_class(
    block: &photon_data::block::registry::Block,
) -> photon_data::block::classes::BlockClass {
    use photon_data::block::classes::BlockClass as C;
    if block.name.ends_with(":spruce_leaves") && Version::active_before_26_3() {
        return C::TintedParticleLeavesBlock;
    }
    block.class
}

/// The name [`oracle_version`]'s Java gives a block class: 26.3 renamed two and split the beds'
/// shared parent off.
pub(crate) fn reference_class_name(class: photon_data::block::classes::BlockClass) -> String {
    let name = format!("{class:?}");
    if !Version::active_before_26_3() {
        return name;
    }
    match name.as_str() {
        "AbstractBedBlock" => "BedBlock".to_owned(),
        "PathBlock" => "DirtPathBlock".to_owned(),
        "RedstoneWireBlock" => "RedStoneWireBlock".to_owned(),
        _ => name,
    }
}

/// The name [`oracle_version`]'s Java gives a block's class.
pub(crate) fn block_class_name(block: &photon_data::block::registry::Block) -> String {
    reference_class_name(reference_block_class(block))
}

/// Whether [`oracle_version`]'s Java has the block class: 26.3 added the straw bed and its
/// parent, the falling-particles leaves and the shelf mushroom.
pub(crate) fn reference_has_block_class(class: photon_data::block::classes::BlockClass) -> bool {
    const NEW_IN_26_3: [&str; 4] = [
        "AbstractBedBlock",
        "StrawBedBlock",
        "FallingParticlesLeavesBlock",
        "ShelfMushroomBlock",
    ];
    !Version::active_before_26_3() || !NEW_IN_26_3.contains(&format!("{class:?}").as_str())
}

/// The asset index file of [`oracle_version`]'s game inside an assets folder: 26.3 moved to
/// index 34.
pub(crate) fn asset_index(assets: &std::path::Path) -> std::path::PathBuf {
    let id = if Version::active_before_26_3() {
        "32"
    } else {
        "34"
    };
    assets.join("indexes").join(format!("{id}.json"))
}
