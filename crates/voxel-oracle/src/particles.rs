//! The particle operations: the registry of types, and a particle ticked in a scene.

use super::*;
use std::sync::Arc;
use voxel_world::particles::{
    AddView, LevelParticles, Particle, ParticleEngine, ParticleResources, ParticleStatus, Spawn,
    SpriteSet, TerrainMaterials,
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

/// The particle a row asks for, made by its type's provider with the row's seeds.
fn particle_of(row: &Value, sprites: &Arc<SpriteSet>) -> Result<Particle, String> {
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
    match string(row, "particle")? {
        "smoke" => Ok(Particle::smoke(random, at, push, sprites, 1.0)),
        "large_smoke" => Ok(Particle::smoke(random, at, push, sprites, 2.5)),
        "white_smoke" => Ok(Particle::white_smoke(random, at, push, sprites)),
        "ash" => Ok(Particle::ash(random, at, sprites)),
        "white_ash" => Ok(Particle::white_ash(random, at, sprites, &mut engine_random)),
        "dust_plume" => Ok(Particle::dust_plume(random, at, push, sprites)),
        "soul" => Ok(Particle::soul(random, at, push, sprites, false)),
        "sculk_soul" => Ok(Particle::soul(random, at, push, sprites, true)),
        "campfire_cosy_smoke" => Ok(Particle::campfire_smoke(
            random,
            at,
            push,
            (sprites, false),
            &mut engine_random,
        )),
        "campfire_signal_smoke" => Ok(Particle::campfire_smoke(
            random,
            at,
            push,
            (sprites, true),
            &mut engine_random,
        )),
        "cloud" => Ok(Particle::player_cloud(random, at, push, sprites, false)),
        "sneeze" => Ok(Particle::player_cloud(random, at, push, sprites, true)),
        "bubble" => Ok(Particle::bubble(
            random,
            (at, push),
            sprites,
            &mut engine_random,
        )),
        "bubble_column_up" => Ok(Particle::bubble_column_up(
            random,
            (at, push),
            sprites,
            &mut engine_random,
        )),
        "bubble_pop" => Ok(Particle::bubble_pop(random, (at, push), sprites)),
        "rain" => Ok(Particle::rain(random, at, sprites, &mut engine_random)),
        "splash" => Ok(Particle::splash(
            random,
            (at, push),
            sprites,
            &mut engine_random,
        )),
        "fishing" => Ok(Particle::wake(random, (at, push), sprites)),
        "current_down" => Ok(Particle::current_down(
            random,
            at,
            sprites,
            &mut engine_random,
        )),
        "squid_ink" => Ok(Particle::squid_ink(random, (at, push), sprites, false)),
        "glow_squid_ink" => Ok(Particle::squid_ink(random, (at, push), sprites, true)),
        "flame" => Ok(Particle::flame(
            random,
            at,
            push,
            sprites,
            &mut engine_random,
        )),
        "small_flame" => Ok(Particle::small_flame(
            random,
            at,
            push,
            sprites,
            &mut engine_random,
        )),
        other => Err(format!("unknown particle {other}")),
    }
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
        let mut particle = particle_of(row, &sprites)?;
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
    use voxel_render::particles::{ParticleCamera, extract};
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
        let mut particle = particle_of(row, &sprites)?;
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
            orientation: voxel_render::entity::name_tag::camera_orientation(
                single("y_rot")?,
                single("x_rot")?,
            ),
            position: voxel_core::math::Vec3::new(
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
        let partials = f32_bits(&row["partials"])?;
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
                        &world,
                        &Spawn {
                            type_id,
                            block_state: None,
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
                                block_state: None,
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
            engine.tick(&world, &view, &mut level_random, &mut particle_random);
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

/// What the oracle's flakes are made with: one texture, and the tints the scenarios ask for.
#[derive(Debug)]
struct OracleTerrain;

impl TerrainMaterials for OracleTerrain {
    fn particle_texture(&self, _state: i32) -> String {
        "block/stone".into()
    }

    fn tint(
        &self,
        _world: &ClientWorld,
        state: i32,
        _pos: voxel_core::binary::BlockPos,
    ) -> Option<i32> {
        // The tint sources of these blocks do not read the biome (`BlockColors`).
        match voxel_world::block::state_or_air(i64::from(state))
            .block()
            .name
        {
            "minecraft:spruce_leaves" => Some(-10_380_959),
            "minecraft:birch_leaves" => Some(-8_345_771),
            // `grassBlock().colorAsTerrainParticle`: none.
            "minecraft:grass_block" => Some(-1),
            _ => None,
        }
    }
}

fn state_of(row: &Value) -> Result<i32, String> {
    let described = &row["state"];
    let name = string(described, "block")?;
    let definition =
        voxel_data::block::registry::block(name).ok_or_else(|| format!("unknown block {name}"))?;
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
        .map(|state| state as i32)
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
        let pos = voxel_core::binary::BlockPos::new(
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
                        "DOWN" => voxel_core::direction::Direction::Down,
                        "UP" => voxel_core::direction::Direction::Up,
                        "NORTH" => voxel_core::direction::Direction::North,
                        "SOUTH" => voxel_core::direction::Direction::South,
                        "WEST" => voxel_core::direction::Direction::West,
                        "EAST" => voxel_core::direction::Direction::East,
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
                let view = voxel_world::particles::AddView {
                    camera: voxel_core::math::Vec3::new(0.0, 0.0, 0.0),
                    status: voxel_world::particles::ParticleStatus::All,
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
