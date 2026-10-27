//! The client's part of `Item.useOn` for the tools that change a block: the server changes it,
//! the client only decides whether the tool applies and plays the sound the server leaves out
//! for the one using it.

use super::context::Context;
use super::mining::{block, is_air};
use super::{Aim, UseResult};
use photon_world::local_player::{LocalSound, LocalSoundSource};

/// `HoeItem.TILLABLES`: whether the hoe works on the block (by name), which of them need the
/// face not to be the bottom one and air above.
fn tillable(name: &str) -> Option<bool> {
    match name {
        "minecraft:grass_block"
        | "minecraft:dirt_path"
        | "minecraft:dirt"
        | "minecraft:coarse_dirt" => Some(true),
        "minecraft:rooted_dirt" => Some(false),
        _ => None,
    }
}

/// Plays `event` at the middle of a block, loud and at normal pitch, as
/// `Level.playSound(player, pos, ...)` does for the one using the tool.
fn play_at_block(ctx: &mut Context<'_>, event: &'static str, (x, y, z): (i32, i32, i32)) {
    ctx.world.push_sound(LocalSound::at(
        event,
        LocalSoundSource::Blocks,
        (1.0, 1.0),
        (f64::from(x) + 0.5, f64::from(y) + 0.5, f64::from(z) + 0.5),
    ));
}

/// `ShovelItem.FLATTENABLES`.
fn flattenable(name: &str) -> bool {
    matches!(
        name,
        "minecraft:grass_block"
            | "minecraft:dirt"
            | "minecraft:podzol"
            | "minecraft:coarse_dirt"
            | "minecraft:mycelium"
            | "minecraft:rooted_dirt"
    )
}

