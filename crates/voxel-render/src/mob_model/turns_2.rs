use super::*;

/// What a fish renderer's `setupRotations` does besides the common turns: the
/// pufferfish's bob (before them), the swing about y after the fall (none for
/// the pufferfish), and the
/// shift a fish out of water makes before it lies on its side.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FishTurns {
    pub bob: f32,
    pub swing: Option<f32>,
    pub on_land: Option<[f32; 3]>,
}

/// What a squid's renderer turns the pose by: the body's two turns, and
/// whether it is a baby (it hangs lower).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SquidTurns {
    pub x_body_rot: f32,
    pub z_body_rot: f32,
    pub baby: bool,
}

impl Default for MobPlacement {
    /// A mob of the ordinary size, upright where the mesh starts, unlit.
    fn default() -> Self {
        Self {
            feet: [0.0; 3],
            body_rot: 0.0,
            scale: 1.0,
            light: 0,
            death_time: 0.0,
            flip_degrees: FLIP_DEGREES,
            red_overlay: false,
            white_overlay: 0.0,
            hook_scale: None,
            hook_translate: None,
            roll: None,
            shift: None,
            roll_shift: None,
            pitch: None,
            panda: None,
            shaking: None,
            squid: None,
            fish: None,
            cube_mob: None,
            attach_face: None,
        }
    }
}

/// `LivingEntityRenderer.getFlipDegrees` of most mobs.
pub const FLIP_DEGREES: f32 = 90.0;

/// What the entity shader reads as the alpha of a vertex lit by the world and blended
/// (`RenderTypes.entityTranslucent` with a colour whose alpha is not full): the shade, past
/// every code the overlay uses, plus the alpha in 255ths.
pub const BLENDED_SHADE: f32 = 64.0;
