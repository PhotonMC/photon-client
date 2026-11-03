//! `HangingEntityItem.useOn` as the client predicts it: the place beside the clicked face must be
//! inside the build height, a painting needs a side face, and an item frame is hung when it
//! survives there (the stack shrinks), or the use is consumed without a swing when it does not.
//! A painting is hung when its smallest variant fits; which variant is the server's.

use photon_core::direction::Direction;
use photon_core::ids::ItemId;
use photon_data::item::classes::ItemClass;
use photon_data::item::component_types::{ENTITY_DATA, PAINTING_VARIANT};
use photon_data::item::ids;

use super::context::Context;
use super::{Aim, UseResult};
use crate::item::values::Values;

/// `Item.useOn` for the items that put an entity in the world: an eye of ender looks at a
/// frame, an egg spawns (the server's doing), and hanging entities are hung.
pub(super) fn entity_use_on(
    ctx: &mut Context<'_>,
    (hand, class): (u8, ItemClass),
    aim: Aim,
) -> UseResult {
    match (class, ctx.held(hand)) {
        (ItemClass::EnderEyeItem, _) => super::utility::ender_eye_use_on(ctx, aim),
        (ItemClass::SpawnEggItem, _) => super::spawn_egg::use_on(ctx, hand),
        (_, Some((item, _))) => use_on(ctx, hand, item, aim),
        _ => UseResult::Pass,
    }
}

/// What is hung: an item frame (either kind) or a painting.
enum Hung {
    Frame,
    Painting,
}

fn hung(item: ItemId) -> Option<Hung> {
    match item {
        ids::ITEM_FRAME | ids::GLOW_ITEM_FRAME => Some(Hung::Frame),
        ids::PAINTING => Some(Hung::Painting),
        _ => None,
    }
}

/// Whether the stack in `hand` brings its own `entity_data` or painting variant, which change
/// what is hung (the variant sets the painting's size).
fn configured(ctx: &Context<'_>, hand: u8, item: ItemId) -> bool {
    ctx.world
        .local_player()
        .and_then(|state| state.inventory_slot(ctx.hand_slot(hand)))
        .is_some_and(|stack| {
            let values = Values::new(item, stack.patch_bytes());
            values.raw(ENTITY_DATA) != Values::new(item, &[0, 0]).raw(ENTITY_DATA)
                || values.has(PAINTING_VARIANT)
        })
}

/// `HangingEntityItem.useOn` for the stack in `hand` used on the block `aim` hits.
fn use_on(ctx: &mut Context<'_>, hand: u8, item: ItemId, aim: Aim) -> UseResult {
    let Aim::Block { x, y, z, face, .. } = aim else {
        return UseResult::Pass;
    };
    let direction = Direction::from_3d_data_value(i32::from(face));
    let (dx, dy, dz) = direction.step();
    let pos = (x + dx, y + dy, z + dz);
    match hangs(ctx, item, (pos, direction), configured(ctx, hand, item)) {
        None => UseResult::Fail,
        Some(None) => UseResult::Success { swing: true },
        Some(Some(false)) => UseResult::Success { swing: false },
        Some(Some(true)) => {
            if !ctx.instabuild() {
                let slot = ctx.hand_slot(hand);
                ctx.world.shrink_local_player_slot(slot, 1);
            }
            UseResult::Success { swing: true }
        }
    }
}

/// What the level says of hanging `item` at `pos` facing `direction`: `None` when the place may
/// not be used, `Some(None)` when the server decides (a configured stack or another item), and
/// whether the entity survives there otherwise.
fn hangs(
    ctx: &Context<'_>,
    item: ItemId,
    (pos, direction): ((i32, i32, i32), Direction),
    configured: bool,
) -> Option<Option<bool>> {
    let dimension = ctx.world.dimension();
    let inside = pos.1 >= dimension.min_y && pos.1 < dimension.min_y + dimension.height;
    let kind = hung(item);
    let vertical_painting = matches!(kind, Some(Hung::Painting)) && direction.axis().is_vertical();
    if !inside || vertical_painting {
        return None;
    }
    Some(match kind {
        Some(Hung::Painting) if !configured => {
            Some(ctx.world.small_painting_survives(pos, direction))
        }
        Some(Hung::Frame) if !configured => Some(ctx.world.item_frame_survives(pos, direction)),
        _ => None,
    })
}
