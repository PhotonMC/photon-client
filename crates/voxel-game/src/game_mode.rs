//! Breaking blocks: `MultiPlayerGameMode`'s destroy state and the attack key
//! as `Minecraft.startAttack` and `continueAttack` spend it.
//!
//! Each tick the loop first spends the attack clicks, each starting an
//! attack on what the player aims at, then lets the held key keep breaking
//! the aimed block, then counts the miss wait down. Breaking accumulates the
//! block's destroy progress every tick; a finished block, and every block in
//! creative, is removed at once as a prediction the server confirms or
//! reverts. The packets the reference would send come back in order.
//!
//! Not ported: block particles and hit sounds, `BlockState.attack` and
//! `playerWillDestroy` side effects, neighbour shape updates after a
//! predicted break, entity targets, piercing weapons, item-specific attack
//! charge, adventure mode `can_break` predicates, and game master blocks for
//! operators (the permission level is not tracked; they are never broken).

use voxel_core::math::Vec3;
use voxel_data::block::destroy as block_destroy_table;
use voxel_world::block_rules::{self as rules, Rule, UseResult};
use voxel_world::local_player::LocalPlayer;
use voxel_world::{ClientItemStackState, ClientPlayerGameMode, ClientWorld};

/// What the player aims at when the tick's keys are handled.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Aim {
    /// Nothing within reach.
    Miss,
    /// A block, the face the aim enters it through (as the protocol numbers
    /// directions), where on it the aim lands, and whether it started inside.
    Block {
        x: i32,
        y: i32,
        z: i32,
        face: u8,
        location: [f64; 3],
        inside: bool,
    },
}

/// The attack and use keys for one tick: presses since the last tick and
/// whether each is held with the mouse captured.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Keys {
    pub attack_clicks: u32,
    pub attack_held: bool,
    pub use_clicks: u32,
    pub use_held: bool,
}

pub const MAIN_HAND: u8 = 0;
pub const OFF_HAND: u8 = 1;

