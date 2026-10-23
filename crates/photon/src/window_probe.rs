//! `photon window`: the window and Vulkan probe, which draws one block,
//! a grid of them, or a box of a server's world, and can capture the frame.

use std::env;
use std::net::SocketAddr;

use photon_render::VulkanBackend;
use photon_render::window::{DisplayData, WindowSurface};

use crate::assets::StackAssets;
use crate::ppm;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// The variant a `PHOTON_BLOCK` name and its `PHOTON_STATE` properties select.
struct ChosenBlock {
    root: String,
    model: String,
    /// Quarter turns about x, y and z, in degrees.
    rotation: [i32; 3],
    uvlock: bool,
    /// The block's name, which its fixed tints are looked up by.
    name: String,
}

/// How a model is turned: its transform unless the turn is a whole one, the
/// quarter turns it came from, and whether its UVs stay put.
struct Turn<'a> {
    state: Option<&'a photon_render::Matrix4>,
    rotation: [i32; 3],
    uv_lock: bool,
}

/// Placeholder clear colour. This is not a reference value: no Java frame has
/// been captured or compared yet, so nothing about these pixels is parity.
const CLEAR_COLOUR: [f32; 4] = [0.1, 0.12, 0.16, 1.0];

/// Where the player stands inside a drawn world box, eyes included.
///
/// Set by the world loader rather than read from the environment, because it
/// depends on where the server put the player. The eye height is the
/// reference's standing value.
static CAMERA_EYE: std::sync::OnceLock<[f32; 3]> = std::sync::OnceLock::new();

mod version;
pub use version::*;
mod world;
use world::*;
