//! `BucketItem.use` as the client predicts it: the fluid put into the world or picked up from it,
//! and the stack the bucket turns into.

use super::*;
use crate::menus::container::max_stack_size;
use photon_core::aabb::BlockHitResult;
use photon_core::ids::{BlockStateId, ItemId};
use photon_data::item::ids;
use photon_world::local_player::{LocalSound, LocalSoundSource};

/// `BucketItem.content`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Content {
    Empty,
    /// A mob bucket of `Fluids.EMPTY`: it puts the mob out and nothing else.
    Cube,
    Water,
    Lava,
}

fn content_of(item: ItemId) -> Content {
    match item {
        ids::BUCKET => Content::Empty,
        ids::LAVA_BUCKET => Content::Lava,
        ids::SULFUR_CUBE_BUCKET => Content::Cube,
        _ => Content::Water,
    }
}

type Pos = (i32, i32, i32);

/// The mob buckets that make the fish sound.
const FISH: [&str; 4] = [
    "minecraft:cod_bucket",
    "minecraft:salmon_bucket",
    "minecraft:pufferfish_bucket",
    "minecraft:tropical_fish_bucket",
];

fn pos_of(hit: &BlockHitResult) -> Pos {
    (hit.block_pos.x, hit.block_pos.y, hit.block_pos.z)
}

fn relative(pos: Pos, direction: photon_core::direction::Direction) -> Pos {
    let (dx, dy, dz) = direction.step();
    (pos.0 + dx, pos.1 + dy, pos.2 + dz)
}

/// `Item.getPlayerPOVHitResult`: the block the look reaches within the interaction range, the
/// fluid sources of an empty bucket's look included.
pub(super) fn pov_hit(ctx: &Context<'_>, sources: bool) -> Option<BlockHitResult> {
    let (eye, to) = super::placing::look_ray(ctx);
    if sources {
        ctx.world.clip_outlines_and_sources(eye, to)
    } else {
        ctx.world.clip_outlines(eye, to)
    }
}

/// `BucketItem.use`.
pub(super) fn bucket_use(ctx: &mut Context<'_>, hand: u8, item: ItemId) -> UseResult {
    let content = content_of(item);
    let Some(hit) = pov_hit(ctx, content == Content::Empty) else {
        return UseResult::Pass;
    };
    let (pos, direction) = (pos_of(&hit), hit.direction);
    // `Player.mayUseItemAt` of the block beside the face: a player who may not build needs the
    // stack's `can_place_on` to name the block the face is on.
    if !ctx.player.abilities().may_build()
        && !ctx.held_adventure_matches(
            hand,
            photon_world::item_components::ids::CAN_PLACE_ON,
            ctx.state_at(pos),
        )
    {
        return UseResult::Fail;
    }
    let clicked = ctx.state_at(pos);
    let container = water_container(ctx, content, clicked).is_some();
    let place_at = if container {
        pos
    } else {
        relative(pos, direction)
    };
    if empty_contents(ctx, (item, content), place_at, Some(&hit)) {
        let emptied = if ctx.instabuild() { item } else { ids::BUCKET };
        return filled_result(ctx, hand, emptied);
    }
    if content != Content::Empty {
        return UseResult::Fail;
    }
    let Some(pickup) = rules::pickup(clicked) else {
        return UseResult::Fail;
    };
    let position = ctx.player.position();
    ctx.world
        .set_block_with_updates(pos, pickup.after, pickup.flags, 512, position);
    ctx.player.play_own_sound(pickup.sound, (1.0, 1.0));
    match crate::item::id_of(pickup.bucket) {
        Some(taken) => filled_result(ctx, hand, taken),
        None => UseResult::Success { swing: true },
    }
}

/// `LiquidBlockContainer.canPlaceLiquid` of the block for the content; `None` for no container.
fn water_container(ctx: &Context<'_>, content: Content, state: BlockStateId) -> Option<bool> {
    if content == Content::Water {
        rules::water_container(state, ctx.mode.is_creative())
    } else {
        None
    }
}