/// A packet the game mode sends, in order.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Outgoing {
    /// `ServerboundPlayerActionPacket`.
    PlayerAction {
        action: PlayerAction,
        x: i32,
        y: i32,
        z: i32,
        face: u8,
        sequence: i32,
    },
    /// `ServerboundSwingPacket`.
    Swing { hand: u8 },
    /// `ServerboundUseItemOnPacket`: the hit relative to the block.
    UseItemOn {
        hand: u8,
        x: i32,
        y: i32,
        z: i32,
        face: u8,
        cursor: [f32; 3],
        inside: bool,
        sequence: i32,
    },
    /// `ServerboundUseItemPacket`.
    UseItem {
        hand: u8,
        sequence: i32,
        y_rot: f32,
        x_rot: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerAction {
    StartDestroyBlock = 0,
    AbortDestroyBlock = 1,
    StopDestroyBlock = 2,
}

/// What breaking reads of the player besides its movement state: the held
/// item, the effects and the attributes. Taken before a tick's attack.
#[derive(Debug, Clone, Default)]
pub struct Miner {
    /// The selected hotbar stack; `None` is empty.
    pub main_hand: Option<ClientItemStackState>,
    /// The greater of the haste and conduit power amplifiers, if either.
    pub dig_speed_amplifier: Option<i32>,
    pub mining_fatigue_amplifier: Option<i32>,
    pub mining_efficiency: f64,
    pub block_break_speed: f64,
    pub submerged_mining_speed: f64,
}

const HASTE: i32 = 2;
const MINING_FATIGUE: i32 = 3;
const CONDUIT_POWER: i32 = 28;
const BLOCK_BREAK_SPEED: i32 = 7;
const MINING_EFFICIENCY: i32 = 24;
const SUBMERGED_MINING_SPEED: i32 = 34;
const DIRECTION_DOWN: u8 = 0;

impl Miner {
    /// The local player's held item, effects and attributes as the world
    /// last heard them, with the attributes' defaults when unsent.
    pub fn of(world: &ClientWorld, selected_slot: i32) -> Self {
        let Some(state) = world.local_player() else {
            return Self::with_defaults(None);
        };
        let main_hand = state.inventory_slot((36 + selected_slot) as i16).cloned();
        let amplifier = |id: i32| {
            state
                .effects()
                .iter()
                .find(|effect| effect.effect_id() == id)
                .map(|effect| effect.amplifier())
        };
        let attribute = |id: i32, default: f64, max: f64| {
            state
                .attributes()
                .iter()
                .find(|attribute| attribute.attribute_id() == id)
                .map_or(default, |attribute| attribute.effective_value(0.0, max))
        };
        let haste = amplifier(HASTE);
        let conduit = amplifier(CONDUIT_POWER);
        Self {
            main_hand,
            dig_speed_amplifier: match (haste, conduit) {
                (None, None) => None,
                (a, b) => Some(a.unwrap_or(0).max(b.unwrap_or(0))),
            },
            mining_fatigue_amplifier: amplifier(MINING_FATIGUE),
            mining_efficiency: attribute(MINING_EFFICIENCY, 0.0, 1024.0),
            block_break_speed: attribute(BLOCK_BREAK_SPEED, 1.0, 1024.0),
            submerged_mining_speed: attribute(SUBMERGED_MINING_SPEED, 0.2, 20.0),
        }
    }

    /// A player with the attributes' defaults and no effects.
    pub fn with_defaults(main_hand: Option<ClientItemStackState>) -> Self {
        Self {
            main_hand,
            dig_speed_amplifier: None,
            mining_fatigue_amplifier: None,
            mining_efficiency: 0.0,
            block_break_speed: 1.0,
            submerged_mining_speed: 0.2,
        }
    }

    fn tool(&self) -> Option<&'static voxel_data::item::registry::Tool> {
        self.main_hand
            .as_ref()
            .and_then(|stack| crate::item::definition(stack.item_registry_id()))
            .and_then(|item| item.tool)
    }

    /// `Player.getDestroySpeed`.
    fn destroy_speed(&self, world: &ClientWorld, player: &LocalPlayer, state: i32) -> f32 {
        let registry = world.block_state_registry();
        let mut speed = self
            .tool()
            .map_or(1.0, |tool| crate::item::mining_speed(tool, registry, state));
        if speed > 1.0 {
            speed += self.mining_efficiency as f32;
        }
        if let Some(amplifier) = self.dig_speed_amplifier {
            speed *= 1.0 + (amplifier + 1) as f32 * 0.2;
        }
        if let Some(amplifier) = self.mining_fatigue_amplifier {
            speed *= match amplifier {
                0 => 0.3,
                1 => 0.09,
                2 => 0.0027,
                _ => 8.1e-4,
            };
        }
        speed *= self.block_break_speed as f32;
        if player.is_eye_in_water() {
            speed *= self.submerged_mining_speed as f32;
        }
        if !player.on_ground() {
            speed /= 5.0;
        }
        speed
    }

    /// `Player.hasCorrectToolForDrops`.
    fn has_correct_tool(&self, world: &ClientWorld, state: i32) -> bool {
        !traits(state).requires_correct_tool
            || self.tool().is_some_and(|tool| {
                crate::item::is_correct_for_drops(tool, world.block_state_registry(), state)
            })
    }

    /// `BlockBehaviour.getDestroyProgress`: how much one tick of breaking
    /// does.
    pub fn destroy_progress(&self, world: &ClientWorld, player: &LocalPlayer, state: i32) -> f32 {
        let destroy_time = f32::from_bits(traits(state).destroy_time);
        if destroy_time == -1.0 {
            return 0.0;
        }
        let modifier = if self.has_correct_tool(world, state) {
            30
        } else {
            100
        };
        self.destroy_speed(world, player, state) / destroy_time / modifier as f32
    }
}

fn traits(state: i32) -> &'static block_destroy_table::DestroyTraits {
    let (index, offset) =
        voxel_data::block::registry::block_index_for_state(state as u32).unwrap_or((0, 0));
    let states = block_destroy_table::BLOCKS[index];
    let id = states[if states.len() == 1 {
        0
    } else {
        offset as usize
    }];
    &block_destroy_table::TRAITS[usize::from(id)]
}

fn is_air(state: i32) -> bool {
    traits(state).air
}

