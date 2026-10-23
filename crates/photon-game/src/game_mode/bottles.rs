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
