use super::*;

/// A sky vertex, in view-rotated space around the camera.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SkyVertex {
    pub position: [f32; 3],
    /// Atlas coordinate, or negative for an untextured vertex.
    pub uv: [f32; 2],
    pub colour: [f32; 4],
    /// The spherical and cylindrical distances fog is measured over, or
    /// negative for no fog.
    pub fog_distance: [f32; 2],
}

/// How a run of sky vertices blends onto what is below it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkyBlend {
    /// Source alpha over the destination.
    Translucent,
    /// Source times its alpha added to the destination.
    Overlay,
}

/// Everything one frame of sky needs.
#[derive(Debug, Clone)]
pub struct SkyFrame {
    /// The colour the frame is cleared to, which is the fog colour.
    pub clear_colour: [f32; 4],
    pub fog_colour: [f32; 4],
    /// Where fog on the sky is complete, in blocks.
    pub sky_fog_end: f32,
    /// The terrain's fog, in blocks: where the environmental fog starts and
    /// ends over spherical distance, and where the render distance fog
    /// starts and ends over cylindrical distance.
    pub terrain_fog: [f32; 4],
    /// The cloud colour (red, green, blue, alpha, 0 to 1), the height of the
    /// clouds' bottom, and where cloud fog is complete, in blocks.
    pub cloud_colour: [f32; 4],
    pub cloud_height: f32,
    pub cloud_fog_end: f32,
    /// Triangles, three vertices each.
    pub vertices: Vec<SkyVertex>,
    /// Runs of triangles and how each blends.
    pub batches: Vec<(SkyBlend, u32, u32)>,
}

/// Where the sky's sprites sit on the atlas.
pub struct SkySprites {
    pub sun: [f32; 4],
    pub moon: [[f32; 4]; 8],
}

/// The inputs a frame of sky reads besides the attributes.
pub struct SkyView {
    /// The camera's forward direction.
    pub forward: [f32; 3],
    /// The camera's eye height above the horizon (sea level on a normal world).
    pub eye_above_horizon: f64,
    pub eye_in_water: bool,
    /// The effective render distance in chunks.
    pub render_distance: i32,
    /// The cloud range option, in chunks.
    pub cloud_range: i32,
    /// The level's rain and thunder at this frame.
    pub weather: Weather,
    /// How far rain has pulled the fog in, 0 to 1.
    pub rain_fog: f32,
}

/// The moon phase sprite names, in the phases' order.
pub fn moon_sprites() -> impl Iterator<Item = String> {
    MOON_PHASES
        .iter()
        .map(|phase| format!("minecraft:environment/celestial/moon/{phase}"))
}

