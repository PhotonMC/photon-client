use super::*;
use photon_core::ids::BlockStateId;

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
    /// `Attributes.ATTACK_SPEED`.
    pub attack_speed: f64,
}

pub(super) const HASTE: i32 = 2;

pub(super) const MINING_FATIGUE: i32 = 3;

pub(super) const CONDUIT_POWER: i32 = 28;

pub(super) const BLOCK_BREAK_SPEED: i32 = 7;

pub(super) const MINING_EFFICIENCY: i32 = 24;

pub(super) const SUBMERGED_MINING_SPEED: i32 = 34;

pub(super) const BLOCK_INTERACTION_RANGE: i32 = 8;

pub(super) const ENTITY_INTERACTION_RANGE: i32 = 13;

pub(super) const ATTACK_SPEED: i32 = 5;

pub(super) const DIRECTION_DOWN: u8 = 0;

/// The block a state belongs to; a number outside the registry is air.
pub(super) fn block(state: BlockStateId) -> &'static photon_data::block::registry::Block {
    photon_world::block::state_or_air(state).block()
}

/// `BlockState.isAir`.
pub(super) fn is_air(state: BlockStateId) -> bool {
    block(state).air
}

pub(super) fn is_game_master_block(state: BlockStateId) -> bool {
    block(state)
        .class
        .implements(photon_data::block::classes::BlockInterface::GameMasterBlock)
}

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
            attack_speed: attribute(ATTACK_SPEED, 4.0, 1024.0),
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
            attack_speed: 4.0,
        }
    }

    pub(super) fn tool(&self) -> Option<&'static photon_data::item::registry::Tool> {
        self.main_hand
            .as_ref()
            .and_then(|stack| crate::item::definition(stack.item_registry_id()))
            .and_then(|item| item.tool)
    }

    /// `Player.getDestroySpeed`.
    pub(super) fn destroy_speed(
        &self,
        world: &ClientWorld,
        player: &LocalPlayer,
        state: BlockStateId,
    ) -> f32 {
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
    pub(super) fn has_correct_tool(&self, world: &ClientWorld, state: BlockStateId) -> bool {
        !block(state).requires_correct_tool_for_drops
            || self.tool().is_some_and(|tool| {
                crate::item::is_correct_for_drops(tool, world.block_state_registry(), state)
            })
    }

    /// `BlockBehaviour.getDestroyProgress`: how much one tick of breaking
    /// does.
    pub fn destroy_progress(
        &self,
        world: &ClientWorld,
        player: &LocalPlayer,
        state: BlockStateId,
    ) -> f32 {
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
