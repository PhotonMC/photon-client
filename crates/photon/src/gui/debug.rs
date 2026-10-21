//! The debug overlay F3 shows (`DebugScreenOverlay`), laid out from the camera entity.

use super::*;
use photon_gui::hud::debug::{
    self, CameraEntity, Clouds, Filtering, FooterKeys, Impactors, Input, ServerInfo, TargetedBlock,
};

/// The two columns of the overlay's lines, or none when nothing is shown: no entry is on, or the
/// interface is hidden with no screen over it.
pub(super) type DebugColumns = (Vec<String>, Vec<String>);

impl Gui {
    pub(super) fn debug_columns(
        session: &PlaySession,
        controls: &Controls,
    ) -> Option<DebugColumns> {
        // `Options.reducedDebugInfo` is not an option here yet.
        let reduced = session.reduced_debug_info();
        let list = &controls.debug.entries;
        if list.enabled(reduced).is_empty() || controls.hud_hidden && !controls.any_screen_open() {
            return None;
        }
        let camera = session.player().map(|player| {
            let target = Self::targeted_block(session, player);
            Self::camera_entity(session, player.position(), player.rotation(), target)
        });
        let input = Input {
            system: controls.debug.system.clone(),
            memory: controls.debug.memory.usage(),
            // The video options these read are not stored yet: the game's own defaults.
            impactors: Some(Impactors {
                improved_transparency: false,
                clouds: Clouds::Fancy,
                biome_blend: 2,
                filtering: Filtering::None,
                anisotropy: 4,
            }),
            version: {
                let version = photon_net::resources::KnownPack::VANILLA_VERSION;
                debug::version_line(version, version)
            },
            gpu_utilization: Some(controls.debug.gpu.utilization()),
            day_count: Self::day_count(session),
            camera,
            frame: controls.debug.frame,
            server: Some({
                let (sent, received) = session.average_packets();
                let (stepping, frozen) = session.tick_rate_status();
                ServerInfo {
                    brand: session.server_brand().map(str::to_owned),
                    stepping,
                    frozen,
                    sent,
                    received,
                }
            }),
        };
        let footer = list.is_overlay_visible().then(|| {
            debug::footer(
                &FooterKeys::defaults(),
                (false, false, false, false),
                (false, false),
            )
        });
        Some(debug::lines(list, reduced, &input, footer))
    }

    /// `Timeline.getPeriodCount` of the overworld day timeline (a 24000-tick period on the
    /// overworld clock); an unset clock counts from zero (`ClientClockManager.getInstance`).
    fn day_count(session: &PlaySession) -> Option<i32> {
        const DAY_PERIOD: i64 = 24000;
        session
            .registry_names("minecraft:timeline")
            .iter()
            .any(|name| name == "minecraft:day")
            .then(|| (session.clock_ticks("minecraft:overworld").unwrap_or(0) / DAY_PERIOD) as i32)
    }

    /// What the entries read of the camera entity: where it is and what is around its feet.
    fn camera_entity(
        session: &PlaySession,
        position: photon_core::math::Vec3,
        (yaw, pitch): (f32, f32),
        target: Option<TargetedBlock>,
    ) -> CameraEntity {
        let world = session.world();
        let bounds = world.dimension();
        let [x, y, z] = [position.x, position.y, position.z].map(|axis| axis.floor() as i32);
        CameraEntity {
            position: (position.x, position.y, position.z),
            yaw,
            pitch,
            dimension: world.dimension_key().to_owned(),
            build_height: (bounds.min_y, bounds.height),
            target,
            light: (
                i32::from(world.sky_brightness(x, y, z)),
                i32::from(world.block_brightness(x, y, z)),
            ),
            biome: Self::feet_biome(session, position),
        }
    }

    /// `Entity.pick(20.0, 0.0F, false)`: the block outline the view meets within 20 blocks, from
    /// the eyes as they were a tick ago, and what the level holds there.
    fn targeted_block(
        session: &PlaySession,
        player: &photon_world::local_player::LocalPlayer,
    ) -> Option<TargetedBlock> {
        const RANGE: f64 = 20.0;
        let eye = crate::view::eye_position(player, 0.0);
        let (yaw, pitch) = player.rotation();
        let look = photon_core::math::Vec3::direction_from_rotation_components(pitch, yaw);
        let to = photon_core::math::Vec3::new(
            eye.x + look.x * RANGE,
            eye.y + look.y * RANGE,
            eye.z + look.z * RANGE,
        );
        let world = session.world();
        let hit = world.clip_outlines(eye, to)?;
        let pos = (hit.block_pos.x, hit.block_pos.y, hit.block_pos.z);
        TargetedBlock::of(pos, world.block_state_or_air(pos.0, pos.1, pos.2))
    }

    /// `Level.getBiome` at the feet: the biome of the quart position the feet are in, the plains
    /// where the chunk is not loaded (`ClientLevel.getUncachedNoiseBiome`).
    fn feet_biome(session: &PlaySession, feet: photon_core::math::Vec3) -> String {
        let quart = |value: f64| (value.floor() as i32) >> 2;
        session
            .world()
            .noise_biome(quart(feet.x), quart(feet.y), quart(feet.z))
            .and_then(|id| usize::try_from(id).ok())
            .and_then(|id| session.registry_names("minecraft:worldgen/biome").get(id))
            .map_or_else(|| "minecraft:plains".to_owned(), Clone::clone)
    }
}