/// `canPlaceFluidInsideBlock`: air, or a block the fluid replaces or a container takes it, unless
/// a sneaking player clicked it.
fn fits_inside(ctx: &Context<'_>, state: BlockStateId, container: bool, no_hit: bool) -> bool {
    let place_liquid = rules::replaceable_by_fluid(state) || container;
    rules::state_is_air(state) || place_liquid && (!ctx.player.is_shift_key_down() || no_hit)
}

/// `BucketItem.emptyContents`: whether the fluid went in at `pos`; a click on the side of a block
/// that cannot take it tries the block beside.
fn empty_contents(
    ctx: &mut Context<'_>,
    (item, content): (ItemId, Content),
    pos: Pos,
    hit: Option<&BlockHitResult>,
) -> bool {
    match content {
        Content::Empty => return false,
        Content::Cube => {
            play_empty_sound(ctx, (item, content), pos);
            return true;
        }
        _ => {}
    }
    let state = ctx.state_at(pos);
    let container = water_container(ctx, content, state);
    let inside = fits_inside(ctx, state, container == Some(true), hit.is_none());
    if !inside {
        return hit.is_some_and(|hit| {
            let beside = relative(pos_of(hit), hit.direction);
            empty_contents(ctx, (item, content), beside, None)
        });
    }
    // The client sets nothing in most containers; the server's waterlogging arrives.
    if container.is_some() {
        if let Some((wet, flags)) = rules::client_waterlogging(state) {
            super::tools::dowse_lit_campfire(ctx, pos, state);
            let position = ctx.player.position();
            ctx.world
                .set_block_with_updates(pos, wet, flags, 512, position);
        }
        play_empty_sound(ctx, (item, content), pos);
        return true;
    }
    let name = if content == Content::Lava {
        "minecraft:lava"
    } else {
        "minecraft:water"
    };
    let fluid = photon_data::block::registry::default_state_named(name);
    let position = ctx.player.position();
    let set = ctx
        .world
        .set_block_with_updates(pos, fluid, 11, 512, position);
    let source = photon_world::block::state_or_air(state).fluid.is_source();
    if !set && !source {
        return false;
    }
    play_empty_sound(ctx, (item, content), pos);
    true
}

/// `BucketItem.playEmptySound`.
fn play_empty_sound(ctx: &mut Context<'_>, (item, content): (ItemId, Content), pos: Pos) {
    // A mob bucket plays its mob's sound, as neutral.
    let mob = match crate::item::definition(item).map(|row| row.name) {
        Some("minecraft:axolotl_bucket") => Some("minecraft:item.bucket.empty_axolotl"),
        Some("minecraft:tadpole_bucket") => Some("minecraft:item.bucket.empty_tadpole"),
        Some("minecraft:sulfur_cube_bucket") => Some("minecraft:item.bucket.empty_sulfur_cube"),
        Some(name) if FISH.contains(&name) => Some("minecraft:item.bucket.empty_fish"),
        _ => None,
    };
    let source = if mob.is_some() {
        LocalSoundSource::Neutral
    } else {
        LocalSoundSource::Blocks
    };
    let event = mob.unwrap_or(if content == Content::Lava {
        "minecraft:item.bucket.empty_lava"
    } else {
        "minecraft:item.bucket.empty"
    });
    let at = (
        f64::from(pos.0) + 0.5,
        f64::from(pos.1) + 0.5,
        f64::from(pos.2) + 0.5,
    );
    ctx.world
        .push_sound(LocalSound::at(event, source, (1.0, 1.0), at));
}

/// `ItemUtils.createFilledResult`: with infinite materials the held stack stays and the new item
/// joins the inventory unless one is there; otherwise one is used up and the new item takes its
/// place or joins the inventory.
fn filled_result(ctx: &mut Context<'_>, hand: u8, new: ItemId) -> UseResult {
    filled_with(ctx, hand, ClientItemStackState::new(1, new))
}

