//! What a player wears on the back: the cape (`CapeLayer`) and the wings of an elytra
//! (`WingsLayer`).

use super::*;
use photon_core::math::{mth_cos_f64, mth_sin_f64};
use photon_render::entity::cape::{CapeAngles, cape_vertices};
use photon_render::entity::textures::{EntityTextures, TextureRect};
use photon_world::EntityView;
use photon_world::cloak::Cloak;
use photon_world::local_player::CloakState;

/// What the cape of an avatar hangs from, and how it is worn.
pub(super) struct Hang {
    /// Where the cape is, or none where the avatar has none.
    pub rect: Option<TextureRect>,
    /// Where the profile's own elytra texture is, if it names one.
    pub elytra: Option<TextureRect>,
    /// `state.showCape`.
    pub shows_cape: bool,
    /// `state.isInvisible`.
    pub invisible: bool,
    pub cloak: CloakState,
    /// Where the avatar is between the last two ticks.
    pub position: [f64; 3],
    /// `getFallFlyingTicks`.
    pub glide_ticks: i32,
}

impl Hang {
    /// The cloak of a remote player, who never adds to the walk distance.
    pub(super) fn remote_cloak(cloak: &Cloak, partial: f32) -> CloakState {
        CloakState {
            position: cloak.interpolated(partial),
            walk_distance: 0.0,
            bob: cloak.interpolated_bob(partial),
        }
    }
}

/// `AvatarRenderer.extractCapeState`.
pub(super) fn angles(hang: &Hang, body_rot: f32, partial: f32) -> CapeAngles {
    let delta: [f64; 3] =
        std::array::from_fn(|axis| hang.cloak.position[axis] - hang.position[axis]);
    let yaw = body_rot * (std::f64::consts::PI / 180.0) as f32;
    let forward_x = f64::from(mth_sin_f64(f64::from(yaw)));
    let forward_z = -f64::from(mth_cos_f64(f64::from(yaw)));
    let time = hang.glide_ticks as f32 + partial;
    let glide_scale = (time * time / 100.0).clamp(0.0, 1.0);
    let flap = (delta[1] as f32 * 10.0).clamp(-6.0, 32.0);
    let lean = (delta[0] * forward_x + delta[2] * forward_z) as f32 * 100.0;
    let lean = (lean * (1.0 - glide_scale)).clamp(0.0, 150.0);
    let lean2 = ((delta[0] * forward_z - delta[2] * forward_x) as f32 * 100.0).clamp(-20.0, 20.0);
    let bob_flap = mth_sin_f64(f64::from(hang.cloak.walk_distance * 6.0)) * 32.0 * hang.cloak.bob;
    CapeAngles {
        lean,
        lean2,
        flap: flap + bob_flap,
    }
}

/// `CapeLayer.submit`: the cape of a player who shows one, unless wings are worn on the chest.
pub(super) fn cape(
    armor: &Armor,
    (chest, hang): (Option<&ClientItemStackState>, &Hang),
    (model, placement): (&PlayerModel, &Placement),
    partial: f32,
) -> Vec<Vertex> {
    let (true, Some(rect)) = (hang.shows_cape && !hang.invisible, hang.rect) else {
        return Vec::new();
    };
    if armor.has_layer(chest, "wings") {
        return Vec::new();
    }
    let armored = armor.has_layer(chest, "humanoid");
    let angles = angles(hang, placement.body_rot, partial);
    cape_vertices((model, placement), (rect, angles), armored)
}

/// The cape and the wings of a player: the wings are spread by `spread` (its elytra's angles).
pub(super) fn back(
    armor: &mut Armor,
    (chest, hang, spread): (Option<&ClientItemStackState>, &Hang, [f32; 3]),
    (model, placement): (&PlayerModel, &Placement),
    (textures, partial): (&EntityTextures, f32),
) -> Vec<Vertex> {
    let mut vertices = cape(armor, (chest, hang), (model, placement), partial);
    vertices.extend(armor.wings(
        chest,
        (false, placement.crouching, spread),
        // `WingsLayer.getPlayerElytraTexture`: the cape of a player who shows it is the wings'.
        (
            textures,
            hang.elytra.or(hang.rect.filter(|_| hang.shows_cape)),
        ),
        &|model, rect, colour| {
            photon_render::entity::player_renderer::layer_vertices(model, placement, rect, colour)
        },
    ));
    vertices
}

