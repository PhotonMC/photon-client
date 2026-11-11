//! `CushionItem.useOn` as the client predicts it: a cushion goes on a top face only, where the
//! level lets it stand (`Cushion.canBePlacedAt`). The click on a cauldron, hopper or composter
//! is first moved to where the collision shape is met, which may be another face. The stack
//! shrinks and the cushion appears only on the server, so a placeable spot is a swing and
//! nothing more.

use photon_core::aabb::Aabb;
use photon_core::binary::BlockPos;
use photon_core::direction::Direction;
use photon_core::math::Vec3;

use super::context::Context;
use super::{Aim, UseResult, placing};
use photon_world::block::rules::{self, Rule};

/// The tag of the blocks whose collision shape the click is measured against; the prefix is
/// joined here so it is not mistaken for a block name.
const USES_COLLISION_SHAPE: &str = concat!("minecraft:", "cushion_uses_collision_shape");

/// How far past the click the recalculating ray reaches.
const RAY_EXTENSION: f64 = 0.001;

/// The cushion's width and height (`EntityTypes.CUSHION`), at spawn scale one.
const WIDTH: f32 = 1.0;
const HEIGHT: f32 = 0.25;

/// Where a click lands: the face it enters and where.
struct Click {
    face: Direction,
    location: Vec3,
}

/// `CushionItem.recalculateContextForSpecialCollisionShapes`: for a block of the tag, the click
/// is the first hit of the ray from the eye through the click on the block's collision shape.
fn recalculated(ctx: &Context<'_>, pos: (i32, i32, i32), click: Click) -> Click {
    let state = ctx.state_at(pos);
    if !ctx
        .world
        .block_state_registry()
        .is_state_in_tag(state, USES_COLLISION_SHAPE)
    {
        return click;
    }
    let (eye, _) = placing::look_ray(ctx);
    let ray = Vec3::new(
        click.location.x - eye.x,
        click.location.y - eye.y,
        click.location.z - eye.z,
    )
    .normalized();
    let to = Vec3::new(
        click.location.x + ray.x * RAY_EXTENSION,
        click.location.y + ray.y * RAY_EXTENSION,
        click.location.z + ray.z * RAY_EXTENSION,
    );
    let shape = ctx
        .world
        .block_state_registry()
        .collision_shape_at(state, [pos.0, pos.1, pos.2]);
    match shape.clip(eye, to, BlockPos::new(pos.0, pos.1, pos.2)) {
        Some(hit) => Click {
            face: hit.direction,
            location: hit.location,
        },
        None => click,
    }
}

/// `CushionItem.useOn` for the stack in `hand` used on the block `aim` hits.
pub(super) fn use_on(ctx: &mut Context<'_>, hand: u8, aim: Aim) -> UseResult {
    let Aim::Block {
        x,
        y,
        z,
        face,
        location,
        inside,
    } = aim
    else {
        return UseResult::Pass;
    };
    let Some((item, _)) = ctx.held(hand) else {
        return UseResult::Pass;
    };
    let click = Click {
        face: Direction::from_3d_data_value(i32::from(face)),
        location: Vec3::new(location[0], location[1], location[2]),
    };
    let click = recalculated(ctx, (x, y, z), click);
    if click.face != Direction::Up {
        return UseResult::Fail;
    }
    // `BlockPlaceContext.getClickedPos`: the clicked block when it gives way, else the one above.
    let Some(pos) = clicked_pos(ctx, item, ((x, y, z), &click, inside)) else {
        return UseResult::Success { swing: true };
    };
    let centre = Vec3::new(
        f64::from(pos.0) + 0.5,
        click.location.y,
        f64::from(pos.2) + 0.5,
    );
    let half = f64::from(WIDTH / 2.0);
    let area = Aabb::new(
        centre.x - half,
        centre.y,
        centre.z - half,
        centre.x + half,
        centre.y + f64::from(HEIGHT),
        centre.z + half,
    );
    if ctx.world.cushion_can_be_placed_at(area) {
        UseResult::Success { swing: true }
    } else {
        UseResult::Fail
    }
}

/// The block position the cushion is placed in, `None` where the rule of the clicked block is
/// not ported.
fn clicked_pos(
    ctx: &Context<'_>,
    item: photon_core::ids::ItemId,
    (hit, click, inside): ((i32, i32, i32), &Click, bool),
) -> Option<(i32, i32, i32)> {
    let world: &photon_world::ClientWorld = ctx.world;
    let state_at = |x: i32, y: i32, z: i32| world.block_state_or_air(x, y, z);
    let aimed = (
        hit,
        click.face,
        (click.location.x, click.location.y, click.location.z),
    );
    let tags = world.block_state_registry().tags();
    let mut place_ctx =
        rules::PlaceContext::new(&state_at, aimed, Some(item), ctx.player.rotation(), tags);
    place_ctx.secondary_use = ctx.player.is_shift_key_down();
    place_ctx.inside = inside;
    match rules::can_be_replaced(state_at(hit.0, hit.1, hit.2), &place_ctx) {
        Rule::Known(replaced) => place_ctx.replace_clicked = replaced,
        Rule::Unported => return None,
    }
    Some(place_ctx.clicked_pos())
}
