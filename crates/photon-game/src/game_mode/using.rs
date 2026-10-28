use super::entity_use::EntityUse;
use super::*;
use photon_core::ids::ItemId;
use photon_world::local_player::{LocalSound, LocalSoundSource};

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
        if ctx.mode == ClientPlayerGameMode::Spectator {
            return false;
        }
        let opens =
            ctx.can_use_game_master_blocks() && ctx.world.command_minecart_view(id).is_some();
        if opens {
            self.opened_command_minecart = Some(id);
            ctx.swing_hand(hand, out);
            return true;
        }
        match ctx.interact_entity(id, hand) {
            EntityUse::Success { swing } => {
                if swing {
                    ctx.swing_hand(hand, out);
                }
                true
            }
            EntityUse::Pass => false,
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
            let Rule::Known(item_use) = Self::item_on_result(ctx, hand, state, (x, y, z), aim)
            else {
                return UseResult::Success { swing: true };
            };
            if item_use.consumes_action() {
                return item_use;
            }
            if item_use == UseResult::TryEmptyHand && hand == MAIN_HAND {
                let used = self.use_without_item(ctx, state, (x, y, z), aim);
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
        ctx: &mut Context<'_>,
        state: photon_core::ids::BlockStateId,
        pos: (i32, i32, i32),
        aim: Aim,
    ) -> UseResult {
        if ctx.can_use_game_master_blocks() && ctx.world.command_block_view(pos).is_some() {
            self.opened_command_block = Some(pos);
            return UseResult::Success { swing: true };
        }
        if let Some(result) = self.operator_block(ctx, state, pos) {
            return result;
        }
        if let Some(by_hand) = super::placing::redstone_wire_use(ctx, state, pos) {
            return Self::by_hand(ctx, by_hand);
        }
        if let Rule::Known(by_hand) = rules::use_by_hand(state, pos, user_of(ctx, aim)) {
            return Self::by_hand(ctx, by_hand);
        }
        match rules::use_without_item(state) {
            Rule::Known(result) => result,
            Rule::Unported => UseResult::Success { swing: true },
        }
    }

    /// `FoodData.eat(nutrition, saturation)` on the player's own food.
    fn eat_food(ctx: &mut Context<'_>, nutrition: i32, saturation: f32) {
        let Some(state) = ctx.world.local_player() else {
            return;
        };
        let food = (state.food() + nutrition).clamp(0, 20);
        let saturation =
            (state.saturation() + nutrition as f32 * saturation * 2.0).clamp(0.0, food as f32);
        let health = state.health();
        ctx.world
            .apply_local_player_health(health, food, saturation);
    }

    /// A block used by hand that the client runs itself: the block it sets, and the sound the
    /// server leaves the user out of.
    fn by_hand(ctx: &mut Context<'_>, by_hand: rules::HandUse) -> UseResult {
        let opened = match by_hand {
            rules::HandUse::Result(result) => return result,
            rules::HandUse::Set(opened) => opened,
        };
        let write = opened.write;
        if opened.eats {
            Self::eat_food(ctx, 2, 0.1);
        }
        if let Some(item) = opened.gives.and_then(crate::item::id_of) {
            super::buckets::add_to_inventory(ctx, ClientItemStackState::new(1, item));
        }
        let position = ctx.player.position();
        ctx.world
            .set_block_with_updates(write.pos, write.state, write.flags, 512, position);
        if let Some(sound) = opened.sound {
            let pitch = sound
                .pitch
                .unwrap_or_else(|| ctx.world.block_use_sound_pitch());
            let at = (
                f64::from(write.pos.0) + 0.5,
                f64::from(write.pos.1) + 0.5,
                f64::from(write.pos.2) + 0.5,
            );
            ctx.world.push_sound(LocalSound::at(
                sound.event,
                LocalSoundSource::Blocks,
                (sound.volume, pitch),
                at,
            ));
        }
        UseResult::Success { swing: true }
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
        // allows the block.
        if ctx.mode == ClientPlayerGameMode::Adventure && !ctx.can_place_on(hand, aim) {
            return UseResult::Pass;
        }
        match rules::item_use_on_owner(item) {
            ItemClass::Item => UseResult::Pass,
            // `PlaceOnWaterBlockItem.useOn` passes: the item is placed by its `use`.
            ItemClass::PlaceOnWaterBlockItem => UseResult::Pass,
            ItemClass::BlockItem | ItemClass::SolidBucketItem
                if rules::item_placement(item).is_some() =>
            {
                super::placing::place(ctx, hand, item, aim)
            }
            class => super::tools::use_on(ctx, (hand, class), aim)
                .unwrap_or(UseResult::Success { swing: true }),
        }
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
        let before = ctx
            .world
            .local_player()
            .and_then(|state| state.inventory_slot(ctx.hand_slot(hand)))
            .cloned();
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
                    // Only a class added by a later version lands here.
                    .unwrap_or(UseResult::Success { swing: false })
            }
            None => UseResult::Pass,
        };
        super::base_use::after_instant_use(ctx, hand, before.as_ref(), result);
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
        for _ in 0..keys.drop_clicks {
            super::dropping::drop_selected(ctx, keys.control_down, &mut out);
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
        super::utility::trident_release(ctx);
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

/// Who uses a block by hand, and where the click landed.
fn user_of(ctx: &Context<'_>, aim: Aim) -> rules::User {
    let Aim::Block {
        x,
        y,
        z,
        face,
        location,
        ..
    } = aim
    else {
        unreachable!("a block is used from an aim at a block");
    };
    rules::User {
        facing: photon_core::direction::Direction::from_y_rot(f64::from(ctx.player.rotation().0)),
        may_build: ctx.mode != ClientPlayerGameMode::Adventure,
        face: photon_core::direction::Direction::from_3d_data_value(i32::from(face)),
        click_y: location[1] - f64::from(y),
        location,
        can_eat: ctx.player.abilities().invulnerable()
            || ctx
                .world
                .local_player()
                .is_some_and(|state| state.food() < 20),
        main_hand_empty: ctx.held(MAIN_HAND).is_none(),
        block_entity: ctx.world.block_entity_at(x, y, z).is_some(),
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
