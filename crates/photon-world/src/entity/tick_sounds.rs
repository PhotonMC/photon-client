//! The sounds an entity's own tick asks for with `Level.playLocalSound`, which the states note
//! as they tick and the world plays at the entity's place (`world::entities::tick_sounds`).

/// A sound some entity's tick plays that needs nothing but the entity; a dragon's carries the
/// pitch its own random source drew.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum TickSound {
    /// `Warden.tick`: the heart beats.
    WardenHeartbeat,
    /// `Guardian.aiStep`: it flops out of the water.
    GuardianFlop,
    /// `CopperGolem.playHeadSpinSound`, by how far the golem has weathered.
    CopperSpin(u8),
    /// `EnderDragon.onFlap`.
    DragonFlap(f32),
    /// `EnderDragon.aiStep`: it growls.
    DragonGrowl(f32),
    /// `Breeze.playWhirlSound`.
    BreezeWhirl,
    /// `Illusioner.aiStep`: it vanishes to cast a mirror.
    /// `EnderMan.playStareSound`.
    EndermanStare,
}

// The pitches are drawn finite: never NaN.
impl Eq for TickSound {}
