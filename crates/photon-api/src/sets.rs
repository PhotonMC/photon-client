use bevy_ecs::schedule::SystemSet;

/// The named parts of a frame, in the order they run. Every system of the `Frame` schedule is in
/// one, so a version or a mod can put a system before or after a part, or replace one.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FrameSet {
    /// The frame's start: the time taken, the profile parts dropped.
    Begin,
    /// The window's events, the settings sent to the server, the language and the swapchain.
    Events,
    /// The time since the last frame, and the ticks it owes.
    Clock,
    /// What the connection delivered, the open container and the level reset.
    Session,
    /// The effects the session's events asked for.
    Effects,
    /// The ticks the clock owes.
    Ticks,
    /// The mouse's accumulated movement turning the player.
    Turn,
    /// The picture, or the screen that stands in for it.
    Draw,
    /// The frame rate limit.
    Limit,
}

/// The named parts of a tick, in the order they run.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TickSet {
    /// The sound engine, and the open screen with what it asked for.
    Interface,
    /// The keys, the items they used and the hands.
    Input,
    /// The local player's own tick.
    LocalPlayer,
    /// The effects the tick leaves in the world: textures, level events, particles.
    Effects,
    /// The camera, the screens and the sky.
    View,
}
