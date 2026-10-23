//! The `use` of items the server mostly answers: what the client itself returns.

use super::*;

/// The `use` of an item class whose client part is written here.
pub(super) fn use_of(
    ctx: &mut Context<'_>,
    hand: u8,
    class: photon_data::item::classes::ItemClass,
) -> UseResult {
    use photon_data::item::classes::ItemClass as C;
    match class {
        C::EnderEyeItem => ender_eye_use(ctx, hand),
        C::BoatItem => boat_use(ctx),
        C::SpawnEggItem => spawn_egg_use(ctx),
        C::TridentItem => trident_use(ctx, hand),
        C::InstrumentItem => instrument_use(ctx, hand),
        C::BottleItem => super::bottles::bottle_use(ctx, hand),
        // The cast, the reel and the new map are the server's; the client only swings (a rod's
        // sounds have no excluded player, so none is heard here).
        _ => UseResult::Success { swing: true },
    }
}

/// `EnderEyeItem.use`: a look at an end portal frame leaves the click to `useOn`; otherwise the use
/// starts (the eye itself is the server's) and the swing is the server's too.
pub(super) fn ender_eye_use(ctx: &mut Context<'_>, hand: u8) -> UseResult {
    let (eye, to) = super::placing::look_ray(ctx);
    if let Some(hit) = ctx.world.clip_outlines(eye, to) {
        let state = ctx.state_at((hit.block_pos.x, hit.block_pos.y, hit.block_pos.z));
        if photon_world::block::state_or_air(state).block().name == "minecraft:end_portal_frame" {
            return UseResult::Pass;
        }
    }
    ctx.player.start_using_item(hand != super::MAIN_HAND);
    UseResult::Success { swing: false }
}

/// `BoatItem.use`: a miss of the look (fluids included) passes; a block gets a boat. Not
/// evaluated: a pickable entity around the eye (pass) and a boat that would not fit (fail).
pub(super) fn boat_use(ctx: &mut Context<'_>) -> UseResult {
    let (eye, to) = super::placing::look_ray(ctx);
    match ctx.world.clip_outlines_and_fluids(eye, to) {
        Some(_) => UseResult::Success { swing: true },
        None => UseResult::Pass,
    }
}

/// `SpawnEggItem.use` on the client: a miss of the look (sources only) passes, a block succeeds
/// (the spawn is the server's).
pub(super) fn spawn_egg_use(ctx: &mut Context<'_>) -> UseResult {
    let (eye, to) = super::placing::look_ray(ctx);
    match ctx.world.clip_outlines_and_sources(eye, to) {
        Some(_) => UseResult::Success { swing: true },
        None => UseResult::Pass,
    }
}

/// `EnderEyeItem.useOn`: only an end portal frame without an eye takes it (the client succeeds,
/// the fill is the server's).
pub(super) fn ender_eye_use_on(ctx: &Context<'_>, aim: Aim) -> UseResult {
    let Aim::Block { x, y, z, .. } = aim else {
        return UseResult::Pass;
    };
    let state = ctx.state_at((x, y, z));
    let frame =
        photon_world::block::state_or_air(state).block().name == "minecraft:end_portal_frame";
    if frame && rules::property(state, "eye") == Some("false") {
        UseResult::Success { swing: true }
    } else {
        UseResult::Pass
    }
}

/// `TridentItem.use` for a trident without riptide and with durability left: the use starts and
/// nothing swings. A stack about to break fails, and one with riptide fails out of the rain;
/// neither is read from the stack's components.
fn trident_use(ctx: &mut Context<'_>, hand: u8) -> UseResult {
    ctx.player.start_using_item(hand != super::MAIN_HAND);
    UseResult::Success { swing: false }
}

/// `InstrumentItem.use`: a stack with an instrument starts its use, plays the call as the player's
/// own record-volume sound and waits out the cooldown of the call's length; one without fails.
fn instrument_use(ctx: &mut Context<'_>, hand: u8) -> UseResult {
    let Some((item, _)) = ctx.held(hand) else {
        return UseResult::Pass;
    };
    let name = crate::item::definition(item).map_or("", |row| row.name);
    let slot = ctx.hand_slot(hand);
    let Some(stack) = ctx
        .world
        .local_player()
        .and_then(|state| state.inventory_slot(slot))
        .cloned()
    else {
        return UseResult::Pass;
    };
    let Some(instrument) = crate::item::instrument::of_stack(&stack, name, ctx.instruments) else {
        return UseResult::Fail;
    };
    let (sound, ticks, volume) = (instrument.sound, instrument.use_ticks, instrument.volume);
    ctx.player.start_using_item(hand != super::MAIN_HAND);
    // `level.playSound(player, player, ..)`: bound to the player, who is the excluded one.
    let at = ctx.player.position();
    ctx.world
        .push_sound(photon_world::local_player::LocalSound::at(
            sound,
            photon_world::local_player::LocalSoundSource::Records,
            (volume, 1.0),
            (at.x, at.y, at.z),
        ));
    ctx.cooldowns
        .add(crate::item::cooldowns::group(&stack), ticks);
    UseResult::Success { swing: false }
}
