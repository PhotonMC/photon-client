//! What the HUD shows, read from the session: the player's attributes (by
//! the registry's ids), effects, stacks and status.

use photon_data::item::registry::AIR_ITEM;
use photon_game::game_mode::Aim;
use photon_game::input::pick::pick;
use photon_game::options::AttackIndicator;
use photon_gui::hud::locator::{LocatorCamera, LocatorInput, Viewer};
use photon_net::connection::PlaySession;
use photon_world::ClientPlayerState;
use photon_world::local_player::attack_strength_delay;

use crate::view::eye_position;

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

/// `Minecraft.crosshairPickEntity`: a living entity the crosshair is on that has not died.
fn crosshair_target_is_living(
    session: &PlaySession,
    player: Option<&photon_world::local_player::LocalPlayer>,
) -> bool {
    player.is_some_and(|player| {
        let Aim::Entity { id, .. } = pick(session.world(), player, eye_position(player, 1.0))
        else {
            return false;
        };
        session
            .world()
            .entities()
            .all()
            .find(|entity| entity.id() == id)
            .is_some_and(|entity| entity.is_living_entity() && !entity.is_dead_or_dying())
    })
}

/// The locator bar's input, when the server tracks any waypoints.
fn locator_input(
    session: &PlaySession,
    camera: LocatorCamera,
    player: Option<&photon_world::local_player::LocalPlayer>,
) -> Option<LocatorInput> {
    session.waypoints().has_waypoints().then(|| LocatorInput {
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
    })
}

/// The textures the worn and held items lay over the view (`Equippable.cameraOverlay` of an item
/// in the slot it names), in the order of `EquipmentSlot.values()`. Only the game's own textures
/// are known to the GUI atlas.
fn worn_overlays(session: &PlaySession) -> Vec<String> {
    let selected = 36 + i16::from(session.status().selected_slot as i8);
    // The slot's name and where the player's menu keeps it.
    let slots = [
        ("mainhand", selected),
        ("offhand", 45),
        ("feet", 8),
        ("legs", 7),
        ("chest", 6),
        ("head", 5),
    ];
    let local = session.world().local_player();
    let stacks: Vec<_> = slots
        .into_iter()
        .filter_map(|(name, index)| {
            let stack = local?
                .inventory_slot(index)
                .filter(|stack| stack.item_count() > 0)?;
            Some((name, stack.item_registry_id(), stack.patch_bytes()))
        })
        .collect();
    photon_game::item::values::camera_overlay_textures(&stacks)
}

/// `Hud.extractCameraOverlays`: the vignette (its option is on) and how strongly the border warns
/// at the camera's feet, the powder snow frame, and the portal overlay with where the nether
/// portal's sprite is in the block atlas, as the partial tick between two ticks has them.
pub fn camera_overlays(
    session: &PlaySession,
    renderer: &photon_render::block::world_mesh::BlockRenderer,
    (partial, options): (f32, &photon_game::options::Options),
) -> photon_gui::hud::CameraOverlays {
    let Some(player) = session.player() else {
        return photon_gui::hud::CameraOverlays::default();
    };
    let feet = player.position();
    let intensity = player.portal_effect_intensity(partial);
    let state = photon_data::block::registry::default_state_named("minecraft:nether_portal");
    let sprite = renderer
        .particle_sprite(state)
        .and_then(|name| renderer.sprite_rect(name));
    photon_gui::hud::CameraOverlays {
        vignette: Some(
            session
                .world()
                .world_border()
                .warning_strength(feet.x, feet.z),
        ),
        worn: worn_overlays(session),
        scoping: player.is_scoping(),
        powder_snow: session
            .world()
            .local_player()
            .filter(|state| state.is_freezing())
            .map(ClientPlayerState::percent_frozen),
        portal: sprite.filter(|_| intensity > 0.0).map(|uv| (intensity, uv)),
        nausea: Some((
            crate::world::fog::nausea_blend(session, partial),
            options.screen_effect_scale as f32,
        )),
    }
}

/// Everything the HUD draws from, the camera in first person or not.
pub fn hud_input(
    session: &PlaySession,
    first_person: bool,
    camera: LocatorCamera,
    attack_indicator: AttackIndicator,
) -> photon_gui::HudInput {
    let status = *session.status();
    let game_mode = session.game_mode();
    let local = session.world().local_player();
    let player = session.player();
    let stack = |slot: i16| {
        local
            .and_then(|state| state.inventory_slot(slot))
            .filter(|stack| stack.item_count() > 0 && stack.item_registry_id() != AIR_ITEM)
            .cloned()
    };
    let target_is_living = crosshair_target_is_living(session, player);
    photon_gui::HudInput {
        cooldowns: Vec::new(),
        first_person,
        three_dimensional_crosshair: false,
        spectator: game_mode == photon_world::ClientPlayerGameMode::Spectator,
        survival: game_mode.is_survival(),
        selected_slot: status.selected_slot,
        health: player.map_or(status.health, |player| player.health()),
        max_health: attribute(local, MAX_HEALTH, 20.0, 1.0, 1024.0) as f32,
        absorption: local.map_or(0.0, |state| state.absorption()),
        invulnerable_time: player.map_or(0, |player| player.invulnerable_time()),
        heart_type: photon_gui::HeartType::for_player(
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
        locator: locator_input(session, camera, player),
        overlays: Default::default(),
        attack_strength: player.map_or(1.0, |player| {
            player.attack_strength_scale(0.0, attack_speed(local))
        }),
        attack_delay: attack_strength_delay(attack_speed(local)),
        attack_indicator,
        target_is_living,
        hotbar: std::array::from_fn(|i| stack(36 + i as i16)),
        offhand: stack(45),
    }
}

/// What the HUD draws from this frame: the cooldowns and camera overlays at the frame's partial tick,
/// and whether the 3D crosshair is among the debug entries shown, which the flat one gives way to.
pub fn frame_input(
    (session, controls): (&PlaySession, &crate::controls::Controls),
    (renderer, partial, camera): (
        &photon_render::block::world_mesh::BlockRenderer,
        f32,
        LocatorCamera,
    ),
) -> photon_gui::HudInput {
    let shown = controls.debug.entries.enabled(session.reduced_debug_info());
    photon_gui::HudInput {
        // `getGameTimeDeltaPartialTick(true)`.
        cooldowns: session.item_cooldowns().percents(partial),
        overlays: camera_overlays(session, renderer, (partial, &controls.options)),
        three_dimensional_crosshair: shown.contains(&"3d_crosshair"),
        ..hud_input(
            session,
            controls.camera_type == crate::view::CameraType::FirstPerson,
            camera,
            controls.options.attack_indicator,
        )
    }
}
