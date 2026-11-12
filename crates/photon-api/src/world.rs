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
