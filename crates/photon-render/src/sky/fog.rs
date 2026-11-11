//! The fog a frame is drawn with (`FogRenderer.setupFog`): which environment applies, the distances
//! it sets, the colour it gives and how that colour is darkened and brightened.

use super::lerp::srgb_lerp_vec;
use photon_core::math::{mth_lerp_f32, mth_sin_f64};

/// What the camera is in (`Camera.getFluidInCamera`, the air being atmospheric).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FogType {
    Atmospheric,
    Water,
    Lava,
    PowderSnow,
}

impl FogType {
    /// Whether being in it narrows the field of view (`Camera.getFov`): water and lava do.
    pub fn narrows_view(self) -> bool {
        matches!(self, Self::Water | Self::Lava)
    }
}

/// What the atmospheric environment makes of the sky's attributes: its colour and distances.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AtmosphericFog {
    pub colour: [f32; 3],
    pub start: f32,
    pub end: f32,
    pub sky_end: f32,
    pub cloud_end: f32,
}

impl AtmosphericFog {
    /// `AtmosphericFogEnvironment.setupFog` less the boss bars' fog: the attributes' distances, drawn
    /// in by the rain, and the sky's and the clouds' ends held to the render distance and the cloud
    /// range, in chunks.
    pub fn of(
        colour: [f32; 3],
        (start, end, sky_end, cloud_end): (f32, f32, f32, f32),
        (rain_fog, render_distance, cloud_range): (f32, i32, i32),
    ) -> Self {
        Self {
            colour,
            start: start + -160.0 * rain_fog,
            end: end.min(96.0).max(end + -256.0 * rain_fog),
            sky_end: ((render_distance * 16) as f32).min(sky_end),
            cloud_end: ((cloud_range * 16) as f32).min(cloud_end),
        }
    }
}

/// What the water environment reads of the attributes: its colour and distances.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaterFog {
    pub colour: [f32; 3],
    pub start: f32,
    pub end: f32,
}

/// What the camera's entity and the world give the fog besides the environments' attributes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FogView {
    pub fog_type: FogType,
    pub spectator: bool,
    pub fire_resistance: bool,
    /// The blindness effect's duration in ticks, `-1` without end, if the camera's entity has it.
    pub blindness: Option<i32>,
    /// The darkness effect's blend factor at this frame, if the entity has it.
    pub darkness: Option<f32>,
    /// `GameRenderer.nightVisionScale`, if the entity has night vision.
    pub night_vision: Option<f32>,
    /// `LocalPlayer.getWaterVision`, if the camera is the local player's.
    pub water_vision: Option<f32>,
    /// The effective render distance in chunks.
    pub render_distance: i32,
    /// How far the boss bars have darkened the world (`darkenWorldAmount`).
    pub darken_world: f32,
    /// `voidDarknessOnsetRange`, the level's lowest block, and the camera's height.
    pub void_darkness: (f32, i32, f64),
    /// Whether a boss bar makes world fog (`BossHealthOverlay.shouldCreateWorldFog`).
    pub boss_fog: bool,
}

impl FogView {
    /// Whether the sky is left undrawn (`LevelRenderer.addSkyPass`): in lava or powder snow, for an
    /// entity that has blindness or darkness, and under a boss bar's world fog.
    pub fn hides_sky(&self) -> bool {
        // A boss bar's world fog leaves the sky out too (`shouldRenderSky`).
        self.boss_fog
            || matches!(self.fog_type, FogType::Lava | FogType::PowderSnow)
            || self.blindness.is_some()
            || self.darkness.is_some()
    }
}

/// `FogData`: the colour and distances the fog is drawn with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FogData {
    pub colour: [f32; 4],
    pub environmental_start: f32,
    pub environmental_end: f32,
    pub render_distance_start: f32,
    pub render_distance_end: f32,
    pub sky_end: f32,
    pub cloud_end: f32,
}

/// `ARGB.vector3fFromRGB24`.
fn rgb24(color: u32) -> [f32; 3] {
    [16, 8, 0].map(|shift| ((color >> shift) & 0xff) as f32 / 255.0)
}

/// `LavaFogEnvironment.COLOR`.
fn lava_colour() -> [f32; 3] {
    rgb24(0xFF99_1900)
}