/// The cape and wings of a remote player, whose cape (if it has one) is at `rect`.
pub(super) fn remote_back(
    armor: &mut Armor,
    (entity, rect, elytra): (EntityView<'_>, Option<TextureRect>, Option<TextureRect>),
    (model, placement): (&PlayerModel, &Placement),
    (textures, partial): (&EntityTextures, f32),
) -> Vec<Vertex> {
    let position = entity.interpolated_position(partial);
    let hang = Hang {
        rect,
        elytra,
        shows_cape: entity.shows_cape(),
        invisible: entity.is_invisible(),
        cloak: Hang::remote_cloak(entity.cloak(), partial),
        position: [position.0, position.1, position.2],
        glide_ticks: entity.fall_flying_ticks(),
    };
    let chest = entity.equipment(EquipmentSlot::Chest);
    let spread = entity.elytra_rotations(partial);
    back(
        armor,
        (chest, &hang, spread),
        (model, placement),
        (textures, partial),
    )
}

/// What the cape of the player's own model hangs from.
pub(super) fn own_hang(
    session: &PlaySession,
    player: &LocalPlayer,
    (view, invisible): (FrameView, bool),
) -> cape::Hang {
    let partial = view.partial as f32;
    let position = player.interpolated_position(view.partial);
    let shown = session
        .world()
        .local_player()
        .is_some_and(|state| state.model_customisation() & 1 != 0);
    cape::Hang {
        rect: view.cape,
        elytra: view.elytra,
        shows_cape: shown,
        invisible,
        cloak: player.cloak_state(partial),
        position: [position.x, position.y, position.z],
        glide_ticks: player.fall_flying_ticks(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hang(cloak: [f64; 3], position: [f64; 3], glide_ticks: i32) -> Hang {
        Hang {
            rect: None,
            elytra: None,
            shows_cape: true,
            invisible: false,
            cloak: CloakState {
                position: cloak,
                walk_distance: 0.0,
                bob: 0.0,
            },
            position,
            glide_ticks,
        }
    }

    #[test]
    fn walking_forward_swings_the_cape_back() {
        // Facing south (yaw 0, forward is +z), half a block ahead of the cloak: the lean is the
        // lag along the facing times a hundred.
        let angles = angles(&hang([0.0; 3], [0.0, 0.0, 0.5], 0), 0.0, 0.0);
        assert_eq!(angles.lean, 50.0);
        assert_eq!(angles.lean2, 0.0);
        assert_eq!(angles.flap, 0.0);
    }

    #[test]
    fn a_step_aside_swings_it_across_and_walking_backwards_does_not_lean_it() {
        let angles = angles(&hang([0.0; 3], [0.1, 0.0, -0.2], 0), 0.0, 0.0);
        assert!((angles.lean2 - 10.0).abs() < 1e-4);
        // Moving against the facing makes the lean negative, which is held at zero.
        assert_eq!(angles.lean, 0.0);
    }

    #[test]
    fn the_leans_are_held_in_their_ranges() {
        let far = angles(&hang([0.0; 3], [5.0, 0.0, 5.0], 0), 90.0, 0.0);
        assert!(far.lean2.abs() <= 20.0 && (0.0..=150.0).contains(&far.lean));
        let up = angles(&hang([0.0, 0.0, 0.0], [0.0, -9.0, 0.0], 0), 0.0, 0.0);
        assert_eq!(up.flap, 32.0);
        let down = angles(&hang([0.0, 0.0, 0.0], [0.0, 9.0, 0.0], 0), 0.0, 0.0);
        assert_eq!(down.flap, -6.0);
    }

    #[test]
    fn a_glider_does_not_lean_its_cape() {
        // Ten ticks of gliding make the scale one: the lean is taken out.
        let gliding = angles(&hang([0.0; 3], [0.0, 0.0, 0.5], 10), 0.0, 0.5);
        assert_eq!(gliding.lean, 0.0);
    }

    #[test]
    fn the_walking_bob_lifts_the_cape() {
        let mut walking = hang([0.0; 3], [0.0; 3], 0);
        // Six times this walk distance is a quarter turn, where the sine is one.
        walking.cloak.walk_distance = std::f32::consts::FRAC_PI_2 / 6.0;
        walking.cloak.bob = 0.1;
        let flap = angles(&walking, 0.0, 0.0).flap;
        assert!((flap - 3.2).abs() < 0.01, "{flap}");
    }
}
