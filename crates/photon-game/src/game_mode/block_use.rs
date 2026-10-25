//! `BlockBehaviour.useItemOn` for the blocks whose class is written: the stack in the used hand is
//! described to the rules, and what they answer is applied to the client's level.

use super::context::{Aim, Context, MAIN_HAND, OFF_HAND};
use super::{GameMode, UseResult};
use crate::item::values::Values;
use photon_core::direction::Direction;
use photon_core::ids::BlockStateId;
use photon_data::item::classes::ItemClass;
use photon_world::block::rules::{self, Held, ItemUser, Rule};

const GLOWSTONE: &str = "minecraft:glowstone";

fn name_of(item: photon_core::ids::ItemId) -> &'static str {
    rules::item(item).map_or("", |row| row.name)
}

/// What the stack in a hand is.
fn describe(ctx: &Context<'_>, hand: u8) -> Held {
    let Some(stack) = ctx
        .world
        .local_player()
        .and_then(|state| state.inventory_slot(ctx.hand_slot(hand)))
        .filter(|stack| stack.item_count() > 0)
    else {
        return Held {
            empty: true,
            ..Held::default()
        };
    };
    let item = stack.item_registry_id();
    let name = name_of(item);
    let tagged = |tag| ctx.world.item_in_tag(item.0, tag);
    let values = Values::new(item, stack.patch_bytes());
    Held {
        empty: false,
        fire_starter: matches!(name, "minecraft:flint_and_steel" | "minecraft:fire_charge"),
        shears: name == "minecraft:shears",
        honeycomb: name == "minecraft:honeycomb",
        axe: tagged("minecraft:axes"),
        note_block_top: tagged("minecraft:noteblock_top_instruments"),
        glowstone: name == GLOWSTONE,
        lectern_book: tagged("minecraft:lectern_books"),
        campfire_input: ctx
            .world
            .recipe_property_set("minecraft:campfire_input")
            .contains(&item.0),
        block: rules::item(item).and_then(|row| row.block),
        compostable: photon_data::compost::chance(name).is_some(),
        bone_meal: name == "minecraft:bone_meal",
        sign_applicator: rules::item(item).is_some_and(|row| {
            matches!(
                row.class,
                ItemClass::DyeItem
                    | ItemClass::InkSacItem
                    | ItemClass::GlowInkSacItem
                    | ItemClass::HoneycombItem
            )
        }),
        hanging_sign: rules::item(item).is_some_and(|row| row.class == ItemClass::HangingSignItem),
        candle: tagged("minecraft:candles").then_some(name),
        record: values.has(photon_data::item::component_types::JUKEBOX_PLAYABLE),
    }
}

impl GameMode {
    /// `BlockBehaviour.useItemOn`: the written class answers, then the generic table; `Unported`
    /// for a class neither knows.
    pub(super) fn item_on_result(
        ctx: &mut Context<'_>,
        hand: u8,
        state: BlockStateId,
        pos: (i32, i32, i32),
        aim: Aim,
    ) -> Rule<UseResult> {
        match Self::block_use_item_on(ctx, hand, state, pos, aim) {
            Some(result) => Rule::Known(result),
            None => rules::use_item_on(state),
        }
    }

    /// A block's `useItemOn` with the stack in the hand; `None` when the block's class is not
    /// written.
    pub(super) fn block_use_item_on(
        ctx: &mut Context<'_>,
        hand: u8,
        state: BlockStateId,
        pos: (i32, i32, i32),
        aim: Aim,
    ) -> Option<UseResult> {
        let Aim::Block { face, location, .. } = aim else {
            return None;
        };
        if !rules::knows_item_on(state) {
            return None;
        }
        let other = if hand == MAIN_HAND {
            OFF_HAND
        } else {
            MAIN_HAND
        };
        let user = ItemUser {
            main_hand: hand == MAIN_HAND,
            may_build: ctx.player.abilities().may_build(),
            face: Direction::from_3d_data_value(i32::from(face)),
            click_y: location[1] - f64::from(pos.1),
            block_entity: ctx.world.block_entity_at(pos.0, pos.1, pos.2).is_some(),
            off_hand_glowstone: describe(ctx, other).glowstone,
            waxed: ctx
                .world
                .sign_view(pos)
                .is_some_and(|sign| crate::sign::is_waxed(sign.tag)),
        };
        let answer = rules::use_item_with(state, pos, describe(ctx, hand), user)?;
        if let Some(write) = answer.write {
            let position = ctx.player.position();
            ctx.world
                .set_block_with_updates(write.pos, write.state, write.flags, 512, position);
        }
        if answer.shrink && !ctx.instabuild() {
            let slot = ctx.hand_slot(hand);
            ctx.world.shrink_local_player_slot(slot, 1);
        }
        Some(answer.result)
    }
}
