//! `BlockItem.place` as the client predicts it: the state, the blocks it sets and the sound.

use super::*;
use photon_core::ids::ItemId;

/// `BlockItem.place` for the block item classes whose rules are written.
pub(super) fn place(ctx: &mut Context<'_>, hand: u8, item: ItemId, aim: Aim) -> UseResult {
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
    let (Some(kind), Some(block)) = (
        rules::item_placement(item),
        rules::item(item)
            .and_then(|row| row.block)
            .and_then(photon_data::block::registry::block_index),
    ) else {
        return UseResult::Success { swing: true };
    };
    let world: &ClientWorld = ctx.world;
    let state_at = |x: i32, y: i32, z: i32| world.block_state_or_air(x, y, z);
    let face = photon_core::direction::Direction::from_3d_data_value(i32::from(face));
    let aimed = ((x, y, z), face, (location[0], location[1], location[2]));
    let tags = world.block_state_registry().tags();
    // `getRawBrightness(pos, 0)`: the brighter of the block light and the sky light.
    let brightness = |x: i32, y: i32, z: i32| {
        i32::from(
            world
                .block_brightness(x, y, z)
                .max(world.sky_brightness(x, y, z)),
        )
    };
    let mut place_ctx =
        rules::PlaceContext::new(&state_at, aimed, Some(item), ctx.player.rotation(), tags);
    place_ctx.brightness = Some(&brightness);
    place_ctx.secondary_use = ctx.player.is_shift_key_down();
    place_ctx.inside = inside;
    place_ctx.game_master = ctx.can_use_game_master_blocks();
    place_ctx.limits = super::place_prediction::limits_of(world);
    let (pos, state) = match super::place_prediction::predicted_state(kind, block, &mut place_ctx) {
        Ok(predicted) => predicted,
        Err(result) => return result,
    };
    if is_obstructed(ctx, pos, state) {
        return UseResult::Fail;
    }
    let writes = rules::place_writes(kind, state, pos, &place_ctx);
    if !set_placed_blocks(ctx, &writes) {
        return UseResult::Fail;
    }
    ctx.world.play_place_sound(pos, state);
    // `ItemStack.consume`: used up unless materials are infinite.
    if !ctx.instabuild() {
        let slot = ctx.hand_slot(hand);
        ctx.world.shrink_local_player_slot(slot, 1);
    }
    UseResult::Success { swing: true }
}

/// The segment `Item.getPlayerPOVHitResult` clips: from the eye along the look, as far as the
/// block interaction range.
pub(super) fn look_ray(ctx: &Context<'_>) -> (photon_core::math::Vec3, photon_core::math::Vec3) {
    let position = ctx.player.position();
    let eye = photon_core::math::Vec3::new(
        position.x,
        position.y + f64::from(ctx.player.eye_height()),
        position.z,
    );
    let (y_rot, x_rot) = ctx.player.rotation();
    let look = photon_core::math::Vec3::direction_from_rotation_components(x_rot, y_rot);
    let range = ctx.miner.block_interaction_range;
    let to = photon_core::math::Vec3::new(
        eye.x + look.x * range,
        eye.y + look.y * range,
        eye.z + look.z * range,
    );
    (eye, to)
}

/// `PlaceOnWaterBlockItem.use`: the block goes on top of the water source (or block) the
/// look reaches within the interaction range, as a use on that block's top face. A look
/// that reaches nothing puts the block in the air, where it cannot stand.
pub(super) fn place_on_water(ctx: &mut Context<'_>, hand: u8, item: ItemId) -> UseResult {
    let (eye, to) = look_ray(ctx);
    let Some(hit) = ctx.world.clip_outlines_and_sources(eye, to) else {
        return UseResult::Fail;
    };
    let above = Aim::Block {
        x: hit.block_pos.x,
        y: hit.block_pos.y + 1,
        z: hit.block_pos.z,
        face: hit.direction.data3d() as u8,
        location: [hit.location.x, hit.location.y, hit.location.z],
        inside: hit.inside,
    };
    place(ctx, hand, item, above)
}

/// `BlockItem.placeBlock` and `setPlacedBy`: what the item clears first (not checked), the
/// block itself, which must be set, and the other part of a door, a tall plant or a bed.
fn set_placed_blocks(ctx: &mut Context<'_>, writes: &rules::PlaceWrites) -> bool {
    let position = ctx.player.position();
    let mut set = |write: rules::Write| {
        ctx.world
            .set_block_with_updates(write.pos, write.state, write.flags, 512, position)
    };
    if let Some(before) = writes.before {
        set(before);
    }
    if !set(writes.block) {
        return false;
    }
    if let Some(after) = writes.after {
        set(after);
    }
    true
}

/// `Level.isUnobstructed` for a state put at `pos`: a state with a collision shape may not go
/// where an entity that blocks building stands. The local player (unless spectating) counts, as
/// do the other entities the client has.
fn is_obstructed(
    ctx: &Context<'_>,
    pos: (i32, i32, i32),
    state: photon_core::ids::BlockStateId,
) -> bool {
    let block_pos = photon_core::binary::BlockPos::new(pos.0, pos.1, pos.2);
    let registry = ctx.world.block_state_registry();
    if registry.collision_shape(state).is_empty() {
        return false;
    }
    if !ctx.player.spectating() && ctx.player.is_colliding(ctx.world, block_pos, state) {
        return true;
    }
    let shape = registry
        .collision_shape_at(state, [pos.0, pos.1, pos.2])
        .move_by_block_pos(block_pos);
    !ctx.world.is_unobstructed(&shape)
}
