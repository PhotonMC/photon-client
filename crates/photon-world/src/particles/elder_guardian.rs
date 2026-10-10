//! `ElderGuardianParticle`: the elder guardian that flashes over the screen. It has no position
//! the picture uses and no motion, only an age; the `ELDER_GUARDIANS` group draws it.

/// `ElderGuardianParticle.lifetime`.
const LIFETIME: i32 = 30;

/// An elder guardian particle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ElderGuardian {
    age: i32,
    lifetime: i32,
}

impl ElderGuardian {
    pub(super) fn new() -> Self {
        Self {
            age: 0,
            lifetime: LIFETIME,
        }
    }

    pub fn age(&self) -> i32 {
        self.age
    }

    pub fn lifetime(&self) -> i32 {
        self.lifetime
    }

    /// `Particle.tick`: the age counts on, and a particle whose age had reached its lifetime is
    /// removed.
    pub(super) fn tick(&mut self) -> bool {
        let alive = self.age < self.lifetime;
        self.age += 1;
        alive
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_ticks_thirty_times_and_goes_on_the_next() {
        let mut particle = ElderGuardian::new();
        for _ in 0..30 {
            assert!(particle.tick());
        }
        assert!(!particle.tick());
    }
}
