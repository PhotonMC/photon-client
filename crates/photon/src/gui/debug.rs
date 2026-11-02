//! The debug overlay F3 shows (`DebugScreenOverlay`), laid out from the camera entity.

use super::*;
use photon_gui::hud::debug::{
    self, CameraEntity, Clouds, Filtering, FooterKeys, Impactors, Input, ServerInfo, TargetedBlock,
};
use photon_gui::hud::debug_chart as chart;

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
            let eye = crate::view::eye_position(player, 0.0);
            let targets = debug::targets(session.world(), eye, player.rotation());
            Self::camera_entity(session, player.position(), player.rotation(), targets)
        });
        let input = Input {
            system: controls.debug.system.clone(),
            memory: controls.debug.memory.usage(),
            impactors: Some(impactors(&controls.options.quality)),
            version: {
                let version = photon_net::resources::KnownPack::VANILLA_VERSION;
                debug::version_line(version, version)
            },
            crosshair_entity: Self::crosshair_entity(session),
            gpu_utilization: Some(controls.debug.gpu.utilization()),
            sound_mood: controls.debug.sound_mood,
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
                controls.debug.charts.footer_state(),
                (false, false),
            )
        });
        Some(debug::lines(list, reduced, &input, footer))
    }

    /// The overlay's lines and the charts under them.
    pub(super) fn draw_debug_overlay(
        frame: &mut photon_render::gui::GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        ((left, right), session, controls): (&DebugColumns, &PlaySession, &Controls),
        size: (i32, i32),
    ) {
        debug::draw(frame, atlas, font, size.0, (left, right));
        Self::draw_debug_charts(frame, (atlas, font), (session, controls), size);
    }

    /// The charts under the overlay's lines (`DebugScreenOverlay.extractRenderState`): the frame
    /// and tick time charts at the bottom corners while the FPS charts are on, the bandwidth and
    /// ping charts while the network charts are.
    pub(super) fn draw_debug_charts(
        frame: &mut photon_render::gui::GuiFrame,
        fonts: (&GuiAtlas, &Font),
        (session, controls): (&PlaySession, &Controls),
        (gui_width, gui_height): (i32, i32),
    ) {
        let flags = controls.debug.charts;
        // `showFpsCharts` and `showNetworkCharts`: the overlay must be up.
        let shown = controls.debug.entries.is_overlay_visible();
        let max_width = gui_width / 2;
        let one = |value: i64| [value, 0, 0, 0];
        let mut put = |kind: chart::Kind, samples: &[[i64; chart::DIMENSIONS]], right: bool| {
            let width = chart::width(max_width);
            let left = if right { gui_width - width } else { 0 };
            chart::draw(frame, fonts, kind, samples, (left, width, gui_height));
        };
        if shown && flags.fps {
            let frames: Vec<_> = controls.debug.frame_log.samples().map(one).collect();
            let limit = controls.options.framerate_limit;
            put(chart::Kind::Fps { limit }, &frames, false);
            let log = session.tick_time_samples();
            if !log.is_empty() {
                let ticks: Vec<_> = (0..log.len()).map(|i| dimensions(log, i)).collect();
                let mspt = session.milliseconds_per_tick();
                put(chart::Kind::Tps { mspt }, &ticks, true);
            }
        }
        if shown && flags.network {
            let bytes: Vec<_> = column(session.bandwidth_samples())
                .into_iter()
                .map(one)
                .collect();
            put(chart::Kind::Bandwidth, &bytes, false);
            let pings: Vec<_> = column(session.ping_samples())
                .into_iter()
                .map(one)
                .collect();
            put(chart::Kind::Ping, &pings, true);
        }
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
        (target, fluid_target): (Option<TargetedBlock>, Option<TargetedBlock>),
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
            fluid_target,
            light: (
                i32::from(world.sky_brightness(x, y, z)),
                i32::from(world.block_brightness(x, y, z)),
            ),
            biome: Self::feet_biome(session, position),
            heights: Self::feet_heights(session, [x, z]),
        }
    }

    /// `Minecraft.crosshairPickEntity`: the type of the entity the crosshair is on.
    fn crosshair_entity(session: &PlaySession) -> Option<String> {
        let id = crate::view::crosshair_entity(session)?;
        let entity = session.world().entities().at(id)?;
        photon_data::entity_type::name(entity.type_id).map(str::to_owned)
    }

    /// The client chunk's heightmaps at the feet's column (`DebugEntryHeightmap`).
    fn feet_heights(session: &PlaySession, [x, z]: [i32; 2]) -> Option<[i32; 3]> {
        use photon_world::HeightmapType::{MotionBlocking, MotionBlockingNoLeaves, WorldSurface};
        let world = session.world();
        // A chunk that is not loaded is the level's empty one, whose heights are one below the world.
        let empty = world.dimension().min_y - 1;
        let height = |kind| world.chunk_height(kind, x, z).unwrap_or(empty);
        Some([
            height(WorldSurface),
            height(MotionBlocking),
            height(MotionBlockingNoLeaves),
        ])
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

/// `DebugEntrySimplePerformanceImpactors`' reads of the stored video options.
fn impactors(quality: &photon_game::options::QualityOptions) -> Impactors {
    use photon_game::options::{CloudStatus, TextureFiltering};
    Impactors {
        improved_transparency: quality.improved_transparency,
        clouds: match quality.cloud_status {
            CloudStatus::Off => Clouds::Off,
            CloudStatus::Fast => Clouds::Fast,
            CloudStatus::Fancy => Clouds::Fancy,
        },
        biome_blend: quality.biome_blend_radius,
        filtering: match quality.texture_filtering {
            TextureFiltering::None => Filtering::None,
            TextureFiltering::Rgss => Filtering::Rgss,
            TextureFiltering::Anisotropic => Filtering::Anisotropic,
        },
        anisotropy: 1 << quality.max_anisotropy_bit.clamp(1, 3),
    }
}

/// A sample of every dimension.
fn dimensions(log: &photon_net::connection::SampleLog, index: usize) -> [i64; chart::DIMENSIONS] {
    std::array::from_fn(|dimension| log.get(index, dimension).unwrap_or(0))
}

/// The first dimension of every sample, oldest first.
fn column(log: &photon_net::connection::SampleLog) -> Vec<i64> {
    (0..log.len()).filter_map(|i| log.get(i, 0)).collect()
}
