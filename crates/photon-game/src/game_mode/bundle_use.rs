//! `BundleItem.onUseTick`: what a bundle in use does on a client.

use super::{Context, MAIN_HAND, OFF_HAND};
use photon_data::item::classes::ItemClass;
use photon_world::block::rules;
use photon_world::local_player::{LocalSound, LocalSoundSource};

/// `BundleItem.getUseDuration`.
const USE_DURATION: i32 = 200;

/// The first tick of a use takes one stack out of the bundle, and so does every second tick from
/// the eleventh on: the stack leaves the bundle with its sound, and `Player.drop` on a client only
/// swings the main hand.
pub(super) fn use_tick(ctx: &mut Context<'_>) {
    if !ctx.player.is_using_item() {
        return;
    }
    let hand = if ctx.player.used_item_is_off_hand() {
        OFF_HAND
    } else {
        MAIN_HAND
    };
    let Some((item, _)) = ctx.held(hand) else {
        return;
    };
    if rules::item_use_owner(item) != ItemClass::BundleItem {
        return;
    }
    let used = ctx.player.used_ticks();
    let remaining = USE_DURATION - used;
    if used != 0 && !(remaining < USE_DURATION - 10 && remaining % 2 == 0) {
        return;
    }
    let slot = ctx.hand_slot(hand);
    let Some(mut bundle) = ctx
        .world
        .local_player()
        .and_then(|state| state.inventory_slot(slot).cloned())
    else {
        return;
    };
    if crate::menus::container::remove_one_from_bundle(&mut bundle).is_none() {
        return;
    }
    ctx.world.put_local_player_slot(slot, bundle);
    // `playRemoveOneSound` is the player's own, heard here; `playDropContentsSound` excludes no
    // player, so the server's copy is the one heard and only its pitch is drawn.
    let random = ctx.world.level_random();
    let pitch = 0.8 + random.next_float() * 0.4;
    // `ClientLevel.playLocalSound` seeds the sound from the level's random.
    random.next_long();
    random.next_float();
    let at = ctx.player.position();
    ctx.world.push_sound(LocalSound::at(
        crate::menus::container::BUNDLE_REMOVE_ONE,
        LocalSoundSource::Players,
        (0.8, pitch),
        (at.x, at.y, at.z),
    ));
    ctx.player.swing_for_drop();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_mode::Miner;
    use photon_world::{ClientItemStackState, ClientPlayerGameMode, ClientWorld, DimensionBounds};

    /// A bundle held in use gives a stack up on its first tick, with the main hand's swing.
    #[test]
    fn a_bundle_in_use_drops_a_stack_on_its_first_tick() {
        let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
        let mut player = photon_world::local_player::LocalPlayer::new(
            photon_core::ids::EntityId(1),
            photon_core::math::Vec3::new(0.5, 1.0, 0.5),
            0.0,
            0.0,
            ClientPlayerGameMode::Survival,
            true,
        );
        world.set_local_player_id(player.entity_id());
        let id = |name| crate::item::id_of(name).expect("an item");
        let mut bundle = ClientItemStackState::new(1, id("minecraft:bundle"));
        let stone = ClientItemStackState::new(4, id("minecraft:cobblestone"));
        crate::menus::container::fill_bundle_for_test(&mut bundle, stone);
        world.apply_local_player_container_slot(0, 0, 36, Some(&bundle));
        player.set_main_hand_item(id("minecraft:bundle"));
        player.start_using_item(false);
        let mut cooldowns = crate::item::cooldowns::ItemCooldowns::default();
        let mut ctx = Context {
            miner: &Miner::of(&world, 0),
            world: &mut world,
            player: &mut player,
            mode: ClientPlayerGameMode::Survival,
            selected_slot: 0,
            cooldowns: &mut cooldowns,
            instruments: &[],
            spin_attack: &[],
            prevents_armor_change: &[],
            difficulty: 2,
            game_master: false,
            profile_id: None,
            creaking_active: false,
        };
        use_tick(&mut ctx);
        let after = ctx
            .world
            .local_player()
            .and_then(|state| state.inventory_slot(36).cloned());
        let emptied =
            after.is_some_and(|stack| crate::menus::container::bundle_fullness(&stack) == 0.0);
        assert!(emptied);
        assert!(ctx.player.is_swinging());
        assert!(!ctx.world.take_sounds().is_empty());
    }
}
