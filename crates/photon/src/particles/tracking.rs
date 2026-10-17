//! The emitters that follow an entity (`TrackingEmitter`): totems, crits.

use super::*;

impl Particles {
    /// Where the local player is, for the emitters that follow it.
    pub(super) fn follow_local_player(&mut self, session: &photon_net::connection::PlaySession) {
        let body = session
            .world()
            .local_player()
            .map(photon_world::ClientPlayerState::id)
            .zip(session.player())
            .map(|(id, player)| {
                let position = player.position();
                let width = player.bounding_box().x_size() as f32;
                let body = photon_world::particles::EntityBody {
                    position: (position.x, position.y, position.z),
                    width,
                    height: player.bb_height(),
                };
                (id, body)
            });
        self.engine.set_local_body(body);
    }

    /// `ParticleEngine.createTrackingEmitter`, and the sound that goes with it
    /// (`playLocalSound`, after the emitter's first spray).
    pub(crate) fn tracking(
        &mut self,
        session: &photon_net::connection::PlaySession,
        requests: Vec<photon_world::particles::TrackingRequest>,
    ) -> Vec<photon_world::local_player::LocalSound> {
        use photon_world::local_player::{LocalSound, LocalSoundSource};
        self.follow_local_player(session);
        let world = session.world();
        let view = AddView {
            camera: self.camera,
            status: ParticleStatus::All,
        };
        let mut sounds = Vec::new();
        for request in requests {
            let Some(type_id) = photon_data::particle_type::id(request.particle) else {
                continue;
            };
            let mut uniquifier = self.uniquifier;
            self.engine.create_tracking_emitter(
                world,
                (request.entity, type_id, request.life_time),
                request.body,
                &view,
                &mut self.level_random,
                || RandomSource::create(unique_seed(&mut uniquifier)),
            );
            self.uniquifier = uniquifier;
            let Some(event) = request.sound else {
                continue;
            };
            // `playLocalSound`'s seed.
            self.level_random.next_long();
            let source = if request.is_player {
                LocalSoundSource::Players
            } else {
                LocalSoundSource::Neutral
            };
            sounds.push(LocalSound::at(
                event,
                source,
                (1.0, 1.0),
                request.body.position,
            ));
        }
        sounds
    }
}
