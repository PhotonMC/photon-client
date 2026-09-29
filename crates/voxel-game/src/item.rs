//! Items as the client knows them without the server: the registry and the
//! default components, from the reference's table.

use voxel_data::item::registry::{ITEMS, ItemDefinition, RuleBlocks, Tool};
use voxel_world::ClientBlockStateRegistry;

/// The item with a registry id.
pub fn definition(id: i32) -> Option<&'static ItemDefinition> {
    usize::try_from(id).ok().and_then(|index| ITEMS.get(index))
}

/// The registry id of a named item.
pub fn id_of(name: &str) -> Option<i32> {
    ITEMS
        .iter()
        .position(|item| item.name == name)
        .map(|index| index as i32)
}

/// `HolderSet.contains` for a block state.
fn rule_covers(blocks: &RuleBlocks, registry: &ClientBlockStateRegistry, state: i32) -> bool {
    match blocks {
        RuleBlocks::Tag(tag) => registry.is_state_in_tag(state, tag),
        RuleBlocks::Blocks(names) => voxel_data::block::registry::block_for_state(state as u32)
            .is_some_and(|(block, _)| names.contains(&block.name)),
    }
}

/// `Tool.getMiningSpeed`: the first rule with a speed that covers the block,
/// or the default.
pub fn mining_speed(tool: &Tool, registry: &ClientBlockStateRegistry, state: i32) -> f32 {
    tool.rules
        .iter()
        .find(|rule| rule.speed.is_some() && rule_covers(&rule.blocks, registry, state))
        .and_then(|rule| rule.speed)
        .unwrap_or(tool.default_mining_speed)
}

/// `Tool.isCorrectForDrops`: the first rule that decides it and covers the
/// block, or not.
pub fn is_correct_for_drops(tool: &Tool, registry: &ClientBlockStateRegistry, state: i32) -> bool {
    tool.rules
        .iter()
        .find(|rule| rule.correct_for_drops.is_some() && rule_covers(&rule.blocks, registry, state))
        .and_then(|rule| rule.correct_for_drops)
        .unwrap_or(false)
}
