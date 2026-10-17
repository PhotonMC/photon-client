//! `TrackingEmitter`: a particle that is never drawn and, for a few ticks, sprays another type of
//! particle about an entity.

use super::engine::{Emitted, ParticleData, Spawn};
use photon_core::ids::EntityId;
use photon_core::random::RandomSource;

/// Where an entity is and how big: its feet, `getBbWidth` and `getBbHeight`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EntityBody {
    pub position: (f64, f64, f64),
    pub width: f32,
    pub height: f32,
}

/// A tracking emitter the packets asked for, and the sound that goes with it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrackingRequest {
    pub entity: EntityId,
    /// The sprayed type's name, without the namespace.
    pub particle: &'static str,
    pub life_time: i32,
    /// A sound played where the entity is, after the emitter is made.
    pub sound: Option<&'static str>,
}

/// One emitter, spraying the particles of `type_id` about `entity`.
#[derive(Debug, Clone)]
pub struct TrackingEmitter {
    entity: EntityId,
    type_id: i32,
    life: i32,
    life_time: i32,
    random: RandomSource,
}

impl TrackingEmitter {
    /// `new TrackingEmitter(level, entity, type, lifeTime)`: its first spray is made here, as the
    /// constructor ticks once.
    pub fn new(
        entity: EntityId,
        type_id: i32,
        life_time: i32,
        random: RandomSource,
        body: EntityBody,
    ) -> (Self, Vec<Emitted>) {
        let mut emitter = Self {
            entity,
            type_id,
            life: 0,
            life_time,
            random,
        };
        let sprayed = emitter.tick(Some(body));
        (emitter, sprayed)
    }

    pub fn entity(&self) -> EntityId {
        self.entity
    }

    /// `TrackingEmitter.tick`: sixteen tries at a point in the cube about the entity's middle, each
    /// kept if it falls in the ball inside it. An entity gone from the level sprays nothing and its
    /// emitter still runs out its time.
    pub fn tick(&mut self, body: Option<EntityBody>) -> Vec<Emitted> {
        let mut sprayed = Vec::new();
        for _ in 0..16 {
            let xa = f64::from(self.random.next_float() * 2.0 - 1.0);
            let ya = f64::from(self.random.next_float() * 2.0 - 1.0);
            let za = f64::from(self.random.next_float() * 2.0 - 1.0);
            if xa * xa + ya * ya + za * za > 1.0 {
                continue;
            }
            let Some(body) = body else { continue };
            let (x, y, z) = body.position;
            let width = f64::from(body.width);
            let height = f64::from(body.height);
            sprayed.push(Emitted::Particle(Spawn {
                type_id: self.type_id,
                data: ParticleData::Plain,
                override_limiter: false,
                always_show: false,
                at: (
                    x + width * (xa / 4.0),
                    y + height * (0.5 + ya / 4.0),
                    z + width * (za / 4.0),
                ),
                push: (xa, ya + 0.2, za),
            }));
        }
        self.life += 1;
        sprayed
    }

    /// `Particle.isAlive`: not past its lifetime.
    pub fn is_alive(&self) -> bool {
        self.life < self.life_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body() -> EntityBody {
        EntityBody {
            position: (10.0, 64.0, -3.0),
            width: 0.6,
            height: 1.8,
        }
    }

    fn emitter(life_time: i32) -> (TrackingEmitter, Vec<Emitted>) {
        TrackingEmitter::new(EntityId(7), 3, life_time, RandomSource::create(42), body())
    }

    /// Every spray stays in the ball a quarter of the entity's size about its middle and rises by
    /// a fifth, and the constructor has sprayed once already.
    #[test]
    fn a_spray_stays_about_the_middle_and_rises() {
        let (mut emitter, first) = emitter(3);
        let mut seen = first.len();
        assert!(emitter.life == 1 && seen > 0 && seen <= 16);
        for sprayed in std::iter::once(first).chain((0..2).map(|_| emitter.tick(Some(body())))) {
            for item in sprayed {
                let Emitted::Particle(spawn) = item else {
                    panic!("only particles are sprayed")
                };
                assert_eq!(spawn.type_id, 3);
                assert!((spawn.at.0 - 10.0).abs() <= 0.6 / 4.0);
                assert!((spawn.at.1 - 64.9).abs() <= 1.8 / 4.0);
                assert!((spawn.at.2 + 3.0).abs() <= 0.6 / 4.0);
                let (xa, ya, za) = (spawn.push.0, spawn.push.1 - 0.2, spawn.push.2);
                assert!(xa * xa + ya * ya + za * za <= 1.0);
                seen += 1;
            }
        }
        assert!(seen > 16);
    }

    /// An emitter runs out its time, one tick of it spent in the constructor, and sprays nothing
    /// for an entity that is gone.
    #[test]
    fn an_emitter_dies_when_its_time_is_out() {
        let (mut emitter, _) = emitter(3);
        assert!(emitter.is_alive());
        assert!(emitter.tick(None).is_empty());
        assert!(emitter.is_alive());
        emitter.tick(Some(body()));
        assert!(!emitter.is_alive());
    }
}
