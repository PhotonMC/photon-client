use super::*;

/// `axolotl_model`: an axolotl's model posed and drawn, an adult from its blend factors, a baby
/// from the animation states it runs.
pub(super) fn axolotl_model_result(request: &Value) -> Result<Value, String> {
    use photon_render::entity::mob_model::{AxolotlRenderState, MobModel, QuadrupedState};
    use photon_world::animation_state::AnimationState;
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let id = row["layer"].as_str().ok_or("layer")?;
        let layer = photon_data::entity_model::layer(id).ok_or("no such layer")?;
        let bits = |key: &str| -> Result<f32, String> {
            u32::from_str_radix(
                row[key].as_str().ok_or(format!("{key} must be a string"))?,
                16,
            )
            .map(f32::from_bits)
            .map_err(|error| error.to_string())
        };
        // A state that starts at the given tick, or none.
        let started = |key: &str| -> AnimationState {
            let mut state = AnimationState::default();
            if let Some(tick) = row[key].as_i64() {
                state.start(tick as i32);
            }
            state
        };
        let state = AxolotlRenderState {
            quadruped: QuadrupedState {
                head_y_rot: bits("y_rot")?,
                x_rot: bits("x_rot")?,
                walk_position: bits("walk_position")?,
                walk_speed: bits("walk_speed")?,
            },
            age_in_ticks: bits("age_in_ticks")?,
            playing_dead_factor: bits("playing_dead")?,
            moving_factor: bits("moving")?,
            in_water_factor: bits("in_water")?,
            on_ground_factor: bits("on_ground")?,
            swim: started("swim"),
            walk: started("walk"),
            idle_under_water: started("idle_under_water"),
            idle_under_water_on_ground: started("idle_under_water_on_ground"),
            idle_on_ground: started("idle_on_ground"),
            play_dead: started("play_dead"),
        };
        let mut model = MobModel::bake(layer);
        if id.contains("_baby") {
            model.setup_axolotl_baby(&state);
        } else {
            model.setup_axolotl_adult(&state);
        }
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `frog_model`: a frog's model posed and drawn from the animation states it runs, a tadpole's
/// from its age and whether it is in water.
pub(super) fn frog_model_result(request: &Value) -> Result<Value, String> {
    use photon_render::entity::mob_model::{
        FrogRenderState, MobModel, QuadrupedState, TadpoleState,
    };
    use photon_world::animation_state::AnimationState;
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let id = row["layer"].as_str().ok_or("layer")?;
        let layer = photon_data::entity_model::layer(id).ok_or("no such layer")?;
        let bits = |key: &str| -> Result<f32, String> {
            u32::from_str_radix(
                row[key].as_str().ok_or(format!("{key} must be a string"))?,
                16,
            )
            .map(f32::from_bits)
            .map_err(|error| error.to_string())
        };
        // A state that starts at the given tick, or none.
        let started = |key: &str| -> AnimationState {
            let mut state = AnimationState::default();
            if let Some(tick) = row[key].as_i64() {
                state.start(tick as i32);
            }
            state
        };
        let mut model = MobModel::bake(layer);
        if id.contains("tadpole") {
            model.setup_tadpole(&TadpoleState {
                age_in_ticks: bits("age_in_ticks")?,
                is_in_water: row["in_water"].as_bool().ok_or("in_water")?,
            });
        } else {
            model.setup_frog(&FrogRenderState {
                quadruped: QuadrupedState {
                    head_y_rot: 0.0,
                    x_rot: 0.0,
                    walk_position: bits("walk_position")?,
                    walk_speed: bits("walk_speed")?,
                },
                age_in_ticks: bits("age_in_ticks")?,
                is_swimming: row["swimming"].as_bool().ok_or("swimming")?,
                jump: started("jump"),
                croak: started("croak"),
                tongue: started("tongue"),
                swim_idle: started("swim_idle"),
            });
        }
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}

/// `airborne_model`: a blaze's, ghast's or phantom's model posed and drawn.
pub(super) fn airborne_model_result(request: &Value) -> Result<Value, String> {
    use photon_render::entity::mob_model::{
        AllayRenderState, BatRenderState, MobModel, QuadrupedState, VexRenderState,
    };
    use photon_world::animation_state::AnimationState;
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let id = row["layer"].as_str().ok_or("layer")?;
        let layer = photon_data::entity_model::layer(id).ok_or("no such layer")?;
        let bits = |key: &str| -> Result<f32, String> {
            u32::from_str_radix(
                row[key].as_str().ok_or(format!("{key} must be a string"))?,
                16,
            )
            .map(f32::from_bits)
            .map_err(|error| error.to_string())
        };
        let flag = |key: &str| row[key].as_bool().ok_or(format!("{key} must be a bool"));
        let mut model = MobModel::bake(layer);
        if id.contains("blaze") {
            model.setup_blaze(bits("age_in_ticks")?, bits("y_rot")?, bits("x_rot")?);
        } else if id.contains("happy_ghast") {
            if id.contains("harness") {
                model.setup_happy_ghast_harness(flag("ridden")?);
            } else {
                model.setup_happy_ghast(bits("age_in_ticks")?, flag("body_item")?);
            }
        } else if id.contains("breeze") {
            use photon_render::entity::mob_model::BreezeRenderState;
            let started = |key: &str| {
                let mut state = AnimationState::default();
                if let Some(tick) = row[key].as_i64() {
                    state.start(tick as i32);
                }
                state
            };
            model.setup_breeze(&BreezeRenderState {
                age_in_ticks: bits("age_in_ticks")?,
                idle: started("idle"),
                shoot: started("shoot"),
                slide: started("slide"),
                slide_back: started("slide_back"),
                inhale: started("inhale"),
                long_jump: started("long_jump"),
            });
        } else if id.contains("wind_charge") {
            model.setup_wind_charge(bits("age_in_ticks")?);
        } else if id.contains("shulker_bullet") {
            model.setup_shulker_bullet(bits("y_rot")?, bits("x_rot")?);
        } else if id.contains("leash_knot") {
            model.setup_leash_knot();
        } else if id.contains("llama_spit") {
            model.setup_llama_spit();
        } else if id.contains("evoker_fangs") {
            model.setup_evoker_fangs(bits("bite_progress")?);
        } else if id.contains("trident") {
            model.setup_trident();
        } else if id.contains("arrow") {
            use photon_render::entity::mob_model::ArrowRenderState;
            model.setup_arrow(&ArrowRenderState {
                shake: bits("shake")?,
            });
        } else if id.contains("end_crystal") {
            use photon_render::entity::mob_model::EndCrystalRenderState;
            model.setup_end_crystal(&EndCrystalRenderState {
                age_in_ticks: bits("age_in_ticks")?,
                shows_bottom: flag("shows_bottom")?,
            });
        } else if id.contains("ender_dragon") {
            use photon_render::entity::mob_model::EnderDragonRenderState;
            use photon_world::mobs::ender_dragon::FlightHistory;
            let mut history = FlightHistory::default();
            for sample in row["history"].as_array().ok_or("history")? {
                let y = u64::from_str_radix(sample["y"].as_str().ok_or("y")?, 16)
                    .map(f64::from_bits)
                    .map_err(|error| error.to_string())?;
                let y_rot = u32::from_str_radix(sample["y_rot"].as_str().ok_or("y_rot")?, 16)
                    .map(f32::from_bits)
                    .map_err(|error| error.to_string())?;
                history.record(y, y_rot);
            }
            let distance = u64::from_str_radix(
                row["distance_to_egg"].as_str().ok_or("distance_to_egg")?,
                16,
            )
            .map(f64::from_bits)
            .map_err(|error| error.to_string())?;
            model.setup_ender_dragon(&EnderDragonRenderState {
                flap_time: bits("flap_time")?,
                history,
                partial_ticks: bits("partial_ticks")?,
                is_landing_or_taking_off: flag("landing_or_taking_off")?,
                is_sitting: flag("sitting")?,
                distance_to_egg: distance,
            });
        } else if id.contains("wither") {
            use photon_render::entity::mob_model::WitherRenderState;
            model.setup_wither(&WitherRenderState {
                age_in_ticks: bits("age_in_ticks")?,
                y_rot: bits("y_rot")?,
                x_rot: bits("x_rot")?,
                body_rot: bits("body_rot")?,
                x_head_rots: [bits("x_head_0")?, bits("x_head_1")?],
                y_head_rots: [bits("y_head_0")?, bits("y_head_1")?],
            });
        } else if id.contains("sniffer") {
            use photon_render::entity::mob_model::SnifferRenderState;
            let started = |key: &str| {
                let mut state = AnimationState::default();
                if let Some(tick) = row[key].as_i64() {
                    state.start(tick as i32);
                }
                state
            };
            model.setup_sniffer(&SnifferRenderState {
                quadruped: QuadrupedState {
                    head_y_rot: bits("y_rot")?,
                    x_rot: bits("x_rot")?,
                    walk_position: bits("walk_position")?,
                    walk_speed: bits("walk_speed")?,
                },
                age_in_ticks: bits("age_in_ticks")?,
                is_searching: flag("searching")?,
                digging: started("digging"),
                sniffing: started("sniffing"),
                rising: started("rising"),
                feeling_happy: started("happy"),
                scenting: started("scenting"),
            });
        } else if id.contains("armadillo") {
            use photon_render::entity::mob_model::ArmadilloRenderState;
            let started = |key: &str| {
                let mut state = AnimationState::default();
                if let Some(tick) = row[key].as_i64() {
                    state.start(tick as i32);
                }
                state
            };
            model.setup_armadillo(
                &ArmadilloRenderState {
                    quadruped: QuadrupedState {
                        head_y_rot: bits("y_rot")?,
                        x_rot: bits("x_rot")?,
                        walk_position: bits("walk_position")?,
                        walk_speed: bits("walk_speed")?,
                    },
                    age_in_ticks: bits("age_in_ticks")?,
                    is_hiding_in_shell: flag("hiding")?,
                    roll_out: started("roll_out"),
                    roll_up: started("roll_up"),
                    peek: started("peek"),
                },
                id.contains("baby"),
            );
        } else if id.contains("creaking") {
            use photon_render::entity::mob_model::CreakingRenderState;
            let started = |key: &str| {
                let mut state = AnimationState::default();
                if let Some(tick) = row[key].as_i64() {
                    state.start(tick as i32);
                }
                state
            };
            model.setup_creaking(&CreakingRenderState {
                quadruped: QuadrupedState {
                    head_y_rot: bits("y_rot")?,
                    x_rot: bits("x_rot")?,
                    walk_position: bits("walk_position")?,
                    walk_speed: bits("walk_speed")?,
                },
                age_in_ticks: bits("age_in_ticks")?,
                can_move: flag("can_move")?,
                attack: started("attack"),
                invulnerable: started("invulnerable"),
                death: started("death"),
            });
        } else if id.contains("warden") {
            use photon_render::entity::mob_model::WardenRenderState;
            let started = |key: &str| {
                let mut state = AnimationState::default();
                if let Some(tick) = row[key].as_i64() {
                    state.start(tick as i32);
                }
                state
            };
            model.setup_warden(&WardenRenderState {
                quadruped: QuadrupedState {
                    head_y_rot: bits("y_rot")?,
                    x_rot: bits("x_rot")?,
                    walk_position: bits("walk_position")?,
                    walk_speed: bits("walk_speed")?,
                },
                age_in_ticks: bits("age_in_ticks")?,
                tendril_animation: bits("tendril")?,
                roar: started("roar"),
                sniff: started("sniff"),
                emerge: started("emerge"),
                digging: started("digging"),
                attack: started("attack"),
                sonic_boom: started("sonic_boom"),
            });
        } else if id.contains("parrot") {
            use photon_render::entity::mob_model::{ParrotPose, ParrotRenderState};
            let pose = match row["pose"].as_str().ok_or("pose")? {
                "FLYING" => ParrotPose::Flying,
                "STANDING" => ParrotPose::Standing,
                "SITTING" => ParrotPose::Sitting,
                "PARTY" => ParrotPose::Party,
                _ => ParrotPose::OnShoulder,
            };
            model.setup_parrot(&ParrotRenderState {
                quadruped: QuadrupedState {
                    head_y_rot: bits("y_rot")?,
                    x_rot: bits("x_rot")?,
                    walk_position: bits("walk_position")?,
                    walk_speed: bits("walk_speed")?,
                },
                age_in_ticks: bits("age_in_ticks")?,
                flap_angle: bits("flap_angle")?,
                pose,
            });
        } else if id.contains("bat") {
            let started = |key: &str| {
                let mut state = AnimationState::default();
                if let Some(tick) = row[key].as_i64() {
                    state.start(tick as i32);
                }
                state
            };
            model.setup_bat(&BatRenderState {
                head_y_rot: bits("y_rot")?,
                age_in_ticks: bits("age_in_ticks")?,
                is_resting: flag("resting")?,
                fly: started("fly"),
                rest: started("rest"),
            });
        } else if id.contains("vex") {
            model.setup_vex(&VexRenderState {
                head_y_rot: bits("y_rot")?,
                x_rot: bits("x_rot")?,
                age_in_ticks: bits("age_in_ticks")?,
                is_charging: flag("charging")?,
                right_hand_item: flag("right_item")?,
                left_hand_item: flag("left_item")?,
            });
        } else if id.contains("allay") {
            model.setup_allay(&AllayRenderState {
                quadruped: QuadrupedState {
                    head_y_rot: bits("y_rot")?,
                    x_rot: bits("x_rot")?,
                    walk_position: bits("walk_position")?,
                    walk_speed: bits("walk_speed")?,
                },
                age_in_ticks: bits("age_in_ticks")?,
                is_dancing: flag("dancing")?,
                is_spinning: flag("spinning")?,
                spinning_progress: bits("spinning_progress")?,
                holding_animation_progress: bits("holding_progress")?,
            });
        } else if id.contains("ghast") {
            model.setup_ghast(bits("age_in_ticks")?);
        } else {
            model.setup_phantom(bits("flap_time")?);
        }
        out.push(json!({"label": row["label"], "quads": posed_quads(&model)}));
    }
    Ok(json!({"rows": out}))
}