fn is_game_master_block(state: i32) -> bool {
    voxel_data::block::registry::block_index_for_state(state as u32)
        .is_some_and(|(index, _)| block_destroy_table::GAME_MASTER.contains(&(index as u16)))
}

/// `MultiPlayerGameMode`'s breaking state and `Minecraft.missTime`.
#[derive(Debug, Clone)]
pub struct GameMode {
    destroy_pos: (i32, i32, i32),
    destroying_item: Option<ClientItemStackState>,
    destroy_progress: f32,
    destroy_ticks: f32,
    destroy_delay: i32,
    is_destroying: bool,
    miss_time: i32,
    /// `Minecraft.rightClickDelay`.
    right_click_delay: i32,
    /// Hands whose stack was used since the caller last asked:
    /// `ItemInHandRenderer.itemUsed`.
    items_used: Vec<u8>,
}

impl GameMode {
    /// The hands whose stack was used since the last call, in order.
    pub fn take_items_used(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.items_used)
    }
}

impl Default for GameMode {
    fn default() -> Self {
        Self {
            destroy_pos: (-1, -1, -1),
            destroying_item: None,
            destroy_progress: 0.0,
            destroy_ticks: 0.0,
            destroy_delay: 0,
            is_destroying: false,
            miss_time: 0,
            right_click_delay: 0,
            items_used: Vec::new(),
        }
    }
}

/// Everything one tick of breaking works on.
pub struct Context<'a> {
    pub world: &'a mut ClientWorld,
    pub player: &'a mut LocalPlayer,
    pub miner: &'a Miner,
    pub mode: ClientPlayerGameMode,
    /// The selected hotbar slot, 0 to 8.
    pub selected_slot: i32,
}

impl Context<'_> {
    fn state_at(&self, (x, y, z): (i32, i32, i32)) -> i32 {
        self.world.block_state_at(x, y, z).unwrap_or(0)
    }

    fn instabuild(&self) -> bool {
        self.player.abilities().instabuild()
    }

    fn within_border(&self, (x, _, z): (i32, i32, i32)) -> bool {
        let border = self.world.world_border();
        let (x, z) = (f64::from(x), f64::from(z));
        x >= border.min_x(0.0)
            && x < border.max_x(0.0)
            && z >= border.min_z(0.0)
            && z < border.max_z(0.0)
    }

    /// `Player.blockActionRestricted`: spectators cannot break, and in
    /// adventure mode nothing can be broken without a `can_break` predicate,
    /// which is not evaluated.
    fn block_action_restricted(&self) -> bool {
        matches!(
            self.mode,
            ClientPlayerGameMode::Adventure | ClientPlayerGameMode::Spectator
        )
    }

    fn swing(&mut self, out: &mut Vec<Outgoing>) {
        self.swing_hand(MAIN_HAND, out);
    }

    /// `LocalPlayer.swing`: the arm swings and the server hears of it. The
    /// off hand's arm is not animated yet.
    fn swing_hand(&mut self, hand: u8, out: &mut Vec<Outgoing>) {
        if hand == MAIN_HAND {
            self.player.swing();
        }
        out.push(Outgoing::Swing { hand });
    }

    /// The inventory menu slot a hand holds.
    fn hand_slot(&self, hand: u8) -> i16 {
        if hand == MAIN_HAND {
            (36 + self.selected_slot) as i16
        } else {
            45
        }
    }

    /// `getItemInHand`: the stack's item and count, or nothing.
    fn held(&self, hand: u8) -> Option<(i32, i32)> {
        self.world
            .local_player()
            .and_then(|state| state.inventory_slot(self.hand_slot(hand)))
            .map(|stack| (stack.item_registry_id(), stack.item_count()))
            .filter(|(_, count)| *count > 0)
    }
}

impl GameMode {
    pub fn is_destroying(&self) -> bool {
        self.is_destroying
    }

    pub fn destroy_progress(&self) -> f32 {
        self.destroy_progress
    }

    pub fn destroy_ticks(&self) -> f32 {
        self.destroy_ticks
    }

    pub fn destroy_delay(&self) -> i32 {
        self.destroy_delay
    }

    pub fn miss_time(&self) -> i32 {
        self.miss_time
    }