/// A bucket milked from a cow or goat: `createFilledResult` with a milk bucket.
pub(super) fn milk_result(ctx: &mut Context<'_>, hand: u8) {
    filled_result(ctx, hand, ids::MILK_BUCKET);
}

/// `createFilledResult` for a stack that carries components.
pub(super) fn filled_with(
    ctx: &mut Context<'_>,
    hand: u8,
    fresh: ClientItemStackState,
) -> UseResult {
    filled_result_of(ctx, hand, fresh, true)
}

/// A bowl filled with stew (`name`) by a mooshroom: `createFilledResult` that does not limit the
/// creative stack size, so a creative player always gets the stew added.
pub(super) fn stew_result(ctx: &mut Context<'_>, hand: u8, name: &str) {
    if let Some(stew) = crate::item::id_of(name) {
        filled_result_of(ctx, hand, ClientItemStackState::new(1, stew), false);
    }
}

/// `createFilledResult` with a plain item that never limits the creative stack size, as a mob
/// pickup and an axolotl's emptied bucket do.
pub(super) fn plain_result(ctx: &mut Context<'_>, hand: u8, new: ItemId) {
    filled_result_of(ctx, hand, ClientItemStackState::new(1, new), false);
}

/// `ItemUtils.createFilledResult(held, player, fresh, limitCreativeStackSize)`.
fn filled_result_of(
    ctx: &mut Context<'_>,
    hand: u8,
    fresh: ClientItemStackState,
    limit_creative: bool,
) -> UseResult {
    let result = UseResult::Success { swing: true };
    if ctx.instabuild() {
        let contained = ctx.world.local_player().is_some_and(|state| {
            (5..=45).any(|slot| {
                state.inventory_slot(slot).is_some_and(|stack| {
                    stack.item_count() > 0 && stack.is_same_item_same_components(&fresh)
                })
            })
        });
        if !(limit_creative && contained) {
            add_to_inventory(ctx, fresh);
        }
        return result;
    }
    let slot = ctx.hand_slot(hand);
    ctx.world.shrink_local_player_slot(slot, 1);
    let emptied = ctx
        .world
        .local_player()
        .and_then(|state| state.inventory_slot(slot))
        .is_none_or(|stack| stack.item_count() <= 0);
    if emptied {
        ctx.world.put_local_player_slot(slot, fresh);
    } else {
        // What does not fit is dropped, which the client does not predict.
        add_to_inventory(ctx, fresh);
    }
    result
}

/// `Inventory.add` of one item: onto a stack with room (the selected slot, the off hand, then the
/// hotbar and the rest in order), else into the first free slot.
pub(super) fn add_to_inventory(ctx: &mut Context<'_>, fresh: ClientItemStackState) {
    let menu_slot = |index: i16| if index < 9 { 36 + index } else { index };
    let order: Vec<i16> = [menu_slot(ctx.selected_slot as i16), 45]
        .into_iter()
        .chain((0..36).map(menu_slot))
        .collect();
    let Some(state) = ctx.world.local_player() else {
        return;
    };
    let has_room = |slot: i16| {
        state.inventory_slot(slot).is_some_and(|stack| {
            stack.item_count() > 0
                && stack.is_same_item_same_components(&fresh)
                && max_stack_size(stack) > 1
                && stack.item_count() < max_stack_size(stack)
        })
    };
    let free = |slot: i16| {
        state
            .inventory_slot(slot)
            .is_none_or(|stack| stack.item_count() <= 0)
    };
    if let Some(slot) = order.iter().copied().find(|slot| has_room(*slot)) {
        ctx.world.grow_local_player_slot(slot, 1);
    } else if let Some(slot) = (0..36).map(menu_slot).find(|slot| free(*slot)) {
        ctx.world.put_local_player_slot(slot, fresh);
    }
}