/// `PowderedSnowFogEnvironment.COLOR`.
fn powder_snow_colour() -> [f32; 3] {
    rgb24(0xFF9F_BBCC)
}

/// Which environment sets the distances (the first that applies, in `FogRenderer`'s order).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Environment {
    Lava,
    PowderSnow,
    Blindness,
    Darkness,
    Water,
    Atmospheric,
}

fn environment_of(view: &FogView) -> Environment {
    match view.fog_type {
        FogType::Lava => Environment::Lava,
        FogType::PowderSnow => Environment::PowderSnow,
        _ if view.blindness.is_some() => Environment::Blindness,
        _ if view.darkness.is_some() => Environment::Darkness,
        FogType::Water => Environment::Water,
        FogType::Atmospheric => Environment::Atmospheric,
    }
}

/// How dark the fog is: the void's, raised by the first environment that modifies the darkness
/// (blindness, else the darkness effect).
fn darkness_of(view: &FogView) -> f32 {
    let (onset, min_y, camera_y) = view.void_darkness;
    let darkness = ((onset + min_y as f32 - camera_y as f32) / onset).clamp(0.0, 1.0);
    match (view.blindness, view.darkness) {
        (Some(duration), _) if duration != -1 && duration <= 19 => {
            (duration as f32 / 20.0).max(darkness)
        }
        (Some(_), _) => 1.0,
        (None, Some(blend)) => blend.max(darkness),
        (None, None) => darkness,
    }
}

/// How far the fog colour is brightened: by the water vision under water, else by night vision
/// unless darkness is on.
fn brighten_factor(view: &FogView) -> f32 {
    match (view.fog_type, view.night_vision, view.darkness) {
        (FogType::Water, _, _) => view.water_vision.unwrap_or(1.0),
        (_, Some(scale), None) => scale,
        _ => 0.0,
    }
}

/// `FogRenderer.computeFogColor` after the base colour is chosen: the void's darkness, the effects',
/// the bosses', and the brightening of the night vision and the water vision.
fn finish_colour(base: [f32; 3], view: &FogView) -> [f32; 4] {
    let [mut red, mut green, mut blue] = base;
    let darkness = darkness_of(view);
    if darkness > 0.0 && !matches!(view.fog_type, FogType::Lava | FogType::PowderSnow) {
        let brightness = (1.0 - darkness) * (1.0 - darkness);
        red *= brightness;
        green *= brightness;
        blue *= brightness;
    }
    if view.darken_world > 0.0 {
        red = mth_lerp_f32(view.darken_world, red, red * 0.7);
        green = mth_lerp_f32(view.darken_world, green, green * 0.6);
        blue = mth_lerp_f32(view.darken_world, blue, blue * 0.6);
    }
    [red, green, blue] = brightened([red, green, blue], brighten_factor(view));
    [red, green, blue, 1.0]
}

/// The colour pulled towards its brightest form by `factor` (`FogRenderer.computeFogColor`): each
/// channel is scaled so the brightest reaches one. 26.2 scales the channels that are not the
/// brightest by one over the brightest held between 0.07 and one.
fn brightened(colour: [f32; 3], factor: f32) -> [f32; 3] {
    let [red, green, blue] = colour;
    if red == 0.0 || green == 0.0 || blue == 0.0 {
        return colour;
    }
    let brightest = red.max(green.max(blue));
    let top_scale = 1.0 / brightest;
    let other_scale = if photon_core::versions::rules()
        .render
        .fog_brighten_clamps_scale
    {
        1.0 / brightest.clamp(0.07, 1.0)
    } else {
        top_scale
    };
    colour.map(|channel| {
        let scale = if channel == brightest {
            top_scale
        } else {
            other_scale
        };
        mth_lerp_f32(factor, channel, channel * scale)
    })
}

