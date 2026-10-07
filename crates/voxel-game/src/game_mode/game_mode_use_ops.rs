use super::*;

impl GameMode {
    /// `Minecraft.startUseItem`: each hand in turn tries the aimed block,
    /// then its item on its own, until one does something.
    pub(super) fn start_use_item(
        &mut self,
        ctx: &mut Context<'_>,
        aim: Aim,
        out: &mut Vec<Outgoing>,
    ) {
        if self.is_destroying {
            return;
        }
        self.right_click_delay = 4;
        for hand in [MAIN_HAND, OFF_HAND] {
            if let Aim::Entity {
                id,
                location,
                position,
                bounds,
            } = aim
            {
                let block = position.map(|value| value.floor() as i32);
                if !ctx.within_border((block[0], block[1], block[2])) {
                    return;
                }
                if ctx.within_entity_interaction_range(bounds) {
                    // `MultiPlayerGameMode.interact`. What the entity does on
                    // its own side of the click is not predicted: the click
                    // passes on to the hand's item as if it had not been
                    // consumed.
                    out.push(Outgoing::Interact {
                        entity_id: id,
                        hand,
                        location: [
                            location[0] - position[0],
                            location[1] - position[1],
                            location[2] - position[2],
                        ],
                        secondary_use: ctx.player.is_shift_key_down(),
                    });
                }
            }
            if let Aim::Block { .. } = aim {
                let old_count = ctx.held(hand).map(|(_, count)| count);
                match self.use_item_on(ctx, hand, aim, out) {
                    Some(UseResult::Success { swing }) => {
                        if swing {
                            ctx.swing_hand(hand, out);
                            // The hand dips when the use spent some of the
                            // stack, or the player never runs out.
                            if let Some((_, count)) = ctx.held(hand)
                                && (Some(count) != old_count || ctx.instabuild())
                            {
                                self.items_used.push(hand);
                            }
                        }
                        return;
                    }
                    Some(UseResult::Fail) => return,
                    _ => {}
                }
            }
            if ctx.held(hand).is_some()
                && let Some(UseResult::Success { swing }) = self.use_item(ctx, hand, out)
            {
                if swing {
                    ctx.swing_hand(hand, out);
                }
                self.items_used.push(hand);
                return;
            }
        }
    }

    /// `MultiPlayerGameMode.useItemOn`: the use runs as a prediction whose
    /// packet goes whatever it comes to. `None` is outside the border.
    pub(super) fn use_item_on(
        &mut self,
        ctx: &mut Context<'_>,
        hand: u8,
        aim: Aim,
        out: &mut Vec<Outgoing>,
    ) -> Option<UseResult> {
        let Aim::Block {
            x,
            y,
            z,
            face,
            location,
            inside,
        } = aim
        else {
            return None;
        };
        if !ctx.within_border((x, y, z)) {
            return Some(UseResult::Fail);
        }
        let sequence = ctx.world.start_predicting();
        let result = self.perform_use_item_on(ctx, hand, aim);
        out.push(Outgoing::UseItemOn {
            hand,
            x,
            y,
            z,
            face,
            cursor: [
                (location[0] - f64::from(x)) as f32,
                (location[1] - f64::from(y)) as f32,
                (location[2] - f64::from(z)) as f32,
            ],
            inside,
            sequence,
        });
        ctx.world.stop_predicting();
        Some(result)
    }

    /// `performUseItemOn`. A block or item whose class's rule is not written
    /// is taken to have done its thing, with a swing, and predicts nothing.
    pub(super) fn perform_use_item_on(
        &mut self,
        ctx: &mut Context<'_>,
        hand: u8,
        aim: Aim,
    ) -> UseResult {
        if ctx.mode == ClientPlayerGameMode::Spectator {
            return UseResult::Success { swing: false };
        }
        let Aim::Block { x, y, z, .. } = aim else {
            return UseResult::Pass;
        };
        let item = ctx.held(hand);
        let have_something = ctx.held(MAIN_HAND).is_some() || ctx.held(OFF_HAND).is_some();
        let suppress_using_block = ctx.player.is_shift_key_down() && have_something;
        if !suppress_using_block {
            let state = ctx.state_at((x, y, z));
            let item_use = match rules::use_item_on(state) {
                Rule::Known(result) => result,
                Rule::Unported => return UseResult::Success { swing: true },
            };
            if item_use.consumes_action() {
                return item_use;
            }
            if item_use == UseResult::TryEmptyHand && hand == MAIN_HAND {
                let used = match rules::use_without_item(state) {
                    Rule::Known(result) => result,
                    Rule::Unported => return UseResult::Success { swing: true },
                };
                if used.consumes_action() {
                    return used;
                }
            }
        }
        match item {
            // No item cooldowns are tracked yet.
            Some((item, _)) => self.item_use_on(ctx, hand, item, aim),
            None => UseResult::Pass,
        }
    }

