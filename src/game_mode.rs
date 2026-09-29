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

use crate::math::Vec3;
use crate::world::local_player::LocalPlayer;
use crate::world::{ClientItemStackState, ClientPlayerGameMode, ClientWorld, block_destroy_table};

/// What the player aims at when the tick's keys are handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Aim {
    /// Nothing within reach.
    Miss,
    /// A block and the face the aim enters it through, as the protocol
    /// numbers directions.
    Block { x: i32, y: i32, z: i32, face: u8 },
}

/// A packet the game mode sends, in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    /// `ServerboundSwingPacket` for the main hand.
    Swing,
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

    fn tool(&self) -> Option<&'static crate::item_table::Tool> {
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
            .map_or(1.0, |tool| tool.mining_speed(registry, state));
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
                tool.is_correct_for_drops(world.block_state_registry(), state)
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
    let (index, offset) = crate::world::block_registry::block_index_for_state(state as u32)
        .unwrap_or((0, 0));
    let states = block_destroy_table::BLOCKS[index];
    let id = states[if states.len() == 1 { 0 } else { offset as usize }];
    &block_destroy_table::TRAITS[usize::from(id)]
}

fn is_air(state: i32) -> bool {
    traits(state).air
}

fn is_game_master_block(state: i32) -> bool {
    crate::world::block_registry::block_index_for_state(state as u32).is_some_and(|(index, _)| {
        block_destroy_table::GAME_MASTER.contains(&(index as u16))
    })
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
        }
    }
}

/// Everything one tick of breaking works on.
pub struct Context<'a> {
    pub world: &'a mut ClientWorld,
    pub player: &'a mut LocalPlayer,
    pub miner: &'a Miner,
    pub mode: ClientPlayerGameMode,
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
        x >= border.min_x(0.0) && x < border.max_x(0.0) && z >= border.min_z(0.0) && z < border.max_z(0.0)
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
        self.player.swing();
        out.push(Outgoing::Swing);
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

    /// One tick of the attack key: `clicks` presses since the last tick,
    /// then whether the key is held with the mouse captured.
    pub fn attack_tick(
        &mut self,
        ctx: &mut Context<'_>,
        clicks: u32,
        held: bool,
        aim: Aim,
    ) -> Vec<Outgoing> {
        let mut out = Vec::new();
        let mut instant_attack = false;
        for _ in 0..clicks {
            instant_attack |= self.start_attack(ctx, aim, &mut out);
        }
        self.continue_attack(ctx, held && !instant_attack, aim, &mut out);
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
            Aim::Block { x, y, z, face } if !is_air(ctx.state_at((x, y, z))) => Some((x, y, z, face)),
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
    fn continue_attack(&mut self, ctx: &mut Context<'_>, down: bool, aim: Aim, out: &mut Vec<Outgoing>) {
        if !down {
            self.miss_time = 0;
        }
        if self.miss_time > 0 {
            return;
        }
        match aim {
            Aim::Block { x, y, z, face } if down => {
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

    fn action(action: PlayerAction, (x, y, z): (i32, i32, i32), face: u8, sequence: i32) -> Outgoing {
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
        ctx.world.destroy_block_progress(id, x, y, z, self.destroy_stage());
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
    let (x, y, z) = pos;
    ctx.world.set_block_predicted(x, y, z, fluid, position)
}
