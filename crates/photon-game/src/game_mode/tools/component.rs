//! `Item.useOn` of a stack that carries the `block_transformer` component (26.3): the axe, the
//! hoe and the shovel, and any stack a patch gives the component, work through the registry
//! entry the component names. The client picks the first transform whose face is allowed and
//! whose provider offers a state, and does what the reference does on a client: it uses up a
//! stackable stack, sets the block (flags 11), plays the sound and shows the particle. The loot,
//! the item's damage and the advancement are the server's.

use super::super::context::Context;
use super::super::{Aim, UseResult};
use super::{blocking_intent, play_at_block};
use crate::block_transformer::{Transform, Transformer, Unported};
use crate::item::values::Values;
use crate::item_components::Cursor;
use photon_core::ids::ItemId;
use photon_data::item::component::Component;
use photon_data::item::component_types::BLOCK_TRANSFORMER;

type Pos = (i32, i32, i32);

/// The registry entry the stack in `hand` carries: the patch's, or the item's own default by its
/// name. `None` without the component, or with an id the registry lacks.
fn carried<'a>(ctx: &Context<'a>, hand: u8, item: ItemId) -> Option<&'a Transformer> {
    let list: &'a [Transformer] = ctx.block_transformers;
    let patch = ctx
        .world
        .local_player()?
        .inventory_slot(ctx.hand_slot(hand))?
        .patch_bytes()
        .to_vec();
    let values = Values::new(item, &patch);
    if !values.has(BLOCK_TRANSFORMER) {
        return None;
    }
    if let Some(raw) = values.raw(BLOCK_TRANSFORMER) {
        let id = Cursor::new(raw).varint().ok()?;
        return list.get(usize::try_from(id).ok()?);
    }
    let own = photon_world::block::rules::item(item)?
        .components
        .iter()
        .find_map(|component| match component {
            Component::BlockTransformer(name) => Some(*name),
            _ => None,
        })?;
    list.iter().find(|entry| entry.name == own)
}

/// What the client does once a transform has found its state: the stack is used up if it
/// stacks (an item that is damaged instead is the server's), the block is set, and the sound and
/// the particle show.
fn apply(
    ctx: &mut Context<'_>,
    hand: u8,
    pos: Pos,
    (transform, state): (&Transform, photon_core::ids::BlockStateId),
) {
    let old = ctx.state_at(pos);
    let shaped = if transform.update_from_neighbors {
        ctx.world.update_from_neighbour_shapes(state, pos)
    } else {
        state
    };
    let slot = ctx.hand_slot(hand);
    let stackable = ctx
        .world
        .local_player()
        .and_then(|local| local.inventory_slot(slot))
        .is_some_and(crate::menus::container::is_stackable);
    // `ItemStack.consume`: not for a player with infinite materials.
    if stackable && transform.consume_on_use && !ctx.instabuild() {
        ctx.world.shrink_local_player_slot(slot, 1);
    }
    let position = ctx.player.position();
    ctx.world
        .set_block_with_updates(pos, shaped, 11, 512, position);
    if let Some(event) = transform.sound {
        play_at_block(ctx, event, pos);
    }
    let particle = transform.particle.level_event();
    if let Some(event) = particle {
        ctx.world.apply_level_event(event, pos, 0);
    }
    // A double copper chest tells its other half too.
    if transform.copper_chest
        && let Some(other) = photon_world::block::rules::other_chest_half(old, pos)
        && let Some(event) = particle
    {
        ctx.world.apply_level_event(event, other, 0);
    }
}

