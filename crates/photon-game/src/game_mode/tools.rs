//! The client's part of `Item.useOn` for the tools that change a block: the server changes it,
//! the client only decides whether the tool applies and plays the sound the server leaves out
//! for the one using it.

use super::context::Context;
use super::mining::{block, is_air};
use super::{Aim, UseResult};
use photon_world::local_player::{LocalSound, LocalSoundSource};

mod class_path;
pub(super) mod component;

/// `playerHasBlockingItemUseIntent`: the main hand does not use a block while the off hand holds
/// something that blocks attacks, unless the player is sneaking.
pub(super) fn blocking_intent(ctx: &Context<'_>, hand: u8) -> bool {
    let off_hand_blocks = ctx
        .world
        .local_player()
        .and_then(|state| state.inventory_slot(ctx.hand_slot(super::OFF_HAND)))
        .is_some_and(|stack| {
            crate::item::values::Values::new(stack.item_registry_id(), stack.patch_bytes())
                .has(photon_data::item::component_types::BLOCKS_ATTACKS)
        });
    hand == super::MAIN_HAND && off_hand_blocks && !ctx.player.is_shift_key_down()
}

/// Plays `event` at the middle of a block, loud and at normal pitch, as
/// `Level.playSound(player, pos, ...)` does for the one using the tool.
pub(super) fn play_at_block(
    ctx: &mut Context<'_>,
    event: &'static str,
    (x, y, z): (i32, i32, i32),
) {
    ctx.world.push_sound(LocalSound::at(
        event,
        LocalSoundSource::Blocks,
        (1.0, 1.0),
        (f64::from(x) + 0.5, f64::from(y) + 0.5, f64::from(z) + 0.5),
    ));
}

/// `CampfireBlock.dowse` on a client, for the shovel and the water of a bucket (`placeLiquid`):
/// a lit campfire gives off twenty puffs of smoke. Whether `state` was one.
pub(super) fn dowse_lit_campfire(
    ctx: &mut Context<'_>,
    pos: (i32, i32, i32),
    state: photon_core::ids::BlockStateId,
) -> bool {
    let campfire = matches!(
        block(state).name,
        "minecraft:campfire" | "minecraft:soul_campfire"
    );
    let flag = |key| photon_world::block::rules::property(state, key) == Some("true");
    let lit = campfire && flag("lit");
    if lit {
        ctx.world.push_campfire_dowse(pos, flag("signal_fire"));
    }
    lit
}

/// `HoneycombItem.useOn` for the client: a copper block takes its wax (flags 11), the stack
/// shrinks, and the wax event plays for the block and the other half of a double chest; any other
/// block passes.
pub(super) fn honeycomb_use_on(ctx: &mut Context<'_>, hand: u8, aim: Aim) -> UseResult {
    let Aim::Block { x, y, z, .. } = aim else {
        return UseResult::Pass;
    };
    let old = ctx.state_at((x, y, z));
    let Some(waxed) = photon_world::block::rules::waxed(old) else {
        return UseResult::Pass;
    };
    // `performUseItemOn` gives a creative player's stack back its count.
    if !ctx.instabuild() {
        let slot = ctx.hand_slot(hand);
        ctx.world.shrink_local_player_slot(slot, 1);
    }
    let position = ctx.player.position();
    ctx.world
        .set_block_with_updates((x, y, z), waxed, 11, 512, position);
    ctx.world.apply_level_event(3003, (x, y, z), 0);
    if let Some(other) = photon_world::block::rules::other_chest_half(old, (x, y, z)) {
        ctx.world.apply_level_event(3003, other, 0);
    }
    UseResult::Success { swing: true }
}

/// `ShearsItem.useOn` for the client: a growing plant's head that is not at its full age (kelp,
/// twisting, weeping and cave vines) sounds the shears and takes the full age at once; anything
/// else passes (`Item.useOn`).
pub(super) fn shears_use_on(ctx: &mut Context<'_>, aim: Aim) -> UseResult {
    let Aim::Block { x, y, z, .. } = aim else {
        return UseResult::Pass;
    };
    let state = ctx.state_at((x, y, z));
    let head = matches!(
        block(state).name,
        "minecraft:kelp"
            | "minecraft:twisting_vines"
            | "minecraft:weeping_vines"
            | "minecraft:cave_vines"
    );
    let age = photon_world::block::rules::property(state, "age");
    if !head || age == Some("25") {
        return UseResult::Pass;
    }
    let Some(grown) = photon_world::block::rules::with_property(state, "age", "25") else {
        return UseResult::Success { swing: true };
    };
    play_at_block(ctx, "minecraft:block.growing_plant.crop", (x, y, z));
    let position = ctx.player.position();
    ctx.world
        .set_block_with_updates((x, y, z), grown, 3, 512, position);
    UseResult::Success { swing: true }
}