    /// `getDestroyStage`: the crack from 0 to 9, or -1 before any progress.
    pub fn destroy_stage(&self) -> i32 {
        if self.destroy_progress > 0.0 {
            (self.destroy_progress * 10.0) as i32
        } else {
            -1
        }
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
        let mut instant_attack = false;
        for _ in 0..keys.attack_clicks {
            instant_attack |= self.start_attack(ctx, aim, &mut out);
        }
        for _ in 0..keys.use_clicks {
            self.start_use_item(ctx, aim, &mut out);
        }
        if keys.use_held && self.right_click_delay == 0 {
            self.start_use_item(ctx, aim, &mut out);
        }
        self.continue_attack(ctx, keys.attack_held && !instant_attack, aim, &mut out);
        if self.miss_time > 0 {
            self.miss_time -= 1;
        }
        out
    }

    /// `Minecraft.startAttack`; whether the attack ended the key's hold.
    fn start_attack(&mut self, ctx: &mut Context<'_>, aim: Aim, out: &mut Vec<Outgoing>) -> bool {
        if self.miss_time > 0 {
            return false;
        }
        if ctx.mode == ClientPlayerGameMode::Spectator {
            // `spectatorNoAction`: nothing to send without an entity.
            return true;
        }
        let mut end_attack = false;
        let aimed_block = match aim {
            Aim::Block { x, y, z, face, .. } if !is_air(ctx.state_at((x, y, z))) => {
                Some((x, y, z, face))
            }
            _ => None,
        };
        match aimed_block {
            Some((x, y, z, face)) => {
                self.start_destroy_block(ctx, (x, y, z), face, out);
                if is_air(ctx.state_at((x, y, z))) {
                    end_attack = true;
                }
            }
            None => {
                if ctx.mode != ClientPlayerGameMode::Creative {
                    self.miss_time = 10;
                }
                ctx.player.reset_attack_strength();
            }
        }
        ctx.swing(out);
        end_attack
    }

    /// `Minecraft.continueAttack`.
    fn continue_attack(
        &mut self,
        ctx: &mut Context<'_>,
        down: bool,
        aim: Aim,
        out: &mut Vec<Outgoing>,
    ) {
        if !down {
            self.miss_time = 0;
        }
        if self.miss_time > 0 {
            return;
        }
        match aim {
            Aim::Block { x, y, z, face, .. } if down => {
                if !is_air(ctx.state_at((x, y, z)))
                    && self.continue_destroy_block(ctx, (x, y, z), face, out)
                {
                    ctx.swing(out);
                }
            }
            _ => self.stop_destroy_block(ctx, out),
        }
    }

    fn same_destroy_target(&self, ctx: &Context<'_>, pos: (i32, i32, i32)) -> bool {
        let same_item = match (&ctx.miner.main_hand, &self.destroying_item) {
            (None, None) => true,
            (Some(held), Some(destroying)) => held.is_same_item_same_components(destroying),
            _ => false,
        };
        pos == self.destroy_pos && same_item
    }

