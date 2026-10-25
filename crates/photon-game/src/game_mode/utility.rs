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
        C::FoodOnAStickItem | C::FireworkRocketItem | C::KnowledgeBookItem | C::BundleItem => {
            carried_use(ctx, hand, class)
        }
        // The cast, the reel and the new map are the server's; the client only swings (a rod's
        // sounds have no excluded player, so none is heard here).
        _ => UseResult::Success { swing: true },
    }
}

/// The `use` of the food on a stick, the firework rocket, the knowledge book and the bundle.
fn carried_use(
    ctx: &mut Context<'_>,
    hand: u8,
    class: photon_data::item::classes::ItemClass,
) -> UseResult {
    use photon_data::item::classes::ItemClass as C;
    match class {
        // `isClientSide` returns Pass for the food on a stick.
        C::FoodOnAStickItem => UseResult::Pass,
        // The rocket flies only while gliding; the server spawns it.
        C::FireworkRocketItem if ctx.player.is_fall_flying() => UseResult::Success { swing: true },
        C::FireworkRocketItem => UseResult::Pass,
        C::KnowledgeBookItem => knowledge_book_use(ctx, hand),
        // `Player.startUsingItem`, then a success: the bundle drops its contents while used.
        _ => {
            ctx.player.start_using_item(hand != super::MAIN_HAND);
            UseResult::Success { swing: true }
        }
    }
}

/// `KnowledgeBookItem.use`: the book is used up before anything is checked (unless materials are
/// infinite); a book without recipes fails, the rest is the server's.
fn knowledge_book_use(ctx: &mut Context<'_>, hand: u8) -> UseResult {
    use crate::item::values::Values;
    let Some(stack) = ctx
        .world
        .local_player()
        .and_then(|state| state.inventory_slot(ctx.hand_slot(hand)))
    else {
        return UseResult::Pass;
    };
    let values = Values::new(stack.item_registry_id(), stack.patch_bytes());
    // `getOrDefault(RECIPES, List.of())`: the component has no stream codec of its own, so a
    // value the server patches in is a network NBT list (tag 9, element type, int count); it is
    // empty when the count is 0. Anything shorter than that header (no value, or the item's
    // default) counts as empty.
    let recipes = values
        .raw(photon_data::item::component_types::RECIPES)
        .is_some_and(|bytes| bytes.len() >= 6 && bytes[2..6] != [0, 0, 0, 0]);
    if !ctx.instabuild() {
        let slot = ctx.hand_slot(hand);
        ctx.world.shrink_local_player_slot(slot, 1);
    }
    if recipes {
        UseResult::Success { swing: true }
    } else {
        UseResult::Fail
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

/// `TridentItem.use`: a stack about to break fails, and so does one with the spin attack effect
/// (riptide) out of the water; otherwise the use starts and nothing swings. The rain is not read,
/// so a riptide trident in the rain on land fails here where the reference starts the use.
fn trident_use(ctx: &mut Context<'_>, hand: u8) -> UseResult {
    let held = ctx
        .world
        .local_player()
        .and_then(|state| state.inventory_slot(ctx.hand_slot(hand)));
    let Some(stack) = held else {
        return UseResult::Pass;
    };
    let spins = crate::item::values::Values::new(stack.item_registry_id(), stack.patch_bytes())
        .enchantments(photon_world::item_components::ids::ENCHANTMENTS)
        .iter()
        .any(|&(id, level)| {
            level > 0
                && usize::try_from(id)
                    .ok()
                    .and_then(|id| ctx.spin_attack.get(id))
                    .copied()
                    .unwrap_or(false)
        });
    if stack.breaks_on_next_damage() || (spins && !ctx.player.is_in_water()) {
        return UseResult::Fail;
    }
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
