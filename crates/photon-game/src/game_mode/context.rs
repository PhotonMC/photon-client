use super::*;
use photon_core::ids::BlockStateId;
use photon_core::ids::EntityId;
use photon_core::ids::ItemId;

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
        id: EntityId,
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
    /// Presses of the drop key (Q): one item, or the whole stack with control down.
    pub drop_clicks: u32,
    /// `Minecraft.hasControlDown`: either control key, which asks for the
    /// picked block's or entity's data as well.
    pub control_down: bool,
}

pub const MAIN_HAND: u8 = 0;

pub const OFF_HAND: u8 = 1;

/// Everything one tick of breaking works on.
pub struct Context<'a> {
    pub world: &'a mut ClientWorld,
    pub player: &'a mut LocalPlayer,
    pub miner: &'a Miner,
    pub mode: ClientPlayerGameMode,
    /// The selected hotbar slot, 0 to 8.
    pub selected_slot: i32,
    /// `Player.getCooldowns`.
    pub cooldowns: &'a mut crate::item::cooldowns::ItemCooldowns,
    /// The instruments of the registry, by id.
    pub instruments: &'a [Option<crate::item::instrument::Instrument>],
    /// Whether each enchantment of the registry, by id, has the trident spin attack effect.
    pub spin_attack: &'a [Option<crate::registries::SpinStrength>],
    /// Whether each enchantment of the registry, by id, has the prevent armor change effect.
    pub prevents_armor_change: &'a [bool],
    /// The level's difficulty, 0 (peaceful) to 3 (hard).
    pub difficulty: u8,
    /// The permission level is at least game master (`COMMANDS_GAMEMASTER`).
    pub game_master: bool,
    /// The player's profile id, as the server logged it in (`Player.getUUID`).
    pub profile_id: Option<photon_core::binary::Uuid>,
}

impl Context<'_> {
    pub(super) fn state_at(&self, (x, y, z): (i32, i32, i32)) -> BlockStateId {
        self.world.block_state_or_air(x, y, z)
    }

    /// `Player.canUseGameMasterBlocks`.
    pub(super) fn can_use_game_master_blocks(&self) -> bool {
        self.game_master && self.instabuild()
    }

    pub(super) fn instabuild(&self) -> bool {
        self.player.abilities().instabuild()
    }

    pub(super) fn within_border(&self, (x, _, z): (i32, i32, i32)) -> bool {
        let border = self.world.world_border();
        let (x, z) = (f64::from(x), f64::from(z));
        x >= border.min_x(0.0)
            && x < border.max_x(0.0)
            && z >= border.min_z(0.0)
            && z < border.max_z(0.0)
    }

    /// `Player.isWithinEntityInteractionRange(entity, 0.0)` for the box.
    pub(super) fn within_entity_interaction_range(&self, b: [f64; 6]) -> bool {
        let position = self.player.position();
        let eye = Vec3::new(
            position.x,
            position.y + f64::from(self.player.eye_height()),
            position.z,
        );
        let range = self.miner.entity_interaction_range;
        photon_core::aabb::Aabb::new(b[0], b[1], b[2], b[3], b[4], b[5]).distance_to_sqr_point(eye)
            < range * range
    }

    /// `Player.blockActionRestricted`: spectators cannot break, and an adventure player only
    /// with a main-hand item whose `can_break` predicate matches the block.
    pub(super) fn block_action_restricted(&self, pos: (i32, i32, i32)) -> bool {
        match self.mode {
            ClientPlayerGameMode::Spectator => true,
            ClientPlayerGameMode::Adventure => !self.held_adventure_matches(
                MAIN_HAND,
                photon_world::item_components::ids::CAN_BREAK,
                self.state_at(pos),
            ),
            _ => false,
        }
    }

    /// `ItemStack.canPlaceOnBlockInAdventureMode` for the block the aim is on.
    pub(super) fn can_place_on(&self, hand: u8, aim: super::Aim) -> bool {
        let super::Aim::Block { x, y, z, .. } = aim else {
            return false;
        };
        self.held_adventure_matches(
            hand,
            photon_world::item_components::ids::CAN_PLACE_ON,
            self.state_at((x, y, z)),
        )
    }

    /// `ItemStack.canBreakBlockInAdventureMode` / `canPlaceOnBlockInAdventureMode` of the
    /// stack in `hand`, by the component's id, for a block state.
    pub(super) fn held_adventure_matches(
        &self,
        hand: u8,
        component: i32,
        state: BlockStateId,
    ) -> bool {
        use photon_world::item_components::{adventure_matches, adventure_predicate};
        let Some(Some(value)) = self
            .world
            .local_player()
            .and_then(|local| local.inventory_slot(self.hand_slot(hand)))
            .and_then(|stack| stack.patched(component))
        else {
            return false;
        };
        let registry = self.world.block_state_registry();
        adventure_predicate(&value).is_some_and(|all| {
            adventure_matches(&all, state, |tag| registry.is_state_in_tag(state, tag))
        })
    }

    pub(super) fn swing(&mut self, out: &mut Vec<Outgoing>) {
        self.swing_hand(MAIN_HAND, out);
    }

    /// `LocalPlayer.swing`: the arm swings and the server hears of it.
    pub(super) fn swing_hand(&mut self, hand: u8, out: &mut Vec<Outgoing>) {
        self.player.swing_hand(hand != MAIN_HAND);
        out.push(Outgoing::Swing { hand });
    }

    /// The inventory menu slot a hand holds.
    pub(super) fn hand_slot(&self, hand: u8) -> i16 {
        if hand == MAIN_HAND {
            (36 + self.selected_slot) as i16
        } else {
            45
        }
    }

    /// `getItemInHand`: the stack's item and count, or nothing.
    pub(super) fn held(&self, hand: u8) -> Option<(ItemId, i32)> {
        self.world
            .local_player()
            .and_then(|state| state.inventory_slot(self.hand_slot(hand)))
            .map(|stack| (stack.item_registry_id(), stack.item_count()))
            .filter(|(_, count)| *count > 0)
    }

    /// `ItemCooldowns.isOnCooldown` of the stack in a hand.
    pub(super) fn on_cooldown(&self, hand: u8) -> bool {
        self.world
            .local_player()
            .and_then(|state| state.inventory_slot(self.hand_slot(hand)))
            .is_some_and(|stack| stack.item_count() > 0 && self.cooldowns.is_on_cooldown(stack))
    }
}
