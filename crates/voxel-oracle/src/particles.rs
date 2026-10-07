//! The particle operations: the registry of types, and a particle ticked in a scene.

use super::*;
use std::sync::Arc;
use voxel_world::particles::{
    AddView, LevelParticles, Particle, ParticleEngine, ParticleResources, ParticleStatus, Spawn,
    SpriteSet,
};

fn bits32(value: f32) -> String {
    format!("{:08x}", value.to_bits())
}

fn bits64(value: f64) -> String {
    format!("{:016x}", value.to_bits())
}

fn parsed64(row: &Value, key: &str) -> Result<f64, String> {
    u64::from_str_radix(string(row, key)?, 16)
        .map(f64::from_bits)
        .map_err(|error| format!("{key}: {error}"))
}

/// `particle_types`: the table of the types, with the reference's own registry's order.
pub(super) fn particle_types_result(request: &Value) -> Result<Value, String> {
    let rows = request["rows"].as_array().ok_or("rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let types: Vec<Value> = voxel_data::particle_type::PARTICLE_TYPES
            .iter()
            .map(|particle| {
                json!({
                    "name": format!("minecraft:{}", particle.name),
                    "override_limiter": particle.overrides_limiter,
                    "simple": particle.simple,
                })
            })
            .collect();
        out.push(json!({"label": string(row, "label")?, "types": types}));
    }
    Ok(json!({"rows": out}))
}

/// What a particle is: its state as bits, and what it would be drawn with at each partial tick.
fn describe(particle: &Particle, world: &ClientWorld, partials: &[f32]) -> Value {
    let (width, height) = particle.size();
    let delta = particle.delta();
    let position = particle.position();
    let previous = particle.previous_position();
    let bounding_box = particle.bounding_box();
    let [r, g, b, alpha] = particle.colour();
    let (o_roll, roll) = particle.roll();
    let drawn: Vec<Value> = partials
        .iter()
        .map(|partial| {
            json!({
                "quad_size": bits32(particle.quad_size_at(*partial)),
                "light": particle.light_coords_at(world, *partial),
            })
        })
        .collect();
    json!({
        "age": particle.age(),
        "lifetime": particle.lifetime(),
        "x": bits64(position.x), "y": bits64(position.y), "z": bits64(position.z),
        "xo": bits64(previous.x), "yo": bits64(previous.y), "zo": bits64(previous.z),
        "xd": bits64(delta.x), "yd": bits64(delta.y), "zd": bits64(delta.z),
        "on_ground": particle.on_ground(),
        "removed": !particle.is_alive(),
        "width": bits32(width), "height": bits32(height),
        "min_x": bits64(bounding_box.min_x), "min_y": bits64(bounding_box.min_y),
        "min_z": bits64(bounding_box.min_z), "max_x": bits64(bounding_box.max_x),
        "max_y": bits64(bounding_box.max_y), "max_z": bits64(bounding_box.max_z),
        "quadSize": bits32(particle.quad_size()),
        "rCol": bits32(r), "gCol": bits32(g), "bCol": bits32(b), "alpha": bits32(alpha),
        "roll": bits32(roll), "oRoll": bits32(o_roll),
        "gravity": bits32(particle.gravity()), "friction": bits32(particle.friction()),
        "sprite": particle.sprite(),
        "drawn": drawn,
    })
}

/// `particle_ticks`: a particle made by its type's provider, seeded, then ticked in a scene.
pub(super) fn particle_ticks_result(request: &Value) -> Result<Value, String> {
    let rows = request["rows"].as_array().ok_or("rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let world = movement_world(row)?;
        let sprites = Arc::new(SpriteSet::new(
            (0..i32_field(row, "sprites")?)
                .map(|index| format!("sprite_{index}"))
                .collect(),
        ));
        let at = (
            parsed64(row, "x")?,
            parsed64(row, "y")?,
            parsed64(row, "z")?,
        );
        let push = (
            parsed64(row, "xa")?,
            parsed64(row, "ya")?,
            parsed64(row, "za")?,
        );
        let random = RandomSource::create(row["seed"].as_i64().ok_or("seed must be an integer")?);
        let mut engine_random = RandomSource::create(
            row["engine_seed"]
                .as_i64()
                .ok_or("engine_seed must be an integer")?,
        );
        let mut particle = match string(row, "particle")? {
            "smoke" => Particle::smoke(random, at, push, &sprites),
            "flame" => Particle::flame(random, at, push, &sprites, &mut engine_random),
            "small_flame" => Particle::small_flame(random, at, push, &sprites, &mut engine_random),
            other => return Err(format!("unknown particle {other}")),
        };
        let partials: Vec<f32> = row["partials"]
            .as_array()
            .ok_or("partials must be an array")?
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .and_then(|text| u32::from_str_radix(text, 16).ok())
                    .map(f32::from_bits)
                    .ok_or_else(|| "partials are f32 bits".to_string())
            })
            .collect::<Result<_, _>>()?;
        let mut states = vec![describe(&particle, &world, &partials)];
        for _ in 0..i32_field(row, "tick_count")? {
            particle.tick(&world);
            states.push(describe(&particle, &world, &partials));
        }
        out.push(json!({"label": string(row, "label")?, "states": states}));
    }
    Ok(json!({"rows": out}))
}

