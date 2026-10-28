//! Keyframe animation definitions (`AnimationDefinition`, `AnimationChannel`,
//! `Keyframe`): the data a model's parts are animated from. The table is the
//! reference's own, written from its definitions and compared on every parity
//! run; vectors are already in the units a part's pose takes (positions with
//! y flipped, rotations in radians, scale less one).

/// What a channel moves: `AnimationChannel.Targets`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Position,
    Rotation,
    Scale,
}

/// How a keyframe is reached from the one before: `AnimationChannel.Interpolations`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interpolation {
    Linear,
    CatmullRom,
}

/// `Keyframe`: a time in seconds and the vector the channel has just before and
/// just after it (the same unless the keyframe jumps).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Keyframe {
    pub time: f32,
    pub pre: [f32; 3],
    pub post: [f32; 3],
    pub interpolation: Interpolation,
}

impl Keyframe {
    pub const fn new(time: f32, target: [f32; 3], interpolation: Interpolation) -> Self {
        Self {
            time,
            pre: target,
            post: target,
            interpolation,
        }
    }

    pub const fn split(
        time: f32,
        pre: [f32; 3],
        post: [f32; 3],
        interpolation: Interpolation,
    ) -> Self {
        Self {
            time,
            pre,
            post,
            interpolation,
        }
    }
}

/// `AnimationChannel`: one target of a bone and its keyframes.
#[derive(Debug, Clone, Copy)]
pub struct Channel {
    pub target: Target,
    pub keyframes: &'static [Keyframe],
}

/// A bone's channels (`boneAnimations`).
#[derive(Debug, Clone, Copy)]
pub struct Bone {
    pub name: &'static str,
    pub channels: &'static [Channel],
}

/// `AnimationDefinition`.
#[derive(Debug, Clone, Copy)]
pub struct Definition {
    /// `Class.FIELD` of the reference (`Rabbit.HOP`).
    pub name: &'static str,
    pub length: f32,
    pub looping: bool,
    pub bones: &'static [Bone],
}

/// The definition with this name.
pub fn definition(name: &str) -> Option<&'static Definition> {
    crate::tables::animations::ANIMATIONS
        .iter()
        .find(|definition| definition.name == name)
}

/// Every definition, in the order the reference's classes declare them.
pub fn definitions() -> &'static [Definition] {
    &crate::tables::animations::ANIMATIONS
}
