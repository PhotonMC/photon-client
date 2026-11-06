//! What covers the player's view from inside (`ScreenEffectRenderer`): the block the eyes are
//! in, the water over the eyes and the fire the player burns in.

use super::{Level, LocalPlayer};
use crate::ClientWorld;
use photon_core::aabb::Aabb;
use photon_core::binary::BlockPos;
use photon_core::ids::BlockStateId;

impl LocalPlayer {
    /// `ScreenEffectRenderer.getViewBlockingState`: the first of eight points around the eyes
    /// that is inside a block that blocks the view, as a block state number. `near_plane` is the
    /// box round the camera's near plane, as an offset from the eyes.
    pub fn view_blocking_state(
        &self,
        world: &ClientWorld,
        near_plane: ([f64; 3], [f64; 3]),
    ) -> Option<BlockStateId> {
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
            if crate::shadow::is_invisible_render_shape(state.block().name) {
                continue;
            }
            let blocks = if state.near_plane_view_blocking {
                let eyes = [self.position.x, eye_y, self.position.z];
                near_plane_meets_outline(state, pos, near_plane, eyes)
            } else {
                state.view_blocking
            };
            if blocks {
                return Some(id);
            }
        }
        None
    }
}

/// `NEAR_PLANE_INTERSECTS_OUTLINE`: whether any box of the state's occlusion shape, placed at
/// `pos`, overlaps the near plane's box moved to the eyes.
fn near_plane_meets_outline(
    state: &crate::block::StateInfo,
    pos: BlockPos,
    (low, high): ([f64; 3], [f64; 3]),
    eyes: [f64; 3],
) -> bool {
    let near = Aabb::new(
        low[0] + eyes[0],
        low[1] + eyes[1],
        low[2] + eyes[2],
        high[0] + eyes[0],
        high[1] + eyes[1],
        high[2] + eyes[2],
    );
    crate::block::states()
        .occlusion(state)
        .to_aabbs()
        .into_iter()
        .any(|part| {
            part.move_by(f64::from(pos.x), f64::from(pos.y), f64::from(pos.z))
                .intersects(near)
        })
}
