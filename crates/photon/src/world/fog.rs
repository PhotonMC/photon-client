//! What the fog reads of the camera and of the player it is the eyes of (`FogRenderer.setupFog`):
//! what the camera is in, the effects the player has and how well they see under water.

use photon_net::connection::PlaySession;
use photon_render::sky::{FogType, FogView, LightmapView, night_vision_intensity};
use photon_world::ClientMobEffectState;

/// The effect of a name the local player has, if they have it.
fn effect<'a>(effects: &'a [ClientMobEffectState], name: &str) -> Option<&'a ClientMobEffectState> {
    let id = photon_data::item::mob_effects::MOB_EFFECTS
        .iter()
        .position(|effect| effect.name == name)?;
    effects
        .iter()
        .find(|state| usize::try_from(state.effect_id()) == Ok(id))
}

/// The fog's view of the camera, in `fluid`, at `camera_y`, between two ticks.
pub fn fog_view(
    session: &PlaySession,
    (fluid, camera_y): (FogType, f64),
    render_distance: i32,
    partial: f32,
) -> FogView {
    let world = session.world();
    let player = session.player();
    let effects = world
        .local_player()
        .map_or(&[][..], |local| local.effects());
    let flat = world.is_flat();
    FogView {
        fog_type: fluid,
        spectator: player.is_some_and(|player| player.is_spectator()),
        fire_resistance: effect(effects, "minecraft:fire_resistance").is_some(),
        blindness: effect(effects, "minecraft:blindness").map(ClientMobEffectState::duration_ticks),
        darkness: effect(effects, "minecraft:darkness").map(|state| state.blend_factor(partial)),
        night_vision: effect(effects, "minecraft:night_vision")
            .map(|state| state.night_vision_scale(partial)),
        water_vision: player.map(|player| player.water_vision()),
        render_distance,
        // The boss bars' darkening and fog are not kept.
        darken_world: 0.0,
        void_darkness: (
            if flat { 1.0 } else { 32.0 },
            world.dimension().min_y,
            camera_y,
        ),
        boss_fog: false,
    }
}

/// What the lightmap reads of the local player (`LightmapRenderStateExtractor.extract`): the
/// darkness effect, the tick count, and the night vision the effect or the conduit gives.
pub fn lightmap_view(session: &PlaySession, partial: f32) -> LightmapView {
    let local = session.world().local_player();
    let effects = local.map_or(&[][..], |local| local.effects());
    let water_vision = session.player().map_or(0.0, |player| player.water_vision());
    let night_vision = night_vision_intensity(
        effect(effects, "minecraft:night_vision").map(|state| state.night_vision_scale(partial)),
        effect(effects, "minecraft:conduit_power").is_some(),
        water_vision,
    );
    LightmapView {
        darkness_blend: effect(effects, "minecraft:darkness")
            .map_or(0.0, |state| state.blend_factor(partial)),
        tick_count: local.map_or(0, |local| local.tick_count()),
        night_vision,
    }
}
