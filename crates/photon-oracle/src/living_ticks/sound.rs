use super::*;
use photon_game::sounds::instances::{
    BEE_AGGRESSIVE, BEE_FLYING, BeeView, Buzz, GUARDIAN_ATTACK, MINECART, MinecartView,
    SNIFFER_DIGGING, Setup, Voice, horizontal_speed, tick_guardian_attack, tick_minecart,
    tick_sniffer_digging,
};
use photon_world::EntityView;

/// The namespace of the event names the instances keep without it.
const NAMESPACE: &str = "minecraft:";

/// The sound a row follows: what the entity's own sound instance has become.
pub(crate) enum Following {
    Minecart(Voice),
    Bee(Buzz, Voice),
    Guardian(Voice),
    Sniffer(Voice),
}

/// How each sound is made.
fn setup_of(following: &Following) -> Setup {
    match following {
        Following::Minecart(_) => MINECART,
        Following::Bee(buzz, _) if buzz.aggressive => BEE_AGGRESSIVE,
        Following::Bee(..) => BEE_FLYING,
        Following::Guardian(_) => GUARDIAN_ATTACK,
        Following::Sniffer(_) => SNIFFER_DIGGING,
    }
}

/// The sound an entity starts when it is added (`postAddEntitySoundInstance`, the guardian and
/// sniffer events): its volume then, whether it loops and whether it may start silent.
fn made(entity: EntityView<'_>, row: &Value) -> Result<Following, String> {
    let name = photon_data::entity_type::name(entity.type_id).ok_or("no type")?;
    Ok(match name {
        "minecraft:bee" => Following::Bee(
            Buzz::new(row["aggressive_start"].as_bool().unwrap_or(false)),
            Voice::new(0.0),
        ),
        "minecraft:guardian" | "minecraft:elder_guardian" => Following::Guardian(Voice::new(1.0)),
        "minecraft:sniffer" => Following::Sniffer(Voice::new(1.0)),
        _ => Following::Minecart(Voice::new(0.0)),
    })
}

/// The tick of whatever the entity's sound is; whether a bee's buzz was handed over.
fn tick_following(following: &mut Following, entity: EntityView<'_>, world: &ClientWorld) -> bool {
    let position = entity.position();
    let motion = entity.motion();
    let speed = horizontal_speed(motion.0, motion.2);
    match following {
        Following::Minecart(voice) => tick_minecart(
            voice,
            &MinecartView {
                removed: false,
                position,
                speed,
                off_rail: world.minecart_off_rail(entity),
                runs_normally: true,
            },
        ),
        Following::Bee(buzz, voice) => {
            return buzz.tick(
                voice,
                &BeeView {
                    removed: false,
                    position,
                    speed,
                    baby: entity.is_baby(),
                    angry: entity.is_angry(world.game_time()),
                },
            );
        }
        Following::Guardian(voice) => {
            let scale = entity.guardian().map_or(0.0, |g| {
                g.attack_animation_scale(0.0, entity.guardian_attack_duration())
            });
            tick_guardian_attack(voice, false, position, scale);
        }
        Following::Sniffer(voice) => tick_sniffer_digging(
            voice,
            false,
            position,
            entity.sniffer().is_some_and(|s| s.can_play_digging_sound()),
        ),
    }
    false
}

pub(crate) fn sound_extra(
    following: &mut Option<Following>,
    entity: EntityView<'_>,
    described: &mut Value,
    (world, row, tick): (&ClientWorld, &Value, i32),
) -> Result<(), String> {
    // A sound that starts later (an event brings it) is not asked before.
    if row["sound_from"]
        .as_i64()
        .is_some_and(|from| i64::from(tick) < from)
    {
        return Ok(());
    }
    let following = match following {
        Some(following) => following,
        slot => {
            let mut sound = made(entity, row)?;
            // A minecart's and a bee's constructors place the sound where they are.
            if let Following::Minecart(voice) | Following::Bee(_, voice) = &mut sound {
                voice.follow(entity.position());
            }
            slot.insert(sound)
        }
    };
    let position = entity.position();
    let handed_over = tick_following(following, entity, world);
    let setup = setup_of(following);
    let (Following::Minecart(voice)
    | Following::Bee(_, voice)
    | Following::Guardian(voice)
    | Following::Sniffer(voice)) = following;
    let voice = *voice;
    described["sound_event"] = json!(format!("{NAMESPACE}{}", setup.event));
    // `stop` ends the looping too.
    described["sound_loops"] = json!(setup.looping && !voice.stopped);
    described["sound_silent_start"] = json!(setup.can_start_silent);
    described["sound_can_play"] = json!(true);
    described["sound_volume"] = json!(hex(voice.volume));
    described["sound_pitch"] = json!(hex(voice.pitch));
    for (key, value) in [
        ("x", voice.position.0),
        ("y", voice.position.1),
        ("z", voice.position.2),
    ] {
        described[format!("sound_{key}")] = json!(format!("{:016x}", value.to_bits()));
    }
    described["sound_stopped"] = json!(voice.stopped);
    described["sound_handed_over"] = json!(handed_over);
    // The buzz the bee handed over to starts at its place and ticks from the next tick on.
    if handed_over && let Following::Bee(buzz, _) = following {
        let mut next = Voice::new(0.0);
        next.follow(position);
        *following = Following::Bee(Buzz::new(!buzz.aggressive), next);
    }
    Ok(())
}