/// What the first environment that applies sets for the distances: the start and end of the fog, and
/// the ends the sky and the clouds are fogged to.
fn environment_distances(
    environment: Environment,
    view: &FogView,
    render_blocks: f32,
    (atmosphere, water): (&AtmosphericFog, &WaterFog),
) -> (f32, f32, f32, f32) {
    match environment {
        Environment::Lava => {
            let (start, end) = if view.spectator {
                (-8.0, render_blocks * 0.5)
            } else if view.fire_resistance {
                (0.0, 5.0)
            } else {
                (0.25, 1.0)
            };
            (start, end, end, end)
        }
        Environment::PowderSnow => {
            let (start, end) = if view.spectator {
                (-8.0, render_blocks * 0.5)
            } else {
                (0.0, 2.0)
            };
            (start, end, end, end)
        }
        Environment::Blindness => {
            let duration = view.blindness.unwrap_or(0);
            let distance = if duration == -1 {
                5.0
            } else {
                mth_lerp_f32((duration as f32 / 20.0).min(1.0), render_blocks, 5.0)
            };
            (distance * 0.25, distance, distance * 0.8, distance * 0.8)
        }
        Environment::Darkness => {
            let distance = mth_lerp_f32(view.darkness.unwrap_or(0.0), render_blocks, 15.0);
            (distance * 0.75, distance, distance, distance)
        }
        Environment::Water => {
            let end = water.end * view.water_vision.map_or(1.0, |vision| vision.max(0.25));
            (water.start, end, end, end)
        }
        Environment::Atmospheric => atmospheric_distances(view, atmosphere),
    }
}

/// `AtmosphericFogEnvironment.setupFog`'s last step: a boss bar that makes world fog pulls it in.
fn atmospheric_distances(view: &FogView, atmosphere: &AtmosphericFog) -> (f32, f32, f32, f32) {
    if view.boss_fog {
        let end = atmosphere.end.min(96.0);
        return (atmosphere.start.min(10.0), end, end, end);
    }
    (
        atmosphere.start,
        atmosphere.end,
        atmosphere.sky_end,
        atmosphere.cloud_end,
    )
}

/// `FogRenderer.setupFog`: the fog for a camera in a fluid or the air, from what the environments
/// the camera could be in read of the attributes.
pub fn fog_data(view: &FogView, atmosphere: &AtmosphericFog, water: &WaterFog) -> FogData {
    let render_blocks = (view.render_distance * 16) as f32;
    let environment = environment_of(view);
    let base = match view.fog_type {
        FogType::Lava => lava_colour(),
        FogType::PowderSnow => powder_snow_colour(),
        FogType::Water => water.colour,
        FogType::Atmospheric => atmosphere.colour,
    };
    let (start, end, sky_end, cloud_end) =
        environment_distances(environment, view, render_blocks, (atmosphere, water));
    let span = (render_blocks / 10.0).clamp(4.0, 64.0);
    FogData {
        colour: finish_colour(base, view),
        environmental_start: start,
        environmental_end: end,
        render_distance_start: render_blocks - span,
        render_distance_end: render_blocks,
        sky_end,
        cloud_end,
    }
}

/// The attributes `AtmosphericFogEnvironment.getBaseColor` reads.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AtmosphereAttributes {
    pub fog_colour: [f32; 3],
    /// The sunrise colour with its alpha.
    pub sunrise: [f32; 4],
    pub sky_colour: [f32; 3],
    /// The sun's angle in degrees.
    pub sun_angle: f32,
    pub sky_fog_end: f32,
}