fn status_of(name: &str) -> Result<ParticleStatus, String> {
    match name {
        "ALL" => Ok(ParticleStatus::All),
        "DECREASED" => Ok(ParticleStatus::Decreased),
        "MINIMAL" => Ok(ParticleStatus::Minimal),
        other => Err(format!("unknown particle status {other}")),
    }
}

/// The fold the oracle digests a group with: FNV-1a over 64-bit words.
fn fold(hash: u64, value: u64) -> u64 {
    (hash ^ value).wrapping_mul(0x0000_0100_0000_01b3)
}

/// `particle_engine`: the engine asked for particles by the level and by the packet.
pub(super) fn particle_engine_result(request: &Value) -> Result<Value, String> {
    let rows = request["rows"].as_array().ok_or("rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let world = movement_world(row)?;
        let sprites = SpriteSet::new(
            (0..i32_field(row, "sprites")?)
                .map(|index| format!("sprite_{index}"))
                .collect(),
        );
        let mut resources = ParticleResources::new();
        for particle in voxel_data::particle_type::PARTICLE_TYPES {
            resources.set_sprites(particle.name, sprites.clone());
        }
        let seed_of = |key: &str| {
            row[key]
                .as_i64()
                .ok_or_else(|| format!("{key} must be an integer"))
        };
        let mut engine =
            ParticleEngine::new(RandomSource::create(seed_of("engine_seed")?), resources);
        let mut level_random = RandomSource::create(seed_of("level_seed")?);
        let mut listener_random = RandomSource::create(seed_of("listener_seed")?);
        let base = seed_of("particle_seed_base")?;
        let step = seed_of("particle_seed_step")?;
        let mut made = 0_i64;
        let mut particle_random = || {
            let seed = base.wrapping_add(step.wrapping_mul(made));
            made += 1;
            RandomSource::create(seed)
        };
        let view = AddView {
            camera: voxel_core::math::Vec3::new(
                parsed64(row, "camera_x")?,
                parsed64(row, "camera_y")?,
                parsed64(row, "camera_z")?,
            ),
            status: status_of(string(row, "status")?)?,
        };
        let partials: Vec<f32> = row["partials"]
            .as_array()
            .ok_or("partials must be an array")?
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .and_then(|text| u32::from_str_radix(text, 16).ok())
                    .map(f32::from_bits)
                    .ok_or_else(|| "partials are f32 bits".to_string())
            })
            .collect::<Result<_, _>>()?;
        let events = row["events"].as_array().ok_or("events must be an array")?;
        let mut ticks = Vec::new();
        for tick in 0..i32_field(row, "tick_count")? {
            for event in events {
                if i32_field(event, "tick")? != tick {
                    continue;
                }
                let name = string(event, "particle")?;
                let type_id = voxel_data::particle_type::id(name)
                    .ok_or_else(|| format!("unknown particle {name}"))?;
                let at = (
                    parsed64(event, "x")?,
                    parsed64(event, "y")?,
                    parsed64(event, "z")?,
                );
                let override_limiter = event["override"]
                    .as_bool()
                    .ok_or("override must be a bool")?;
                let always_show = event["always_show"]
                    .as_bool()
                    .ok_or("always_show must be a bool")?;
                match string(event, "type")? {
                    "add" => engine.add_particle(
                        &Spawn {
                            type_id,
                            override_limiter,
                            always_show,
                            at,
                            push: (
                                parsed64(event, "xa")?,
                                parsed64(event, "ya")?,
                                parsed64(event, "za")?,
                            ),
                        },
                        &view,
                        &mut level_random,
                        &mut particle_random,
                    ),
                    "packet" => {
                        let single = |key: &str| -> Result<f32, String> {
                            u32::from_str_radix(string(event, key)?, 16)
                                .map(f32::from_bits)
                                .map_err(|error| format!("{key}: {error}"))
                        };
                        engine.level_particles(
                            &LevelParticles {
                                type_id,
                                override_limiter,
                                always_show,
                                at,
                                spread: (single("x_dist")?, single("y_dist")?, single("z_dist")?),
                                max_speed: single("max_speed")?,
                                count: i32_field(event, "count")?,
                            },
                            &view,
                            (&mut listener_random, &mut level_random),
                            &mut particle_random,
                        );
                    }
                    other => return Err(format!("unknown particle event {other}")),
                }
            }
            let pending = engine.pending_count();
            engine.tick(&world);
            ticks.push(json!({"pending": pending, "particles": engine.particle_count()}));
        }
        let limit =
            usize::try_from(i32_field(row, "describe")?).map_err(|error| error.to_string())?;
        let mut digest = 0xcbf2_9ce4_8422_2325_u64;
        let mut first = Vec::new();
        for particle in engine.particles() {
            let (position, delta) = (particle.position(), particle.delta());
            for value in [
                position.x, position.y, position.z, delta.x, delta.y, delta.z,
            ] {
                digest = fold(digest, value.to_bits());
            }
            digest = fold(digest, i64::from(particle.age()) as u64);
            digest = fold(digest, i64::from(particle.lifetime()) as u64);
            digest = fold(digest, particle.sprite() as u64);
            if first.len() < limit {
                first.push(describe(particle, &world, &partials));
            }
        }
        out.push(json!({
            "label": string(row, "label")?,
            "ticks": ticks,
            "first": first,
            "digest": format!("{digest:x}"),
        }));
    }
    Ok(json!({"rows": out}))
}
