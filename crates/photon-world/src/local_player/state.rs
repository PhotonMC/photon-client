use super::*;
use photon_core::ids::{BlockStateId, EntityId};
use photon_data::item::registry::AIR_ITEM;

impl LocalPlayer {
    /// How many ticks the player has lived, which the model's idle sway
    /// reads.
    pub fn tick_count(&self) -> i32 {
        self.tick_count
    }

    /// What the mount the player rides reads of it (`controller.xxa`, `zza` and where it looks).
    pub fn controller(&self) -> crate::Controller {
        crate::Controller {
            xxa: self.xxa,
            zza: self.zza,
            y_rot: self.y_rot,
            x_rot: self.x_rot,
            holds_carrot_on_a_stick: false,
            holds_warped_fungus_on_a_stick: false,
            sprinting: self.sprinting,
        }
    }

    /// A player the connection just made, snapped to its spawn pose, with the
    /// game mode's abilities applied.
    pub fn new(
        entity_id: EntityId,
        position: Vec3,
        y_rot: f32,
        x_rot: f32,
        game_mode: ClientPlayerGameMode,
        auto_jump_option: bool,
    ) -> Self {
        let dimensions = ClientEntityPose::Standing.player_dimensions();
        Self {
            entity_id,
            position,
            old_position: position,
            delta: Vec3::ZERO,
            y_rot,
            x_rot,
            bounding_box: dimensions.make_bounding_box(position),
            pose: ClientEntityPose::Standing,
            dimensions,
            on_ground: false,
            horizontal_collision: false,
            vertical_collision: false,
            vertical_collision_below: false,
            minor_horizontal_collision: false,
            main_supporting_block: None,
            on_ground_no_blocks: false,
            fall_distance: 0.0,
            stuck_speed_multiplier: Vec3::ZERO,
            walk_dist: 0.0,
            walk_dist_old: 0.0,
            // The reference starts a new player facing at random; this one
            // starts with head and body where it looks.
            y_body_rot: y_rot,
            y_body_rot_old: y_rot,
            y_head_rot: y_rot,
            y_head_rot_old: y_rot,
            walk: crate::living::WalkAnimation::default(),
            riding: None,
            jump_riding_ticks: 0,
            jump_riding_scale: 0.0,
            ride_jump: None,
            x_bob: 0.0,
            y_bob: 0.0,
            x_bob_old: 0.0,
            y_bob_old: 0.0,
            swinging: false,
            swing_time: 0,
            swinging_off_hand: false,
            attack_anim: 0.0,
            attack_anim_old: 0.0,
            attack_strength_ticker: 0,
            elytra: crate::elytra::ElytraState::default(),
            main_hand_item: AIR_ITEM,
            last_main_hand_item: AIR_ITEM,
            hands: HandState::default(),
            use_edge: Default::default(),
            item_swap_ticker: 0,
            health: 20.0,
            flash_on_set_health: false,
            invulnerable_time: 0,
            hurt_time: 0,
            hurt_duration: 0,
            hurt_dir: 0.0,
            death_time: 0,
            swim_amount: 0.0,
            swim_amount_old: 0.0,
            bob: 0.0,
            bob_old: 0.0,
            first_tick: true,
            tick_count: 0,
            random: photon_core::random::RandomSource::create(i64::from(entity_id.0)),
            underwater: Default::default(),
            portal: Default::default(),
            move_dist: 0.0,
            next_step: 1.0,
            pending_sounds: Vec::new(),
            was_touching_water: false,
            eyes: EyeHistory::default(),
            water: FluidTracker::default(),
            lava: FluidTracker::default(),
            sprinting: false,
            swimming: false,
            crouching: false,
            input: KeyboardInput::new(Input::EMPTY),
            xxa: 0.0,
            zza: 0.0,
            jumping: false,
            no_jump_delay: 0,
            jump_trigger_time: 0,
            sprint_trigger_time: 0,
            auto_jump_time: 0,
            abilities: game_mode.update_player_abilities(ClientPlayerAbilities::default()),
            game_mode,
            no_physics: false,
            auto_jump_option,
            auto_jump_enabled: true,
            trace: Default::default(),
            x_last: 0.0,
            y_last: 0.0,
            z_last: 0.0,
            y_rot_last: 0.0,
            x_rot_last: 0.0,
            last_on_ground: false,
            last_horizontal_collision: false,
            was_sprinting: false,
            position_reminder: 0,
            last_sent_input: Input::EMPTY,
            sent: Vec::new(),
        }
    }

    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    pub fn abilities(&self) -> ClientPlayerAbilities {
        self.abilities
    }

