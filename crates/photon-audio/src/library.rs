//! What the sound engine needs of the channels there are (`Library`, `Channel`): where a sound
//! plays and how it is asked to. `photon-game` drives these; the mixer implements them.

/// `Library.Pool`: sounds loaded whole share one set of channels, streamed ones another.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pool {
    Static,
    Streaming,
}

/// One source of sound the engine drives (`Channel`).
pub trait Channel {
    fn set_pitch(&mut self, pitch: f32);
    fn set_volume(&mut self, volume: f32);
    /// Linear fall-off to silence at this distance, in blocks.
    fn linear_attenuation(&mut self, max_distance: f32);
    fn disable_attenuation(&mut self);
    fn set_looping(&mut self, looping: bool);
    fn set_position(&mut self, position: (f64, f64, f64));
    fn set_relative(&mut self, relative: bool);
    /// Play a sound file loaded whole (`sounds/<name>.ogg` of the assets).
    fn attach_static_buffer(&mut self, path: &str);
    /// Play a sound file as it is read, from the start again when it ends if `looping`.
    fn attach_stream(&mut self, path: &str, looping: bool);
    fn play(&mut self);
    fn pause(&mut self);
    fn unpause(&mut self);
    fn stop(&mut self);
    /// Whether the sound has ended (by itself or by [`Self::stop`]).
    fn stopped(&self) -> bool;
    fn update_stream(&mut self);
    /// Give the channel back for good.
    fn destroy(&mut self);
    /// Which channel this is, for those that count them (a test's recorder).
    fn number(&self) -> Option<usize> {
        None
    }
}

/// The channels there are (`Library`): none once a pool is used up.
pub trait ChannelSource {
    type Channel: Channel;
    fn acquire(&mut self, pool: Pool) -> Option<Self::Channel>;
}