/// Build one frame of sky from the blended attributes.
pub fn build_frame(
    attributes: &Attributes,
    view: &SkyView,
    stars: &[[[f32; 3]; 4]],
    sprites: &SkySprites,
    skybox: Skybox,
) -> SkyFrame {
    let sky_colour = attributes.get(Attribute::SkyColor).color();
    let sunrise = attributes.get(Attribute::SunriseSunsetColor).color();
    let sun_angle = attributes.get(Attribute::SunAngle).float() * std::f32::consts::PI / 180.0;
    let moon_angle = attributes.get(Attribute::MoonAngle).float() * std::f32::consts::PI / 180.0;
    let star_angle = attributes.get(Attribute::StarAngle).float() * std::f32::consts::PI / 180.0;
    let star_brightness = attributes.get(Attribute::StarBrightness).float();
    let moon_phase = usize::from(attributes.get(Attribute::MoonPhase).phase()).min(7);
    let render_blocks = (view.render_distance * 16) as f32;

    // AtmosphericFogEnvironment.getBaseColor, then the fog renderer's scaling;
    // under water the water fog colour takes its place.
    let fog = if view.eye_in_water {
        attributes.get(Attribute::WaterFogColor).color()
    } else {
        let mut fog = attributes.get(Attribute::FogColor).color();
        if view.render_distance >= 4 {
            let sun_x = if mth_sin_f64(f64::from(sun_angle)) > 0.0 {
                -1.0
            } else {
                1.0
            };
            let looking = view.forward[0] * sun_x;
            if looking > 0.0 {
                let alpha = channel(sunrise, 24) as f32 / 255.0;
                if alpha > 0.0 {
                    fog = srgb_lerp(looking * alpha, fog, sunrise | 0xFF00_0000);
                }
            }
        }
        let sky_fog_end = (attributes.get(Attribute::SkyFogEndDistance).float() / 16.0)
            .min(view.render_distance as f32);
        let delta = sky_fog_end / 32.0;
        let mix = if delta < 0.0 {
            0.25
        } else if delta > 1.0 {
            1.0
        } else {
            0.25 + delta * 0.75
        };
        let mix = 1.0 - (f64::from(mix)).powf(0.25) as f32;
        // The sky colour the fog blends towards is darkened by the weather.
        let scale = |colour: u32, red: f32, green: f32, blue: f32| {
            argb(
                channel(colour, 24),
                ((channel(colour, 16) as f32 * red) as i32).clamp(0, 255),
                ((channel(colour, 8) as f32 * green) as i32).clamp(0, 255),
                ((channel(colour, 0) as f32 * blue) as i32).clamp(0, 255),
            )
        };
        let mut darkened = sky_colour;
        if view.weather.rain > 0.0 {
            let modifier = 1.0 - view.weather.rain * 0.5;
            darkened = scale(darkened, modifier, modifier, 1.0 - view.weather.rain * 0.4);
        }
        if view.weather.thunder > 0.0 {
            let modifier = 1.0 - view.weather.thunder * 0.5;
            darkened = scale(darkened, modifier, modifier, modifier);
        }
        srgb_lerp(mix, fog, darkened)
    };
    let fog_colour = {
        let [r, g, b, _] = unpack(fog);
        [r, g, b, 1.0]
    };
    // FogRenderer.setupFog: the water or atmospheric environment's
    // distances, then the render distance's own band. The player's water
    // vision is taken as zero, so under water the end is its lowest quarter.
    let (environmental_start, environmental_end, sky_fog_end, cloud_fog_end) = if view.eye_in_water
    {
        let end = attributes.get(Attribute::WaterFogEndDistance).float() * 0.25;
        (
            attributes.get(Attribute::WaterFogStartDistance).float(),
            end,
            end,
            end,
        )
    } else {
        let end = attributes.get(Attribute::FogEndDistance).float();
        (
            attributes.get(Attribute::FogStartDistance).float() - 160.0 * view.rain_fog,
            end.min(96.0).max(end - 256.0 * view.rain_fog),
            render_blocks.min(attributes.get(Attribute::SkyFogEndDistance).float()),
            ((view.cloud_range * 16) as f32)
                .min(attributes.get(Attribute::CloudFogEndDistance).float()),
        )
    };
    let cloud_colour = unpack(attributes.get(Attribute::CloudColor).color());
    let render_fog_span = (render_blocks / 10.0).clamp(4.0, 64.0);
    let terrain_fog = [
        environmental_start,
        environmental_end,
        render_blocks - render_fog_span,
        render_blocks,
    ];

    let mut frame = SkyFrame {
        clear_colour: fog_colour,
        fog_colour,
        sky_fog_end,
        terrain_fog,
        cloud_colour,
        cloud_height: attributes.get(Attribute::CloudHeight).float(),
        cloud_fog_end,
        vertices: Vec::new(),
        batches: Vec::new(),
    };
    if skybox != Skybox::Overworld {
        return frame;
    }

    let untextured = [-1.0, -1.0];
    let push_batch = |frame: &mut SkyFrame, blend: SkyBlend, vertices: Vec<SkyVertex>| {
        if vertices.is_empty() {
            return;
        }
        let first = frame.vertices.len() as u32;
        let count = vertices.len() as u32;
        frame.vertices.extend(vertices);
        match frame.batches.last_mut() {
            Some((last_blend, last_first, last_count))
                if *last_blend == blend && *last_first + *last_count == first =>
            {
                *last_count += count;
            }
            _ => frame.batches.push((blend, first, count)),
        }
    };
    let fan = |centre: SkyVertex, ring: &[SkyVertex]| -> Vec<SkyVertex> {
        ring.windows(2)
            .flat_map(|pair| [centre, pair[0], pair[1]])
            .collect()
    };
    let quad = |corners: [SkyVertex; 4]| -> Vec<SkyVertex> {
        vec![
            corners[0], corners[1], corners[2], corners[0], corners[2], corners[3],
        ]
    };
    let disc = |height: f32, colour: [f32; 4], lift: f32| -> Vec<SkyVertex> {
        let x = height.signum() * 512.0;
        let point = |position: [f32; 3]| SkyVertex {
            position: [position[0], position[1] + lift, position[2]],
            uv: untextured,
            colour,
            fog_distance: [
                (position[0] * position[0] + position[1] * position[1] + position[2] * position[2])
                    .sqrt(),
                (position[0] * position[0] + position[2] * position[2])
                    .sqrt()
                    .max(position[1].abs()),
            ],
        };
        let centre = point([0.0, height, 0.0]);
        let ring: Vec<SkyVertex> = (-180..=180)
            .step_by(45)
            .map(|degrees| {
                let angle = degrees as f32 * (std::f32::consts::PI / 180.0);
                point([
                    x * mth_cos_f64(f64::from(angle)),
                    height,
                    512.0 * mth_sin_f64(f64::from(angle)),
                ])
            })
            .collect();
        fan(centre, &ring)
    };

    // The sky disc, in the sky colour, fogged with distance.
    push_batch(
        &mut frame,
        SkyBlend::Translucent,
        disc(16.0, unpack(sky_colour | 0xFF00_0000), 0.0),
    );

    // The sunrise fan, stretched by its colour's alpha.
    let sunrise_alpha = channel(sunrise, 24) as f32 / 255.0;
    if sunrise_alpha > 0.001 {
        let flip = if mth_sin_f64(f64::from(sun_angle)) < 0.0 {
            180.0_f32
        } else {
            0.0
        };
        let pose = mul(
            mul(
                rotate_x(90.0_f32.to_radians()),
                rotate_z((flip + 90.0).to_radians()),
            ),
            scale(1.0, 1.0, sunrise_alpha),
        );
        let [r, g, b, a] = unpack(sunrise);
        let point = |position: [f32; 3], alpha: f32| SkyVertex {
            position: transform(pose, position),
            uv: untextured,
            colour: [r, g, b, a * alpha],
            fog_distance: [-1.0, -1.0],
        };
        let centre = point([0.0, 100.0, 0.0], 1.0);
        let ring: Vec<SkyVertex> = (0..=16)
            .map(|i| {
                let angle = i as f32 * std::f32::consts::TAU / 16.0;
                let (s, c) = (mth_sin_f64(f64::from(angle)), mth_cos_f64(f64::from(angle)));
                point([s * 120.0, c * 120.0, -c * 40.0], 0.0)
            })
            .collect();
        push_batch(&mut frame, SkyBlend::Translucent, fan(centre, &ring));
    }

    let base = rotate_y((-90.0_f32).to_radians());
    let celestial = |angle: f32, size: f32, uv: [[f32; 2]; 4]| -> Vec<SkyVertex> {
        let pose = mul(
            mul(mul(base, rotate_x(angle)), translate(0.0, 100.0, 0.0)),
            scale(size, 1.0, size),
        );
        let corners = [
            [-1.0, 0.0, -1.0],
            [1.0, 0.0, -1.0],
            [1.0, 0.0, 1.0],
            [-1.0, 0.0, 1.0],
        ];
        let vertex = |i: usize| SkyVertex {
            position: transform(pose, corners[i]),
            uv: uv[i],
            // Rain fades the sun and the moon.
            colour: [1.0, 1.0, 1.0, 1.0 - view.weather.rain],
            fog_distance: [-1.0, -1.0],
        };
        quad([vertex(0), vertex(1), vertex(2), vertex(3)])
    };
    let [u0, v0, u1, v1] = sprites.sun;
    push_batch(
        &mut frame,
        SkyBlend::Overlay,
        celestial(sun_angle, 30.0, [[u0, v0], [u1, v0], [u1, v1], [u0, v1]]),
    );
    let [u0, v0, u1, v1] = sprites.moon[moon_phase];
    push_batch(
        &mut frame,
        SkyBlend::Overlay,
        celestial(moon_angle, 20.0, [[u1, v1], [u0, v1], [u0, v0], [u1, v0]]),
    );

    if star_brightness > 0.0 {
        let pose = mul(base, rotate_x(star_angle));
        let colour = [star_brightness; 4];
        let mut vertices = Vec::with_capacity(stars.len() * 6);
        for star in stars {
            let vertex = |i: usize| SkyVertex {
                position: transform(pose, star[i]),
                uv: untextured,
                colour,
                fog_distance: [-1.0, -1.0],
            };
            vertices.extend(quad([vertex(0), vertex(1), vertex(2), vertex(3)]));
        }
        push_batch(&mut frame, SkyBlend::Overlay, vertices);
    }

    if view.eye_above_horizon < 0.0 && !view.eye_in_water {
        // The dark disc sits lifted by twelve, but its fog is measured from
        // where it was built.
        push_batch(
            &mut frame,
            SkyBlend::Translucent,
            disc(-16.0, [0.0, 0.0, 0.0, 1.0], 12.0),
        );
    }
    frame
}