    /// `MultiPlayerGameMode.setLocalMode`: the new mode's abilities.
    pub fn set_game_mode(&mut self, game_mode: ClientPlayerGameMode) {
        self.game_mode = game_mode;
        self.abilities = game_mode.update_player_abilities(self.abilities);
    }

    /// `handlePlayerAbilities`: every field but the build permission.
    pub fn set_abilities(&mut self, abilities: ClientPlayerAbilities) {
        let may_build = self.abilities.may_build;
        self.abilities = abilities;
        self.abilities.may_build = may_build;
    }

    pub fn health(&self) -> f32 {
        self.health
    }

    pub(super) fn on_update_abilities(&mut self) {
        self.sent.push(ServerboundMovement::PlayerAbilities {
            flying: self.abilities.flying,
        });
    }
}

pub(crate) fn property(state_id: Option<BlockStateId>, name: &str) -> Option<&'static str> {
    let (block, offset) = registry::block_for_state(state_id?)?;
    block
        .properties_at(offset)?
        .into_iter()
        .find(|(property, _)| *property == name)
        .map(|(_, value)| value)
}

/// `ServerboundPlayerCommandPacket.Action`, by ordinal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerCommandAction {
    StopSleeping = 0,
    StartSprinting = 1,
    StopSprinting = 2,
    StartRidingJump = 3,
    StartFallFlying = 6,
}

// Player attribute defaults, as the reference registers them.
pub(super) const MOVEMENT_SPEED_BASE: f32 = 0.1;

pub(super) const SPRINTING_MODIFIER: f32 = 0.3;

pub(super) const JUMP_STRENGTH: f32 = 0.42;

pub(super) const STEP_HEIGHT: f64 = 0.6;

pub(super) const GRAVITY: f64 = 0.08;

pub(super) const SNEAKING_SPEED: f64 = 0.3;

pub(super) const SPRINT_WINDOW: i32 = 7;

/// What the player's moves leave behind: the stretches for the block-effects pass, and (as
/// `Entity.pistonDeltas` and `pistonDeltasGameTime`) how far pistons moved the player on each
/// axis during one game tick.
#[derive(Debug, Clone)]
pub(crate) struct MoveTrace {
    pub(crate) movements: Vec<Movement>,
    pub(crate) piston_deltas: [f64; 3],
    pub(crate) piston_time: i64,
}

impl Default for MoveTrace {
    fn default() -> Self {
        Self {
            movements: Vec::new(),
            piston_deltas: [0.0; 3],
            piston_time: i64::MIN,
        }
    }
}

/// A stretch of movement recorded for the block-effects pass.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Movement {
    pub(crate) from: Vec3,
    pub(crate) to: Vec3,
    /// The movement asked for, when the move was split per axis.
    pub(crate) original: Option<Vec3>,
}

/// What one tick of the player sitting in a boat gives the session: the packets to send, and the
/// keys to hand the boat (`AbstractBoat.setInput`).
#[derive(Debug, Clone)]
pub struct RideTick {
    pub sent: Vec<ServerboundMovement>,
    pub boat_input: crate::entity::BoatInput,
    /// The jump the rider let go of, as `PlayerRideableJumping.onPlayerJump` is told it
    /// (the charge, in hundredths).
    pub jump: Option<i32>,
}

/// A block the walk reached, and what its visit needs to act on.
pub(crate) struct InsideVisit {
    pub(crate) pos: BlockPos,
    pub(crate) state: Option<BlockStateId>,
    pub(crate) info: StateInfo,
    /// `isPrecise`: the entity's box really overlaps the block, or it moved a long way.
    pub(crate) precise: bool,
}

/// A body whose movement is walked for the blocks it passes through (`Entity.checkInsideBlocks`).
pub(crate) trait InsideBody {
    /// The body's box with its feet at a position.
    fn box_at(&self, position: Vec3) -> Aabb;
    /// The collision shape powder snow has for this body, which depends on how far it has fallen.
    fn powder_snow_shape(&self) -> VoxelShape;
    /// What the block does to the body.
    fn inside_block_effect(&mut self, level: &Level<'_>, visit: &InsideVisit);
}

/// `SwingAnimation.DEFAULT`'s duration: an empty hand's swing, in ticks.
/// How long a swing lasts with nothing special in the hand (`SwingAnimation.DEFAULT`).
pub(super) const SWING_DURATION: i32 = 6;
