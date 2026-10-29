use super::*;

impl ClientEntity {
    /// What `getDefaultDimensions` scales a fish's box by, from the size the
    /// type's table holds (that of a new one: a medium salmon, an unpuffed
    /// pufferfish, whose half is undone by doubling): a salmon by its size, a
    /// pufferfish by how puffed it is.
    pub(crate) fn fish_dimension_scale(&self) -> f32 {
        match photon_data::entity_type::name(self.type_id) {
            Some("minecraft:salmon") => self.salmon_size().bounding_box_scale(),
            Some("minecraft:pufferfish") => {
                crate::mobs::fish::pufferfish_bounding_box_scale(self.puff_state()) * 2.0
            }
            _ => 1.0,
        }
    }

    /// `Frog.onSyncedDataUpdated(DATA_POSE)`: the pose the data names starts
    /// or stops the frog's animations.
    pub(crate) fn set_pose(&mut self, pose: ClientEntityPose) {
        self.pose = pose;
        if self.is_frog() {
            let tick_count = self.tick_count;
            self.kind.frog_state().set_pose(pose, tick_count);
        }
        if self.is_breeze() {
            let tick_count = self.tick_count;
            self.kind.breeze_state().set_pose(pose, tick_count);
        }
        if self.is_warden() {
            let tick_count = self.tick_count;
            self.kind.warden_state().set_pose(pose, tick_count);
        }
    }

    pub fn pose(&self) -> ClientEntityPose {
        self.pose
    }

    /// Return player/Avatar dimensions for a remote player entity.
    pub fn player_dimensions(&self) -> Option<ClientEntityDimensions> {
        (self.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID).then(|| self.pose.player_dimensions())
    }
}

/// Resolve Java `LivingEntity.getDimensions` from default dimensions and scales.
pub fn living_entity_dimensions(
    pose: ClientEntityPose,
    default_dimensions: ClientEntityDimensions,
    age_scale: f32,
    entity_scale: f32,
) -> ClientEntityDimensions {
    if pose == ClientEntityPose::Sleeping {
        ClientEntityDimensions::fixed(0.2, 0.2).with_eye_height(0.2)
    } else {
        default_dimensions
            .scale(age_scale, age_scale)
            .scale(entity_scale, entity_scale)
    }
}

/// What Java's resize gate reads besides the sizes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ResizeContext {
    pub level_is_client_side: bool,
    pub first_tick: bool,
    pub no_physics: bool,
    pub is_player: bool,
}

/// Refresh dimensions at the current position and evaluate Java's resize gate.
pub fn refresh_entity_dimensions(
    old_dimensions: ClientEntityDimensions,
    new_dimensions: ClientEntityDimensions,
    position: Vec3,
    context: ResizeContext,
) -> ClientEntityDimensionsRefresh {
    let ResizeContext {
        level_is_client_side,
        first_tick,
        no_physics,
        is_player,
    } = context;
    let is_small = new_dimensions.width() <= 4.0 && new_dimensions.height() <= 4.0;
    let grew = new_dimensions.width() > old_dimensions.width()
        || new_dimensions.height() > old_dimensions.height();
    ClientEntityDimensionsRefresh {
        dimensions: new_dimensions,
        bounding_box: new_dimensions.make_bounding_box(position),
        should_fudge_position: !level_is_client_side
            && !first_tick
            && !no_physics
            && is_small
            && grew
            && !is_player,
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ClientEntityPose {
    #[default]
    Standing,
    FallFlying,
    Sleeping,
    Swimming,
    SpinAttack,
    Crouching,
    LongJumping,
    Dying,
    Croaking,
    UsingTongue,
    Sitting,
    Roaring,
    Sniffing,
    Emerging,
    Digging,
    Sliding,
    Shooting,
    Inhaling,
}

/// Entity dimensions used to derive Java's feet-origin bounding box.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientEntityDimensions {
    pub(crate) width: JavaF32,
    pub(crate) height: JavaF32,
    pub(crate) eye_height: JavaF32,
    pub(crate) fixed: bool,
}

/// State produced by Java Entity.refreshDimensions before optional server
/// position correction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientEntityDimensionsRefresh {
    pub dimensions: ClientEntityDimensions,
    pub bounding_box: Aabb,
    pub should_fudge_position: bool,
}

/// How a living entity's pose is ticked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LivingKind {
    Player,
    /// A `Mob`: its body follows the head by `BodyRotationControl`.
    Mob,
    /// A living entity that is neither, an armor stand.
    Other,
}

