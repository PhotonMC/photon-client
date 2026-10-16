//! The particles the scenarios ask for, made by each type's provider from a row or an event.

use super::*;

/// The particle a row asks for, made by its type's provider with the row's seeds.
pub(super) fn particle_of(
    row: &Value,
    world: &ClientWorld,
    sprites: &Arc<SpriteSet>,
) -> Result<Particle, String> {
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
    let mut spell_random = RandomSource::create(row["spell_seed"].as_i64().unwrap_or(0));
    let color = row["color"].as_i64().unwrap_or(0) as u32;
    let to_color = row["to_color"].as_i64().unwrap_or(0) as u32;
    let float = |key: &str| {
        row[key]
            .as_str()
            .and_then(|text| u32::from_str_radix(text, 16).ok())
            .map_or(0.0, f32::from_bits)
    };
    let motion = (at, push);
    let mut level_random = RandomSource::create(row["level_seed"].as_i64().unwrap_or(0));
    match string(row, "particle")? {
        "portal" => Ok(Particle::portal(
            random,
            motion,
            sprites,
            &mut engine_random,
            false,
        )),
        "reverse_portal" => Ok(Particle::portal(
            random,
            motion,
            sprites,
            &mut engine_random,
            true,
        )),
        "enchant" | "nautilus" => Ok(Particle::fly_towards_provider(
            random,
            motion,
            sprites,
            &mut engine_random,
            false,
        )),
        "vault_connection" => Ok(Particle::fly_towards_provider(
            random,
            motion,
            sprites,
            &mut engine_random,
            true,
        )),
        "ominous_spawning" => Ok(Particle::ominous_spawning(
            random,
            motion,
            sprites,
            (&mut engine_random, &mut level_random),
        )),
        name @ ("underwater" | "crimson_spore" | "warped_spore" | "spore_blossom_air") => Ok(
            Particle::suspended(name, random, at, sprites, &mut engine_random),
        ),
        name @ ("mycelium" | "happy_villager" | "egg_crack" | "composter" | "dolphin") => {
            Ok(Particle::suspended_town(
                name,
                random,
                motion,
                sprites,
                (&mut engine_random, &mut level_random),
            ))
        }
        "dust" => Ok(Particle::dust(
            random,
            motion,
            sprites,
            (color, float("scale").clamp(0.01, 4.0)),
        )),
        "dust_color_transition" => Ok(Particle::dust_transition(
            random,
            motion,
            sprites,
            (color, to_color, float("scale").clamp(0.01, 4.0)),
        )),
        "infested" => Ok(Particle::spell(random, motion, sprites, &mut spell_random)),
        "effect" => Ok(Particle::instant_spell(
            random,
            motion,
            sprites,
            &mut spell_random,
            (color, float("power")),
        )),
        "entity_effect" => Ok(Particle::mob_effect(
            random,
            motion,
            sprites,
            &mut spell_random,
            color,
        )),
        "witch" => Ok(Particle::witch(
            random,
            motion,
            sprites,
            (&mut spell_random, &mut engine_random),
        )),
        "totem_of_undying" => Ok(Particle::totem(random, motion, sprites)),
        "end_rod" => Ok(Particle::end_rod(random, motion, sprites)),
        "vibration" => {
            let destination = photon_world::particles::PositionSource::Block(BlockPos {
                x: i32_field(row, "destination_x")?,
                y: i32_field(row, "destination_y")?,
                z: i32_field(row, "destination_z")?,
            });
            let sprite = sprites.from_random(&mut engine_random);
            Ok(Particle::vibration(
                random,
                at,
                destination,
                i32_field(row, "arrival_in_ticks")?,
                (sprites, sprite),
                world,
            ))
        }
        "shriek" => {
            let data = photon_world::particles::ParticleData::Shriek {
                delay: i32_field(row, "delay")?,
            };
            Particle::hit_provider(
                "shriek",
                world,
                random,
                motion,
                sprites,
                &mut engine_random,
                data,
            )
            .ok_or_else(|| "no shriek".to_owned())
        }
        "trail" => {
            let target = (
                parsed64(row, "target_x")?,
                parsed64(row, "target_y")?,
                parsed64(row, "target_z")?,
            );
            let sprite = sprites.from_random(&mut engine_random);
            Ok(Particle::trail(
                random,
                motion,
                target,
                color,
                i32_field(row, "duration")?,
                (sprites, sprite),
            ))
        }
        "item_crumb" => Ok(Particle::breaking_item(
            random,
            motion,
            (sprites, 0),
            (true, true),
        )),
        "item_crumb_still" => Ok(Particle::breaking_item(
            random,
            motion,
            (sprites, 0),
            (true, false),
        )),
        "sulfur_cube_goo" => Particle::breaking_item_provider(
            "sulfur_cube_goo",
            random,
            motion,
            (photon_world::particles::ParticleData::Plain, sprites),
            (None, &mut engine_random),
        )
        .ok_or_else(|| "no goo".to_owned()),
        name @ ("cherry_leaves" | "pale_oak_leaves" | "tinted_leaves") => {
            Particle::falling_leaves_provider(
                name,
                random,
                at,
                sprites,
                (
                    &mut engine_random,
                    photon_world::particles::ParticleData::Color(color),
                ),
            )
            .ok_or_else(|| format!("unknown particle {name}"))
        }
        name @ ("glow" | "electric_spark" | "scrape" | "wax_on" | "wax_off") => Ok(Particle::glow(
            name,
            random,
            motion,
            sprites,
            &mut engine_random,
        )),
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
        name if name.starts_with("dripping_")
            || name.starts_with("landing_")
            || (name.starts_with("falling_") && name != "falling_dust") =>
        {
            Particle::drip_provider(name, random, at, sprites, &mut engine_random)
                .ok_or_else(|| format!("unknown particle {name}"))
        }
        name => Particle::hit_provider(
            name,
            world,
            random,
            motion,
            sprites,
            &mut engine_random,
            event_data(row, name),
        )
        .ok_or_else(|| format!("unknown particle {name}")),
    }
}

/// What an event's particle carries (`color`, `to_color`, `power` and `scale` as the type reads
/// them, the floats as their bits).
pub(super) fn event_data(event: &Value, name: &str) -> photon_world::particles::ParticleData {
    use photon_world::particles::ParticleData;
    let color = event["color"].as_i64().unwrap_or(0) as u32;
    let float = |key: &str| {
        event[key]
            .as_str()
            .and_then(|text| u32::from_str_radix(text, 16).ok())
            .map_or(0.0, f32::from_bits)
    };
    match name {
        "dust" => ParticleData::Dust {
            color,
            scale: float("scale").clamp(0.01, 4.0),
        },
        "dust_color_transition" => ParticleData::DustTransition {
            from: color,
            to: event["to_color"].as_i64().unwrap_or(0) as u32,
            scale: float("scale").clamp(0.01, 4.0),
        },
        "effect" | "instant_effect" => ParticleData::Spell {
            color,
            power: float("power"),
        },
        "entity_effect" | "flash" => ParticleData::Color(color),
        "dragon_breath" => ParticleData::Power(float("power")),
        "sculk_charge" => ParticleData::Roll(float("roll")),
        "falling_dust" => ParticleData::Block(
            event["block"]
                .as_str()
                .and_then(photon_data::block::registry::block)
                .map_or(photon_world::VANILLA_AIR_BLOCK_STATE_ID, |block| {
                    block.default_state_id()
                }),
        ),
        _ => ParticleData::Plain,
    }
}
