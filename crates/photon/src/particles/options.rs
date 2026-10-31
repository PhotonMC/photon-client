//! What the particles read of the options.

use super::{ParticleStatus, Particles};

impl Particles {
    /// The options the particles read, as they are now.
    pub(crate) fn follow_options(&mut self, options: &photon_game::options::Options) {
        use photon_game::options::ParticleStatus as Chosen;
        self.weather_radius = options.quality.weather_radius;
        self.status = match options.particles {
            Chosen::All => ParticleStatus::All,
            Chosen::Decreased => ParticleStatus::Decreased,
            Chosen::Minimal => ParticleStatus::Minimal,
        };
    }
}