impl ClientEntityPose {
    /// The reference's `Pose` constant name.
    pub const fn java_name(self) -> &'static str {
        match self {
            Self::Standing => "STANDING",
            Self::FallFlying => "FALL_FLYING",
            Self::Sleeping => "SLEEPING",
            Self::Swimming => "SWIMMING",
            Self::SpinAttack => "SPIN_ATTACK",
            Self::Crouching => "CROUCHING",
            Self::LongJumping => "LONG_JUMPING",
            Self::Dying => "DYING",
            Self::Croaking => "CROAKING",
            Self::UsingTongue => "USING_TONGUE",
            Self::Sitting => "SITTING",
            Self::Roaring => "ROARING",
            Self::Sniffing => "SNIFFING",
            Self::Emerging => "EMERGING",
            Self::Digging => "DIGGING",
            Self::Sliding => "SLIDING",
            Self::Shooting => "SHOOTING",
            Self::Inhaling => "INHALING",
        }
    }

    pub const fn id(self) -> u8 {
        match self {
            Self::Standing => 0,
            Self::FallFlying => 1,
            Self::Sleeping => 2,
            Self::Swimming => 3,
            Self::SpinAttack => 4,
            Self::Crouching => 5,
            Self::LongJumping => 6,
            Self::Dying => 7,
            Self::Croaking => 8,
            Self::UsingTongue => 9,
            Self::Sitting => 10,
            Self::Roaring => 11,
            Self::Sniffing => 12,
            Self::Emerging => 13,
            Self::Digging => 14,
            Self::Sliding => 15,
            Self::Shooting => 16,
            Self::Inhaling => 17,
        }
    }

    pub(crate) fn from_wire_id(id: i32) -> Self {
        match id {
            1 => Self::FallFlying,
            2 => Self::Sleeping,
            3 => Self::Swimming,
            4 => Self::SpinAttack,
            5 => Self::Crouching,
            6 => Self::LongJumping,
            7 => Self::Dying,
            8 => Self::Croaking,
            9 => Self::UsingTongue,
            10 => Self::Sitting,
            11 => Self::Roaring,
            12 => Self::Sniffing,
            13 => Self::Emerging,
            14 => Self::Digging,
            15 => Self::Sliding,
            16 => Self::Shooting,
            17 => Self::Inhaling,
            _ => Self::Standing,
        }
    }

    /// Resolve a wire pose ID with Java's out-of-range standing fallback.
    pub fn from_id(id: i32) -> Self {
        Self::from_wire_id(id)
    }

    /// Return the default Avatar dimensions for this pose at scale one.
    pub fn player_dimensions(self) -> ClientEntityDimensions {
        match self {
            Self::Standing => ClientEntityDimensions::scalable(0.6, 1.8).with_eye_height(1.62),
            Self::Sleeping => ClientEntityDimensions::fixed(0.2, 0.2).with_eye_height(0.2),
            Self::FallFlying | Self::Swimming | Self::SpinAttack => {
                ClientEntityDimensions::scalable(0.6, 0.6).with_eye_height(0.4)
            }
            Self::Crouching => ClientEntityDimensions::scalable(0.6, 1.5).with_eye_height(1.27),
            Self::Dying => ClientEntityDimensions::fixed(0.2, 0.2).with_eye_height(1.62),
            _ => Self::Standing.player_dimensions(),
        }
    }
}

impl ClientEntityDimensions {
    /// Create scalable dimensions with Java's default eye-height multiplier.
    pub fn scalable(width: f32, height: f32) -> Self {
        Self::from_values(width, height, height * 0.85_f32, false)
    }

    /// Create dimensions that ignore later scale changes.
    pub fn fixed(width: f32, height: f32) -> Self {
        Self::from_values(width, height, height * 0.85_f32, true)
    }

    pub(crate) fn from_values(width: f32, height: f32, eye_height: f32, fixed: bool) -> Self {
        Self {
            width: JavaF32::new(width),
            height: JavaF32::new(height),
            eye_height: JavaF32::new(eye_height),
            fixed,
        }
    }

    pub fn width(self) -> f32 {
        self.width.get()
    }

    pub fn height(self) -> f32 {
        self.height.get()
    }

    pub fn eye_height(self) -> f32 {
        self.eye_height.get()
    }

    pub fn is_fixed(self) -> bool {
        self.fixed
    }

    /// Apply Java's width/height scale path, including the fixed early return.
    pub fn scale(self, width_scale: f32, height_scale: f32) -> Self {
        if !self.fixed && (width_scale != 1.0_f32 || height_scale != 1.0_f32) {
            Self::from_values(
                self.width() * width_scale,
                self.height() * height_scale,
                self.eye_height() * height_scale,
                false,
            )
        } else {
            self
        }
    }

    pub fn with_eye_height(self, eye_height: f32) -> Self {
        Self::from_values(self.width(), self.height(), eye_height, self.fixed)
    }

    /// Make the feet-origin AABB used by `EntityDimensions.makeBoundingBox`.
    pub fn make_bounding_box(self, position: Vec3) -> Aabb {
        let half_width = f64::from(self.width() / 2.0_f32);
        let height = f64::from(self.height());
        Aabb::new(
            position.x - half_width,
            position.y,
            position.z - half_width,
            position.x + half_width,
            position.y + height,
            position.z + half_width,
        )
    }
}
