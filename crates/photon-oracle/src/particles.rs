//! The particle operations: the registry of types, and a particle ticked in a scene.

use super::*;
use std::sync::Arc;
mod factory;
use factory::*;
mod digest;
mod events;
mod extras;
mod terrain;
use digest::digest_group;
use terrain::OracleTerrain;
mod pickup;
use photon_world::particles::{
    AddView, LevelEventContext, LevelParticles, Particle, ParticleEngine, ParticleResources,
    ParticleStatus, Spawn, SpriteSet, TerrainMaterials,
};
pub(super) use pickup::item_pickup_result;

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
        let types: Vec<Value> = photon_data::particle_type::PARTICLE_TYPES
            .iter()
            .map(|particle| {
                json!({
                    "name": format!("minecraft:{}", particle.name),
                    "override_limiter": particle.overrides_limiter,
                    "simple": particle.simple,
                    "block_state": particle.block_state,
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

/// An array of numbers given as the bits of an `f32` each.
fn f32_bits(list: &Value) -> Result<Vec<f32>, String> {
    list.as_array()
        .ok_or("f32 bits must be an array")?
        .iter()
        .map(|value| {
            value
                .as_str()
                .and_then(|text| u32::from_str_radix(text, 16).ok())
                .map(f32::from_bits)
                .ok_or_else(|| "f32 bits are hex text".to_string())
        })
        .collect()
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
        let mut particle = particle_of(row, &world, &sprites)?;
        let partials = f32_bits(&row["partials"])?;
        let mut states = vec![describe(&particle, &world, &partials)];
        for _ in 0..i32_field(row, "tick_count")? {
            particle.tick(&world);
            states.push(describe(&particle, &world, &partials));
        }
        out.push(json!({"label": string(row, "label")?, "states": states}));
    }
    Ok(json!({"rows": out}))
}

/// `particle_quads`: a particle ticked, then extracted for a camera as the renderer writes it.
pub(super) fn particle_quads_result(request: &Value) -> Result<Value, String> {
    use photon_render::particles::{ParticleCamera, extract};
    let rows = request["rows"].as_array().ok_or("rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let world = movement_world(row)?;
        let uvs = row["sprite_uvs"]
            .as_array()
            .ok_or("sprite_uvs must be an array")?;
        let sprites = Arc::new(SpriteSet::new(
            (0..uvs.len())
                .map(|index| format!("sprite_{index}"))
                .collect(),
        ));
        let mut particle = particle_of(row, &world, &sprites)?;
        for _ in 0..i32_field(row, "tick_count")? {
            particle.tick(&world);
        }
        let single = |key: &str| -> Result<f32, String> {
            u32::from_str_radix(string(row, key)?, 16)
                .map(f32::from_bits)
                .map_err(|error| format!("{key}: {error}"))
        };
        if row.get("roll").is_some() {
            particle.set_roll(single("o_roll")?, single("roll")?);
        }
        let camera = ParticleCamera {
            orientation: photon_render::entity::name_tag::camera_orientation(
                single("y_rot")?,
                single("x_rot")?,
            ),
            position: photon_core::math::Vec3::new(
                parsed64(row, "camera_x")?,
                parsed64(row, "camera_y")?,
                parsed64(row, "camera_z")?,
            ),
        };
        let partial = single("partial")?;
        let sprite_uvs = f32_bits(&uvs[particle.sprite()])?;
        let light = particle.light_coords_at(&world, partial);
        let vertices: Vec<Value> = extract(
            &particle,
            &camera,
            partial,
            (
                [sprite_uvs[0], sprite_uvs[1], sprite_uvs[2], sprite_uvs[3]],
                light,
            ),
        )
        .iter()
        .map(|vertex| {
            json!({
                "x": bits32(vertex.position[0]), "y": bits32(vertex.position[1]),
                "z": bits32(vertex.position[2]), "u": bits32(vertex.uv[0]),
                "v": bits32(vertex.uv[1]), "color": vertex.color, "light": vertex.light,
            })
        })
        .collect();
        out.push(json!({"label": string(row, "label")?, "vertices": vertices}));
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

/// `particle_engine`: the engine asked for particles by the level and by the packet.
pub(super) fn particle_engine_result(request: &Value) -> Result<Value, String> {
    let rows = request["rows"].as_array().ok_or("rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let mut world = movement_world(row)?;
        extras::place(&mut world, row)?;
        let sprites = SpriteSet::new(
            (0..i32_field(row, "sprites")?)
                .map(|index| format!("sprite_{index}"))
                .collect(),
        );
        let mut resources = ParticleResources::new();
        for particle in photon_data::particle_type::PARTICLE_TYPES {
            resources.set_sprites(particle.name, sprites.clone());
        }
        // The tints a block's dust takes (`BlockColors`), as the flakes' rows know them.
        resources.set_terrain(Arc::new(OracleTerrain));
        let seed_of = |key: &str| {
            row[key]
                .as_i64()
                .ok_or_else(|| format!("{key} must be an integer"))
        };
        resources.set_spell_random(RandomSource::create(
            row["spell_seed"].as_i64().unwrap_or(0),
        ));
        let engine_random = RandomSource::create(seed_of("engine_seed")?);
        let mut engine = events::engine_with_songs(engine_random, resources, row);
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
            camera: photon_core::math::Vec3::new(
                parsed64(row, "camera_x")?,
                parsed64(row, "camera_y")?,
                parsed64(row, "camera_z")?,
            ),
            status: status_of(string(row, "status")?)?,
        };
        let partials = f32_bits(&row["partials"])?;
        let events = row["events"].as_array().ok_or("events must be an array")?;
        let mut ticks = Vec::new();
        for tick in 0..i32_field(row, "tick_count")? {
            let mut sounds = Vec::new();
            for event in events {
                if i32_field(event, "tick")? != tick {
                    continue;
                }
                if string(event, "type")? == "animate_tick" {
                    let mut context = LevelEventContext::new(
                        &world,
                        &view,
                        &mut level_random,
                        &mut particle_random,
                    );
                    sounds.extend(events::animate_tick(event, &mut engine, &mut context)?);
                    continue;
                }
                // A level or block event goes through the level as the world queues it.
                if events::queues(event, &mut world)? {
                    let randoms = (&mut level_random, &mut particle_random);
                    sounds.extend(events::play_queued(
                        (&mut engine, &mut world),
                        &view,
                        randoms,
                    ));
                    continue;
                }
                let name = string(event, "particle")?;
                let type_id = photon_data::particle_type::id(name)
                    .ok_or_else(|| format!("unknown particle {name}"))?;
                let at = (
                    parsed64(event, "x")?,
                    parsed64(event, "y")?,
                    parsed64(event, "z")?,
                );
                let data = event_data(event, name);
                let override_limiter = event["override"]
                    .as_bool()
                    .ok_or("override must be a bool")?;
                let always_show = event["always_show"]
                    .as_bool()
                    .ok_or("always_show must be a bool")?;
                match string(event, "type")? {
                    "add" => engine.add_particle(
                        &world,
                        &Spawn {
                            type_id,
                            data,
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
                            &world,
                            &LevelParticles {
                                type_id,
                                data,
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
            let pending = events::tick_block_entities(&mut world, row, &engine);
            // What the block entities asked of the level as they ticked.
            let randoms = (&mut level_random, &mut particle_random);
            sounds.extend(events::play_queued(
                (&mut engine, &mut world),
                &view,
                randoms,
            ));
            let mut context =
                LevelEventContext::new(&world, &view, &mut level_random, &mut particle_random);
            engine.tick_explosions(&mut context);
            let ticked = engine.tick(&world, &view, &mut level_random, &mut particle_random);
            sounds.extend(events::ticked_lines(&ticked, (&world, row)));
            ticks.push(json!({
                "pending": pending,
                "particles": engine.particle_count(),
                "sounds": sounds,
                "emitters": engine.emitter_count(),
            }));
        }
        let limit =
            usize::try_from(i32_field(row, "describe")?).map_err(|error| error.to_string())?;
        let (digest, first) = digest_group(&engine, (&world, &partials), limit);
        out.push(json!({
            "label": string(row, "label")?,
            "ticks": ticks,
            "first": first,
            "digest": digest,
        }));
    }
    Ok(json!({"rows": out}))
}

fn state_of(row: &Value) -> Result<BlockStateId, String> {
    let described = &row["state"];
    let name = string(described, "block")?;
    let definition =
        photon_data::block::registry::block(name).ok_or_else(|| format!("unknown block {name}"))?;
    let mut chosen = std::collections::BTreeMap::new();
    if let Some(properties) = described["properties"].as_object() {
        for (key, value) in properties {
            chosen.insert(
                key.clone(),
                value
                    .as_str()
                    .ok_or("property values are strings")?
                    .to_owned(),
            );
        }
    }
    definition
        .state_id(&chosen)
        .ok_or_else(|| format!("bad properties for {name}"))
}

/// `particle_terrain`: a flake made by a provider and ticked, the flakes of a block breaking, and the
/// flake of one being broken.
pub(super) fn particle_terrain_result(request: &Value) -> Result<Value, String> {
    let rows = request["rows"].as_array().ok_or("rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let world = movement_world(row)?;
        let state = state_of(row)?;
        let pos = photon_core::binary::BlockPos::new(
            i32_field(row, "pos_x")?,
            i32_field(row, "pos_y")?,
            i32_field(row, "pos_z")?,
        );
        let partials = f32_bits(&row["partials"])?;
        let sprite_uv = f32_bits(&row["sprite_uv"])?;
        let materials = OracleTerrain;
        let mut entry = serde_json::Map::new();
        entry.insert("label".into(), json!(string(row, "label")?));
        let seeds = {
            let mut next = 0_i64;
            move || {
                next += 1;
                RandomSource::create(next)
            }
        };
        match string(row, "mode")? {
            "single" => {
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
                let random =
                    RandomSource::create(row["seed"].as_i64().ok_or("seed must be an integer")?);
                let mut engine_random = RandomSource::create(
                    row["engine_seed"]
                        .as_i64()
                        .ok_or("engine_seed must be an integer")?,
                );
                let env = (&world, &materials as &dyn TerrainMaterials);
                let made = match string(row, "provider")? {
                    "block" => Particle::terrain_provider(random, (at, push), state, env),
                    "crumbling" => Particle::crumbling_provider(
                        random,
                        (at, push),
                        state,
                        env,
                        &mut engine_random,
                    ),
                    "dust_pillar" => Particle::dust_pillar_provider(
                        random,
                        (at, push),
                        state,
                        env,
                        &mut engine_random,
                    ),
                    other => return Err(format!("unknown provider {other}")),
                };
                entry.insert("made".into(), json!(made.is_some()));
                let mut states = Vec::new();
                if let Some(mut particle) = made {
                    let describe_terrain = |particle: &Particle| {
                        let mut described = describe(particle, &world, &partials);
                        let window = particle.sprite_window().unwrap_or([0.0, 1.0, 0.0, 1.0]);
                        // `TextureAtlasSprite.getU` and `getV`: a place within the sprite.
                        let along =
                            |low: f32, high: f32, fraction: f32| low + (high - low) * fraction;
                        let [u0, u1, v0, v1] =
                            [sprite_uv[0], sprite_uv[1], sprite_uv[2], sprite_uv[3]];
                        described["uvs"] = json!([
                            bits32(along(u0, u1, window[0])),
                            bits32(along(u0, u1, window[1])),
                            bits32(along(v0, v1, window[2])),
                            bits32(along(v0, v1, window[3])),
                        ]);
                        described
                            .as_object_mut()
                            .expect("an object")
                            .remove("sprite");
                        described
                    };
                    states.push(describe_terrain(&particle));
                    for _ in 0..i32_field(row, "tick_count")? {
                        particle.tick(&world);
                        states.push(describe_terrain(&particle));
                    }
                }
                entry.insert("states".into(), json!(states));
            }
            "destroy" | "breaking" => {
                let mut resources = ParticleResources::new();
                resources.set_terrain(std::sync::Arc::new(materials));
                let mut engine = ParticleEngine::new(RandomSource::create(0), resources);
                if string(row, "mode")? == "destroy" {
                    engine.add_destroy_block_effect(&world, (pos, state), seeds);
                } else {
                    let direction = match string(row, "direction")? {
                        "DOWN" => photon_core::direction::Direction::Down,
                        "UP" => photon_core::direction::Direction::Up,
                        "NORTH" => photon_core::direction::Direction::North,
                        "SOUTH" => photon_core::direction::Direction::South,
                        "WEST" => photon_core::direction::Direction::West,
                        "EAST" => photon_core::direction::Direction::East,
                        other => return Err(format!("unknown direction {other}")),
                    };
                    let mut level_random = RandomSource::create(
                        row["level_seed"]
                            .as_i64()
                            .ok_or("level_seed must be an integer")?,
                    );
                    engine.add_breaking_block_effect(
                        &world,
                        (pos, direction),
                        &mut level_random,
                        seeds,
                    );
                }
                let breaking = string(row, "mode")? == "breaking";
                // The engine keeps them until its next tick: tick it once with nothing to move them.
                let view = photon_world::particles::AddView {
                    camera: photon_core::math::Vec3::new(0.0, 0.0, 0.0),
                    status: photon_world::particles::ParticleStatus::All,
                };
                engine.tick(&world, &view, &mut RandomSource::create(0), || {
                    RandomSource::create(0)
                });
                let made: Vec<Value> = engine
                    .particles()
                    .map(|particle| {
                        let position = particle.position();
                        let mut out = json!({"x": bits64(position.x), "y": bits64(position.y), "z": bits64(position.z)});
                        if breaking {
                            let (width, height) = particle.size();
                            out["width"] = json!(bits32(width));
                            out["height"] = json!(bits32(height));
                        }
                        out
                    })
                    .collect();
                entry.insert("particles".into(), json!(made));
            }
            other => return Err(format!("unknown terrain mode {other}")),
        }
        out.push(Value::Object(entry));
    }
    Ok(json!({"rows": out}))
}
