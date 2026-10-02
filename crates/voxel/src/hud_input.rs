//! What the HUD shows, read from the session: the player's attributes (by
//! the registry's ids), effects, stacks and status.

use voxel_net::connection::PlaySession;
use voxel_render::gui::locator::{LocatorCamera, LocatorInput, Viewer};
use voxel_world::ClientPlayerState;

const ARMOR: i32 = 1;
const ATTACK_SPEED: i32 = 5;
const MAX_HEALTH: i32 = 23;
const REGENERATION: i32 = 9;
const HUNGER: i32 = 16;
const POISON: i32 = 18;
const WITHER: i32 = 19;

/// An attribute's value within `min..=max`, or `default` without one.
fn attribute(local: Option<&ClientPlayerState>, id: i32, default: f64, min: f64, max: f64) -> f64 {
    local
        .and_then(|state| {
            state
                .attributes()
                .iter()
                .find(|attribute| attribute.attribute_id() == id)
        })
        .map_or(default, |attribute| attribute.effective_value(min, max))
}

/// The attack speed attribute, which the attack strength recovers by.
pub fn attack_speed(local: Option<&ClientPlayerState>) -> f64 {
    attribute(local, ATTACK_SPEED, 4.0, 0.0, 1024.0)
}

fn has_effect(local: Option<&ClientPlayerState>, id: i32) -> bool {
    local.is_some_and(|state| {
        state
            .effects()
            .iter()
            .any(|effect| effect.effect_id() == id)
    })
}

/// Everything the HUD draws from, the camera in first person or not.
pub fn hud_input(
    session: &PlaySession,
    first_person: bool,
    camera: LocatorCamera,
) -> voxel_render::gui::HudInput {
    let status = *session.status();
    let game_mode = session.game_mode();
    let local = session.world().local_player();
    let player = session.player();
    let stack = |slot: i16| {
        local
            .and_then(|state| state.inventory_slot(slot))
            .filter(|stack| stack.item_count() > 0 && stack.item_registry_id() != 0)
            .cloned()
    };
    voxel_render::gui::HudInput {
        first_person,
        spectator: game_mode == voxel_world::ClientPlayerGameMode::Spectator,
        survival: game_mode.is_survival(),
        selected_slot: status.selected_slot,
        health: player.map_or(status.health, |player| player.health()),
        max_health: attribute(local, MAX_HEALTH, 20.0, 1.0, 1024.0) as f32,
        absorption: local.map_or(0.0, |state| state.absorption()),
        invulnerable_time: player.map_or(0, |player| player.invulnerable_time()),
        heart_type: voxel_render::gui::HeartType::for_player(
            has_effect(local, POISON),
            has_effect(local, WITHER),
            local.is_some_and(|state| state.is_fully_frozen()),
        ),
        regeneration: has_effect(local, REGENERATION),
        hardcore: session.hardcore(),
        armor: attribute(local, ARMOR, 0.0, 0.0, 30.0).floor() as i32,
        food: status.food,
        saturation: status.saturation,
        hunger: has_effect(local, HUNGER),
        air_supply: local.map_or(300, |state| state.air_supply()),
        max_air_supply: 300,
        eye_in_water: player.is_some_and(|player| player.is_eye_in_water()),
        experience_progress: status.experience_progress,
        experience_level: status.experience_level,
        // `willPrioritizeExperienceInfo`.
        experience_prioritised: status.experience_display_start_tick.wrapping_add(100)
            > player.map_or(0, |player| player.tick_count()),
        locator: session.waypoints().has_waypoints().then(|| LocatorInput {
            waypoints: session.waypoints().clone(),
            camera,
            viewer: Viewer {
                uuid: session
                    .profile_id()
                    .map_or(0, |id| u128::from(id.most) << 64 | u128::from(id.least)),
                position: player.map_or([0.0; 3], |player| {
                    let at = player.position();
                    [at.x, at.y, at.z]
                }),
            },
        }),
        attack_strength: player.map_or(1.0, |player| {
            player.attack_strength_scale(0.0, attack_speed(local))
        }),
        hotbar: std::array::from_fn(|i| stack(36 + i as i16)),
        offhand: stack(45),
    }
}
