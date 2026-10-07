//! The particle operations: the registry of types, and a particle ticked in a scene.

use super::*;
use std::sync::Arc;
use voxel_world::particles::{Particle, SpriteSet};

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