    /// `ItemStack.useOn`, then `Item.useOn` as the item's class decides it.
    pub(super) fn item_use_on(
        &mut self,
        ctx: &mut Context<'_>,
        hand: u8,
        item: i32,
        aim: Aim,
    ) -> UseResult {
        // A player who may not build passes, unless the stack's `can_place_on`
        // allows the block; that predicate is not evaluated.
        if ctx.mode == ClientPlayerGameMode::Adventure {
            return UseResult::Pass;
        }
        match rules::item_use_on_owner(item) {
            ItemClass::Item => UseResult::Pass,
            ItemClass::BlockItem
                if rules::item(item).is_some_and(|row| row.class == ItemClass::BlockItem) =>
            {
                self.place(ctx, hand, item, aim)
            }
            _ => UseResult::Success { swing: true },
        }
    }

    /// `BlockItem.place` for a plain block item.
    pub(super) fn place(
        &mut self,
        ctx: &mut Context<'_>,
        hand: u8,
        item: i32,
        aim: Aim,
    ) -> UseResult {
        let Aim::Block {
            x,
            y,
            z,
            face,
            location,
            ..
        } = aim
        else {
            return UseResult::Pass;
        };
        let Some(block) = rules::item(item)
            .and_then(|row| row.block)
            .and_then(voxel_data::block::registry::block_index)
        else {
            return UseResult::Success { swing: true };
        };
        let world: &ClientWorld = ctx.world;
        let state_at = |x: i32, y: i32, z: i32| world.block_state_at(x, y, z).unwrap_or(0);
        let (y_rot, _) = ctx.player.rotation();
        let mut place_ctx = rules::PlaceContext {
            state_at: &state_at,
            hit_pos: (x, y, z),
            face: voxel_core::direction::Direction::from_3d_data_value(i32::from(face)),
            location: (location[0], location[1], location[2]),
            item: Some(item),
            horizontal_direction: voxel_core::direction::Direction::from_y_rot(f64::from(y_rot)),
            replace_clicked: true,
        };
        // `BlockPlaceContext`: the clicked block is replaced if it may be.
        place_ctx.replace_clicked = match rules::can_be_replaced(state_at(x, y, z), &place_ctx) {
            Rule::Known(replaced) => replaced,
            Rule::Unported => return UseResult::Success { swing: true },
        };
        let pos = place_ctx.clicked_pos();
        // `canPlace`.
        if !place_ctx.replace_clicked {
            match rules::can_be_replaced(state_at(pos.0, pos.1, pos.2), &place_ctx) {
                Rule::Known(true) => {}
                Rule::Known(false) => return UseResult::Fail,
                Rule::Unported => return UseResult::Success { swing: true },
            }
        }
        let state = match rules::state_for_placement(block, &place_ctx) {
            Rule::Known(Some(state)) => state,
            Rule::Known(None) => return UseResult::Fail,
            Rule::Unported => return UseResult::Success { swing: true },
        };
        // `BlockItem.canPlace`: it must stand there and not be in the way.
        match rules::can_survive(state) {
            Rule::Known(true) => {}
            Rule::Known(false) => return UseResult::Fail,
            Rule::Unported => return UseResult::Success { swing: true },
        }
        let block_pos = voxel_core::binary::BlockPos::new(pos.0, pos.1, pos.2);
        let obstructed = !ctx
            .world
            .block_state_registry()
            .collision_shape(state)
            .is_empty()
            && !ctx.player.spectating()
            && ctx.player.is_colliding(ctx.world, block_pos, state);
        if obstructed {
            return UseResult::Fail;
        }
        let position = ctx.player.position();
        if !ctx
            .world
            .set_block_with_updates(pos, state, 11, 512, position)
        {
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

    /// `MultiPlayerGameMode.useItem`: the packet goes whatever the item
    /// does. `None` is a spectator's use, which sends nothing.
    pub(super) fn use_item(
        &mut self,
        ctx: &mut Context<'_>,
        hand: u8,
        out: &mut Vec<Outgoing>,
    ) -> Option<UseResult> {
        if ctx.mode == ClientPlayerGameMode::Spectator {
            return Some(UseResult::Pass);
        }
        let sequence = ctx.world.start_predicting();
        let (y_rot, x_rot) = ctx.player.rotation();
        let result = match ctx.held(hand) {
            Some((item, _)) => {
                let row = rules::item(item);
                if rules::item_use_owner(item) == ItemClass::Item
                    && row.is_some_and(voxel_data::item::registry::Item::use_does_nothing)
                {
                    UseResult::Pass
                } else {
                    // Eating, equipping and the rest are not written yet.
                    UseResult::Success { swing: false }
                }
            }
            None => UseResult::Pass,
        };
        out.push(Outgoing::UseItem {
            hand,
            sequence,
            y_rot,
            x_rot,
        });
        ctx.world.stop_predicting();
        Some(result)
    }
}