/// `Item.useOn` of the hoe, axe, honeycomb and shovel classes.
fn ground_tool_use_on(
    ctx: &mut Context<'_>,
    (hand, class): (u8, photon_data::item::classes::ItemClass),
    aim: Aim,
) -> UseResult {
    use photon_data::item::classes::ItemClass as C;
    match class {
        C::HoneycombItem => honeycomb_use_on(ctx, hand, aim),
        _ => class_path::use_on(ctx, (hand, class), aim),
    }
}

/// `Item.useOn` of the tools whose class the client decides: `None` for any other class.
pub(super) fn use_on(
    ctx: &mut Context<'_>,
    (hand, class): (u8, photon_data::item::classes::ItemClass),
    aim: Aim,
) -> Option<UseResult> {
    use photon_data::item::classes::ItemClass as C;
    Some(match class {
        C::HoeItem | C::AxeItem | C::HoneycombItem | C::ShovelItem => {
            ground_tool_use_on(ctx, (hand, class), aim)
        }
        C::FlintAndSteelItem => flint_and_steel_use_on(ctx, aim),
        C::FireChargeItem => fire_charge_use_on(ctx, hand, aim),
        C::ShearsItem => shears_use_on(ctx, aim),
        C::BoneMealItem => return super::bone_meal::use_on(ctx, aim),
        C::PotionItem => super::bottles::potion_use_on(ctx, hand, aim),
        C::BrushItem => super::brush::use_on(ctx, hand),
        C::EnderEyeItem | C::SpawnEggItem | C::HangingEntityItem | C::ItemFrameItem => {
            super::hanging::entity_use_on(ctx, (hand, class), aim)
        }
        _ => return None,
    })
}

/// `CampfireBlock.canLight`, `CandleBlock.canLight` and `CandleCakeBlock.canLight`: an unlit
/// campfire or candle that is not under water, or an unlit candle cake.
fn lightable(state: photon_core::ids::BlockStateId) -> bool {
    use photon_world::block::rules::property;
    let path = block(state).name.trim_start_matches("minecraft:");
    let unlit = property(state, "lit") == Some("false");
    let dry = property(state, "waterlogged") == Some("false");
    match path {
        "campfire" | "soul_campfire" | "candle" => unlit && dry,
        _ if path.ends_with("candle_cake") => unlit,
        _ if path.ends_with("_candle") => unlit && dry,
        _ => false,
    }
}

/// Where flint and steel or a fire charge lights something and the state it leaves there: the
/// clicked campfire, candle or candle cake, or fire in the empty place beside the face when it
/// would survive (not the nether portal case). `None` where nothing lights.
fn ignition(
    ctx: &Context<'_>,
    (x, y, z, face): (i32, i32, i32, u8),
) -> Option<((i32, i32, i32), Option<photon_core::ids::BlockStateId>)> {
    let clicked = ctx.state_at((x, y, z));
    if lightable(clicked) {
        let lit = photon_world::block::rules::with_property(clicked, "lit", "true");
        return Some(((x, y, z), lit));
    }
    let (dx, dy, dz) =
        photon_core::direction::Direction::from_3d_data_value(i32::from(face)).step();
    let beside = (x + dx, y + dy, z + dz);
    let at = |(x, y, z)| ctx.world.block_state_or_air(x, y, z);
    let tags = ctx.world.block_state_registry().tags();
    if !is_air(ctx.state_at(beside)) || !photon_world::block::fire::survives_at(at, beside, tags) {
        return None;
    }
    Some((
        beside,
        photon_world::block::fire::placement_state(at, beside, tags),
    ))
}

/// `FlintAndSteelItem.useOn` for the client: lights what `ignition` finds with the flint's sound
/// (a pitch from the level's random) and the block set at once (flags 11); fails where nothing
/// lights.
pub(super) fn flint_and_steel_use_on(ctx: &mut Context<'_>, aim: Aim) -> UseResult {
    let Aim::Block { x, y, z, face, .. } = aim else {
        return UseResult::Pass;
    };
    let Some((target, placed)) = ignition(ctx, (x, y, z, face)) else {
        return UseResult::Fail;
    };
    let pitch = ctx.world.next_level_float() * 0.4 + 0.8;
    ctx.world.push_sound(LocalSound::at(
        "minecraft:item.flintandsteel.use",
        LocalSoundSource::Blocks,
        (1.0, pitch),
        (
            f64::from(target.0) + 0.5,
            f64::from(target.1) + 0.5,
            f64::from(target.2) + 0.5,
        ),
    ));
    if let Some(state) = placed {
        let position = ctx.player.position();
        ctx.world
            .set_block_with_updates(target, state, 11, 512, position);
    }
    UseResult::Success { swing: true }
}