/// `Item.useOn` for a stack in `hand` whose item is `item`: nothing without the component or
/// for a main hand that blocks use with a shield in the other; a transform that reaches a
/// provider the client does not decide counts as used, with no prediction.
pub(in crate::game_mode) fn use_on(
    ctx: &mut Context<'_>,
    hand: u8,
    item: ItemId,
    aim: Aim,
) -> UseResult {
    let Aim::Block { x, y, z, face, .. } = aim else {
        return UseResult::Pass;
    };
    let Some(transformer) = carried(ctx, hand, item) else {
        return UseResult::Pass;
    };
    if blocking_intent(ctx, hand) {
        return UseResult::Pass;
    }
    let Some(transforms) = &transformer.transforms else {
        return UseResult::Success { swing: true };
    };
    let pos = (x, y, z);
    for transform in transforms {
        if transform.disallows(face) {
            continue;
        }
        match transform.provider.optional_state(ctx.world, pos) {
            Ok(Some(state)) => {
                apply(ctx, hand, pos, (transform, state));
                return UseResult::Success { swing: true };
            }
            Ok(None) => {}
            Err(Unported) => return UseResult::Success { swing: true },
        }
    }
    UseResult::Pass
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_transformer::from_json;
    use crate::game_mode::{Aim, Context, MAIN_HAND, Miner};
    use photon_core::ids::BlockStateId;
    use photon_world::item_components::Patch;
    use photon_world::{
        ClientItemStackState, ClientPlayerGameMode, ClientWorld, DimensionBounds, SectionInput,
        local_player::LocalPlayer,
    };

    fn named(name: &str) -> BlockStateId {
        let index = photon_data::block::registry::block_index(name).expect(name);
        photon_data::block::registry::BLOCKS[index].default_state_id()
    }

    fn entry(provider: &str, extra: &str) -> Transformer {
        let text = format!(r#"[{{"block_state_provider":{provider}{extra}}}]"#);
        let value = photon_core::json::parse(&text).expect("json");
        Transformer {
            name: "minecraft:axe".into(),
            transforms: from_json(&value),
        }
    }

    /// A stack of two sticks, which no class turns blocks with, whose patch gives the component
    /// the registry entry `entry` (or no component at all).
    fn sticks(entry: Option<i32>) -> ClientItemStackState {
        let item = crate::item::id_of("minecraft:stick").expect("an item");
        let mut stack = ClientItemStackState::new(2, item);
        if let Some(entry) = entry {
            stack.set_patch(&Patch {
                added: vec![(BLOCK_TRANSFORMER, vec![entry as u8])],
                removed: Vec::new(),
            });
        }
        stack
    }

    /// What using the stack on the top of an oak log at (1, 1, 1) did: the result, the block
    /// that is there afterwards and the count that is left in the hand.
    fn click(
        stack: ClientItemStackState,
        transformers: &[Transformer],
    ) -> (UseResult, BlockStateId, i32) {
        let mut blocks = vec![0; 4_096];
        blocks[256 + 16 + 1] = named("minecraft:oak_log").0;
        let biomes = vec![0; 64];
        let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
        let section = SectionInput {
            non_empty_block_count: 1,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        };
        assert!(world.replace_chunk(0, 0, [section]));
        let mut player = LocalPlayer::new(
            photon_core::ids::EntityId(1),
            photon_core::math::Vec3::new(1.5, 1.0, 3.5),
            180.0,
            35.0,
            ClientPlayerGameMode::Survival,
            true,
        );
        world.set_local_player_id(player.entity_id());
        let item = stack.item_registry_id();
        world.apply_local_player_container_slot(0, 0, 36, Some(&stack));
        player.set_main_hand_item(item);
        let mut cooldowns = crate::item::cooldowns::ItemCooldowns::default();
        let mut ctx = Context {
            miner: &Miner::of(&world, 0),
            world: &mut world,
            player: &mut player,
            mode: ClientPlayerGameMode::Survival,
            selected_slot: 0,
            cooldowns: &mut cooldowns,
            instruments: &[],
            block_transformers: transformers,
            spin_attack: &[],
            prevents_armor_change: &[],
            difficulty: 2,
            game_master: false,
            profile_id: None,
            creaking_active: false,
        };
        let aim = Aim::Block {
            x: 1,
            y: 1,
            z: 1,
            face: 1,
            location: [1.5, 1.5, 1.5],
            inside: false,
        };
        let result = use_on(&mut ctx, MAIN_HAND, item, aim);
        let block = world.block_state_or_air(1, 1, 1);
        let count = world
            .local_player()
            .and_then(|local| local.inventory_slot(36))
            .map_or(0, |stack| stack.item_count());
        (result, block, count)
    }

    const STRIPPED: &str = r#"{"id":"minecraft:stripped_oak_log"}"#;

    /// Any stack that a patch gives the component works as a tool, and a stack that stacks is
    /// used up by it.
    #[test]
    fn a_patched_stack_turns_the_block_and_is_used_up() {
        let list = [entry(STRIPPED, "")];
        let (result, block, count) = click(sticks(Some(0)), &list);
        assert_eq!(result, UseResult::Success { swing: true });
        assert_eq!(block, named("minecraft:stripped_oak_log"));
        assert_eq!(count, 1);
    }

    /// `consume_on_use: false` keeps the stack whole.
    #[test]
    fn a_transform_that_does_not_consume_keeps_the_count() {
        let list = [entry(STRIPPED, r#","consume_on_use":false"#)];
        let (result, block, count) = click(sticks(Some(0)), &list);
        assert_eq!(result, UseResult::Success { swing: true });
        assert_eq!(block, named("minecraft:stripped_oak_log"));
        assert_eq!(count, 2);
    }

    /// The component names an entry by its place in the registry the server sent.
    #[test]
    fn the_patch_picks_the_entry_by_its_place() {
        let list = [entry(r#"{"id":"minecraft:dirt"}"#, ""), entry(STRIPPED, "")];
        let (_, block, _) = click(sticks(Some(1)), &list);
        assert_eq!(block, named("minecraft:stripped_oak_log"));
        let (_, block, _) = click(sticks(Some(0)), &list);
        assert_eq!(block, named("minecraft:dirt"));
    }

    #[test]
    fn a_stack_without_the_component_or_with_an_unknown_entry_passes() {
        let list = [entry(STRIPPED, "")];
        for stack in [sticks(None), sticks(Some(5))] {
            let (result, block, count) = click(stack, &list);
            assert_eq!(result, UseResult::Pass);
            assert_eq!(block, named("minecraft:oak_log"));
            assert_eq!(count, 2);
        }
    }

    /// A transform whose provider offers nothing here leaves the block and the stack alone.
    #[test]
    fn a_provider_with_nothing_to_offer_passes() {
        let rules = r#"{"type":"minecraft:rule_based","rules":[{"if_true":
            {"type":"minecraft:matching_blocks","blocks":"minecraft:stone"},
            "then":{"id":"minecraft:dirt"}}]}"#;
        let (result, block, count) = click(sticks(Some(0)), &[entry(rules, "")]);
        assert_eq!(result, UseResult::Pass);
        assert_eq!(block, named("minecraft:oak_log"));
        assert_eq!(count, 2);
    }

    /// An entry the client cannot read, or a provider of a kind it does not decide, counts as
    /// used with no prediction: the server answers.
    #[test]
    fn an_entry_the_client_cannot_decide_is_used_without_prediction() {
        let unread = Transformer {
            name: "minecraft:axe".into(),
            transforms: None,
        };
        let weighted = r#"{"type":"minecraft:weighted_state_provider","entries":[]}"#;
        for list in [[unread], [entry(weighted, "")]] {
            let (result, block, count) = click(sticks(Some(0)), &list);
            assert_eq!(result, UseResult::Success { swing: true });
            assert_eq!(block, named("minecraft:oak_log"));
            assert_eq!(count, 2);
        }
    }
}
