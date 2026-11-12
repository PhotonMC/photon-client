//! What the level shows a mod, in plain values: a copy, so a mod can never change the game
//! through it.

/// What a mod with `game.time` reads about the level, as of the tick that is running.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WorldView {
    /// The level's game time in ticks, as the server last set it.
    pub game_time: i64,
    /// The ticks of the dimension's own clock (the day time), zero if it has none.
    pub day_time: i64,
}

/// What a mod with `player.state` reads about the local player, as of the tick that is running.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PlayerView {
    /// Where the player stands (the feet), in blocks.
    pub x: f64,
    pub y: f64,
    pub z: f64,
    /// Health in half hearts, zero before the server has said.
    pub health: f32,
    /// Where the player looks, in degrees.
    pub yaw: f32,
    pub pitch: f32,
}