    /// `startPrediction`: the action runs under a new sequence number, which
    /// the packet it returns carries.
    fn predict(
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

    fn action(
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

    fn crack(&self, ctx: &mut Context<'_>) {
        let (x, y, z) = self.destroy_pos;
        let id = ctx.player.entity_id();
        ctx.world
            .destroy_block_progress(id, x, y, z, self.destroy_stage());
    }

    /// `startDestroyBlock`.
    fn start_destroy_block(
        &mut self,
        ctx: &mut Context<'_>,
        pos: (i32, i32, i32),
        face: u8,
        out: &mut Vec<Outgoing>,
    ) -> bool {
        if ctx.block_action_restricted() || !ctx.within_border(pos) {
            return false;
        }
        if ctx.instabuild() {
            self.predict(ctx, out, |_, ctx, sequence| {
                destroy_block(ctx, pos);
                Self::action(PlayerAction::StartDestroyBlock, pos, face, sequence)
            });
            self.destroy_delay = 5;
        } else if !self.is_destroying || !self.same_destroy_target(ctx, pos) {
            if self.is_destroying {
                out.push(Self::action(
                    PlayerAction::AbortDestroyBlock,
                    self.destroy_pos,
                    face,
                    0,
                ));
            }
            let state = ctx.state_at(pos);
            self.predict(ctx, out, |game_mode, ctx, sequence| {
                let not_air = !is_air(state);
                if not_air && ctx.miner.destroy_progress(ctx.world, ctx.player, state) >= 1.0 {
                    destroy_block(ctx, pos);
                } else {
                    game_mode.is_destroying = true;
                    game_mode.destroy_pos = pos;
                    game_mode.destroying_item = ctx.miner.main_hand.clone();
                    game_mode.destroy_progress = 0.0;
                    game_mode.destroy_ticks = 0.0;
                    game_mode.crack(ctx);
                }
                Self::action(PlayerAction::StartDestroyBlock, pos, face, sequence)
            });
        }
        true
    }

    /// `stopDestroyBlock`: letting go abandons the block.
    fn stop_destroy_block(&mut self, ctx: &mut Context<'_>, out: &mut Vec<Outgoing>) {
        if !self.is_destroying {
            return;
        }
        out.push(Self::action(
            PlayerAction::AbortDestroyBlock,
            self.destroy_pos,
            DIRECTION_DOWN,
            0,
        ));
        self.is_destroying = false;
        self.destroy_progress = 0.0;
        let (x, y, z) = self.destroy_pos;
        let id = ctx.player.entity_id();
        ctx.world.destroy_block_progress(id, x, y, z, -1);
        ctx.player.reset_attack_strength();
    }

    /// `continueDestroyBlock`.
    fn continue_destroy_block(
        &mut self,
        ctx: &mut Context<'_>,
        pos: (i32, i32, i32),
        face: u8,
        out: &mut Vec<Outgoing>,
    ) -> bool {
        if self.destroy_delay > 0 {
            self.destroy_delay -= 1;
            return true;
        }
        if ctx.instabuild() && ctx.within_border(pos) {
            self.destroy_delay = 5;
            self.predict(ctx, out, |_, ctx, sequence| {
                destroy_block(ctx, pos);
                Self::action(PlayerAction::StartDestroyBlock, pos, face, sequence)
            });
            return true;
        }
        if !self.same_destroy_target(ctx, pos) {
            return self.start_destroy_block(ctx, pos, face, out);
        }
        let state = ctx.state_at(pos);
        if is_air(state) {
            self.is_destroying = false;
            return false;
        }
        self.destroy_progress += ctx.miner.destroy_progress(ctx.world, ctx.player, state);
        self.destroy_ticks += 1.0;
        if self.destroy_progress >= 1.0 {
            self.is_destroying = false;
            self.predict(ctx, out, |_, ctx, sequence| {
                destroy_block(ctx, pos);
                Self::action(PlayerAction::StopDestroyBlock, pos, face, sequence)
            });
            self.destroy_progress = 0.0;
            self.destroy_ticks = 0.0;
            self.destroy_delay = 5;
        }
        self.crack(ctx);
        true
    }
}

/// `destroyBlock`: the block becomes the fluid it held, as a prediction.
fn destroy_block(ctx: &mut Context<'_>, pos: (i32, i32, i32)) -> bool {
    if ctx.block_action_restricted() {
        return false;
    }
    let old = ctx.state_at(pos);
    // `Item.canDestroyBlock`: some tools cannot break blocks in creative.
    let tool = ctx
        .miner
        .main_hand
        .as_ref()
        .and_then(|stack| crate::item::definition(stack.item_registry_id()))
        .and_then(|item| item.tool);
    if tool.is_some_and(|tool| !tool.can_destroy_blocks_in_creative) && ctx.instabuild() {
        return false;
    }
    if is_game_master_block(old) || is_air(old) {
        return false;
    }
    let fluid = traits(old).fluid_block as i32;
    let position: Vec3 = ctx.player.position();
    ctx.world
        .set_block_with_updates(pos, fluid, 11, 512, position)
}

impl GameMode {
    /// `Minecraft.startUseItem`: each hand in turn tries the aimed block,
    /// then its item on its own, until one does something.
    fn start_use_item(&mut self, ctx: &mut Context<'_>, aim: Aim, out: &mut Vec<Outgoing>) {
        if self.is_destroying {
            return;
        }
        self.right_click_delay = 4;
        for hand in [MAIN_HAND, OFF_HAND] {
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
    fn use_item_on(
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
    fn perform_use_item_on(&mut self, ctx: &mut Context<'_>, hand: u8, aim: Aim) -> UseResult {
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
    fn item_use_on(&mut self, ctx: &mut Context<'_>, hand: u8, item: i32, aim: Aim) -> UseResult {
        // A player who may not build passes, unless the stack's `can_place_on`
        // allows the block; that predicate is not evaluated.
        if ctx.mode == ClientPlayerGameMode::Adventure {
            return UseResult::Pass;
        }
        match rules::item_use_on_class(item) {
            "Item" => UseResult::Pass,
            "BlockItem" if rules::item_class(item) == "BlockItem" => {
                self.place(ctx, hand, item, aim)
            }
            _ => UseResult::Success { swing: true },
        }
    }

    /// `BlockItem.place` for a plain block item.
    fn place(&mut self, ctx: &mut Context<'_>, hand: u8, item: i32, aim: Aim) -> UseResult {
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
        let Some(block) = rules::item_behaviour(item).and_then(|row| row.block) else {
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
        let state = match rules::state_for_placement(usize::from(block), &place_ctx) {
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
        // `ItemStack.consume`: used up unless materials are infinite.
        if !ctx.instabuild() {
            let slot = ctx.hand_slot(hand);
            ctx.world.shrink_local_player_slot(slot, 1);
        }
        UseResult::Success { swing: true }
    }

    /// `MultiPlayerGameMode.useItem`: the packet goes whatever the item
    /// does. `None` is a spectator's use, which sends nothing.
    fn use_item(
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
                let row = rules::item_behaviour(item);
                if rules::item_use_class(item) == "Item"
                    && row.is_some_and(|row| row.default_use_pass)
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

impl Outgoing {
    /// The packet's type, as the protocol names it.
    pub fn type_name(&self) -> &'static str {
        match self {
            Outgoing::PlayerAction { .. } => "minecraft:player_action",
            Outgoing::Swing { .. } => "minecraft:swing",
            Outgoing::UseItemOn { .. } => "minecraft:use_item_on",
            Outgoing::UseItem { .. } => "minecraft:use_item",
        }
    }

    /// The packet body as its codec writes it, without the packet id.
    pub fn encode_body(&self) -> Vec<u8> {
        let mut out = Vec::new();
        match *self {
            Outgoing::PlayerAction {
                action,
                x,
                y,
                z,
                face,
                sequence,
            } => {
                voxel_core::wire::write_varint(action as i32, &mut out);
                out.extend_from_slice(
                    &voxel_core::binary::BlockPos::new(x, y, z)
                        .as_long()
                        .to_be_bytes(),
                );
                out.push(face);
                voxel_core::wire::write_varint(sequence, &mut out);
            }
            Outgoing::Swing { hand } => voxel_core::wire::write_varint(i32::from(hand), &mut out),
            Outgoing::UseItemOn {
                hand,
                x,
                y,
                z,
                face,
                cursor,
                inside,
                sequence,
            } => {
                voxel_core::wire::write_varint(i32::from(hand), &mut out);
                out.extend_from_slice(
                    &voxel_core::binary::BlockPos::new(x, y, z)
                        .as_long()
                        .to_be_bytes(),
                );
                voxel_core::wire::write_varint(i32::from(face), &mut out);
                for value in cursor {
                    out.extend_from_slice(&value.to_be_bytes());
                }
                out.push(u8::from(inside));
                // Never a world border hit: the pick does not aim at it.
                out.push(0);
                voxel_core::wire::write_varint(sequence, &mut out);
            }
            Outgoing::UseItem {
                hand,
                sequence,
                y_rot,
                x_rot,
            } => {
                voxel_core::wire::write_varint(i32::from(hand), &mut out);
                voxel_core::wire::write_varint(sequence, &mut out);
                out.extend_from_slice(&y_rot.to_be_bytes());
                out.extend_from_slice(&x_rot.to_be_bytes());
            }
        }
        out
    }
}
