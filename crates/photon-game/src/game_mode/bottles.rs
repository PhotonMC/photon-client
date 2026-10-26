//! `BottleItem.use` as the client predicts it: a glass bottle filled from water.

use super::*;
use photon_data::item::component::Component;
use photon_world::local_player::{LocalSound, LocalSoundSource};

/// `PotionContents.createItemStack(Items.POTION, Potions.WATER)`.
fn water_bottle() -> Option<ClientItemStackState> {
    let item = crate::item::id_of("minecraft:potion")?;
    let (id, value) = crate::item::default_components::encode(
        &Component::PotionContents(Some("minecraft:water")),
        None,
    )?;
    let patch = photon_world::item_components::Patch {
        added: vec![(id, value)],
        removed: Vec::new(),
    };
    Some(
        ClientItemStackState::from_wire(1, item, vec![id], Vec::new())
            .with_patch_bytes(patch.trusted()),
    )
}

/// `BottleItem.use`. The dragon's-breath branch needs an area effect cloud owned by an ender
/// dragon, which the client is not told of, so it is not taken.
pub(super) fn bottle_use(ctx: &mut Context<'_>, hand: u8) -> UseResult {
    let Some(hit) = super::buckets::pov_hit(ctx, true) else {
        return UseResult::Pass;
    };
    let pos = (hit.block_pos.x, hit.block_pos.y, hit.block_pos.z);
    if !rules::holds_water(ctx.state_at(pos)) {
        return UseResult::Pass;
    }
    // `level.playSound(player, ..)`: the local player is the excluded one, so it is heard.
    let at = ctx.player.position();
    ctx.world.push_sound(LocalSound::at(
        "minecraft:item.bottle.fill",
        LocalSoundSource::Neutral,
        (1.0, 1.0),
        (at.x, at.y, at.z),
    ));
    match water_bottle() {
        Some(fresh) => super::buckets::filled_with(ctx, hand, fresh),
        None => UseResult::Success { swing: true },
    }
}

/// `PotionItem.useOn`: a water potion turns the dirt it is poured on (not from below) into mud and
/// leaves a glass bottle. The sounds name no player, so none play here. The blocks of
/// `BlockTags.CONVERTABLE_TO_MUD` are the vanilla ones.
pub(super) fn potion_use_on(ctx: &mut Context<'_>, hand: u8, aim: Aim) -> UseResult {
    let Aim::Block { x, y, z, face, .. } = aim else {
        return UseResult::Pass;
    };
    let pos = (x, y, z);
    let convertable = matches!(
        photon_world::block::state_or_air(ctx.state_at(pos))
            .block()
            .name,
        "minecraft:dirt" | "minecraft:coarse_dirt" | "minecraft:rooted_dirt"
    );
    let water = ctx.held(hand).is_some_and(|(item, _)| {
        let stack = ctx
            .world
            .local_player()
            .and_then(|state| state.inventory_slot(ctx.hand_slot(hand)));
        stack.is_some_and(|stack| {
            let values = crate::item::values::Values::new(item, stack.patch_bytes());
            super::block_use::is_water_potion(&values)
        })
    });
    if face == super::mining::DIRECTION_DOWN || !convertable || !water {
        return UseResult::Pass;
    }
    let (Some(bottle), Some(mud)) = (
        crate::item::id_of("minecraft:glass_bottle"),
        photon_data::block::registry::block_index("minecraft:mud")
            .map(|index| photon_data::block::registry::BLOCKS[index].default_state_id()),
    ) else {
        return UseResult::Success { swing: true };
    };
    super::buckets::filled_with(ctx, hand, ClientItemStackState::new(1, bottle));
    let position = ctx.player.position();
    ctx.world.set_block_with_updates(pos, mud, 3, 512, position);
    UseResult::Success { swing: true }
}

/// `BeehiveBlock.useItemOn` with a glass bottle: the stack shrinks in any game mode, the player
/// hears the fill, and the honey replaces an emptied stack or goes to the inventory (what does not
/// fit is dropped, which the client does not predict).
pub(super) fn take_honey(ctx: &mut Context<'_>, hand: u8) {
    let slot = ctx.hand_slot(hand);
    ctx.world.shrink_local_player_slot(slot, 1);
    let at = ctx.player.position();
    ctx.world.push_sound(LocalSound::at(
        "minecraft:item.bottle.fill",
        LocalSoundSource::Blocks,
        (1.0, 1.0),
        (at.x, at.y, at.z),
    ));
    let Some(honey) = crate::item::id_of("minecraft:honey_bottle") else {
        return;
    };
    let fresh = ClientItemStackState::new(1, honey);
    let emptied = ctx
        .world
        .local_player()
        .and_then(|state| state.inventory_slot(slot))
        .is_none_or(|stack| stack.item_count() <= 0);
    if emptied {
        ctx.world.put_local_player_slot(slot, fresh);
    } else {
        super::buckets::add_to_inventory(ctx, fresh);
    }
}
