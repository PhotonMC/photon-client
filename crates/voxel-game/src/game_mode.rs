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
use voxel_data::item::classes::ItemClass;
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
    /// An entity, where the aim lands on it, where the entity stands, and its
    /// box (min then max corner).
    Entity {
        id: i32,
        location: [f64; 3],
        position: [f64; 3],
        bounds: [f64; 6],
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
    /// Presses of the pick item key (the middle button).
    pub pick_clicks: u32,
    /// `Minecraft.hasControlDown`: either control key, which asks for the
    /// picked block's or entity's data as well.
    pub control_down: bool,
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
    /// `ServerboundAttackPacket`.
    Attack { entity_id: i32 },
    /// `ServerboundPickItemFromBlockPacket`.
    PickItemFromBlock {
        x: i32,
        y: i32,
        z: i32,
        include_data: bool,
    },
    /// `ServerboundPickItemFromEntityPacket`.
    PickItemFromEntity { entity_id: i32, include_data: bool },
    /// `ServerboundSpectatorActionPacket`.
    SpectatorAction { entity_id: Option<i32> },
    /// `ServerboundInteractPacket`: the hit relative to the entity, and
    /// whether the secondary use (sneaking) is active.
    Interact {
        entity_id: i32,
        hand: u8,
        location: [f64; 3],
        secondary_use: bool,
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
    /// `Attributes.ENTITY_INTERACTION_RANGE`.
    pub entity_interaction_range: f64,
    /// `Attributes.BLOCK_INTERACTION_RANGE`.
    pub block_interaction_range: f64,
}

const HASTE: i32 = 2;
const MINING_FATIGUE: i32 = 3;
const CONDUIT_POWER: i32 = 28;
const BLOCK_BREAK_SPEED: i32 = 7;
const MINING_EFFICIENCY: i32 = 24;
const SUBMERGED_MINING_SPEED: i32 = 34;
const BLOCK_INTERACTION_RANGE: i32 = 8;
const ENTITY_INTERACTION_RANGE: i32 = 13;
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
            entity_interaction_range: attribute(ENTITY_INTERACTION_RANGE, 3.0, 64.0),
            block_interaction_range: attribute(BLOCK_INTERACTION_RANGE, 4.5, 64.0),
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
            entity_interaction_range: 3.0,
            block_interaction_range: 4.5,
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
        !block(state).requires_correct_tool_for_drops
            || self.tool().is_some_and(|tool| {
                crate::item::is_correct_for_drops(tool, world.block_state_registry(), state)
            })
    }

    /// `BlockBehaviour.getDestroyProgress`: how much one tick of breaking
    /// does.
    pub fn destroy_progress(&self, world: &ClientWorld, player: &LocalPlayer, state: i32) -> f32 {
        let destroy_time = block(state).destroy_time;
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

/// The block a state belongs to; a number outside the registry is air.
fn block(state: i32) -> &'static voxel_data::block::registry::Block {
    voxel_world::block::state_or_air(i64::from(state)).block()
}

/// `BlockState.isAir`.
fn is_air(state: i32) -> bool {
    block(state).air
}

fn is_game_master_block(state: i32) -> bool {
    block(state)
        .class
        .implements(voxel_data::block::classes::BlockInterface::GameMasterBlock)
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

    /// `Player.isWithinEntityInteractionRange(entity, 0.0)` for the box.
    fn within_entity_interaction_range(&self, b: [f64; 6]) -> bool {
        let position = self.player.position();
        let eye = Vec3::new(
            position.x,
            position.y + f64::from(self.player.eye_height()),
            position.z,
        );
        let range = self.miner.entity_interaction_range;
        voxel_core::aabb::Aabb::new(b[0], b[1], b[2], b[3], b[4], b[5]).distance_to_sqr_point(eye)
            < range * range
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

    /// `LocalPlayer.swing`: the arm swings and the server hears of it.
    fn swing_hand(&mut self, hand: u8, out: &mut Vec<Outgoing>) {
        self.player.swing_hand(hand != MAIN_HAND);
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
    let fluid = voxel_world::block::state_or_air(i64::from(old))
        .fluid
        .legacy_block() as i32;
    let position: Vec3 = ctx.player.position();
    let changed = ctx
        .world
        .set_block_with_updates(pos, fluid, 11, 512, position);
    if changed {
        // `Block.playerWillDestroy`: the break sound (and the particles).
        ctx.world.apply_level_event(2001, pos, old);
    }
    changed
}

impl Outgoing {
    /// The packet's type, as the protocol names it.
    pub fn type_name(&self) -> &'static str {
        match self {
            Outgoing::PlayerAction { .. } => "minecraft:player_action",
            Outgoing::Swing { .. } => "minecraft:swing",
            Outgoing::UseItemOn { .. } => "minecraft:use_item_on",
            Outgoing::UseItem { .. } => "minecraft:use_item",
            Outgoing::Attack { .. } => "minecraft:attack",
            Outgoing::PickItemFromBlock { .. } => "minecraft:pick_item_from_block",
            Outgoing::PickItemFromEntity { .. } => "minecraft:pick_item_from_entity",
            Outgoing::SpectatorAction { .. } => "minecraft:spectator_action",
            Outgoing::Interact { .. } => "minecraft:interact",
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
            Outgoing::Attack { entity_id } => voxel_core::wire::write_varint(entity_id, &mut out),
            Outgoing::PickItemFromBlock {
                x,
                y,
                z,
                include_data,
            } => {
                out.extend_from_slice(
                    &voxel_core::binary::BlockPos::new(x, y, z)
                        .as_long()
                        .to_be_bytes(),
                );
                out.push(u8::from(include_data));
            }
            Outgoing::PickItemFromEntity {
                entity_id,
                include_data,
            } => {
                voxel_core::wire::write_varint(entity_id, &mut out);
                out.push(u8::from(include_data));
            }
            // `OptionalInt`: the value and one, or nothing as zero.
            Outgoing::SpectatorAction { entity_id } => {
                voxel_core::wire::write_varint(entity_id.map_or(0, |id| id + 1), &mut out);
            }
            Outgoing::Interact {
                entity_id,
                hand,
                location,
                secondary_use,
            } => {
                voxel_core::wire::write_varint(entity_id, &mut out);
                voxel_core::wire::write_varint(i32::from(hand), &mut out);
                voxel_core::binary::write_lp_vec3(
                    Vec3::new(location[0], location[1], location[2]),
                    &mut out,
                );
                out.push(u8::from(secondary_use));
            }
        }
        out
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
                id: 20,
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

mod game_mode_items;

mod game_mode_destroy;

mod game_mode_use_ops;
