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
    put_fluid(ctx, (item, content), pos, (state, container))
}

/// The end of `BucketItem.emptyContents` once the fluid fits: it boils away where water
/// evaporates, goes into a container, or replaces the block.
fn put_fluid(
    ctx: &mut Context<'_>,
    (item, content): (ItemId, Content),
    pos: Pos,
    (state, container): (BlockStateId, Option<bool>),
) -> bool {
    if content == Content::Water && ctx.world.dimension_environment().water_evaporates() {
        ctx.world.evaporate_water(pos);
        return true;
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
        add_or_drop(ctx, fresh);
    }
    result
}

/// `Inventory.add` or, when nothing has room, `Player.drop`: a client spawns no item entity for the
/// drop, it only swings the main hand (and tells the server so).
pub(super) fn add_or_drop(ctx: &mut Context<'_>, fresh: ClientItemStackState) {
    if !add_to_inventory(ctx, fresh) {
        ctx.player.swing_for_drop();
    }
}

/// `Inventory.add` of one item: onto a stack with room (the selected slot, the off hand, then the
/// hotbar and the rest in order), else into the first free slot.
pub(super) fn add_to_inventory(ctx: &mut Context<'_>, fresh: ClientItemStackState) -> bool {
    let menu_slot = |index: i16| if index < 9 { 36 + index } else { index };
    let order: Vec<i16> = [menu_slot(ctx.selected_slot as i16), 45]
        .into_iter()
        .chain((0..36).map(menu_slot))
        .collect();
    let Some(state) = ctx.world.local_player() else {
        return false;
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
    } else {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_mode::{Context, Miner};
    use photon_world::{
        ClientPlayerGameMode, ClientWorld, DimensionBounds, local_player::LocalPlayer,
    };

    /// A survival player whose slots 9 to 45 hold full stacks of cobblestone, but for `free`.
    fn add_water_bucket(free: Option<i16>) -> (bool, bool) {
        let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
        let mut player = LocalPlayer::new(
            photon_core::ids::EntityId(1),
            photon_core::math::Vec3::new(0.5, 1.0, 0.5),
            0.0,
            0.0,
            ClientPlayerGameMode::Survival,
            true,
        );
        world.set_local_player_id(player.entity_id());
        let stone = crate::item::id_of("minecraft:cobblestone").expect("an item");
        for slot in (9..=45).filter(|slot| Some(*slot) != free) {
            let stack = ClientItemStackState::new(64, stone);
            world.apply_local_player_container_slot(0, 0, slot, Some(&stack));
        }
        let mut cooldowns = crate::item::cooldowns::ItemCooldowns::default();
        let mut ctx = Context {
            miner: &Miner::of(&world, 0),
            world: &mut world,
            player: &mut player,
            mode: ClientPlayerGameMode::Survival,
            selected_slot: 0,
            cooldowns: &mut cooldowns,
            instruments: &[],
            block_transformers: &[],
            spin_attack: &[],
            prevents_armor_change: &[],
            difficulty: 2,
            game_master: false,
            profile_id: None,
            creaking_active: false,
        };
        let bucket = crate::item::id_of("minecraft:water_bucket").expect("an item");
        let fits = add_to_inventory(&mut ctx, ClientItemStackState::new(1, bucket));
        let mut again = ctx;
        add_or_drop(&mut again, ClientItemStackState::new(1, bucket));
        (
            fits,
            again.player.is_swinging() && again.player.take_drop_swings() == 1,
        )
    }

    /// A full inventory drops the bucket, which a client shows as the main hand's swing only.
    #[test]
    fn a_bucket_that_fits_nowhere_swings_the_main_hand() {
        let (fits, swinging) = add_water_bucket(None);
        assert!(!fits && swinging);
        let (fits, _) = add_water_bucket(Some(20));
        assert!(fits);
    }
}
