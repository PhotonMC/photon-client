//! What is drawn over the world's blocks from the player's side: the hands
//! and what they hold, the aimed block's outline, the cracks in blocks being
//! broken, and the player's own model.

pub mod screen_effect;

use photon_core::math::Vec3;
use photon_game::game_mode::{Aim, MAIN_HAND, OFF_HAND};
use photon_game::item::in_hand::ItemInHand;
use photon_net::connection::PlaySession;
use photon_render::block::biome_tint::BiomeTints;
use photon_render::block::outline::LineVertex;
use photon_render::block::world_mesh::BlockRenderer;
use photon_render::entity::player_model::{Arm, PlayerModel, PlayerRenderState};
use photon_render::entity::player_renderer::{HandView, Placement};
use photon_render::scene::Vertex;
use photon_world::local_player::LocalPlayer;
use photon_world::{ClientItemStackState, EquipmentSlot};

use crate::equipment::armor::{Armor, Worn};
use crate::equipment::held_items::{HEAD_TRANSFORMS, Hands, HeldItem};
use crate::equipment::item_layers::ItemLayerCache;
use crate::hud_input::attack_speed;
use crate::view::eye_position;
use photon_game::input::pick::pick;

/// Where the frame is drawn from: how far between ticks, the light at the
/// eyes (packed block and sky light), and the point meshes are measured
/// from.
#[derive(Clone, Copy)]
pub struct FrameView {
    pub partial: f64,
    /// `Options.getEffectiveRenderDistance`, in chunks.
    pub render_distance: i32,
    pub light: i32,
    pub origin: [i32; 3],
    /// Where the camera is, from that origin.
    pub camera: [f32; 3],
    /// The camera's yaw and pitch, in degrees.
    pub rotation: (f32, f32),
    /// Where the skin the local player wears is in the entity textures.
    pub skin: photon_render::entity::textures::TextureRect,
}

impl FrameView {
    /// A block's corner relative to the mesh origin.
    fn relative(&self, [x, y, z]: [i32; 3]) -> [f32; 3] {
        [
            (x - self.origin[0]) as f32,
            (y - self.origin[1]) as f32,
            (z - self.origin[2]) as f32,
        ]
    }
}

/// The player models the frame draws for the players the server added: one
/// wide-armed and one slim, which each remote player is posed in turn.
pub struct RemoteModels {
    wide: PlayerModel,
    slim: PlayerModel,
}

impl RemoteModels {
    pub fn new() -> Self {
        Self {
            wide: PlayerModel::new(false),
            slim: PlayerModel::new(true),
        }
    }
}

impl Default for RemoteModels {
    fn default() -> Self {
        Self::new()
    }
}

/// Where the name tags are seen from.
pub struct TagCamera {
    pub position: Vec3,
    /// The camera relative to the mesh origin.
    pub relative: [f32; 3],
    /// The camera's yaw and pitch.
    pub rotation: (f32, f32),
    /// The local player watches from the spectator's seat.
    pub spectating: bool,
    /// The interface is hidden (F1): so are the names over players.
    pub hud_hidden: bool,
}

/// The path (under `textures/`) of the experience orbs' texture.
pub const ORB_TEXTURE: &str = "entity/experience/experience_orb";

/// What casts a shadow: where it is drawn and where it ticked (the camera's
/// distance is measured to the latter), and its renderer's radius and strength.
struct Caster {
    drawn: (f64, f64, f64),
    ticked: (f64, f64, f64),
    radius: f32,
    strength: f32,
}

/// The shadow texture's path (under `textures/`).
pub const SHADOW_TEXTURE: &str = "misc/shadow";

mod light;
pub use light::*;
mod wear;
pub use wear::*;