/// `AtmosphericFogEnvironment.getBaseColor`: the fog's colour, warmed towards the sunrise or sunset
/// where the camera faces the sun, then mixed towards the sky's, which the weather darkens. `facing`
/// is the camera's forward direction along x and the render distance in chunks; `weather` the rain
/// and thunder levels.
pub fn atmospheric_colour(
    attributes: &AtmosphereAttributes,
    (forward_x, render_distance): (f32, i32),
    weather: (f32, f32),
) -> [f32; 3] {
    let sun = sun_mix(attributes, forward_x, render_distance);
    let mix = sky_mix(attributes, render_distance);
    if photon_core::versions::rules().render.fog_colour_is_packed {
        return packed_colour(attributes, (sun, mix), weather);
    }
    let vec4 = |colour: [f32; 3]| [colour[0], colour[1], colour[2], 1.0];
    let vec3 = |colour: [f32; 4]| [colour[0], colour[1], colour[2]];
    let mut fog = vec4(attributes.fog_colour);
    if let Some(sun) = sun {
        fog = srgb_lerp_vec(sun, fog, vec4(vec3(attributes.sunrise)));
    }
    // The sky colour the fog blends towards is darkened by the weather
    // (`AtmosphericFogEnvironment.applyWeatherDarken`).
    let scale = |colour: [f32; 4], red: f32, green: f32, blue: f32| {
        [
            (colour[0] * red).clamp(0.0, 1.0),
            (colour[1] * green).clamp(0.0, 1.0),
            (colour[2] * blue).clamp(0.0, 1.0),
            1.0,
        ]
    };
    let (rain, thunder) = weather;
    let mut darkened = vec4(attributes.sky_colour);
    if rain > 0.0 {
        let modifier = 1.0 - rain * 0.5;
        darkened = scale(darkened, modifier, modifier, 1.0 - rain * 0.4);
    }
    if thunder > 0.0 {
        let modifier = 1.0 - thunder * 0.5;
        darkened = scale(darkened, modifier, modifier, modifier);
    }
    vec3(srgb_lerp_vec(mix, fog, darkened))
}

/// How far the fog warms towards the sunrise colour: the part of the camera's forward direction
/// that faces the sun, by the colour's alpha. None where it does not warm.
fn sun_mix(attributes: &AtmosphereAttributes, forward_x: f32, render_distance: i32) -> Option<f32> {
    if render_distance < 4 {
        return None;
    }
    let sun_angle = attributes.sun_angle * (std::f64::consts::PI / 180.0) as f32;
    let sun_x = if mth_sin_f64(f64::from(sun_angle)) > 0.0 {
        -1.0
    } else {
        1.0
    };
    let looking = forward_x * sun_x;
    let alpha = attributes.sunrise[3];
    (looking > 0.0 && alpha > 0.0).then_some(looking * alpha)
}

/// How far the fog blends towards the sky's colour, from where the sky's fog ends.
fn sky_mix(attributes: &AtmosphereAttributes, render_distance: i32) -> f32 {
    let sky_fog_end = (attributes.sky_fog_end / 16.0).min(render_distance as f32);
    let delta = sky_fog_end / 32.0;
    let mix = if delta < 0.0 {
        0.25
    } else if delta > 1.0 {
        1.0
    } else {
        0.25 + delta * 0.75
    };
    1.0 - (f64::from(mix)).powf(0.25) as f32
}

/// 26.2's `getBaseColor`, which works on packed colours: every blend and the weather's darkening
/// land on whole 8-bit channels.
fn packed_colour(
    attributes: &AtmosphereAttributes,
    (sun, mix): (Option<f32>, f32),
    (rain, thunder): (f32, f32),
) -> [f32; 3] {
    use super::lerp::srgb_lerp;
    let pack = |colour: [f32; 3]| {
        let byte = |value: f32| (value * 255.0).round().clamp(0.0, 255.0) as u32;
        0xFF00_0000 | byte(colour[0]) << 16 | byte(colour[1]) << 8 | byte(colour[2])
    };
    let scale = |colour: u32, by: [f32; 3]| {
        let channel = |shift: u32, by: f32| {
            let scaled = (((colour >> shift) & 0xFF) as f32 * by) as i32;
            (scaled.clamp(0, 255) as u32) << shift
        };
        0xFF00_0000 | channel(16, by[0]) | channel(8, by[1]) | channel(0, by[2])
    };
    let mut fog = pack(attributes.fog_colour);
    if let Some(sun) = sun {
        let [red, green, blue, _] = attributes.sunrise;
        fog = srgb_lerp(sun, fog, pack([red, green, blue]));
    }
    let mut sky = pack(attributes.sky_colour);
    if rain > 0.0 {
        let modifier = 1.0 - rain * 0.5;
        sky = scale(sky, [modifier, modifier, 1.0 - rain * 0.4]);
    }
    if thunder > 0.0 {
        sky = scale(sky, [1.0 - thunder * 0.5; 3]);
    }
    let mixed = srgb_lerp(mix, fog, sky);
    [16, 8, 0].map(|shift| ((mixed >> shift) & 0xFF) as f32 / 255.0)
}
