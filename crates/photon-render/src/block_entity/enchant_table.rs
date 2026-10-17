//! The enchanting table's book (`EnchantTableRenderer`): its model layer and texture, how far
//! each page is flipped, and the pose it floats and turns in.

use crate::joml::{Pose, Quaternion};
use photon_core::math::{frac_f32, mth_sin_f64};

/// The model layer of the book (`ModelLayers.BOOK`).
pub const LAYER: &str = "minecraft:book#main";

/// The texture under `textures/` (`EnchantTableRenderer.BOOK_TEXTURE`).
pub const TEXTURE: &str = "entity/enchantment/enchanting_table_book";

/// What the book's model is set up with: how open it is, and how far each loose page is flipped
/// (`BookModel.State.forAnimation`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BookPose {
    pub openness: f32,
    pub pages: (f32, f32),
}

/// `BookModel.State.forAnimation` with the page flips `EnchantTableRenderer.submit` works out
/// from the table's `flip`.
pub fn book_pose(time: f32, flip: f32, open: f32) -> BookPose {
    let first = (frac_f32(flip + 0.25) * 1.6 - 0.3).clamp(0.0, 1.0);
    let second = (frac_f32(flip + 0.75) * 1.6 - 0.3).clamp(0.0, 1.0);
    let sin = mth_sin_f64(f64::from(time * 0.02));
    BookPose {
        openness: (sin * 0.1 + 1.25) * open,
        pages: (first, second),
    }
}

/// The pose the book is drawn from, with `base` the pose of the block's corner: floating above
/// the table, bobbing, turned to `y_rot`, and laid over by eighty degrees.
pub fn table_pose(base: &Pose, time: f32, y_rot: f32) -> Pose {
    let mut pose = *base;
    pose.translate(0.5, 0.75, 0.5);
    let bob = mth_sin_f64(f64::from(time * 0.1));
    pose.translate(0.0, 0.1 + bob * 0.01, 0.0);
    pose.rotate(&Quaternion::rotation_y(-y_rot));
    pose.rotate(&Quaternion::rotation_z(Quaternion::degrees_to_radians(
        80.0,
    )));
    pose
}