/// `ShovelItem.useOn` for the client: a flattenable block with air above sounds the shovel, a lit
/// campfire is dowsed (the server puts it out; the client shows the smoke), the bottom face and
/// anything else pass.
pub(super) fn shovel_use_on(ctx: &mut Context<'_>, aim: Aim) -> UseResult {
    let Aim::Block { x, y, z, face, .. } = aim else {
        return UseResult::Pass;
    };
    if face == super::mining::DIRECTION_DOWN {
        return UseResult::Pass;
    }
    let state = ctx.state_at((x, y, z));
    let name = block(state).name;
    if flattenable(name) && is_air(ctx.state_at((x, y + 1, z))) {
        play_at_block(ctx, "minecraft:item.shovel.flatten", (x, y, z));
        return UseResult::Success { swing: true };
    }
    if dowse_lit_campfire(ctx, (x, y, z), state) {
        return UseResult::Success { swing: true };
    }
    UseResult::Pass
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

/// `AxeItem.STRIPPABLES`: the stripped block's name for a log, wood, stem, hyphae or bamboo block.
fn stripped(name: &str) -> Option<String> {
    let path = name.strip_prefix("minecraft:")?;
    let strippable = path == "bamboo_block"
        || [
            "oak", "dark_oak", "pale_oak", "acacia", "cherry", "birch", "jungle", "spruce",
            "mangrove",
        ]
        .iter()
        .any(|tree| path == format!("{tree}_log") || path == format!("{tree}_wood"))
        || ["warped", "crimson"]
            .iter()
            .any(|fungus| path == format!("{fungus}_stem") || path == format!("{fungus}_hyphae"));
    strippable.then(|| format!("minecraft:stripped_{path}"))
}

/// `AxeItem.useOn` for the client: not with a blocking item in the off hand unless sneaking; a
/// log strips to its stripped block (axis kept), sounding the axe and setting the block as the
/// reference does on the client too; weathered copper scrapes one age back and waxed copper loses
/// its wax; anything else passes.
pub(super) fn axe_use_on(ctx: &mut Context<'_>, hand: u8, aim: Aim) -> UseResult {
    let Aim::Block { x, y, z, .. } = aim else {
        return UseResult::Pass;
    };
    // `playerHasBlockingItemUseIntent`.
    let off_hand_blocks = ctx
        .world
        .local_player()
        .and_then(|state| state.inventory_slot(ctx.hand_slot(super::OFF_HAND)))
        .is_some_and(|stack| {
            crate::item::values::Values::new(stack.item_registry_id(), stack.patch_bytes())
                .has(photon_data::item::component_types::BLOCKS_ATTACKS)
        });
    if hand == super::MAIN_HAND && off_hand_blocks && !ctx.player.is_shift_key_down() {
        return UseResult::Pass;
    }
    let state = ctx.state_at((x, y, z));
    let name = block(state).name;
    let Some(target) = stripped(name) else {
        let position = ctx.player.position();
        let (changed, sound) = match photon_world::block::rules::scraped(state) {
            Some(scraped) => (scraped, "minecraft:item.axe.scrape"),
            None => match photon_world::block::rules::waxed_off(state) {
                Some(unwaxed) => (unwaxed, "minecraft:item.axe.wax_off"),
                None => return UseResult::Pass,
            },
        };
        play_at_block(ctx, sound, (x, y, z));
        ctx.world
            .set_block_with_updates((x, y, z), changed, 11, 512, position);
        return UseResult::Success { swing: true };
    };
    let axis = photon_world::block::rules::property(state, "axis");
    let new_state = photon_data::block::registry::BLOCKS
        .iter()
        .find(|candidate| candidate.name == target)
        .map(|candidate| candidate.default_state_id())
        .and_then(|default| {
            photon_world::block::rules::with_property(default, "axis", axis.unwrap_or("y"))
        });
    let Some(new_state) = new_state else {
        return UseResult::Success { swing: true };
    };
    play_at_block(ctx, "minecraft:item.axe.strip", (x, y, z));
    let position = ctx.player.position();
    ctx.world
        .set_block_with_updates((x, y, z), new_state, 11, 512, position);
    UseResult::Success { swing: true }
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

/// `Item.useOn` of the hoe, axe, honeycomb and shovel.
fn ground_tool_use_on(
    ctx: &mut Context<'_>,
    (hand, class): (u8, photon_data::item::classes::ItemClass),
    aim: Aim,
) -> UseResult {
    use photon_data::item::classes::ItemClass as C;
    match class {
        C::HoeItem => hoe_use_on(ctx, aim),
        C::AxeItem => axe_use_on(ctx, hand, aim),
        C::HoneycombItem => honeycomb_use_on(ctx, hand, aim),
        _ => shovel_use_on(ctx, aim),
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
        C::EnderEyeItem => super::utility::ender_eye_use_on(ctx, aim),
        // `SpawnEggItem.useOn` on the client: the spawn is the server's.
        C::SpawnEggItem => super::spawn_egg::use_on(ctx, hand),
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
    if !is_air(ctx.state_at(beside)) || !photon_world::block::fire::survives_at(at, beside) {
        return None;
    }
    Some((
        beside,
        photon_world::block::fire::placement_state(at, beside),
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

/// `HoeItem.useOn` for the client: a tillable block with its predicate met sounds the hoe and
/// succeeds; any other passes.
pub(super) fn hoe_use_on(ctx: &mut Context<'_>, aim: Aim) -> UseResult {
    let Aim::Block { x, y, z, face, .. } = aim else {
        return UseResult::Pass;
    };
    let Some(needs_air) = tillable(block(ctx.state_at((x, y, z))).name) else {
        return UseResult::Pass;
    };
    // `HoeItem.onlyIfAirAbove`: the face is not down, and the block above is air.
    if needs_air && (face == super::mining::DIRECTION_DOWN || !is_air(ctx.state_at((x, y + 1, z))))
    {
        return UseResult::Pass;
    }
    play_at_block(ctx, "minecraft:item.hoe.till", (x, y, z));
    UseResult::Success { swing: true }
}

#[cfg(test)]
mod tests {
    use super::{stripped, tillable};

    #[test]
    fn logs_strip_and_stripped_logs_do_not() {
        assert_eq!(
            stripped("minecraft:oak_log").as_deref(),
            Some("minecraft:stripped_oak_log")
        );
        assert_eq!(
            stripped("minecraft:crimson_hyphae").as_deref(),
            Some("minecraft:stripped_crimson_hyphae")
        );
        assert_eq!(
            stripped("minecraft:bamboo_block").as_deref(),
            Some("minecraft:stripped_bamboo_block")
        );
        assert_eq!(stripped("minecraft:stripped_oak_log"), None);
        assert_eq!(stripped("minecraft:oak_planks"), None);
    }

    #[test]
    fn the_shovel_flattens_six_blocks() {
        for name in [
            "minecraft:grass_block",
            "minecraft:podzol",
            "minecraft:mycelium",
        ] {
            assert!(super::flattenable(name), "{name}");
        }
        assert!(!super::flattenable("minecraft:dirt_path"));
        assert!(!super::flattenable("minecraft:stone"));
    }

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

    #[test]
    fn the_hoe_tills_five_blocks() {
        for name in [
            "minecraft:grass_block",
            "minecraft:dirt_path",
            "minecraft:dirt",
            "minecraft:coarse_dirt",
        ] {
            assert_eq!(tillable(name), Some(true), "{name}");
        }
        assert_eq!(tillable("minecraft:rooted_dirt"), Some(false));
        assert_eq!(tillable("minecraft:stone"), None);
    }

    /// Clicks a lit campfire at (1, 1, 1) with an item, from three blocks off and looking at it
    /// from above, and returns what the world is asked for.
    fn click_on_lit_campfire(item: &str) -> Vec<photon_world::LevelEvent> {
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
            spin_attack: &[],
            prevents_armor_change: &[],
            difficulty: 2,
            game_master: false,
            profile_id: None,
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
        game_mode.keys_tick(&mut ctx, keys, aim);
        world.take_level_events()
    }

    fn dowsed(events: &[photon_world::LevelEvent]) -> bool {
        events
            .iter()
            .any(|event| matches!(event, photon_world::LevelEvent::CampfireDowse { .. }))
    }

    /// The click that puts a lit campfire out with a shovel, played through the game mode: it
    /// counts as done and the world is asked for the smoke of the dousing.
    #[test]
    fn a_shovel_on_a_lit_campfire_asks_for_its_smoke() {
        let events = click_on_lit_campfire("minecraft:iron_shovel");
        assert!(dowsed(&events), "{events:?}");
    }

    /// `CampfireBlock.placeLiquid` dowses on the client too: a bucket of water on a lit campfire
    /// asks for the smoke, and a bucket of lava or a stick does not.
    #[test]
    fn a_water_bucket_on_a_lit_campfire_asks_for_its_smoke() {
        let events = click_on_lit_campfire("minecraft:water_bucket");
        assert!(dowsed(&events), "{events:?}");
        assert!(!dowsed(&click_on_lit_campfire("minecraft:stick")));
    }
}