/// `FireChargeItem.useOn` for the client: the same places as flint, set with `setBlockAndUpdate`
/// (flags 3). Its sound is played by the server for everyone (the call names no player). One
/// charge is used up whatever the game mode (`ItemStack.shrink`, not `consume`).
pub(super) fn fire_charge_use_on(ctx: &mut Context<'_>, hand: u8, aim: Aim) -> UseResult {
    let Aim::Block { x, y, z, face, .. } = aim else {
        return UseResult::Pass;
    };
    let Some((target, placed)) = ignition(ctx, (x, y, z, face)) else {
        return UseResult::Fail;
    };
    if let Some(state) = placed {
        let position = ctx.player.position();
        ctx.world
            .set_block_with_updates(target, state, 3, 512, position);
    }
    if !ctx.instabuild() {
        let slot = ctx.hand_slot(hand);
        ctx.world.shrink_local_player_slot(slot, 1);
    }
    UseResult::Success { swing: true }
}

#[cfg(test)]
mod tests {
    fn named(name: &str) -> photon_core::ids::BlockStateId {
        let index = photon_data::block::registry::block_index(name).expect(name);
        photon_data::block::registry::BLOCKS[index].default_state_id()
    }

    #[test]
    fn flint_lights_what_is_unlit_and_dry() {
        use photon_world::block::rules::with_property;
        let lit = named("minecraft:campfire");
        assert!(!super::lightable(lit));
        assert!(super::lightable(
            with_property(lit, "lit", "false").unwrap()
        ));
        let wet = with_property(lit, "lit", "false")
            .and_then(|s| with_property(s, "waterlogged", "true"));
        assert!(!super::lightable(wet.unwrap()));
        assert!(super::lightable(named("minecraft:red_candle")));
        assert!(super::lightable(named("minecraft:candle")));
        assert!(!super::lightable(named("minecraft:stone")));
        let cake = named("minecraft:white_candle_cake");
        assert!(super::lightable(cake));
        assert!(!super::lightable(
            with_property(cake, "lit", "true").unwrap()
        ));
    }

    /// Clicks a lit campfire at (1, 1, 1) with an item, from three blocks off and looking at it
    /// from above, and returns what the world is asked for.
    /// `class` is the item class whose own `useOn` is run directly; none clicks through the game
    /// mode, which runs the rules of the version under test.
    fn click_on_lit_campfire(
        item: &str,
        class: Option<photon_data::item::classes::ItemClass>,
    ) -> Vec<photon_world::LevelEvent> {
        use crate::game_mode::{Aim, Context, GameMode, Keys, Miner};
        use photon_world::{
            ClientPlayerGameMode, ClientWorld, DimensionBounds, SectionInput,
            local_player::LocalPlayer,
        };

        let mut blocks = vec![0; 4_096];
        blocks[256 + 16 + 1] = named("minecraft:campfire").0;
        let biomes = vec![0; 64];
        let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
        assert!(world.replace_chunk(
            0,
            0,
            [SectionInput {
                non_empty_block_count: 1,
                fluid_count: 0,
                block_states: &blocks,
                biomes: &biomes,
            }],
        ));
        let mut player = LocalPlayer::new(
            photon_core::ids::EntityId(1),
            photon_core::math::Vec3::new(1.5, 1.0, 3.5),
            180.0,
            35.0,
            ClientPlayerGameMode::Survival,
            true,
        );
        world.set_local_player_id(player.entity_id());
        let held = crate::item::id_of(item).expect("an item");
        let stack = photon_world::ClientItemStackState::new(1, held);
        world.apply_local_player_container_slot(0, 0, 36, Some(&stack));
        player.set_main_hand_item(held);

        let mut cooldowns = crate::item::cooldowns::ItemCooldowns::default();
        let mut game_mode = GameMode::default();
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
        let keys = Keys {
            use_clicks: 1,
            ..Default::default()
        };
        let aim = Aim::Block {
            x: 1,
            y: 1,
            z: 1,
            face: 1,
            location: [1.5, 1.4375, 1.5],
            inside: false,
        };
        match class {
            Some(class) => {
                super::class_path::use_on(&mut ctx, (crate::game_mode::MAIN_HAND, class), aim);
            }
            None => {
                game_mode.keys_tick(&mut ctx, keys, aim);
            }
        }
        world.take_level_events()
    }

    fn dowsed(events: &[photon_world::LevelEvent]) -> bool {
        events
            .iter()
            .any(|event| matches!(event, photon_world::LevelEvent::CampfireDowse { .. }))
    }

    /// The shovel class's own `useOn` (before 26.3) puts a lit campfire out: the world is asked
    /// for the smoke of the dousing.
    #[test]
    fn a_shovel_on_a_lit_campfire_asks_for_its_smoke() {
        let class = photon_data::item::classes::ItemClass::ShovelItem;
        let events = click_on_lit_campfire("minecraft:iron_shovel", Some(class));
        assert!(dowsed(&events), "{events:?}");
    }

    /// `CampfireBlock.placeLiquid` dowses on the client too: a bucket of water on a lit campfire
    /// asks for the smoke, and a bucket of lava or a stick does not.
    #[test]
    fn a_water_bucket_on_a_lit_campfire_asks_for_its_smoke() {
        let events = click_on_lit_campfire("minecraft:water_bucket", None);
        assert!(dowsed(&events), "{events:?}");
        assert!(!dowsed(&click_on_lit_campfire("minecraft:stick", None)));
    }
}
