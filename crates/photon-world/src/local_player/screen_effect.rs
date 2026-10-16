//! What covers the player's view from inside (`ScreenEffectRenderer`): the block the eyes are
//! in, the water over the eyes and the fire the player burns in.

use super::{Level, LocalPlayer};
use crate::ClientWorld;
use photon_core::binary::BlockPos;
use photon_core::ids::BlockStateId;

impl LocalPlayer {
    /// `ScreenEffectRenderer.getViewBlockingState`: the first of eight points around the eyes
    /// that is inside a block that blocks the view, as a block state number.
    pub fn view_blocking_state(&self, world: &ClientWorld) -> Option<BlockStateId> {
        if self.no_physics {
            return None;
        }
        let level = Level::new(world);
        let width = self.dimensions.width.get();
        // The reference's `getScale` is one for a player that has no scale attribute.
        let scale = 1.0_f32;
        let eye_y = self.position.y + f64::from(self.eye_height());
        for i in 0..8_i32 {
            let offset = |bit: i32| ((i >> bit) % 2) as f32 - 0.5;
            let x = self.position.x + f64::from(offset(0) * width * 0.8);
            let y = eye_y + f64::from(offset(1) * 0.1 * scale);
            let z = self.position.z + f64::from(offset(2) * width * 0.8);
            let pos = BlockPos::new(x.floor() as i32, y.floor() as i32, z.floor() as i32);
            let Some(id) = level.state_id(pos) else {
                continue;
            };
            let state = crate::block::state_or_air(id);
            if !crate::shadow::is_invisible_render_shape(state.block().name) && state.view_blocking
            {
                return Some(id);
            }
        }
        None
    }
}
