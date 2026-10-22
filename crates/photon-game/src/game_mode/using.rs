use super::*;
use photon_core::ids::ItemId;

impl GameMode {
    /// The hands whose stack was used since the last call, in order.
    pub fn take_items_used(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.items_used)
    }

    /// The hand whose book and quill's screen is to open, once.
    /// The position of the command block used, whose screen is to open.
    pub fn take_opened_command_block(&mut self) -> Option<(i32, i32, i32)> {
        self.opened_command_block.take()
    }

    /// The position of the jigsaw block used, whose screen is to open.
    pub fn take_opened_jigsaw_block(&mut self) -> Option<(i32, i32, i32)> {
        self.opened_jigsaw_block.take()
    }

    /// The position of the test block used, whose screen is to open.
    pub fn take_opened_test_block(&mut self) -> Option<(i32, i32, i32)> {
        self.opened_test_block.take()
    }

    /// The position of the test instance block used, whose screen is to open.
    pub fn take_opened_test_instance_block(&mut self) -> Option<(i32, i32, i32)> {
        self.opened_test_instance_block.take()
    }

    /// The position of the structure block used, whose screen is to open.
    pub fn take_opened_structure_block(&mut self) -> Option<(i32, i32, i32)> {
        self.opened_structure_block.take()
    }

    /// The id of the command block minecart used, whose screen is to open.
    pub fn take_opened_command_minecart(&mut self) -> Option<photon_core::ids::EntityId> {
        self.opened_command_minecart.take()
    }

    pub fn take_opened_book(&mut self) -> Option<u8> {
        self.opened_book.take()
    }

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
            if self.click_entity(ctx, hand, aim, out) {
                return;
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

    /// `MultiPlayerGameMode.interact` for an aimed entity. Whether the use is over: the click fell
    /// outside the border, or the entity took it.
    fn click_entity(
        &mut self,
        ctx: &mut Context<'_>,
        hand: u8,
        aim: Aim,
        out: &mut Vec<Outgoing>,
    ) -> bool {
        let Aim::Entity {
            id,
            location,
            position,
            bounds,
        } = aim
        else {
            return false;
        };
        let block = position.map(|value| value.floor() as i32);
        if !ctx.within_border((block[0], block[1], block[2])) {
            return true;
        }
        if !ctx.within_entity_interaction_range(bounds) {
            return false;
        }
        // What the entity does on its own side of the click is not predicted: the click passes
        // on to the hand's item as if it had not been consumed.
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
        // `MinecartCommandBlock.interact`: a game master's click opens its screen, unless the
        // player only watches.
        let opens = ctx.mode != ClientPlayerGameMode::Spectator
            && ctx.can_use_game_master_blocks()
            && ctx.world.command_minecart_view(id).is_some();
        if opens {
            self.opened_command_minecart = Some(id);
            ctx.swing_hand(hand, out);
        }
        opens
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
            let flint = item.is_some_and(|(item, _)| {
                rules::item(item).is_some_and(|row| {
                    matches!(
                        row.class,
                        ItemClass::FlintAndSteelItem | ItemClass::FireChargeItem
                    )
                })
            });
            let item_use = match flint
                .then(|| super::tools::flint_over_lightable(state))
                .flatten()
                .map_or_else(|| rules::use_item_on(state), Rule::Known)
            {
                Rule::Known(result) => result,
                Rule::Unported => return UseResult::Success { swing: true },
            };
            if item_use.consumes_action() {
                return item_use;
            }
            if item_use == UseResult::TryEmptyHand && hand == MAIN_HAND {
                let used = self.use_without_item(ctx, state, (x, y, z));
                if used.consumes_action() {
                    return used;
                }
            }
        }
        match item {
            Some((item, _)) if !ctx.on_cooldown(hand) => self.item_use_on(ctx, hand, item, aim),
            _ => UseResult::Pass,
        }
    }

    /// The use of a structure, jigsaw or test block (`StructureBlockEntity.usedBy`,
    /// `JigsawBlock.useWithoutItem`, `TestBlock.useWithoutItem`): a game master opens the block's
    /// screen, anyone else passes. `None` for any other block.
    fn operator_block(
        &mut self,
        ctx: &Context<'_>,
        state: photon_core::ids::BlockStateId,
        pos: (i32, i32, i32),
    ) -> Option<UseResult> {
        let (present, opened) = match photon_world::block::state_or_air(state).block().name {
            "minecraft:structure_block" => (
                ctx.world.structure_block_view(pos).is_some(),
                &mut self.opened_structure_block,
            ),
            "minecraft:jigsaw" => (
                ctx.world.jigsaw_block_view(pos).is_some(),
                &mut self.opened_jigsaw_block,
            ),
            "minecraft:test_block" => (
                ctx.world.test_block_view(pos).is_some(),
                &mut self.opened_test_block,
            ),
            "minecraft:test_instance_block" => (
                ctx.world.test_instance_block_view(pos).is_some(),
                &mut self.opened_test_instance_block,
            ),
            _ => return None,
        };
        if ctx.can_use_game_master_blocks() && present {
            *opened = Some(pos);
            return Some(UseResult::Success { swing: true });
        }
        Some(UseResult::Pass)
    }

    /// `BlockBehaviour.useWithoutItem`; a command block opens its screen for a game master.
    fn use_without_item(
        &mut self,
        ctx: &Context<'_>,
        state: photon_core::ids::BlockStateId,
        pos: (i32, i32, i32),
    ) -> UseResult {
        if ctx.can_use_game_master_blocks() && ctx.world.command_block_view(pos).is_some() {
            self.opened_command_block = Some(pos);
            return UseResult::Success { swing: true };
        }
        if let Some(result) = self.operator_block(ctx, state, pos) {
            return result;
        }
        match rules::use_without_item(state) {
            Rule::Known(result) => result,
            Rule::Unported => UseResult::Success { swing: true },
        }
    }

    /// `ItemStack.useOn`, then `Item.useOn` as the item's class decides it.
    pub(super) fn item_use_on(
        &mut self,
        ctx: &mut Context<'_>,
        hand: u8,
        item: ItemId,
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
            class => super::tools::use_on(ctx, (hand, class), aim)
                .unwrap_or(UseResult::Success { swing: true }),
        }
    }

    /// `BlockItem.place` for a plain block item.
    pub(super) fn place(
        &mut self,
        ctx: &mut Context<'_>,
        hand: u8,
        item: ItemId,
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
            .and_then(photon_data::block::registry::block_index)
        else {
            return UseResult::Success { swing: true };
        };
        let world: &ClientWorld = ctx.world;
        let state_at = |x: i32, y: i32, z: i32| world.block_state_or_air(x, y, z);
        let (y_rot, _) = ctx.player.rotation();
        let mut place_ctx = rules::PlaceContext {
            state_at: &state_at,
            hit_pos: (x, y, z),
            face: photon_core::direction::Direction::from_3d_data_value(i32::from(face)),
            location: (location[0], location[1], location[2]),
            item: Some(item),
            horizontal_direction: photon_core::direction::Direction::from_y_rot(f64::from(y_rot)),
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
        let block_pos = photon_core::binary::BlockPos::new(pos.0, pos.1, pos.2);
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
            Some(_) if ctx.on_cooldown(hand) => UseResult::Pass,
            // `WritableBookItem.use`: its screen opens (`LocalPlayer.openItemGui`).
            Some((item, _)) if crate::item::id_of("minecraft:writable_book") == Some(item) => {
                self.opened_book = Some(hand);
                UseResult::Success { swing: true }
            }
            Some((item, _)) if rules::item_use_owner(item) == ItemClass::Item => {
                super::base_use::base_item_use(ctx, hand)
            }
            Some((item, _)) => {
                super::base_use::class_item_use(ctx, hand, rules::item_use_owner(item))
                    // The other classes that override `use` are not predicted yet.
                    .unwrap_or(UseResult::Success { swing: false })
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

    pub fn right_click_delay(&self) -> i32 {
        self.right_click_delay
    }

    /// One tick of the attack and use keys, in `Minecraft.tick`'s order:
    /// the right click delay counts down, then `handleKeybinds` spends the
    /// attack clicks, the use clicks, a held use key and a held attack key,
    /// then the miss wait counts down.
    pub fn keys_tick(&mut self, ctx: &mut Context<'_>, keys: Keys, aim: Aim) -> Vec<Outgoing> {
        let mut out = Vec::new();
        if self.right_click_delay > 0 {
            self.right_click_delay -= 1;
        }
        let instant_attack = self.spend_clicks(ctx, keys, aim, &mut out);
        if keys.use_held && self.right_click_delay == 0 && !ctx.player.is_using_item() {
            self.start_use_item(ctx, aim, &mut out);
        }
        self.continue_attack(ctx, keys.attack_held && !instant_attack, aim, &mut out);
        if self.miss_time > 0 {
            self.miss_time -= 1;
        }
        out
    }

    /// The clicks of one tick, whether an attack was instant. While an item is in use they are
    /// spent for nothing, and a use key that is up lets go of the item.
    fn spend_clicks(
        &mut self,
        ctx: &mut Context<'_>,
        keys: Keys,
        aim: Aim,
        out: &mut Vec<Outgoing>,
    ) -> bool {
        if ctx.player.is_using_item() {
            if !keys.use_held {
                Self::release_using_item(ctx, out);
            }
            return false;
        }
        let mut instant_attack = false;
        for _ in 0..keys.attack_clicks {
            instant_attack |= self.start_attack(ctx, aim, out);
        }
        for _ in 0..keys.use_clicks {
            self.start_use_item(ctx, aim, out);
        }
        for _ in 0..keys.pick_clicks {
            Self::pick_block_or_entity(aim, keys.control_down, out);
        }
        instant_attack
    }

    /// `MultiPlayerGameMode.releaseUsingItem`: the server hears the release, and the use ends.
    fn release_using_item(ctx: &mut Context<'_>, out: &mut Vec<Outgoing>) {
        out.push(Outgoing::PlayerAction {
            action: PlayerAction::ReleaseUseItem,
            x: 0,
            y: 0,
            z: 0,
            face: 0,
            sequence: 0,
        });
        // `SpyglassItem.releaseUsing`.
        if ctx.player.is_scoping() {
            ctx.player
                .play_own_sound("minecraft:item.spyglass.stop_using", (1.0, 1.0));
        }
        ctx.player.release_using_item();
    }

    /// `Minecraft.pickBlockOrEntity`: the server is asked for the aimed
    /// block's or entity's item; a miss asks for nothing.
    pub(super) fn pick_block_or_entity(aim: Aim, include_data: bool, out: &mut Vec<Outgoing>) {
        match aim {
            Aim::Block { x, y, z, .. } => out.push(Outgoing::PickItemFromBlock {
                x,
                y,
                z,
                include_data,
            }),
            Aim::Entity { id, .. } => out.push(Outgoing::PickItemFromEntity {
                entity_id: id,
                include_data,
            }),
            Aim::Miss => {}
        }
    }

    /// `startPrediction`: the action runs under a new sequence number, which
    /// the packet it returns carries.
    pub(super) fn predict(
        &mut self,
        ctx: &mut Context<'_>,
        out: &mut Vec<Outgoing>,
        action: impl FnOnce(&mut Self, &mut Context<'_>, i32) -> Outgoing,
    ) {
        let sequence = ctx.world.start_predicting();
        let packet = action(self, ctx, sequence);
        out.push(packet);
        ctx.world.stop_predicting();
    }

    pub(super) fn action(
        action: PlayerAction,
        (x, y, z): (i32, i32, i32),
        face: u8,
        sequence: i32,
    ) -> Outgoing {
        Outgoing::PlayerAction {
            action,
            x,
            y,
            z,
            face,
            sequence,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// With a control key down the pick asks for the data too; the oracle
    /// cannot hold a key in the reference's window, so the byte is read here.
    #[test]
    fn control_asks_the_pick_for_data() {
        let mut out = Vec::new();
        GameMode::pick_block_or_entity(
            Aim::Entity {
                id: photon_core::ids::EntityId(20),
                location: [0.0; 3],
                position: [0.0; 3],
                bounds: [0.0; 6],
            },
            true,
            &mut out,
        );
        GameMode::pick_block_or_entity(Aim::Miss, true, &mut out);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].encode_body(), vec![20, 1]);
    }
}
